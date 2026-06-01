// Amber — the agent runtime (the spine, per knowledge/wiki/north-star.md).
//
// Converts a turn from single-shot chat into a model-driven tool-use loop: the model
// plans → calls a tool → sees the result → iterates → answers. Vault retrieval and web
// search are no longer forced; they're TOOLS the model chooses when it judges it needs
// them. Build this loop once and every future capability (vault write-back, rituals,
// browser, proactivity) is just another tool hung off it.

use serde_json::{json, Value};
use tauri::ipc::Channel;
use tauri::AppHandle;

use crate::router::Mode;
use crate::{ChatMessage, StreamEvent};

const CLASSIFIER_WEB_MODEL: &str = "anthropic/claude-3.5-haiku";
// Bound on tool round-trips so a confused model can't loop (or bill) forever.
const MAX_STEPS: usize = 6;

/// One streamed completion's accumulated tool-call request (fragments arrive across
/// many SSE deltas and are concatenated by their `index`).
#[derive(Default, Clone)]
struct PartialToolCall {
    id: String,
    name: String,
    args: String,
}

/// The function tools exposed to the model (OpenAI/OpenRouter function-calling schema).
fn tool_schemas() -> Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "search_vault",
                "description": "Search Tucker's personal markdown knowledge vault for notes relevant to a query. Returns the top matching notes as path + snippet. Use this to recall anything specific about Tucker, his work, people, decisions, or projects before answering.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "What to look for, e.g. 'GP1 account challenges'" }
                    },
                    "required": ["query"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "read_note",
                "description": "Read one note from the vault in full, by its vault-relative path (as returned by search_vault). Use after search_vault to open a promising note.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Vault-relative path, e.g. '_work/cosi/resources/people/pam-das.md'" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the live web for current or external information not in Tucker's vault. Returns key findings with their source URLs.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The web search query" }
                    },
                    "required": ["query"]
                }
            }
        }
    ])
}

/// Run the agent loop for one user turn. Streams answer tokens + tool-activity events;
/// returns when the model produces a final answer (no tool calls) or hits MAX_STEPS.
pub async fn run(
    app: &AppHandle,
    mode: Mode,
    messages: &[ChatMessage],
    api_key: &str,
    on_event: &Channel<StreamEvent>,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let use_tools = mode.tools();

    // Build the working conversation: persona + the turn so far.
    let mut convo: Vec<Value> = Vec::new();
    convo.push(json!({ "role": "system", "content": mode.persona() }));
    for m in messages {
        convo.push(json!({ "role": m.role, "content": m.content }));
    }

    for _step in 0..MAX_STEPS {
        let mut body = json!({
            "model": mode.model(),
            "messages": convo,
            "stream": true,
        });
        if use_tools {
            body["tools"] = tool_schemas();
            body["tool_choice"] = json!("auto");
        }

        let calls = stream_once(&client, api_key, &body, on_event).await?;

        // No tool calls → the model just answered. Done.
        if calls.is_empty() {
            let _ = on_event.send(StreamEvent::Done);
            return Ok(());
        }

        // Echo the assistant's tool-call request back into the conversation…
        let tool_calls_json: Vec<Value> = calls
            .iter()
            .map(|c| {
                json!({
                    "id": c.id,
                    "type": "function",
                    "function": { "name": c.name, "arguments": c.args }
                })
            })
            .collect();
        convo.push(json!({ "role": "assistant", "content": null, "tool_calls": tool_calls_json }));

        // …then execute each tool and append its result for the next round.
        for c in &calls {
            let _ = on_event.send(StreamEvent::Tool {
                name: c.name.clone(),
                arg: arg_preview(&c.args),
            });
            let output = execute(app, mode, &c.name, &c.args, api_key, &client, on_event).await;
            convo.push(json!({
                "role": "tool",
                "tool_call_id": c.id,
                "name": c.name,
                "content": output,
            }));
        }
    }

    // Hit the step cap — close the stream with whatever was said.
    let _ = on_event.send(StreamEvent::Done);
    Ok(())
}

/// Stream one completion. Emits `Token` events for content as it arrives; accumulates and
/// returns any tool-call request (empty vec ⇒ the model produced a final answer).
async fn stream_once(
    client: &reqwest::Client,
    api_key: &str,
    body: &Value,
    on_event: &Channel<StreamEvent>,
) -> Result<Vec<PartialToolCall>, String> {
    let resp = client
        .post(crate::OPENROUTER_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("HTTP-Referer", "https://github.com/inkxel/amber")
        .header("X-Title", "Amber")
        .json(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let msg = format!("OpenRouter {status}: {text}");
        let _ = on_event.send(StreamEvent::Error(msg.clone()));
        return Err(msg);
    }

    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut buf = String::new();
    let mut calls: Vec<PartialToolCall> = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        buf.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(nl) = buf.find('\n') {
            let line: String = buf.drain(..=nl).collect();
            let line = line.trim();
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data == "[DONE]" {
                return Ok(calls);
            }
            if let Ok(json) = serde_json::from_str::<Value>(data) {
                accumulate(&json["choices"][0]["delta"], &mut calls, on_event);
            }
        }
    }
    Ok(calls)
}

/// Apply one streaming `delta`: emit content tokens, merge tool-call fragments by index.
fn accumulate(delta: &Value, calls: &mut Vec<PartialToolCall>, on_event: &Channel<StreamEvent>) {
    if let Some(tok) = delta["content"].as_str() {
        if !tok.is_empty() {
            let _ = on_event.send(StreamEvent::Token(tok.to_string()));
        }
    }
    if let Some(tcs) = delta["tool_calls"].as_array() {
        merge_tool_calls(tcs, calls);
    }
}

/// Merge tool-call fragments into the running list by their `index` — id/name arrive once,
/// `arguments` stream across many deltas and are concatenated. Pure, so it's unit-tested.
fn merge_tool_calls(tcs: &[Value], calls: &mut Vec<PartialToolCall>) {
    for tc in tcs {
        let idx = tc["index"].as_u64().unwrap_or(0) as usize;
        while calls.len() <= idx {
            calls.push(PartialToolCall::default());
        }
        let slot = &mut calls[idx];
        if let Some(id) = tc["id"].as_str() {
            if !id.is_empty() {
                slot.id = id.to_string();
            }
        }
        if let Some(name) = tc["function"]["name"].as_str() {
            if !name.is_empty() {
                slot.name = name.to_string();
            }
        }
        if let Some(a) = tc["function"]["arguments"].as_str() {
            slot.args.push_str(a);
        }
    }
}

/// Dispatch a tool call to its implementation, returning the result string fed back to the model.
async fn execute(
    app: &AppHandle,
    mode: Mode,
    name: &str,
    args_json: &str,
    api_key: &str,
    client: &reqwest::Client,
    on_event: &Channel<StreamEvent>,
) -> String {
    let args: Value = serde_json::from_str(args_json).unwrap_or_else(|_| json!({}));
    match name {
        "search_vault" => {
            let q = args["query"].as_str().unwrap_or("");
            match crate::vault_path(app) {
                Some(v) => {
                    let (digest, paths) = crate::vault::search_digest(&v, q);
                    if mode.show_sources() && !paths.is_empty() {
                        let _ = on_event.send(StreamEvent::Sources(paths));
                    }
                    digest
                }
                None => "No vault is connected.".into(),
            }
        }
        "read_note" => {
            let p = args["path"].as_str().unwrap_or("");
            match crate::vault_path(app) {
                Some(v) => crate::vault::read_note(&v, p).unwrap_or_else(|e| e),
                None => "No vault is connected.".into(),
            }
        }
        "web_search" => web_search(client, api_key, args["query"].as_str().unwrap_or("")).await,
        other => format!("Unknown tool: {other}"),
    }
}

/// `web_search` tool: reuse OpenRouter's built-in web plugin (no new API key) via a small
/// completion that returns findings + source URLs. This is the model-decided web access.
async fn web_search(client: &reqwest::Client, api_key: &str, query: &str) -> String {
    let body = json!({
        "model": CLASSIFIER_WEB_MODEL,
        "plugins": [{ "id": "web", "max_results": 5 }],
        "messages": [{
            "role": "user",
            "content": format!("Search the web and report the key findings with their source URLs for: {query}")
        }]
    });
    let resp = client
        .post(crate::OPENROUTER_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("X-Title", "Amber")
        .json(&body)
        .send()
        .await;
    match resp {
        Ok(r) => match r.json::<Value>().await {
            Ok(v) => v["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("(no web results)")
                .to_string(),
            Err(e) => format!("web_search could not parse results: {e}"),
        },
        Err(e) => format!("web_search failed: {e}"),
    }
}

/// The query/path from a tool-call's JSON args, for the UI's "using X" activity line.
fn arg_preview(args_json: &str) -> String {
    let v: Value = serde_json::from_str(args_json).unwrap_or_else(|_| json!({}));
    v["query"]
        .as_str()
        .or_else(|| v["path"].as_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // The fiddly bit: tool-call argument fragments arrive split across deltas and must be
    // concatenated by index. id/name come once; arguments stream.
    #[test]
    fn accumulates_split_tool_call_fragments() {
        let mut calls = Vec::new();
        let d1 = json!([{ "index": 0, "id": "call_1", "function": { "name": "search_vault", "arguments": "{\"que" } }]);
        let d2 = json!([{ "index": 0, "function": { "arguments": "ry\":\"gp1\"}" } }]);
        merge_tool_calls(d1.as_array().unwrap(), &mut calls);
        merge_tool_calls(d2.as_array().unwrap(), &mut calls);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call_1");
        assert_eq!(calls[0].name, "search_vault");
        assert_eq!(calls[0].args, "{\"query\":\"gp1\"}");
        assert_eq!(arg_preview(&calls[0].args), "gp1");
    }
}
