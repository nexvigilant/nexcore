//! Adventure HUD MCP Server - Track adventures via MCP tools

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError { code: i32, message: String }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AdventureState {
    session_id: String,
    name: String,
    started_at: String,
    tasks: Vec<TaskEvent>,
    skills: HashMap<String, u32>,
    measures: HashMap<String, f64>,
    milestones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskEvent {
    id: String,
    subject: String,
    status: String,
    timestamp: String,
}

type State = Arc<Mutex<Option<AdventureState>>>;

fn main() {
    let state: State = Arc::new(Mutex::new(None));
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines().map_while(Result::ok) {
        if line.trim().is_empty() { continue; }
        let resp = handle_line(&line, &state);
        writeln!(stdout, "{}", resp).ok();
        stdout.flush().ok();
    }
}

fn handle_line(line: &str, state: &State) -> String {
    let req: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(_) => return error_response(Value::Null, -32700, "Parse error"),
    };
    let id = req.id.clone().unwrap_or(Value::Null);
    let result = dispatch(&req.method, req.params, state);
    match result {
        Ok(r) => ok_response(id, r),
        Err((c, m)) => error_response(id, c, &m),
    }
}

fn dispatch(method: &str, params: Option<Value>, state: &State) -> Result<Value, (i32, String)> {
    match method {
        "initialize" => Ok(init_result()),
        "tools/list" => Ok(tools_list()),
        "tools/call" => tools_call(params, state),
        _ => Err((-32601, "Method not found".into())),
    }
}

fn init_result() -> Value {
    json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"adventure-hud","version":"1.0.0"}})
}

fn tools_list() -> Value {
    json!({"tools":[
        {"name":"adventure_start","description":"Start adventure","inputSchema":{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}},
        {"name":"adventure_task","description":"Log task","inputSchema":{"type":"object","properties":{"id":{"type":"string"},"subject":{"type":"string"},"status":{"type":"string"}},"required":["id","subject","status"]}},
        {"name":"adventure_skill","description":"Log skill","inputSchema":{"type":"object","properties":{"skill":{"type":"string"}},"required":["skill"]}},
        {"name":"adventure_measure","description":"Record metric","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"value":{"type":"number"}},"required":["name","value"]}},
        {"name":"adventure_milestone","description":"Record milestone","inputSchema":{"type":"object","properties":{"milestone":{"type":"string"}},"required":["milestone"]}},
        {"name":"adventure_status","description":"Get state","inputSchema":{"type":"object"}}
    ]})
}

fn tools_call(params: Option<Value>, state: &State) -> Result<Value, (i32, String)> {
    let p = params.ok_or((-32602, "Missing params".into()))?;
    let name = get_str(&p, "name").ok_or((-32602, "Missing tool name".into()))?;
    let args = p.get("arguments").cloned().unwrap_or(json!({}));
    let text = call_tool(name, &args, state)?;
    Ok(json!({"content":[{"type":"text","text":text}]}))
}

fn call_tool(name: &str, args: &Value, state: &State) -> Result<String, (i32, String)> {
    match name {
        "adventure_start" => tool_start(args, state),
        "adventure_task" => tool_task(args, state),
        "adventure_skill" => tool_skill(args, state),
        "adventure_measure" => tool_measure(args, state),
        "adventure_milestone" => tool_milestone(args, state),
        "adventure_status" => tool_status(state),
        _ => Err((-32602, format!("Unknown tool: {}", name))),
    }
}

fn tool_start(args: &Value, state: &State) -> Result<String, (i32, String)> {
    let name = get_str(args, "name").unwrap_or("Adventure");
    let now = Utc::now();
    let s = AdventureState {
        session_id: format!("adv-{}", now.timestamp()),
        name: name.into(),
        started_at: now.to_rfc3339(),
        ..Default::default()
    };
    *state.lock().unwrap() = Some(s.clone());
    Ok(format!("🗺️ Adventure '{}' started! ID: {}", s.name, s.session_id))
}

fn tool_task(args: &Value, state: &State) -> Result<String, (i32, String)> {
    let mut guard = state.lock().unwrap();
    let s = guard.as_mut().ok_or((-32000, "No adventure".into()))?;
    let t = TaskEvent {
        id: get_str(args, "id").unwrap_or("?").into(),
        subject: get_str(args, "subject").unwrap_or("?").into(),
        status: get_str(args, "status").unwrap_or("pending").into(),
        timestamp: Utc::now().to_rfc3339(),
    };
    let icon = status_icon(&t.status);
    let msg = format!("{} Task #{}: {} [{}]", icon, t.id, t.subject, t.status);
    s.tasks.push(t);
    Ok(msg)
}

fn tool_skill(args: &Value, state: &State) -> Result<String, (i32, String)> {
    let mut guard = state.lock().unwrap();
    let s = guard.as_mut().ok_or((-32000, "No adventure".into()))?;
    let skill = get_str(args, "skill").unwrap_or("unknown");
    *s.skills.entry(skill.into()).or_insert(0) += 1;
    Ok(format!("⚡ Skill /{} (×{})", skill, s.skills.get(skill).unwrap()))
}

fn tool_measure(args: &Value, state: &State) -> Result<String, (i32, String)> {
    let mut guard = state.lock().unwrap();
    let s = guard.as_mut().ok_or((-32000, "No adventure".into()))?;
    let name = get_str(args, "name").unwrap_or("metric");
    let val = args.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
    s.measures.insert(name.into(), val);
    Ok(format!("📏 {} = {:.2}", name, val))
}

fn tool_milestone(args: &Value, state: &State) -> Result<String, (i32, String)> {
    let mut guard = state.lock().unwrap();
    let s = guard.as_mut().ok_or((-32000, "No adventure".into()))?;
    let m = get_str(args, "milestone").unwrap_or("milestone");
    s.milestones.push(m.into());
    Ok(format!("🏆 {}", m))
}

fn tool_status(state: &State) -> Result<String, (i32, String)> {
    let guard = state.lock().unwrap();
    match guard.as_ref() {
        Some(s) => Ok(serde_json::to_string_pretty(s).unwrap_or_default()),
        None => Ok("No adventure. Use adventure_start.".into()),
    }
}

fn get_str<'a>(v: &'a Value, key: &str) -> Option<&'a str> { v.get(key).and_then(|x| x.as_str()) }
fn status_icon(s: &str) -> &'static str { match s { "completed" => "✓", "in_progress" => "⟳", _ => "○" } }

fn ok_response(id: Value, result: Value) -> String {
    serde_json::to_string(&JsonRpcResponse { jsonrpc: "2.0".into(), id, result: Some(result), error: None }).unwrap()
}

fn error_response(id: Value, code: i32, msg: &str) -> String {
    serde_json::to_string(&JsonRpcResponse { jsonrpc: "2.0".into(), id, result: None, error: Some(JsonRpcError { code, message: msg.into() }) }).unwrap()
}
