use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use pine_lang::builtins::{register_namespace_objects, DefaultPineOutput};
use pine_lang::core::PineVersion;
use pine_lang::diagnostics::{Diagnostic as PineDiagnostic, Severity};
use pine_lang::interpreter::{BuiltinSignature, Value};
use pine_lang::sema::{FileId, Symbol, SymbolId, SymbolKind, SymbolTable};
use pine_lang::DirLoader;
use tower_lsp_server::lsp_types::*;
use tower_lsp_server::{jsonrpc, Client, LanguageServer, LspService, Server, UriExt};

thread_local! {
    /// The builtin namespaces (`ta`, `math`, `array`, …) as the analyzer sees
    /// them, for `namespace.` member completion. `Value` is `!Send`, so it can't
    /// live on the shared `Backend`; each worker thread builds its own copy once.
    static BUILTINS: HashMap<String, Value<DefaultPineOutput>> =
        register_namespace_objects(PineVersion::LATEST, None, None).0;
}

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
        let (diagnostics, analyzed) = match analyze(&text, uri_dir(&uri)) {
            Ok(analysis) => (
                analysis
                    .diagnostics
                    .iter()
                    .map(|d| to_lsp(d, &text))
                    .collect(),
                Some(analysis.symbols),
            ),
            // A lex/parse/version error stops analysis; publish it as a single diagnostic.
            Err(err) => (vec![error_diagnostic(&err, &text)], None),
        };
        {
            let mut documents = self.documents.lock().unwrap();
            // A transient parse error yields no table (e.g. right after typing a
            // `.`); keep the last good one so completion and hover still answer.
            let symbols =
                analyzed.or_else(|| documents.get_mut(&uri).and_then(|d| d.symbols.take()));
            documents.insert(uri.clone(), Document { text, symbols });
        }
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
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
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
        let location = {
            let documents = self.documents.lock().unwrap();
            documents.get(&uri).and_then(|doc| {
                let symbols = doc.symbols.as_ref()?;
                let id = symbol_at(symbols, &doc.text, at.position)?;
                let (file, line, column) = symbols.declaration_location(id)?;
                let width = symbols.symbol(id).name.chars().count() as u32;
                Some(location_at(
                    file_uri(&uri, symbols, file)?,
                    line,
                    column,
                    width,
                ))
            })
        };
        Ok(location.map(GotoDefinitionResponse::Scalar))
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
                let mut sites: Vec<(FileId, u32, u32)> = Vec::new();
                if params.context.include_declaration {
                    if let Some(decl) = symbols.declaration_location(id) {
                        sites.push(decl);
                    }
                }
                sites.extend(symbols.references(id));
                Some(
                    sites
                        .into_iter()
                        .filter_map(|(file, line, column)| {
                            let uri = file_uri(&uri, symbols, file)?;
                            Some(location_at(uri, line, column, width))
                        })
                        .collect::<Vec<_>>(),
                )
            })
        };
        Ok(locations.filter(|l| !l.is_empty()))
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        let at = params.text_document_position;
        let items = {
            let documents = self.documents.lock().unwrap();
            documents.get(&at.text_document.uri).and_then(|doc| {
                let symbols = doc.symbols.as_ref()?;
                member_completions(symbols, &doc.text, at.position)
            })
        };
        Ok(items.map(CompletionResponse::Array))
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

/// The URI for a symbol-table `file`: the request document for the main file,
/// or the resolved library file otherwise. `None` when a library file can't be
/// located on disk (e.g. an in-memory loader).
fn file_uri(request: &Uri, symbols: &SymbolTable, file: FileId) -> Option<Uri> {
    if file == SymbolTable::MAIN {
        return Some(request.clone());
    }
    let dir = uri_dir(request)?;
    let path = DirLoader::new(vec![dir]).resolve_path(symbols.file_path(file))?;
    Uri::from_file_path(path)
}

/// A location spanning `width` characters from a 1-based `(line, column)`.
fn location_at(uri: Uri, line: u32, column: u32, width: u32) -> Location {
    let start = Position::new(line - 1, column - 1);
    let end = Position::new(line - 1, column - 1 + width);
    Location {
        uri,
        range: Range::new(start, end),
    }
}

/// Member completions after `receiver.` — a user object's fields/cases/exports,
/// or the members of a builtin namespace (`ta.`, `math.`, …).
fn member_completions(
    symbols: &SymbolTable,
    text: &str,
    position: Position,
) -> Option<Vec<CompletionItem>> {
    let line = text.lines().nth(position.line as usize).unwrap_or("");
    let prefix: String = line.chars().take(position.character as usize).collect();
    let receiver = receiver_before_dot(&prefix)?;

    let root = symbols.file_root(SymbolTable::MAIN);
    match symbols.resolve_id(root, receiver) {
        // A user declaration shadows a builtin namespace of the same name.
        Some(id) => user_member_completions(symbols, id),
        None => builtin_member_completions(receiver),
    }
}

/// The members of a user symbol: the fields/cases of a type (reached directly or
/// through a variable's declared type), or an import's exported symbols.
fn user_member_completions(symbols: &SymbolTable, id: SymbolId) -> Option<Vec<CompletionItem>> {
    let symbol = symbols.symbol(id);
    let members: Vec<&Symbol> = match symbol.kind {
        SymbolKind::Import => {
            let scope = symbol.module?;
            symbols.symbols_in(scope).filter(|s| s.exported).collect()
        }
        SymbolKind::Type | SymbolKind::Enum => symbols.members_of(id).collect(),
        SymbolKind::Var => symbols.members_of(symbol.type_ref?).collect(),
        SymbolKind::Function => return None,
    };
    let items: Vec<CompletionItem> = members.into_iter().map(completion_item).collect();
    (!items.is_empty()).then_some(items)
}

/// The members of a builtin namespace object (`ta.sma`, `math.abs`, …), read
/// straight from the registered builtins.
fn builtin_member_completions(namespace: &str) -> Option<Vec<CompletionItem>> {
    BUILTINS.with(|builtins| {
        let Some(Value::Object { fields, .. }) = builtins.get(namespace) else {
            return None;
        };
        let mut items: Vec<CompletionItem> = fields
            .borrow()
            .iter()
            .map(|(name, value)| builtin_completion_item(name, value))
            .collect();
        items.sort_by(|a, b| a.label.cmp(&b.label));
        (!items.is_empty()).then_some(items)
    })
}

/// A completion for one builtin member: a function (with its signature) when the
/// value is callable, a nested namespace, or otherwise a constant.
fn builtin_completion_item(name: &str, value: &Value<DefaultPineOutput>) -> CompletionItem {
    let (kind, detail) = match value {
        Value::BuiltinFunction(builtin) => (
            CompletionItemKind::FUNCTION,
            signature_detail(name, builtin.signature),
        ),
        Value::Object {
            call: Some(builtin),
            ..
        } => (
            CompletionItemKind::FUNCTION,
            signature_detail(name, builtin.signature),
        ),
        Value::Object { .. } => (CompletionItemKind::MODULE, None),
        _ => (CompletionItemKind::CONSTANT, None),
    };
    CompletionItem {
        label: name.to_string(),
        kind: Some(kind),
        detail,
        ..Default::default()
    }
}

/// `name(param, param, …)` for a builtin whose parameters are declared, else
/// `None` (an undeclared signature would falsely read as taking no arguments).
fn signature_detail(name: &str, signature: &BuiltinSignature) -> Option<String> {
    if signature.params.is_empty() {
        return None;
    }
    let params: Vec<&str> = signature.params.iter().map(|p| p.name.as_str()).collect();
    Some(format!("{name}({})", params.join(", ")))
}

/// The receiver identifier in `…receiver.partial` up to the cursor, if the text
/// before the cursor is a member access. Single hop only.
fn receiver_before_dot(prefix: &str) -> Option<&str> {
    let before_dot = prefix
        .trim_end_matches(|c: char| c.is_alphanumeric() || c == '_')
        .strip_suffix('.')?;
    let start = before_dot
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
        .map_or(0, |i| i + 1);
    let receiver = &before_dot[start..];
    (!receiver.is_empty()).then_some(receiver)
}

fn completion_item(symbol: &Symbol) -> CompletionItem {
    let kind = match symbol.kind {
        SymbolKind::Function => CompletionItemKind::FUNCTION,
        SymbolKind::Type => CompletionItemKind::STRUCT,
        SymbolKind::Enum => CompletionItemKind::ENUM,
        SymbolKind::Var => CompletionItemKind::FIELD,
        SymbolKind::Import => CompletionItemKind::MODULE,
    };
    let detail = match symbol.kind {
        SymbolKind::Function => Some(format!("{}({})", symbol.name, symbol.params.join(", "))),
        _ => symbol.type_annotation.clone(),
    };
    CompletionItem {
        label: symbol.name.clone(),
        kind: Some(kind),
        detail,
        ..Default::default()
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

fn error_diagnostic(err: &pine_lang::Error, text: &str) -> Diagnostic {
    let range = match err.location() {
        Some((line, col)) => token_range(text, line, col),
        None => Range::default(),
    };
    Diagnostic {
        range,
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
    fn parse_error_points_at_its_line() {
        let source = "//@version=6\nindicator(\"x\")\nlog.info(str.tostring. (up))\n";
        let Err(err) = pine_lang::analyze(source, None) else {
            panic!("expected a parse error");
        };
        let diag = error_diagnostic(&err, source);
        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        // Points at the offending `up` token, not the top of the file.
        assert_eq!(diag.range.start, Position::new(2, 24));
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

    #[test]
    fn completes_builtin_namespace_members() {
        let ta = builtin_member_completions("ta").expect("ta namespace");
        assert!(ta.iter().any(|i| i.label == "sma"), "expected ta.sma");

        let math = builtin_member_completions("math").expect("math namespace");
        assert!(math.iter().any(|i| i.label == "abs"), "expected math.abs");

        // A name that is not a builtin namespace yields nothing.
        assert!(builtin_member_completions("definitely_not_a_namespace").is_none());
    }
}
