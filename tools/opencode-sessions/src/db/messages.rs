//! Reads conversation messages from the v2 `session_message` table.

use crate::models::*;
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde_json::{Value, json};

/// Loads one session's conversation from the v2 `session_message` table.
///
/// Only `user` and `assistant` rows become messages; every other kind is a
/// control row (idle, compaction, model switches, ...). Orders rows by `seq`
/// and normalizes them into the `LoadedMessage` shape the export pipeline
/// consumes.
pub(crate) fn load_messages(conn: &Connection, session_id: &str) -> Result<Vec<LoadedMessage>> {
    let mut stmt = conn.prepare(
        r#"
        select id, type, seq, time_created, data
        from session_message
        where session_id = ?1
        order by seq asc
        "#,
    )?;
    let mut rows = stmt.query(params![session_id])?;

    let mut messages = Vec::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let kind: String = row.get(1)?;
        let row_time_created: i64 = row.get(3)?;
        let raw_json: String = row.get(4)?;
        let data: Value = serde_json::from_str(&raw_json)
            .with_context(|| format!("parse session_message json for {id}"))?;
        if let Some(message) = map_session_message(&id, &kind, row_time_created, &data) {
            messages.push(message);
        }
    }

    Ok(messages)
}

/// Converts one v2 row into a loaded message, or `None` for control rows.
fn map_session_message(
    id: &str,
    kind: &str,
    row_time_created: i64,
    data: &Value,
) -> Option<LoadedMessage> {
    let parts = match kind {
        "user" => user_parts(data),
        "assistant" => assistant_parts(data),
        // Control rows carry no conversation turns.
        _ => return None,
    };

    let mut info = MessageInfo {
        role: kind.to_string(),
        ..MessageInfo::default()
    };
    info.time = MessageTime {
        created: nested_ms(data, &["time", "created"]),
        completed: nested_ms(data, &["time", "completed"]),
    };

    if kind == "assistant" {
        info.agent = data.get("agent").and_then(Value::as_str).map(str::to_string);
        info.finish = data.get("finish").and_then(Value::as_str).map(str::to_string);
        info.cost = data.get("cost").and_then(Value::as_f64);
        // Aborted or provider-failed steps carry a top-level error object;
        // `infer_session_status` reads it to mark sessions as errored.
        info.error = data.get("error").filter(|error| !error.is_null()).cloned();
        info.tokens = data
            .get("tokens")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok());
        info.model = data.get("model").map(model_ref);
    }

    Some(LoadedMessage {
        id: id.to_string(),
        time_created: row_time_created,
        info,
        parts,
    })
}

/// Normalizes assistant `content` items into the part shapes the pipeline
/// classifier understands and drops unknown item types.
fn assistant_parts(data: &Value) -> Vec<LoadedPart> {
    data.get("content")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| match item.get("type").and_then(Value::as_str) {
                    Some("text") => assistant_text_part(item),
                    Some("reasoning") => reasoning_part(item),
                    Some("tool") => Some(tool_part(item)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Maps the v2 model ref (`{id, providerID, variant}`) onto `ModelRef`.
fn model_ref(model: &Value) -> ModelRef {
    ModelRef {
        provider_id: model
            .get("providerID")
            .and_then(Value::as_str)
            .map(str::to_string),
        model_id: model.get("id").and_then(Value::as_str).map(str::to_string),
        variant: model
            .get("variant")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn nested_ms(value: &Value, path: &[&str]) -> Option<i64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_i64()
}

/// Builds the text part for a user row.
fn user_parts(data: &Value) -> Vec<LoadedPart> {
    let text = data.get("text").and_then(Value::as_str).unwrap_or_default();
    if text.trim().is_empty() {
        return Vec::new();
    }
    vec![text_part(text)]
}

fn assistant_text_part(item: &Value) -> Option<LoadedPart> {
    let text = item.get("text").and_then(Value::as_str)?.trim();
    if text.is_empty() {
        return None;
    }
    Some(text_part(text))
}

fn reasoning_part(item: &Value) -> Option<LoadedPart> {
    let text = item.get("text").and_then(Value::as_str)?.trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some(LoadedPart {
        raw: json!({ "type": "reasoning", "text": text }),
    })
}

/// Maps a v2 tool call (`name` + `state.content` output list) onto the legacy
/// part shape (`tool` + `state.output` string) the classifier reads.
fn tool_part(item: &Value) -> LoadedPart {
    let name = item
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let state = item.get("state").cloned().unwrap_or(Value::Null);

    let mut state_out = json!({
        "status": state.get("status").cloned().unwrap_or(Value::Null),
        "input": state.get("input").cloned().unwrap_or(Value::Null),
        "time": {
            "start": nested_ms(item, &["time", "created"]),
            "end": nested_ms(item, &["time", "completed"]),
        },
    });
    if let Some(output) = tool_output_text(&state) {
        state_out["output"] = Value::String(output);
    }
    if let Some(error) = tool_error_text(&state) {
        state_out["error"] = Value::String(error);
    }

    LoadedPart {
        raw: json!({ "type": "tool", "tool": name, "state": state_out }),
    }
}

/// Builds the shared text part shape from any source text.
fn text_part(text: &str) -> LoadedPart {
    LoadedPart {
        raw: json!({ "type": "text", "text": text, "synthetic": false }),
    }
}

/// Extracts a readable message from a tool error, which v2 stores as an
/// object (`{type, message}`) or plain string.
fn tool_error_text(state: &Value) -> Option<String> {
    let error = state.get("error")?;
    if let Some(message) = error.get("message").and_then(Value::as_str) {
        return Some(message.to_string());
    }
    error.as_str().map(str::to_string)
}

/// Joins the tool's `state.content` text items into one output string.
fn tool_output_text(state: &Value) -> Option<String> {
    let texts: Vec<&str> = state
        .get("content")?
        .as_array()?
        .iter()
        .filter(|content| content.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|content| content.get("text").and_then(Value::as_str))
        .collect();
    if texts.is_empty() {
        return None;
    }
    Some(texts.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Shared fixture base instant (ms); other timestamps offset from it.
    const BASE_MS: i64 = 1771102727767;
    /// Assistant completion offset: 12 554 ms after `BASE_MS`.
    const COMPLETED_OFFSET_MS: i64 = 12554;
    /// Tool window offsets: start and end around `BASE_MS`.
    const TOOL_START_OFFSET_MS: i64 = 233;
    const TOOL_END_OFFSET_MS: i64 = 1233;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            create table session_message (
              id text primary key,
              session_id text not null,
              type text not null,
              seq integer not null,
              time_created integer not null,
              time_updated integer,
              data text not null
            );
            "#,
        )
        .unwrap();
        conn
    }

    fn insert_message(conn: &Connection, id: &str, kind: &str, seq: i64, data: &str) {
        conn.execute(
            "insert into session_message (id, session_id, type, seq, time_created, data)
             values (?1, 'ses_test', ?2, ?3, ?4, ?5)",
            params![id, kind, seq, BASE_MS, data],
        )
        .unwrap();
    }

    /// Inserts one assistant/user row and returns the loaded messages.
    fn load_single(kind: &str, data: &str) -> Vec<LoadedMessage> {
        let conn = test_conn();
        insert_message(&conn, "msg_a", kind, 1, data);
        load_messages(&conn, "ses_test").unwrap()
    }

    fn user_data(text: &str) -> String {
        json!({ "text": text, "time": { "created": BASE_MS } }).to_string()
    }

    fn assistant_data(content: Vec<Value>) -> String {
        json!({
            "agent": "build",
            "model": { "id": "kimi", "providerID": "moonshot", "variant": "dev" },
            "finish": "stop",
            "cost": 0.25,
            "tokens": {
                "input": 1200, "output": 300, "reasoning": 50,
                "cache": { "read": 900, "write": 10 }
            },
            "time": { "created": BASE_MS, "completed": BASE_MS + COMPLETED_OFFSET_MS },
            "content": content,
        })
        .to_string()
    }

    fn tool_item(status: &str, output_texts: &[&str], error: Option<Value>) -> Value {
        let content: Vec<Value> = output_texts
            .iter()
            .map(|text| json!({ "type": "text", "text": text }))
            .collect();
        let mut state = json!({
            "status": status,
            "input": { "path": "/tmp/a.rs" },
            "content": content,
        });
        if let Some(error) = error {
            state["error"] = error;
        }
        json!({
            "type": "tool", "id": "call_1", "name": "read", "executed": true,
            "state": state,
            "time": {
                "created": BASE_MS + TOOL_START_OFFSET_MS,
                "completed": BASE_MS + TOOL_END_OFFSET_MS,
                "ran": true,
            },
        })
    }

    fn part_type(part: &LoadedPart) -> &str {
        part.raw.get("type").and_then(Value::as_str).unwrap_or_default()
    }

    // Construction: rows arrive in seq order regardless of insert order.

    #[test]
    fn load_messages_should_preserve_seq_order_when_rows_are_interleaved() {
        let conn = test_conn();
        insert_message(&conn, "msg_b", "assistant", 2, &assistant_data(vec![]));
        insert_message(&conn, "msg_a", "user", 1, &user_data("hello"));

        let messages = load_messages(&conn, "ses_test").unwrap();

        assert_eq!(
            messages.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            vec!["msg_a", "msg_b"]
        );
    }

    // Core behavior: user and assistant rows map onto the pipeline model.

    #[test]
    fn load_messages_should_add_text_part_when_user_row_has_text() {
        let conn = test_conn();
        insert_message(&conn, "msg_a", "user", 1, &user_data("  hi there  "));

        let messages = load_messages(&conn, "ses_test").unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].info.role, "user");
        assert_eq!(messages[0].parts.len(), 1);
        assert_eq!(
            messages[0].parts[0].raw.get("text").and_then(Value::as_str),
            Some("  hi there  ")
        );
    }

    #[test]
    fn load_messages_should_map_runtime_fields_when_assistant_has_them() {
        let messages = load_single(
            "assistant",
            &assistant_data(vec![json!({ "type": "text", "text": "done" })]),
        );
        let info = &messages[0].info;

        assert_eq!(info.role, "assistant");
        assert_eq!(info.agent.as_deref(), Some("build"));
        assert_eq!(info.finish.as_deref(), Some("stop"));
        assert_eq!(info.cost, Some(0.25));
        assert_eq!(
            info.model.as_ref().and_then(|m| m.model_id.as_deref()),
            Some("kimi")
        );
        assert_eq!(
            info.model.as_ref().and_then(|m| m.provider_id.as_deref()),
            Some("moonshot")
        );
        let tokens = info.tokens.as_ref().unwrap();
        assert_eq!(tokens.input, 1200);
        assert_eq!(tokens.output, 300);
        assert_eq!(tokens.reasoning, 50);
        assert_eq!(tokens.cache.read, 900);
        assert_eq!(tokens.cache.write, 10);
        assert_eq!(info.time.completed, Some(BASE_MS + COMPLETED_OFFSET_MS));
    }

    #[test]
    fn load_messages_should_normalize_tool_part_when_state_has_content_texts() {
        let messages = load_single(
            "assistant",
            &assistant_data(vec![tool_item("completed", &["line one", "line two"], None)]),
        );
        let tool = &messages[0]
            .parts
            .iter()
            .find(|p| part_type(p) == "tool")
            .unwrap()
            .raw;
        let state = tool.get("state").unwrap();

        assert_eq!(tool.get("tool").and_then(Value::as_str), Some("read"));
        assert_eq!(state.get("status").and_then(Value::as_str), Some("completed"));
        assert_eq!(
            state.get("output").and_then(Value::as_str),
            Some("line one\nline two")
        );
        assert_eq!(
            state.get("time").unwrap().get("start").and_then(Value::as_i64),
            Some(BASE_MS + TOOL_START_OFFSET_MS)
        );
        assert_eq!(
            state.get("time").unwrap().get("end").and_then(Value::as_i64),
            Some(BASE_MS + TOOL_END_OFFSET_MS)
        );
    }

    #[test]
    fn load_messages_should_keep_error_when_assistant_row_was_aborted() {
        let data = json!({
            "error": { "type": "provider.error", "message": "Bad Request" },
            "time": { "created": BASE_MS },
            "content": [],
        })
        .to_string();

        let messages = load_single("assistant", &data);

        let error = messages[0].info.error.as_ref().unwrap();
        assert_eq!(error.get("type").and_then(Value::as_str), Some("provider.error"));
    }

    // Edge cases: control rows, running tools, error objects, empty text.

    #[rstest::rstest]
    #[case::idle("idle")]
    #[case::model_switched("model-switched")]
    #[case::agent_switched("agent-switched")]
    #[case::compaction("compaction")]
    #[case::system("system")]
    #[case::synthetic("synthetic")]
    fn load_messages_should_skip_row_when_kind_is_control(#[case] kind: &str) {
        let conn = test_conn();
        insert_message(
            &conn,
            "msg_ctl",
            kind,
            1,
            &json!({ "time": { "created": BASE_MS } }).to_string(),
        );

        assert!(load_messages(&conn, "ses_test").unwrap().is_empty());
    }

    #[test]
    fn load_messages_should_omit_output_when_tool_is_running() {
        let messages = load_single(
            "assistant",
            &assistant_data(vec![tool_item("running", &[], None)]),
        );
        let state = &messages[0].parts[0].raw.get("state").unwrap();

        assert_eq!(state.get("status").and_then(Value::as_str), Some("running"));
        assert!(state.get("output").is_none());
    }

    #[test]
    fn load_messages_should_extract_error_message_when_state_error_is_object() {
        let messages = load_single(
            "assistant",
            &assistant_data(vec![tool_item(
                "error",
                &[],
                Some(json!({ "type": "unknown", "message": "boom" })),
            )]),
        );
        let state = &messages[0].parts[0].raw.get("state").unwrap();

        assert_eq!(state.get("error").and_then(Value::as_str), Some("boom"));
    }

    #[test]
    fn load_messages_should_keep_empty_text_parts_out_when_content_is_blank() {
        let messages = load_single(
            "assistant",
            &assistant_data(vec![
                json!({ "type": "text", "text": "   " }),
                json!({ "type": "unknown-kind", "text": "x" }),
            ]),
        );

        assert!(messages[0].parts.is_empty());
    }
}
