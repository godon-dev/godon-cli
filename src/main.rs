use clap::{Parser, Subcommand};
use godon_cli::{create_wait_met, EXIT_CREATE_UNMET, EXIT_NAME_TAKEN, EXIT_PURGE_UNMET, Steerwish, SteerwishSummary, Systemtender, SystemtenderSummary, Credential, GodonClient, Target};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "godon_cli")]
#[command(about = "CLI for the Godon API", long_about = None)]
struct Cli {
    #[arg(short = 'H', long, default_value = "localhost")]
    hostname: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(short = 'V', long, default_value = "v0")]
    api_version: String,

    #[arg(short, long, value_enum, default_value = "text")]
    output: OutputFormat,

    #[arg(long)]
    insecure: bool,

    #[arg(long, env = "DEBUG")]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::ValueEnum, Clone, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Yaml,
}

#[derive(Subcommand)]
enum Commands {
    Systemtender {
        #[command(subcommand)]
        subcommand: SystemtenderCommands,
    },
    Credential {
        #[command(subcommand)]
        subcommand: CredentialCommands,
    },
    Target {
        #[command(subcommand)]
        subcommand: TargetCommands,
    },
    Wish {
        #[command(subcommand)]
        subcommand: WishCommands,
    },
}

#[derive(Subcommand)]
enum SystemtenderCommands {
    List,

    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        file: PathBuf,
        /// Wait server-side until the systemtender is active (or the
        /// create fails / the timeout is reached). Default: accept the
        /// 202 (row in `creating`) and return at once.
        #[arg(long, default_value_t = false)]
        wait: bool,
        /// Seconds to wait for the active state (--wait only,
        /// default 60). The client gives itself N+30 seconds.
        #[arg(long, default_value_t = 60)]
        timeout: u64,
    },

    Show {
        #[arg(long)]
        id: String,
    },

    Update {
        #[arg(long)]
        id: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    Stop {
        #[arg(long)]
        id: String,
    },

    Start {
        #[arg(long)]
        id: String,
    },

    Purge {
        #[arg(long)]
        id: String,
        #[arg(long)]
        force: bool,
        /// Wait until the systemtender is fully gone (GET answers
        /// 404). Default: accept the 202 (deletion marker set) and
        /// return at once.
        #[arg(long, default_value_t = false)]
        wait: bool,
        /// Seconds to wait for the 404 (--wait only, default 60).
        #[arg(long, default_value_t = 60)]
        timeout: u64,
    },
}

#[derive(Subcommand)]
enum WishCommands {
    List,

    Declare {
        #[arg(long)]
        file: PathBuf,
    },

    Show {
        #[arg(long)]
        id: String,
    },

    Close {
        #[arg(long)]
        id: String,
    },

    /// The holder corrects the wish: new terms, same identity (YAML file)
    Update {
        #[arg(long)]
        id: String,
        #[arg(long)]
        file: PathBuf,
    },

    /// Purge the wish: close first, then forget
    Delete {
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum CredentialCommands {
    List,

    Create {
        #[arg(long)]
        file: PathBuf,
    },

    Show {
        #[arg(long)]
        id: String,
    },

    Delete {
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum TargetCommands {
    List,

    Create {
        #[arg(long)]
        file: PathBuf,
    },

    Show {
        #[arg(long)]
        id: String,
    },

    Delete {
        #[arg(long)]
        id: String,
    },
}

/// Same as write_error, but with the async contract's distinct exit
/// code (designs/2026-10-10): scripts branch on the number, humans
/// read the prose.
fn write_error_exit(message: &str, code: i32) -> ! {
    eprintln!("Error: {}", message);
    std::process::exit(code);
}

fn write_error(message: &str) -> ! {
    eprintln!("Error: {}", message);
    std::process::exit(1);
}

fn format_output<T: serde::Serialize>(data: &T, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(data).unwrap_or_default());
        }
        OutputFormat::Text => {}
    }
}

async fn handle_target_command(client: &GodonClient, cmd: TargetCommands, output: &OutputFormat) {
    match cmd {
        TargetCommands::List => {
            let response = client.list_targets().await;
            if response.success {
                if let Some(targets) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_target_list(&targets);
                    } else {
                        format_output(&targets, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        TargetCommands::Create { file } => {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => write_error(&format!("Failed to read file: {}", e)),
            };

            let response = client.create_target_from_yaml(&content).await;
            if response.success {
                if let Some(target) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_target_created(&target);
                    } else {
                        format_output(&target, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        TargetCommands::Show { id } => {
            let response = client.get_target(&id).await;
            if response.success {
                if let Some(target) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_target(&target);
                    } else {
                        format_output(&target, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        TargetCommands::Delete { id } => {
            let response = client.delete_target(&id).await;
            if response.success {
                if matches!(output, OutputFormat::Text) {
                    println!("Target deleted successfully: {}", id);
                } else if let Some(data) = response.data {
                    format_output(&data, output);
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }
    }
}

fn format_systemtender_list(systemtenders: &[SystemtenderSummary]) {
    println!("Systemtenders:");
    for systemtender in systemtenders {
        println!("  ID: {}", systemtender.id);
        println!("  Name: {}", systemtender.name);
        println!("  Status: {}", systemtender.status);
        println!("  Created: {}", systemtender.created_at);
        println!("  ---");
    }
}

fn format_steerwish_list(steerwishes: &[SteerwishSummary]) {
    println!("Found {} steerwish(es)", steerwishes.len());
    for steerwish in steerwishes {
        // wish-shape freedom: read the aim from either grammar - the first
        // claim (claims + terms) or the top-level outcome (retired N=1
        // shape, still shown for pre-flip records)
        let aim = steerwish
            .body
            .get("claims")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .and_then(|c| c.get("outcome"))
            .and_then(|v| v.as_str())
            .or_else(|| steerwish.body.get("outcome").and_then(|v| v.as_str()))
            .unwrap_or("(carried by the body)");
        println!("  ID: {}", steerwish.id);
        println!("  Outcome: {}", aim);
        println!("  State: {}", steerwish.state);
        println!("  Created: {}", steerwish.created_at);
        println!();
    }
}

fn format_steerwish(steerwish: &Steerwish) {
    println!("  ID: {}", steerwish.id);
    println!("  State: {}", steerwish.state);
    println!("  Created: {}", steerwish.created_at);
    // wish-shape freedom: the body is the controller's object - shown
    // verbatim, whatever grammar it carries (today claims + terms)
    println!(
        "  Body: {}",
        serde_json::to_string_pretty(&steerwish.body).unwrap_or_default()
    );
    if let Some(events) = &steerwish.events {
        println!("  Events:");
        for event in events {
            match &event.detail {
                Some(detail) => println!("    [{}] {} - {}", event.at, event.kind, detail),
                None => println!("    [{}] {}", event.at, event.kind),
            }
        }
    }
}

fn format_systemtender(systemtender: &Systemtender) {
    println!("Systemtender Details:");
    println!("  ID: {}", systemtender.id);
    println!("  Name: {}", systemtender.name);
    println!("  Status: {}", systemtender.status);
    println!("  Config: {}", serde_json::to_string_pretty(&systemtender.config).unwrap_or_default());
    println!("  Created: {}", systemtender.created_at);
}

fn format_systemtender_summary(systemtender: &SystemtenderSummary) {
    println!("Systemtender created successfully:");
    println!("  ID: {}", systemtender.id);
    println!("  Name: {}", systemtender.name);
    println!("  Status: {}", systemtender.status);
}

fn format_credential_list(credentials: &[Credential]) {
    println!("Credentials:");
    for credential in credentials {
        println!("  ID: {}", credential.id);
        println!("  Name: {}", credential.name);
        println!("  Type: {}", credential.credential_type);
        println!("  Description: {}", credential.description.as_deref().unwrap_or(""));
        println!("  windmillVariable: {}", credential.windmill_variable);
        println!("  Created: {}", credential.created_at.as_deref().unwrap_or(""));
        println!("  ---");
    }
}

fn format_credential(credential: &Credential) {
    println!("Credential Details:");
    println!("  ID: {}", credential.id);
    println!("  Name: {}", credential.name);
    println!("  Type: {}", credential.credential_type);
    println!("  Description: {}", credential.description.as_deref().unwrap_or(""));
    println!("  windmillVariable: {}", credential.windmill_variable);
    println!("  Created: {}", credential.created_at.as_deref().unwrap_or(""));
    println!("  Last Used: {}", credential.last_used_at.as_deref().unwrap_or(""));
    println!("  Content:");
    println!("    {}", credential.content.as_deref().unwrap_or(""));
}

fn format_credential_created(credential: &Credential) {
    println!("Credential created successfully:");
    println!("  ID: {}", credential.id);
    println!("  Name: {}", credential.name);
    println!("  Type: {}", credential.credential_type);
    println!("  windmillVariable: {}", credential.windmill_variable);
}

fn format_target_list(targets: &[Target]) {
    println!("Targets:");
    for target in targets {
        let spec_summary = target
            .spec
            .get("address")
            .or_else(|| target.spec.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        println!("  ID: {}", target.id);
        println!("  Name: {}", target.name);
        println!("  Type: {}", target.target_type);
        println!("  Spec: {}", spec_summary);
        println!("  Created: {}", target.created_at.as_deref().unwrap_or(""));
        println!("  ---");
    }
}

fn format_target(target: &Target) {
    println!("Target Details:");
    println!("  ID: {}", target.id);
    println!("  Name: {}", target.name);
    println!("  Type: {}", target.target_type);
    println!("  Spec: {}", serde_json::to_string_pretty(&target.spec).unwrap_or_default());
    if let Some(ref metadata) = target.metadata {
        println!("  Metadata: {}", serde_json::to_string_pretty(metadata).unwrap_or_default());
    }
    println!("  Created: {}", target.created_at.as_deref().unwrap_or(""));
    println!("  Last Used: {}", target.last_used_at.as_deref().unwrap_or(""));
}

fn format_target_created(target: &Target) {
    println!("Target created successfully:");
    println!("  ID: {}", target.id);
    println!("  Name: {}", target.name);
    println!("  Type: {}", target.target_type);
    println!("  Spec: {}", serde_json::to_string_pretty(&target.spec).unwrap_or_default());
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let client = match GodonClient::new(
        cli.hostname,
        cli.port,
        cli.api_version,
        cli.insecure,
        cli.debug,
    ) {
        Ok(c) => c,
        Err(e) => write_error(&e.to_string()),
    };

    match cli.command {
        Commands::Systemtender { subcommand } => handle_systemtender_command(&client, subcommand, &cli.output).await,
        Commands::Credential { subcommand } => handle_credential_command(&client, subcommand, &cli.output).await,
        Commands::Target { subcommand } => handle_target_command(&client, subcommand, &cli.output).await,
        Commands::Wish { subcommand } => handle_wish_command(&client, subcommand, &cli.output).await,
    }
}

async fn handle_wish_command(client: &GodonClient, cmd: WishCommands, output: &OutputFormat) {
    match cmd {
        WishCommands::List => {
            let response = client.list_steerwishes().await;
            if response.success {
                if let Some(steerwishes) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_steerwish_list(&steerwishes);
                    } else {
                        format_output(&steerwishes, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        WishCommands::Declare { file } => {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => write_error(&format!("Failed to read file: {}", e)),
            };

            let response = client.declare_steerwish_from_yaml(&content).await;
            if response.success {
                if let Some(steerwish) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_steerwish(&steerwish);
                    } else {
                        format_output(&steerwish, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        WishCommands::Show { id } => {
            let response = client.get_steerwish(&id).await;
            if response.success {
                if let Some(steerwish) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_steerwish(&steerwish);
                    } else {
                        format_output(&steerwish, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        WishCommands::Update { id, file } => {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => write_error(&format!("Failed to read file: {}", e)),
            };

            let body: serde_json::Value = match serde_yaml::from_str(&content) {
                Ok(v) => v,
                Err(e) => write_error(&format!("Failed to parse wish YAML: {}", e)),
            };

            let response = client.update_steerwish(&id, body).await;
            if response.success {
                if let Some(steerwish) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_steerwish(&steerwish);
                    } else {
                        format_output(&steerwish, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        WishCommands::Delete { id } => {
            let response = client.delete_steerwish(&id).await;
            if response.success {
                if let Some(result) = response.data {
                    format_output(&result, output);
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        WishCommands::Close { id } => {
            let response = client.close_steerwish(&id).await;
            if response.success {
                if let Some(steerwish) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_steerwish(&steerwish);
                    } else {
                        format_output(&steerwish, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }
    }
}

async fn handle_systemtender_command(client: &GodonClient, cmd: SystemtenderCommands, output: &OutputFormat) {
    match cmd {
        SystemtenderCommands::List => {
            let response = client.list_systemtenders().await;
            if response.success {
                if let Some(systemtenders) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_systemtender_list(&systemtenders);
                    } else {
                        format_output(&systemtenders, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        SystemtenderCommands::Create { name, file, wait, timeout } => {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => write_error(&format!("Failed to read file: {}", e)),
            };

            // Wait mode (designs/2026-10-10): the server polls its read
            // path until the row is terminal-for-create or the bound;
            // the verdict rides the row state, the exit code carries it.
            if wait {
                let response = client.create_systemtender_from_yaml_wait(&content, &name, timeout).await;
                if response.status == Some(409) {
                    write_error_exit(
                        &format!("Name already taken: {}", response.error.as_deref().unwrap_or("a systemtender with this name exists")),
                        EXIT_NAME_TAKEN,
                    );
                }
                if response.success {
                    if let Some(row) = response.data {
                        if matches!(output, OutputFormat::Text) {
                            format_systemtender(&row);
                        } else {
                            format_output(&row, output);
                        }
                        if create_wait_met(&row.status) {
                            return;
                        }
                        // Unmet: create-failed (reason on the row) or
                        // still creating at the bound (executor runs on)
                        if let Some(reason) = row.creation_reason.as_deref() {
                            eprintln!("Create failed: {}", reason);
                        }
                        eprintln!("Create expectation unmet, status: {}", row.status);
                        std::process::exit(EXIT_CREATE_UNMET);
                    }
                } else {
                    write_error(response.error.as_deref().unwrap_or("Unknown error"));
                }
            }

            // Async default: 202 + the row in `creating`
            let response = client.create_systemtender_from_yaml(&content, &name).await;
            if response.status == Some(409) {
                write_error_exit(
                    &format!("Name already taken: {}", response.error.as_deref().unwrap_or("a systemtender with this name exists")),
                    EXIT_NAME_TAKEN,
                );
            }
            if response.success {
                if let Some(systemtender) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        println!("Systemtender create accepted (async):");
                        println!("  ID: {}", systemtender.id);
                        println!("  Name: {}", systemtender.name);
                        println!("  Status: {}", systemtender.status);
                        println!("  Poll with: systemtender show --id {}", systemtender.id);
                    } else {
                        format_output(&systemtender, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        SystemtenderCommands::Show { id } => {
            let response = client.get_systemtender(&id).await;
            if response.success {
                if let Some(systemtender) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_systemtender(&systemtender);
                    } else {
                        format_output(&systemtender, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        SystemtenderCommands::Update { id, file, force } => {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => write_error(&format!("Failed to read file: {}", e)),
            };

            let response = client.update_systemtender_from_yaml(&id, &content, force).await;
            if response.success {
                if let Some(data) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        let systemtender_id = data.get("systemtender_id").and_then(|v| v.as_str()).unwrap_or(&id);
                        let trials_cleared = data.get("trials_cleared").and_then(|v| v.as_bool()).unwrap_or(false);
                        let history = data.get("config_history_entries").and_then(|v| v.as_u64()).unwrap_or(0);
                        println!("Systemtender updated successfully:");
                        println!("  ID: {}", systemtender_id);
                        println!("  Trials cleared: {}", trials_cleared);
                        println!("  Config history entries: {}", history);
                    } else {
                        format_output(&data, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        SystemtenderCommands::Stop { id } => {
            let response = client.stop_systemtender(&id).await;
            if response.success {
                if matches!(output, OutputFormat::Text) {
                    println!("Systemtender stop requested (graceful shutdown): {}", id);
                    println!("Workers will finish current trial before stopping.");
                } else if let Some(data) = response.data {
                    format_output(&data, output);
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        SystemtenderCommands::Start { id } => {
            let response = client.start_systemtender(&id).await;
            if response.success {
                if matches!(output, OutputFormat::Text) {
                    println!("Systemtender started/resumed: {}", id);
                } else if let Some(data) = response.data {
                    format_output(&data, output);
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        SystemtenderCommands::Purge { id, force, wait, timeout } => {
            let response = client.delete_systemtender(&id, force).await;
            if !response.success {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }

            if matches!(output, OutputFormat::Text) {
                if force {
                    println!("Systemtender deletion requested (force, workers cancelled): {}", id);
                } else {
                    println!("Systemtender deletion requested: {}", id);
                }
            } else if let Some(data) = response.data {
                format_output(&data, output);
            }

            // Sync delete as the fallback (designs/2026-10-10): no
            // server-side wait on DELETE — the CLI polls GET until the
            // row is gone; 404 is the deletion-done receipt.
            if wait {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout);
                loop {
                    if std::time::Instant::now() >= deadline {
                        eprintln!("Purge expectation unmet: the systemtender is still visible after {}s", timeout);
                        std::process::exit(EXIT_PURGE_UNMET);
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    let poll = client.get_systemtender(&id).await;
                    if poll.status == Some(404) {
                        if matches!(output, OutputFormat::Text) {
                            println!("Systemtender gone: {}", id);
                        }
                        return;
                    }
                    if !poll.success {
                        write_error(poll.error.as_deref().unwrap_or("Unknown error"));
                    }
                    if let Some(row) = poll.data {
                        if let Some(reason) = row.deletion_reason.as_deref() {
                            eprintln!("Deletion state: {} ({})", row.status, reason);
                        }
                    }
                }
            }
        }
    }
}

async fn handle_credential_command(client: &GodonClient, cmd: CredentialCommands, output: &OutputFormat) {
    match cmd {
        CredentialCommands::List => {
            let response = client.list_credentials().await;
            if response.success {
                if let Some(credentials) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_credential_list(&credentials);
                    } else {
                        format_output(&credentials, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        CredentialCommands::Create { file } => {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => write_error(&format!("Failed to read file: {}", e)),
            };

            let response = client.create_credential_from_yaml(&content).await;
            if response.success {
                if let Some(credential) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_credential_created(&credential);
                    } else {
                        format_output(&credential, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        CredentialCommands::Show { id } => {
            let response = client.get_credential(&id).await;
            if response.success {
                if let Some(credential) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_credential(&credential);
                    } else {
                        format_output(&credential, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        CredentialCommands::Delete { id } => {
            let response = client.delete_credential(&id).await;
            if response.success {
                if matches!(output, OutputFormat::Text) {
                    println!("Credential deleted successfully: {}", id);
                } else if let Some(data) = response.data {
                    format_output(&data, output);
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }
    }
}

#[cfg(test)]
mod async_contract_tests {
    use super::{Cli, Commands, SystemtenderCommands};
    use clap::Parser;
    use godon_cli::{create_wait_met, EXIT_CREATE_UNMET, EXIT_ERROR, EXIT_NAME_TAKEN, EXIT_OK, EXIT_PURGE_UNMET};

    // ── arg parse: the wait/timeout surface (designs/2026-10-10) ──

    #[test]
    fn create_defaults_to_async_with_timeout_60() {
        let cli = Cli::try_parse_from([
            "godon_cli", "systemtender", "create",
            "--name", "bench", "--file", "cfg.yaml",
        ]).expect("parse");
        match cli.command {
            Commands::Systemtender { subcommand: SystemtenderCommands::Create { wait, timeout, .. } } => {
                assert!(!wait, "async is the default");
                assert_eq!(timeout, 60);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn create_parses_wait_and_timeout() {
        let cli = Cli::try_parse_from([
            "godon_cli", "systemtender", "create",
            "--name", "bench", "--file", "cfg.yaml",
            "--wait", "--timeout", "90",
        ]).expect("parse");
        match cli.command {
            Commands::Systemtender { subcommand: SystemtenderCommands::Create { wait, timeout, .. } } => {
                assert!(wait);
                assert_eq!(timeout, 90);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn purge_parses_wait_and_timeout() {
        let cli = Cli::try_parse_from([
            "godon_cli", "systemtender", "purge",
            "--id", "550e8400-e29b-41d4-a716-446655440000",
            "--wait", "--timeout", "30",
        ]).expect("parse");
        match cli.command {
            Commands::Systemtender { subcommand: SystemtenderCommands::Purge { wait, timeout, .. } } => {
                assert!(wait);
                assert_eq!(timeout, 30);
            }
            _ => panic!("wrong command"),
        }
    }

    // ── exit-code map: distinct numbers, scripts branch on them ──

    #[test]
    fn exit_codes_are_distinct() {
        let codes = [EXIT_OK, EXIT_ERROR, EXIT_NAME_TAKEN, EXIT_CREATE_UNMET, EXIT_PURGE_UNMET];
        for i in 0..codes.len() {
            for j in (i + 1)..codes.len() {
                assert_ne!(codes[i], codes[j], "exit codes must be distinct");
            }
        }
    }

    // ── the create wait verdict rides the state ──

    #[test]
    fn create_wait_verdicts() {
        // met: the row is live (active = post-create window, running =
        // first heartbeat landed, finished = it already walked)
        assert!(create_wait_met("active"));
        assert!(create_wait_met("running"));
        assert!(create_wait_met("finished"));
        // unmet: still creating at the bound, failed with a reason, or
        // the workers died (presumed_dead is not the asked-for active)
        assert!(!create_wait_met("creating"));
        assert!(!create_wait_met("create-failed"));
        assert!(!create_wait_met("presumed_dead"));
    }
}
