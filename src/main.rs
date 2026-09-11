use clap::{Parser, Subcommand};
use godon_cli::{Steerwish, SteerwishSummary, Systemtender, SystemtenderSummary, Credential, GodonClient, Target};
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
        println!("  ID: {}", steerwish.id);
        println!("  Outcome: {}", steerwish.outcome);
        println!("  State: {}", steerwish.state);
        println!("  Created: {}", steerwish.created_at);
        println!();
    }
}

fn format_steerwish(steerwish: &Steerwish) {
    println!("  ID: {}", steerwish.id);
    println!("  Outcome: {}", steerwish.outcome);
    println!("  State: {}", steerwish.state);
    println!(
        "  Band: [{}, {}]",
        steerwish.band.lo, steerwish.band.hi
    );
    if let Some(target) = steerwish.band.target {
        println!("  Target: {} (receipt only)", target);
    }
    if let Some(limits) = &steerwish.limits {
        if let Some(exclude) = &limits.exclude {
            println!("  Excluded inputs: {:?}", exclude);
        }
        if let Some(max_change) = limits.max_change {
            println!("  Max change: {} of range from neutral", max_change);
        }
    }
    println!(
        "  Budget: {}",
        steerwish
            .budget
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unbounded".to_string())
    );
    println!("  Regime: {}", steerwish.regime.as_deref().unwrap_or("standing"));
    println!("  Created: {}", steerwish.created_at);
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

        SystemtenderCommands::Create { name, file } => {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => write_error(&format!("Failed to read file: {}", e)),
            };

            let response = client.create_systemtender_from_yaml(&content, &name).await;
            if response.success {
                if let Some(systemtender) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_systemtender_summary(&systemtender);
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

        SystemtenderCommands::Purge { id, force } => {
            let response = client.delete_systemtender(&id, force).await;
            if response.success {
                if matches!(output, OutputFormat::Text) {
                    if force {
                        println!("Systemtender force deleted (workers cancelled): {}", id);
                    } else {
                        println!("Systemtender deleted: {}", id);
                    }
                } else if let Some(data) = response.data {
                    format_output(&data, output);
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
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
