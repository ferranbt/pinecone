use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use pine_lang::diagnostics::{Diagnostic as PineDiagnostic, Severity};
use pine_lang::sema::{Symbol, SymbolId, SymbolKind, SymbolTable};
use tower_lsp_server::lsp_types::*;
use tower_lsp_server::{jsonrpc, Client, LanguageServer, LspService, Server, UriExt};

/// Serve the language server over stdio until the client disconnects.
#[tokio::main]
pub async fn run() {
    let (service, socket) = LspService::new(Backend::new);
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}

/// An open document and the analysis last computed for it.
struct Document {
    text: String,
    /// The symbol table, absent while the document does not parse.
    symbols: Option<SymbolTable>,
}

struct Backend {
    client: Client,
    documents: Mutex<HashMap<Uri, Document>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Mutex::new(HashMap::new()),
        }
    }

    /// Re-analyze `text`, cache the result for `uri`, and publish its diagnostics.
    async fn update(&self, uri: Uri, text: String) {
        let (diagnostics, symbols) = match analyze(&text, uri_dir(&uri)) {
            Ok(analysis) => (
                analysis
                    .diagnostics
                    .iter()
                    .map(|d| to_lsp(d, &text))
                    .collect(),
                Some(analysis.symbols),
            ),
            // A lex/parse/version error stops analysis before any position is known.
            Err(err) => (vec![error_diagnostic(&err)], None),
        };
        self.documents
            .lock()
            .unwrap()
            .insert(uri.clone(), Document { text, symbols });
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "pine-lsp".to_string(),
                version: None,
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "pine-lsp ready")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        self.update(doc.uri, doc.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // Full sync: the final change carries the whole document.
        if let Some(change) = params.content_changes.into_iter().next_back() {
            self.update(params.text_document.uri, change.text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = self
            .documents
            .lock()
            .unwrap()
            .get(&uri)
            .map(|d| d.text.clone());
        if let Some(text) = text {
            self.update(uri, text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.lock().unwrap().remove(&uri);
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn formatting(
        &self,
        params: DocumentFormattingParams,
    ) -> jsonrpc::Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let Some(text) = self
            .documents
            .lock()
            .unwrap()
            .get(&uri)
            .map(|d| d.text.clone())
        else {
            return Ok(None);
        };
        match pine_lang::format::format(&text) {
            Ok(formatted) if formatted != text => Ok(Some(vec![TextEdit {
                range: Range::new(Position::new(0, 0), end_position(&text)),
                new_text: formatted,
            }])),
            _ => Ok(None),
        }
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let at = params.text_document_position_params;
        let markdown = {
            let documents = self.documents.lock().unwrap();
            documents.get(&at.text_document.uri).and_then(|doc| {
                let symbols = doc.symbols.as_ref()?;
                let id = symbol_at(symbols, &doc.text, at.position)?;
                Some(hover_markdown(symbols, id, &at.text_document.uri))
            })
        };
        Ok(markdown.map(|value| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let at = params.text_document_position_params;
        let uri = at.text_document.uri;
        let decl = {
            let documents = self.documents.lock().unwrap();
            documents.get(&uri).and_then(|doc| {
                let symbols = doc.symbols.as_ref()?;
                let id = symbol_at(symbols, &doc.text, at.position)?;
                let (file, line, column) = symbols.declaration_location(id)?;
                // Cross-file (library) declarations are not resolved yet.
                (file == SymbolTable::MAIN).then(|| Position::new(line - 1, column - 1))
            })
        };
        Ok(decl.map(|decl| {
            GotoDefinitionResponse::Scalar(Location {
                uri,
                range: Range::new(decl, decl),
            })
        }))
    }

    async fn references(&self, params: ReferenceParams) -> jsonrpc::Result<Option<Vec<Location>>> {
        let at = params.text_document_position;
        let uri = at.text_document.uri;
        let locations = {
            let documents = self.documents.lock().unwrap();
            documents.get(&uri).and_then(|doc| {
                let symbols = doc.symbols.as_ref()?;
                let id = symbol_at(symbols, &doc.text, at.position)?;
                let width = symbols.symbol(id).name.chars().count() as u32;
                let mut sites: Vec<(u32, u32)> = Vec::new();
                if params.context.include_declaration {
                    if let Some((file, line, column)) = symbols.declaration_location(id) {
                        if file == SymbolTable::MAIN {
                            sites.push((line, column));
                        }
                    }
                }
                for (file, line, column) in symbols.references(id) {
                    if file == SymbolTable::MAIN {
                        sites.push((line, column));
                    }
                }
                Some(
                    sites
                        .into_iter()
                        .map(|(line, column)| main_location(&uri, line, column, width))
                        .collect::<Vec<_>>(),
                )
            })
        };
        Ok(locations.filter(|l| !l.is_empty()))
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }
}

/// Analyze `text` (diagnostics + symbol table). `dir` roots `import` resolution.
fn analyze(text: &str, dir: Option<PathBuf>) -> Result<pine_lang::Analysis, pine_lang::Error> {
    let loader = dir.map(|dir| pine_lang::DirLoader::new(vec![dir]));
    let loader = loader.as_ref().map(|l| l as &dyn pine_lang::LibraryLoader);
    pine_lang::analyze(text, loader)
}

fn symbol_at(
    symbols: &SymbolTable,
    text: &str,
    position: Position,
) -> Option<pine_lang::sema::SymbolId> {
    let line = text.lines().nth(position.line as usize).unwrap_or("");
    let start = identifier_start(line, position.character as usize);
    symbols.symbol_at(SymbolTable::MAIN, position.line + 1, start as u32 + 1)
}

/// A location in the main document spanning `width` characters from a 1-based
/// `(line, column)`.
fn main_location(uri: &Uri, line: u32, column: u32, width: u32) -> Location {
    let start = Position::new(line - 1, column - 1);
    let end = Position::new(line - 1, column - 1 + width);
    Location {
        uri: uri.clone(),
        range: Range::new(start, end),
    }
}

/// The start column of the identifier the cursor sits in or just after.
fn identifier_start(line: &str, column: usize) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let mut start = column.min(chars.len());
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }
    start
}

fn render_symbol(symbol: &Symbol) -> String {
    let signature = match symbol.kind {
        SymbolKind::Function => format!("{}({})", symbol.name, symbol.params.join(", ")),
        _ => match &symbol.type_annotation {
            Some(ty) => format!("{}: {ty}", symbol.name),
            None => symbol.name.clone(),
        },
    };
    format!("```pine\n{signature}\n```\n\n*{}*", symbol.kind.noun())
}

/// Hover text: the symbol's signature, plus — for a function — where it is
/// defined and every place it is called. Positions in this file are rendered as
/// links so the reader can jump to them.
fn hover_markdown(symbols: &SymbolTable, id: SymbolId, uri: &Uri) -> String {
    let symbol = symbols.symbol(id);
    let mut md = render_symbol(symbol);
    if symbol.kind != SymbolKind::Function {
        return md;
    }

    let here = uri.as_str();
    let link = |line: u32, column: u32| format!("[{line}:{column}]({here}#L{line},{column})");

    match symbols.declaration_location(id) {
        Some((file, line, column)) if file == SymbolTable::MAIN => {
            md.push_str(&format!("\n\nDefined at {}", link(line, column)));
        }
        Some((file, _, _)) => {
            md.push_str(&format!("\n\nDefined in `{}`", symbols.file_path(file)));
        }
        None => {}
    }

    let calls: Vec<String> = symbols
        .references(id)
        .filter(|(file, _, _)| *file == SymbolTable::MAIN)
        .map(|(_, line, column)| link(line, column))
        .collect();
    match calls.len() {
        0 => md.push_str("\n\nNo calls in this file."),
        1 => md.push_str(&format!("\n\n**1 call:** {}", calls[0])),
        n => md.push_str(&format!("\n\n**{n} calls:** {}", calls.join(", "))),
    }
    md
}

fn to_lsp(diagnostic: &PineDiagnostic, text: &str) -> Diagnostic {
    let range = diagnostic
        .pos
        .map(|(line, col)| token_range(text, line, col))
        .unwrap_or_default();
    Diagnostic {
        range,
        severity: Some(match diagnostic.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        }),
        source: Some("pinecone".to_string()),
        code: Some(NumberOrString::String(diagnostic.rule.to_string())),
        message: diagnostic.message.clone(),
        ..Default::default()
    }
}

fn error_diagnostic(err: &pine_lang::Error) -> Diagnostic {
    Diagnostic {
        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("pinecone".to_string()),
        message: err.to_string(),
        ..Default::default()
    }
}

/// The range of the token at a 1-based `(line, col)` — the run of non-whitespace
/// starting there, or one column when there is nothing to underline.
fn token_range(text: &str, line: u32, col: u32) -> Range {
    let line0 = line.saturating_sub(1);
    let col0 = col.saturating_sub(1);
    let chars: Vec<char> = text
        .lines()
        .nth(line0 as usize)
        .unwrap_or("")
        .chars()
        .collect();
    let start = col0 as usize;
    let mut end = start;
    while end < chars.len() && !chars[end].is_whitespace() {
        end += 1;
    }
    let end = end.max(start + 1) as u32;
    Range::new(Position::new(line0, col0), Position::new(line0, end))
}

/// The position just past the last character — the end of a full-document range.
fn end_position(text: &str) -> Position {
    let mut line = 0u32;
    let mut character = 0u32;
    for ch in text.chars() {
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }
    Position::new(line, character)
}

fn uri_dir(uri: &Uri) -> Option<PathBuf> {
    uri.to_file_path()?.parent().map(|p| p.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_warning_becomes_a_diagnostic() {
        let src = "//@version=6\nindicator(\"x\")\nx = request.security(syminfo.tickerid, \"D\", close)\nplot(x)\n";
        let analysis = pine_lang::analyze(src, None).unwrap();
        let diags: Vec<_> = analysis
            .diagnostics
            .iter()
            .map(|d| to_lsp(d, src))
            .collect();
        let repaint = diags
            .iter()
            .find(|d| d.code == Some(NumberOrString::String("security-repaint".into())))
            .expect("repaint diagnostic");
        assert_eq!(repaint.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(repaint.range.start.line, 2);
    }

    #[test]
    fn parse_error_becomes_a_single_error() {
        let Err(err) = pine_lang::analyze("indicator(\n", None) else {
            panic!("expected a parse error");
        };
        assert_eq!(
            error_diagnostic(&err).severity,
            Some(DiagnosticSeverity::ERROR)
        );
    }

    #[test]
    fn token_range_underlines_the_word() {
        let range = token_range("x = close\n", 1, 5);
        assert_eq!(range.start, Position::new(0, 4));
        assert_eq!(range.end, Position::new(0, 9));
    }

    #[test]
    fn end_position_is_past_the_last_char() {
        assert_eq!(end_position("ab\ncd"), Position::new(1, 2));
    }
}
