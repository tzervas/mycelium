//! The **LSP wire protocol** (M-310; FR-S5; SC-5): JSON-RPC 2.0 framing over stdio plus the
//! mapping of the [`Feedback`](crate::feedback::Feedback) surface into LSP-shaped messages — the
//! "mechanical wrapping" the facade doc (M-140) flagged as the later step.
//!
//! What this layer **is**: the byte-level [`read_message`]/[`write_message`] codec (the
//! `Content-Length` header framing every LSP transport uses), the
//! [`Diagnostic`](crate::lint::Diagnostic) → LSP-`Diagnostic` mapping with the proper
//! `DiagnosticSeverity` codes, the `textDocument/publishDiagnostics` notification builder, and a
//! minimal [`serve`] lifecycle loop (`initialize` → capabilities, `shutdown`/`exit`).
//!
//! What it deliberately is **not** (honest scope, VR-5): a document-syncing server. The facade
//! analyzes **Core IR nodes**, not source text — there is no text → `Node` path yet, so the server
//! advertises `TextDocumentSyncKind.None` and the diagnostic `range` is a **zero placeholder** with
//! the navigable location carried as the structured breadcrumb in `data.breadcrumb`. Real source
//! spans (and `didOpen`/`didChange` document sync) arrive with the L1 surface (M-320); this layer is
//! ready to carry them without a protocol change.

use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use crate::feedback::Feedback;
use crate::lint::{Diagnostic, Severity};

/// The advertised server name (LSP `serverInfo.name`).
pub const SERVER_NAME: &str = "mycelium-lsp";

/// LSP `DiagnosticSeverity` code for a [`Severity`] (LSP spec: Error=1, Warning=2, Information=3,
/// Hint=4). The lint lattice only has Error/Warning, mapped to 1/2.
#[must_use]
pub fn lsp_severity(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 1,
        Severity::Warning => 2,
    }
}

/// Map a [`Diagnostic`] to an LSP-`Diagnostic` JSON value. The `range` is a **zero placeholder**
/// (L0 Core IR has no source spans yet) and the navigable location is the structured breadcrumb in
/// `data.breadcrumb` — never a fabricated line/column (M-310; spans arrive with the L1 surface).
#[must_use]
pub fn to_lsp_diagnostic(diag: &Diagnostic) -> Value {
    json!({
        "range": {
            "start": { "line": 0, "character": 0 },
            "end": { "line": 0, "character": 0 },
        },
        "severity": lsp_severity(diag.severity),
        "code": diag.code,
        "source": SERVER_NAME,
        "message": diag.message,
        // The breadcrumb path the client navigates by until real spans exist (M-310).
        "data": { "breadcrumb": diag.path() },
    })
}

/// The `params` of a `textDocument/publishDiagnostics` notification for `feedback` at `uri`.
#[must_use]
pub fn publish_diagnostics_params(uri: &str, feedback: &Feedback) -> Value {
    json!({
        "uri": uri,
        "diagnostics": feedback
            .diagnostics
            .iter()
            .map(to_lsp_diagnostic)
            .collect::<Vec<_>>(),
    })
}

/// Build the full `textDocument/publishDiagnostics` JSON-RPC **notification** (server → client) that
/// reports `feedback`'s diagnostics for the document `uri`. This is the LSP wrapping of the M-140
/// diagnostics channel; the (future) document-analysis path emits it after each [`crate::analyze`].
#[must_use]
pub fn publish_diagnostics_notification(uri: &str, feedback: &Feedback) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": publish_diagnostics_params(uri, feedback),
    })
}

/// The `initialize` result: the server's advertised capabilities. Honestly minimal —
/// `textDocumentSync: 0` (`TextDocumentSyncKind.None`) because there is no text → `Node` path yet
/// (the facade analyzes Core IR nodes); diagnostics are pushed via [`publish_diagnostics_notification`].
#[must_use]
pub fn initialize_result() -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": 0,
        },
        "serverInfo": { "name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
    })
}

/// Read one JSON-RPC message off `reader`, decoding the `Content-Length` header framing. Returns
/// `Ok(None)` at a **clean** EOF (no partial header), and an `io::Error` for a malformed frame
/// (truncated body, missing/invalid `Content-Length`, or non-JSON body) — never a silent drop.
pub fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            // EOF: clean only if we were between messages (no header seen yet).
            return if content_length.is_some() {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "EOF inside LSP message headers",
                ))
            } else {
                Ok(None)
            };
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break; // blank line terminates the headers
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
            let n = rest.trim().parse::<usize>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length")
            })?;
            content_length = Some(n);
        }
        // Any other header (e.g. Content-Type) is ignored — LSP defines only these two.
    }
    let len = content_length.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length header")
    })?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    let value =
        serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(value))
}

/// Write one JSON-RPC message to `writer` with the `Content-Length` framing, then flush.
pub fn write_message<W: Write>(writer: &mut W, msg: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(msg)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

fn response(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Drive the minimal LSP lifecycle over `reader`/`writer` (stdio in the real server): answer
/// `initialize` with [`initialize_result`], acknowledge `shutdown` with a null result, stop on
/// `exit`, and reply to any other **request** (a message carrying an `id`) with JSON-RPC
/// `MethodNotFound` (-32601) — never silently. Unknown **notifications** (no `id`) are ignored, as
/// the protocol requires. Returns when the stream ends or `exit` is received.
///
/// This is the handshake skeleton: it does not yet synchronize documents (no text → `Node` path —
/// see the module note), so it does not, on its own, emit diagnostics. The
/// [`publish_diagnostics_notification`] builder is the channel the document path will push through
/// once the L1 surface lands.
pub fn serve<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    while let Some(msg) = read_message(reader)? {
        let method = msg
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let id = msg.get("id").cloned();
        match (method, id) {
            ("initialize", Some(id)) => write_message(writer, &response(id, initialize_result()))?,
            ("shutdown", Some(id)) => write_message(writer, &response(id, Value::Null))?,
            ("exit", _) => break,
            // Any other request must get a response (never a silent hang); -32601 = MethodNotFound.
            (other, Some(id)) => write_message(
                writer,
                &error_response(id, -32601, &format!("method not handled: {other}")),
            )?,
            // Unknown notification (no id, e.g. `initialized`): nothing to answer.
            (_, None) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn framing_round_trips_one_and_many_messages() {
        let a = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" });
        let b = json!({ "jsonrpc": "2.0", "method": "exit" });
        let mut buf = Vec::new();
        write_message(&mut buf, &a).unwrap();
        write_message(&mut buf, &b).unwrap();
        // The frame is the documented header + body shape.
        let text = String::from_utf8(buf.clone()).unwrap();
        assert!(text.starts_with("Content-Length: "));
        assert!(text.contains("\r\n\r\n"));

        let mut cur = Cursor::new(buf);
        assert_eq!(read_message(&mut cur).unwrap(), Some(a));
        assert_eq!(read_message(&mut cur).unwrap(), Some(b));
        // Clean EOF after the last message.
        assert_eq!(read_message(&mut cur).unwrap(), None);
    }

    #[test]
    fn empty_stream_is_clean_eof_not_an_error() {
        let mut cur = Cursor::new(Vec::new());
        assert_eq!(read_message(&mut cur).unwrap(), None);
    }

    #[test]
    fn truncated_body_is_an_explicit_error() {
        // Mutant-witness: a header promising more bytes than the body holds must error, never return
        // a partial/silent message.
        let framed = b"Content-Length: 50\r\n\r\n{\"jsonrpc\":\"2.0\"}".to_vec();
        let mut cur = Cursor::new(framed);
        assert!(read_message(&mut cur).is_err());
    }

    #[test]
    fn severity_maps_to_lsp_codes() {
        assert_eq!(lsp_severity(Severity::Error), 1);
        assert_eq!(lsp_severity(Severity::Warning), 2);
    }

    #[test]
    fn publish_diagnostics_has_the_lsp_shape() {
        let feedback = Feedback {
            diagnostics: vec![Diagnostic {
                code: "implicit-swap",
                severity: Severity::Error,
                at: "let a/swap".to_string(),
                message: "a swap must be explicit".to_string(),
            }],
            guarantees: Vec::new(),
            swaps: Vec::new(),
            stages: Vec::new(),
            explanations: Vec::new(),
        };
        let note = publish_diagnostics_notification("mem://demo", &feedback);
        assert_eq!(note["method"], "textDocument/publishDiagnostics");
        assert_eq!(note["params"]["uri"], "mem://demo");
        let d = &note["params"]["diagnostics"][0];
        assert_eq!(d["severity"], 1); // Error
        assert_eq!(d["code"], "implicit-swap");
        assert_eq!(d["source"], SERVER_NAME);
        // Honest scope: zero range placeholder, breadcrumb carries the navigable location.
        assert_eq!(d["range"]["start"]["line"], 0);
        assert_eq!(d["data"]["breadcrumb"], json!(["let a", "swap"]));
    }

    #[test]
    fn serve_answers_initialize_and_shutdown_then_exits() {
        // Scripted client: initialize → shutdown → exit. The loop must answer the two requests and
        // stop on exit (mutant-witness: dropping the `exit` arm would block on read past EOF).
        let mut input = Vec::new();
        write_message(
            &mut input,
            &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
        )
        .unwrap();
        write_message(
            &mut input,
            &json!({ "jsonrpc": "2.0", "id": 2, "method": "shutdown" }),
        )
        .unwrap();
        write_message(&mut input, &json!({ "jsonrpc": "2.0", "method": "exit" })).unwrap();

        let mut reader = Cursor::new(input);
        let mut out = Vec::new();
        serve(&mut reader, &mut out).unwrap();

        let mut rout = Cursor::new(out);
        let init = read_message(&mut rout).unwrap().unwrap();
        assert_eq!(init["id"], 1);
        assert_eq!(init["result"]["serverInfo"]["name"], SERVER_NAME);
        assert_eq!(init["result"]["capabilities"]["textDocumentSync"], 0);
        let shut = read_message(&mut rout).unwrap().unwrap();
        assert_eq!(shut["id"], 2);
        assert_eq!(shut["result"], Value::Null);
        // Nothing after the shutdown response (exit produced no message).
        assert_eq!(read_message(&mut rout).unwrap(), None);
    }

    #[test]
    fn unknown_request_gets_method_not_found_not_silence() {
        let mut input = Vec::new();
        write_message(
            &mut input,
            &json!({ "jsonrpc": "2.0", "id": 7, "method": "textDocument/hover" }),
        )
        .unwrap();
        write_message(&mut input, &json!({ "jsonrpc": "2.0", "method": "exit" })).unwrap();
        let mut reader = Cursor::new(input);
        let mut out = Vec::new();
        serve(&mut reader, &mut out).unwrap();
        let mut rout = Cursor::new(out);
        let resp = read_message(&mut rout).unwrap().unwrap();
        assert_eq!(resp["id"], 7);
        assert_eq!(resp["error"]["code"], -32601);
    }
}
