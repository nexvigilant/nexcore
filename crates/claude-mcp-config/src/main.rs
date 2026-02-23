use std::fs;
use std::path::PathBuf;

use nexcore_error::{Result, nexerror};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value = "/home/matthew/.claude/settings.json")]
    settings_path: PathBuf,
    #[arg(long, default_value = "/home/matthew/.mcp.json")]
    mcp_path: PathBuf,
    #[arg(long, default_value = "/home/matthew/.config/claude/config.json")]
    mirror_claude_path: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    AddServer {
        #[arg(long)]
        scope: Scope,
        #[arg(long)]
        name: String,
        #[arg(long)]
        command: String,
        #[arg(long, action = clap::ArgAction::Append)]
        server_args: Vec<String>,
        #[arg(long)]
        description: Option<String>,
    },
    RemoveServer {
        #[arg(long)]
        scope: Scope,
        #[arg(long)]
        name: String,
    },
    AllowTool {
        #[arg(long)]
        pattern: String,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Scope {
    Claude,
    Global,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let claude_path = args.settings_path.clone();
    let global_path = args.mcp_path.clone();
    let mirror_claude_path = args.mirror_claude_path.clone();

    match args.command {
        Command::AddServer {
            scope,
            name,
            command,
            server_args,
            description,
        } => {
            let paths = scope_paths(&scope, &claude_path, &global_path, &mirror_claude_path);
            for path in paths {
                let mut root = load_json(&path)?;
                let mcp = ensure_object(root.pointer_mut("/mcpServers"));
                mcp.as_object_mut()
                    .ok_or_else(|| nexerror!("mcpServers is not an object"))?
                    .insert(
                        name.clone(),
                        json!({
                            "command": command,
                            "args": server_args,
                            "env": {},
                            "description": description.clone().unwrap_or_default(),
                            "type": "stdio"
                        }),
                    );
                write_json(&path, &root)?;
            }
        }
        Command::RemoveServer { scope, name } => {
            let paths = scope_paths(&scope, &claude_path, &global_path, &mirror_claude_path);
            for path in paths {
                let mut root = load_json(&path)?;
                if let Some(mcp) = root.get_mut("mcpServers") {
                    if let Some(obj) = mcp.as_object_mut() {
                        obj.remove(&name);
                    }
                }
                write_json(&path, &root)?;
            }
        }
        Command::AllowTool { pattern } => {
            let mut paths = vec![claude_path.clone()];
            if mirror_claude_path.exists() && mirror_claude_path != claude_path {
                paths.push(mirror_claude_path.clone());
            }
            for path in paths {
                let mut root = load_json(&path)?;
                let perms = ensure_object(root.pointer_mut("/permissions"));
                let allow = perms
                    .as_object_mut()
                    .ok_or_else(|| nexerror!("permissions is not an object"))?
                    .entry("allow")
                    .or_insert_with(|| Value::Array(Vec::new()));
                let list = allow
                    .as_array_mut()
                    .ok_or_else(|| nexerror!("permissions.allow is not an array"))?;
                if !list.iter().any(|v| v.as_str() == Some(&pattern)) {
                    list.push(Value::String(pattern.clone()));
                }
                write_json(&path, &root)?;
            }
        }
    }

    Ok(())
}

fn scope_paths(
    scope: &Scope,
    claude_path: &PathBuf,
    global_path: &PathBuf,
    mirror_claude_path: &PathBuf,
) -> Vec<PathBuf> {
    match scope {
        Scope::Claude => {
            let mut paths = vec![claude_path.clone()];
            if mirror_claude_path.exists() && mirror_claude_path != claude_path {
                paths.push(mirror_claude_path.clone());
            }
            paths
        }
        Scope::Global => vec![global_path.clone()],
    }
}

fn load_json(path: &PathBuf) -> Result<Value> {
    let text = fs::read_to_string(path)
        .map_err(|err| nexerror!("failed to read {}: {err}", path.display()))?;
    let value = serde_json::from_str(&text)
        .map_err(|err| nexerror!("invalid json {}: {err}", path.display()))?;
    Ok(value)
}

fn write_json(path: &PathBuf, value: &Value) -> Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, text).map_err(|err| nexerror!("failed to write {}: {err}", path.display()))?;
    Ok(())
}

fn ensure_object(node: Option<&mut Value>) -> &mut Value {
    match node {
        Some(v) => {
            if !v.is_object() {
                *v = json!({});
            }
            v
        }
        None => panic!("invalid JSON path"),
    }
}
