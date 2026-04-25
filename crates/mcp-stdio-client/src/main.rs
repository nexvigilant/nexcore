use std::process::Stdio;
use std::time::Duration;

use clap::{ArgAction, Parser};
use nexcore_error::{Result, nexerror};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

use mcp_stdio_client::*;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// MCP server command to run
    #[arg(long)]
    command: String,
    /// Arguments for the MCP server command (repeatable)
    #[arg(long, action = ArgAction::Append)]
    arg: Vec<String>,
    /// List tools
    #[arg(long)]
    list_tools: bool,
    /// Call tool name
    #[arg(long)]
    call: Option<String>,
    /// JSON arguments for tool call
    #[arg(long, default_value = "{}")]
    args_json: String,
    /// Timeout in milliseconds
    #[arg(long, default_value_t = 10000)]
    timeout_ms: u64,
    /// Print raw JSON responses
    #[arg(long)]
    raw: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut child = Command::new(&args.command)
        .args(&args.arg)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| nexerror!("missing stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| nexerror!("missing stdout"))?;

    let mut writer = tokio::io::BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout).lines();

    // Initialize
    let init_id = 1;
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": init_id,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "mcp-stdio-client", "version": "0.1"}
        }
    });
    send(&mut writer, &init_req).await?;
    let init_resp = read_response(&mut reader, init_id, args.timeout_ms).await?;
    if !args.raw {
        eprintln!("initialized: {}", summarize(&init_resp));
    }

    let initialized = json!({"jsonrpc": "2.0", "method": "notifications/initialized"});
    send(&mut writer, &initialized).await?;

    if args.list_tools {
        let id = 2;
        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/list",
            "params": {}
        });
        send(&mut writer, &req).await?;
        let resp = read_response(&mut reader, id, args.timeout_ms).await?;
        println!("{}", format_output(&resp, args.raw));
    }

    if let Some(tool) = args.call {
        let id = 3;
        let params: Value = serde_json::from_str(&args.args_json)
            .map_err(|err| nexerror!("invalid --args-json: {err}"))?;
        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {"name": tool, "arguments": params}
        });
        send(&mut writer, &req).await?;
        let resp = read_response(&mut reader, id, args.timeout_ms).await?;
        println!("{}", format_output(&resp, args.raw));
    }

    let _ = child.kill().await;
    Ok(())
}

async fn send(
    writer: &mut tokio::io::BufWriter<tokio::process::ChildStdin>,
    value: &Value,
) -> Result<()> {
    let line = serde_json::to_string(value)?;
    writer.write_all(line.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

async fn read_response(
    reader: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    id: i64,
    timeout_ms: u64,
) -> Result<Value> {
    let deadline = Duration::from_millis(timeout_ms);
    let resp = timeout(deadline, find_id_in_stream(reader, id))
        .await
        .map_err(|e| nexcore_error::NexError::msg(format!("timeout: {e}")))??;
    Ok(resp)
}

async fn find_id_in_stream(
    reader: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    id: i64,
) -> Result<Value> {
    loop {
        let line = reader
            .next_line()
            .await?
            .ok_or_else(|| nexerror!("EOF before response"))?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)?;
        if value.get("id").and_then(|v| v.as_i64()) == Some(id) {
            return Ok(value);
        }
    }
}
