use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, Options};
use headless_chrome::Browser;
use scraper::{Html, Selector};
use std::fs;
use std::path::Path;
use std::time::Duration;

const PINE_REFERENCE_URL: &str = "https://www.tradingview.com/pine-script-reference/v6/";
const REFERENCE_MARKDOWN: &str = include_str!("../spec/v6.md");

/// Downloads the Pine Script reference page and saves it to spec/v6.md
pub fn download_and_save_reference() -> eyre::Result<()> {
    let browser = Browser::default().map_err(|e| eyre::eyre!("Failed to start browser: {}", e))?;
    let tab = browser
        .new_tab()
        .map_err(|e| eyre::eyre!("Failed to open tab: {}", e))?;

    tab.navigate_to(PINE_REFERENCE_URL)
        .map_err(|e| eyre::eyre!("Failed to navigate: {}", e))?;

    // Wait for actual content items to load
    tab.wait_for_element_with_custom_timeout("#var_ask", Duration::from_secs(60))
        .map_err(|e| eyre::eyre!("Failed to wait for content element: {}", e))?;

    // Give it more time to ensure all content is loaded
    std::thread::sleep(Duration::from_secs(5));

    let html_content = tab
        .get_content()
        .map_err(|e| eyre::eyre!("Failed to get content: {}", e))?;

    // Convert HTML to markdown
    let document = Html::parse_document(&html_content);
    let selector = Selector::parse(".tv-script-reference__content-container")
        .map_err(|e| eyre::eyre!("Failed to parse selector: {:?}", e))?;

    let markdown_parts: Vec<_> = document
        .select(&selector)
        .map(|element| htmd::convert(&element.inner_html()))
        .collect::<Result<Vec<_>, _>>()?;

    if markdown_parts.is_empty() {
        return Err(eyre::eyre!("No content found with selector"));
    }

    let markdown = markdown_parts.join("\n\n");

    // Write to file
    let spec_dir = Path::new("crates/pine-reference/spec");
    fs::create_dir_all(spec_dir)?;

    let output_path = spec_dir.join("v6.md");
    fs::write(&output_path, markdown)?;

    eprintln!("Successfully wrote reference to {:?}", output_path);
    Ok(())
}

#[derive(Debug)]
pub struct Section {
    pub title: String,
    pub level: u8,
    pub content: String,
}

/// Parses a markdown file and extracts all sections with their headers
fn parse_markdown_sections(markdown_content: &str) -> eyre::Result<Vec<Section>> {
    let arena = Arena::new();
    let options = Options::default();
    let root = parse_document(&arena, markdown_content, &options);

    let mut sections = Vec::new();
    let lines: Vec<&str> = markdown_content.lines().collect();

    // Collect all headings first
    fn collect_headings<'a>(node: &'a AstNode<'a>, headings: &mut Vec<(String, u8, usize)>) {
        if let NodeValue::Heading(heading) = &node.data.borrow().value {
            let level = heading.level;
            if level == 2 || level == 3 {
                let mut heading_text = String::new();
                for child in node.children() {
                    if let NodeValue::Text(text) = &child.data.borrow().value {
                        heading_text.push_str(text);
                    }
                }
                let line_num = node.data.borrow().sourcepos.start.line;
                headings.push((heading_text, level, line_num));
            }
        }

        for child in node.children() {
            collect_headings(child, headings);
        }
    }

    let mut headings = Vec::new();
    collect_headings(root, &mut headings);

    // Now create sections with content between headings
    for (i, (title, level, start_line)) in headings.iter().enumerate() {
        let end_line = if i + 1 < headings.len() {
            headings[i + 1].2 - 1 // Exclude the next heading line
        } else {
            lines.len()
        };

        let content: Vec<String> = lines[*start_line..end_line]
            .iter()
            .skip(1) // Skip the heading line itself
            .map(|s| s.to_string())
            .collect();

        sections.push(Section {
            title: title.clone(),
            level: *level,
            content: content.join("\n").trim_end().to_string(),
        });
    }

    Ok(sections)
}

pub enum QueryResult {
    /// List of matching items
    List(Vec<String>),
    /// Full content of an exact match
    Content(String),
}

/// Query the reference with smart matching:
/// - If path matches exactly one item, return its content
/// - If path is a prefix or matches multiple items, return the list
/// - If no path given, return all top-level sections
///
/// Examples:
///   None -> List all top-level sections (Variables, Constants, etc.)
///   Some("Variables") -> List all items under Variables
///   Some("Variables.bar") -> List items starting with "bar" (bar_index, barstate.*)
///   Some("Variables.ask") -> Content of Variables.ask if exact match
pub fn query(path: Option<&str>) -> eyre::Result<QueryResult> {
    let sections = parse_markdown_sections(REFERENCE_MARKDOWN)?;

    match path {
        None => {
            // List all top-level sections
            let results: Vec<String> = sections
                .into_iter()
                .filter(|s| s.level == 2)
                .map(|s| s.title)
                .collect();
            Ok(QueryResult::List(results))
        }
        Some(path) => {
            let parts: Vec<&str> = path.split('.').collect();
            if parts.is_empty() {
                return Ok(QueryResult::List(Vec::new()));
            }

            // Try exact match first
            if parts.len() == 2 {
                let section_name = parts[0];
                let item_name = parts[1];
                let mut in_target_section = false;

                for section in &sections {
                    if section.level == 2 {
                        in_target_section = section.title.eq_ignore_ascii_case(section_name);
                    } else if section.level == 3 && in_target_section {
                        if section.title.eq_ignore_ascii_case(item_name) {
                            return Ok(QueryResult::Content(section.content.clone()));
                        }
                    }
                }
            }

            // No exact match, do prefix search
            let section_name = parts[0];
            let prefix = if parts.len() > 1 {
                parts[1..].join(".")
            } else {
                String::new()
            };

            let mut results = Vec::new();
            let mut in_target_section = false;

            for section in sections {
                if section.level == 2 {
                    in_target_section = section.title.eq_ignore_ascii_case(section_name);
                } else if section.level == 3 && in_target_section {
                    if prefix.is_empty() || section.title.starts_with(&prefix) {
                        results.push(section.title);
                    }
                }
            }

            Ok(QueryResult::List(results))
        }
    }
}
