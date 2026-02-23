use std::fs;
use std::path::{Path, PathBuf};

use nexcore_error::{Result, nexerror};
use clap::{Parser, Subcommand};
use toml_edit::{DocumentMut, value};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value = "/home/matthew/.codex/config.toml")]
    codex_config: PathBuf,
    #[arg(long, default_value = "/home/matthew/.mcp.json")]
    global_mcp: PathBuf,
    #[arg(long, default_value = "/home/matthew/.codex/mcp.json")]
    codex_mcp: PathBuf,
    #[arg(long, default_value = "/home/matthew/.codex/skills")]
    codex_skills: PathBuf,
    #[arg(long, default_value = "/home/matthew/.agents/skills")]
    agents_skills: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Install,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Install => {
            install_symlink(&args.codex_mcp, &args.global_mcp)?;
            update_codex_config(&args.codex_config, &args.global_mcp)?;
            install_default_servers(&args.codex_config)?;
            install_skills_symlink(&args.agents_skills, &args.codex_skills)?;
        }
    }

    Ok(())
}

fn update_codex_config(path: &Path, global_mcp: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .map_err(|err| nexerror!("failed to read {}: {err}", path.display()))?;
    let mut doc = text.parse::<DocumentMut>()?;

    if !doc.contains_table("mcp") {
        doc["mcp"] = toml_edit::table();
    }

    doc["mcp"]["config_path"] = value(global_mcp.display().to_string());

    fs::write(path, doc.to_string())
        .map_err(|err| nexerror!("failed to write {}: {err}", path.display()))?;
    Ok(())
}

fn install_default_servers(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .map_err(|err| nexerror!("failed to read {}: {err}", path.display()))?;
    let mut doc = text.parse::<DocumentMut>()?;

    if !doc.contains_table("mcp_servers") {
        doc["mcp_servers"] = toml_edit::table();
    }

    set_stdio_server(
        &mut doc,
        "claude-repl",
        "/home/matthew/.nexcore/target/release/claude-repl-mcp",
        &[],
    );
    set_stdio_server(
        &mut doc,
        "claude-fs",
        "/home/matthew/.nexcore/target/release/claude-fs-mcp",
        &[],
    );

    set_http_server(
        &mut doc,
        "openaiDeveloperDocs",
        "https://developers.openai.com/mcp",
    );

    fs::write(path, doc.to_string())
        .map_err(|err| nexerror!("failed to write {}: {err}", path.display()))?;
    Ok(())
}

fn set_stdio_server(doc: &mut DocumentMut, name: &str, command: &str, args: &[&str]) {
    doc["mcp_servers"][name]["command"] = value(command);
    let mut arr = toml_edit::Array::new();
    for arg in args {
        arr.push(*arg);
    }
    doc["mcp_servers"][name]["args"] = toml_edit::Item::Value(toml_edit::Value::Array(arr));
    doc["mcp_servers"][name]["env"] = toml_edit::table();
}

fn set_http_server(doc: &mut DocumentMut, name: &str, url: &str) {
    doc["mcp_servers"][name]["url"] = value(url);
}

fn install_symlink(codex_mcp: &Path, global_mcp: &Path) -> Result<()> {
    if !global_mcp.exists() {
        return Err(nexerror!(
            "global MCP config missing: {}",
            global_mcp.display()
        ));
    }

    if let Some(parent) = codex_mcp.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| nexerror!("failed to create {}: {err}", parent.display()))?;
    }

    if codex_mcp.exists() {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        let backup = codex_mcp.with_extension(format!("bak-{stamp}"));
        fs::rename(codex_mcp, &backup)
            .map_err(|err| nexerror!("failed to backup {}: {err}", codex_mcp.display()))?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(global_mcp, codex_mcp)
            .map_err(|err| nexerror!("failed to symlink: {err}"))?;
    }

    #[cfg(not(unix))]
    {
        fs::copy(global_mcp, codex_mcp).map_err(|err| nexerror!("failed to copy: {err}"))?;
    }

    Ok(())
}

fn install_skills_symlink(agents_skills: &Path, codex_skills: &Path) -> Result<()> {
    if let Some(parent) = agents_skills.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| nexerror!("failed to create {}: {err}", parent.display()))?;
    }

    if agents_skills.exists() {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        let backup = agents_skills.with_extension(format!("bak-{stamp}"));
        fs::rename(agents_skills, &backup)
            .map_err(|err| nexerror!("failed to backup {}: {err}", agents_skills.display()))?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(codex_skills, agents_skills)
            .map_err(|err| nexerror!("failed to symlink: {err}"))?;
    }

    #[cfg(not(unix))]
    {
        fs::create_dir_all(agents_skills)
            .map_err(|err| nexerror!("failed to create {}: {err}", agents_skills.display()))?;
    }

    Ok(())
}
