//! nexcore-vault CLI — zero-dep local secret manager.

use std::io::{self, BufRead, Write as IoWrite};

use clap::{Parser, Subcommand};

use nexcore_vault::config::VaultConfig;
use nexcore_vault::error::{Result, VaultError};
use nexcore_vault::store::Vault;
use nexcore_vault::types::{PlaintextExport, SecretName};

#[derive(Parser)]
#[command(name = "nexcore-vault")]
#[command(about = "Zero-dep local encrypted secret manager")]
#[command(version)]
struct Cli {
    /// Path to vault config file.
    #[arg(short, long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new vault.
    Init,
    /// Set a secret value.
    Set {
        /// Secret name.
        name: String,
        /// Secret value (omit to read from stdin).
        value: Option<String>,
    },
    /// Get a secret value.
    Get {
        /// Secret name.
        name: String,
    },
    /// List all secret names.
    List,
    /// Delete a secret.
    Delete {
        /// Secret name.
        name: String,
    },
    /// Export all secrets as plaintext JSON (dangerous).
    Export,
    /// Import secrets from plaintext JSON on stdin.
    Import,
    /// Change the vault password.
    ChangePassword,
}

fn main() {
    let cli = Cli::parse();

    let config = load_config(&cli);
    let config = match config {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {e}");
            std::process::exit(1);
        }
    };

    let result = run_command(cli.command, config);
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn load_config(cli: &Cli) -> Result<VaultConfig> {
    cli.config.as_ref().map_or_else(
        || {
            let default_path = VaultConfig::default_config_path();
            VaultConfig::load(&default_path)
        },
        |path| VaultConfig::load(std::path::Path::new(path)),
    )
}

fn run_command(command: Commands, config: VaultConfig) -> Result<()> {
    match command {
        Commands::Init => cmd_init(config),
        Commands::Set { name, value } => cmd_set(config, &name, value),
        Commands::Get { name } => cmd_get(config, &name),
        Commands::List => cmd_list(config),
        Commands::Delete { name } => cmd_delete(config, &name),
        Commands::Export => cmd_export(config),
        Commands::Import => cmd_import(config),
        Commands::ChangePassword => cmd_change_password(config),
    }
}

fn cmd_init(config: VaultConfig) -> Result<()> {
    let password = get_password("Enter new vault password: ")?;
    let confirm = get_password("Confirm password: ")?;

    if password != confirm {
        return Err(VaultError::Config("passwords do not match".into()));
    }

    Vault::create(config, &password)?;
    eprintln!("Vault created successfully.");
    Ok(())
}

fn cmd_set(config: VaultConfig, name: &str, value: Option<String>) -> Result<()> {
    let password = get_password("Password: ")?;
    let name = SecretName::new(name)?;

    let value = match value {
        Some(v) => v,
        None => read_stdin_line()?,
    };

    let mut vault = Vault::open(config, &password)?;
    vault.set(&name, &value)?;
    eprintln!("Secret '{name}' set.");
    Ok(())
}

fn cmd_get(config: VaultConfig, name: &str) -> Result<()> {
    let password = get_password("Password: ")?;
    let name = SecretName::new(name)?;
    let vault = Vault::open(config, &password)?;
    let value = vault.get(&name)?;
    println!("{value}");
    Ok(())
}

fn cmd_list(config: VaultConfig) -> Result<()> {
    let password = get_password("Password: ")?;
    let vault = Vault::open(config, &password)?;
    let names = vault.list();

    if names.is_empty() {
        eprintln!("Vault is empty.");
    } else {
        for name in &names {
            println!("{name}");
        }
        eprintln!("{} secret(s)", names.len());
    }
    Ok(())
}

fn cmd_delete(config: VaultConfig, name: &str) -> Result<()> {
    let password = get_password("Password: ")?;
    let name = SecretName::new(name)?;
    let mut vault = Vault::open(config, &password)?;
    vault.delete(&name)?;
    eprintln!("Secret '{name}' deleted.");
    Ok(())
}

fn cmd_export(config: VaultConfig) -> Result<()> {
    let password = get_password("Password: ")?;
    let vault = Vault::open(config, &password)?;
    let export = vault.export()?;
    let json = serde_json::to_string_pretty(&export)?;
    println!("{json}");
    Ok(())
}

fn cmd_import(config: VaultConfig) -> Result<()> {
    let password = get_password("Password: ")?;
    let mut vault = Vault::open(config, &password)?;

    let stdin = io::stdin();
    let mut input = String::new();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| VaultError::Io {
            path: std::path::PathBuf::from("<stdin>"),
            source: e,
        })?;
        input.push_str(&line);
        input.push('\n');
    }

    let export: PlaintextExport = serde_json::from_str(&input)?;
    let count = vault.import(&export)?;
    eprintln!("Imported {count} secret(s).");
    Ok(())
}

fn cmd_change_password(config: VaultConfig) -> Result<()> {
    let old_pass = get_password("Current password: ")?;
    let new_pass = get_password("New password: ")?;
    let confirm = get_password("Confirm new password: ")?;

    if new_pass != confirm {
        return Err(VaultError::Config("passwords do not match".into()));
    }

    let mut vault = Vault::open(config, &old_pass)?;
    vault.change_password(&new_pass)?;
    eprintln!("Password changed successfully.");
    Ok(())
}

/// Get password from NEXCORE_VAULT_PASSWORD env var or stderr prompt.
fn get_password(prompt: &str) -> Result<String> {
    if let Ok(pass) = std::env::var("NEXCORE_VAULT_PASSWORD") {
        if !pass.is_empty() {
            return Ok(pass);
        }
    }

    eprint!("{prompt}");
    io::stderr().flush().map_err(|e| VaultError::Io {
        path: std::path::PathBuf::from("<stderr>"),
        source: e,
    })?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| VaultError::Io {
            path: std::path::PathBuf::from("<stdin>"),
            source: e,
        })?;

    let pass = line.trim_end().to_string();
    if pass.is_empty() {
        return Err(VaultError::PasswordRequired);
    }
    Ok(pass)
}

/// Read a single line from stdin.
fn read_stdin_line() -> Result<String> {
    eprintln!("Enter secret value (single line):");
    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| VaultError::Io {
            path: std::path::PathBuf::from("<stdin>"),
            source: e,
        })?;
    Ok(line.trim_end().to_string())
}
