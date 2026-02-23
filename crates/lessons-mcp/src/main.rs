//! Lessons Learned MCP Server Entry Point
//! Tier: T1 (Sequence - main loop)

use lessons_mcp::{protocol::Response, tools};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};

fn main() {
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();

    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let response = process_line(&line);
        write_response(&mut stdout, &response);
    }
}

fn process_line(line: &str) -> Response {
    let request: lessons_mcp::protocol::Request = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => return Response::parse_error(&e.to_string()),
    };

    let id = request.id.unwrap_or(json!(null));
    dispatch(&request.method, &request.params, id)
}

fn dispatch(method: &str, params: &Value, id: Value) -> Response {
    match method {
        "initialize" => initialize_response(id),
        "tools/list" => Response::success(id, tools::definitions()),
        "tools/call" => tools::call(params, id),
        _ => Response::method_not_found(id, method),
    }
}

fn initialize_response(id: Value) -> Response {
    Response::success(
        id,
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "lessons-mcp", "version": "0.1.0" }
        }),
    )
}

fn write_response(stdout: &mut std::io::Stdout, response: &Response) {
    if let Ok(json) = serde_json::to_string(response) {
        let _ = writeln!(stdout, "{}", json);
        let _ = stdout.flush();
    }
}
