//! Borrow Miner MCP Server
//!
//! Exposes game actions as MCP tools for Claude to play

mod ore;
mod state;
mod tools;

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn main() {
    state::init_game();
    run_server();
}

fn run_server() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines().flatten() {
        if line.is_empty() { continue; }

        if let Ok(req) = serde_json::from_str::<Value>(&line) {
            let resp = route_request(&req);
            let _ = writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap());
            let _ = stdout.flush();
        }
    }
}

fn route_request(req: &Value) -> Value {
    let method = req["method"].as_str().unwrap_or("");
    let id = req["id"].clone();

    match method {
        "initialize" => init_response(id),
        "tools/list" => tools_list(id),
        "tools/call" => tools_call(req, id),
        _ => json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
    }
}

fn init_response(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "borrow-miner", "version": "0.1.0" }
        }
    })
}

fn tools_list(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": tool_definitions() }
    })
}

fn tool_definitions() -> Vec<Value> {
    vec![
        tool_def("mine", "Mine for ore. Get points based on combo and depth."),
        tool_def("drop_ore", "Drop oldest ore for bonus points."),
        tool_def("get_state", "Get current game state."),
        signal_tool_def(),
    ]
}

fn tool_def(name: &str, desc: &str) -> Value {
    json!({
        "name": name,
        "description": desc,
        "inputSchema": { "type": "object", "properties": {} }
    })
}

fn signal_tool_def() -> Value {
    json!({
        "name": "signal_check",
        "description": "Check FDA signal for drug-event pair.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "drug": { "type": "string" },
                "event": { "type": "string" }
            },
            "required": ["drug", "event"]
        }
    })
}

fn tools_call(req: &Value, id: Value) -> Value {
    let name = req["params"]["name"].as_str().unwrap_or("");
    let args = &req["params"]["arguments"];
    let result = dispatch_tool(name, args);

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "content": [{ "type": "text", "text": result }] }
    })
}

fn dispatch_tool(name: &str, args: &Value) -> String {
    match name {
        "mine" => tools::mine(),
        "drop_ore" => tools::drop_ore(),
        "get_state" => tools::get_state(),
        "signal_check" => tools::signal_check(args),
        _ => "Unknown tool".into(),
    }
}
