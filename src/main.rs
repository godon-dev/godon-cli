use clap::{Parser, Subcommand};
use godon_cli::{Breeder, BreederSummary, Credential, GodonClient};
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
    Breeder {
        #[command(subcommand)]
        subcommand: BreederCommands,
    },
    Credential {
        #[command(subcommand)]
        subcommand: CredentialCommands,
    },
}

#[derive(Subcommand)]
enum BreederCommands {
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
        file: PathBuf,
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

fn format_breeder_list(breeders: &[BreederSummary]) {
    println!("Breeders:");
    for breeder in breeders {
        println!("  ID: {}", breeder.id);
        println!("  Name: {}", breeder.name);
        println!("  Status: {}", breeder.status);
        println!("  Created: {}", breeder.created_at);
        println!("  ---");
    }
}

fn format_breeder(breeder: &Breeder) {
    println!("Breeder Details:");
    println!("  ID: {}", breeder.id);
    println!("  Name: {}", breeder.name);
    println!("  Status: {}", breeder.status);
    println!("  Config: {}", serde_json::to_string_pretty(&breeder.config).unwrap_or_default());
    println!("  Created: {}", breeder.created_at);
}

fn format_breeder_summary(breeder: &BreederSummary) {
    println!("Breeder created successfully:");
    println!("  ID: {}", breeder.id);
    println!("  Name: {}", breeder.name);
    println!("  Status: {}", breeder.status);
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
        Commands::Breeder { subcommand } => handle_breeder_command(&client, subcommand, &cli.output).await,
        Commands::Credential { subcommand } => handle_credential_command(&client, subcommand, &cli.output).await,
    }
}

async fn handle_breeder_command(client: &GodonClient, cmd: BreederCommands, output: &OutputFormat) {
    match cmd {
        BreederCommands::List => {
            let response = client.list_breeders().await;
            if response.success {
                if let Some(breeders) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_breeder_list(&breeders);
                    } else {
                        format_output(&breeders, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        BreederCommands::Create { name, file } => {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => write_error(&format!("Failed to read file: {}", e)),
            };

            let response = client.create_breeder_from_yaml(&content, &name).await;
            if response.success {
                if let Some(breeder) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_breeder_summary(&breeder);
                    } else {
                        format_output(&breeder, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        BreederCommands::Show { id } => {
            let response = client.get_breeder(&id).await;
            if response.success {
                if let Some(breeder) = response.data {
                    if matches!(output, OutputFormat::Text) {
                        format_breeder(&breeder);
                    } else {
                        format_output(&breeder, output);
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        BreederCommands::Update { file } => {
            let content = match std::fs::read_to_string(&file) {
                Ok(c) => c,
                Err(e) => write_error(&format!("Failed to read file: {}", e)),
            };

            let response = client.update_breeder_from_yaml(&content).await;
            if response.success {
                if let Some(data) = response.data {
                    match data.get("id").and_then(|v| v.as_str()) {
                        Some(id) => {
                            if matches!(output, OutputFormat::Text) {
                                println!("Breeder updated successfully: {}", id);
                            } else {
                                format_output(&data, output);
                            }
                        }
                        None => write_error("Unexpected response format: missing 'id' field"),
                    }
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        BreederCommands::Stop { id } => {
            let response = client.stop_breeder(&id).await;
            if response.success {
                if matches!(output, OutputFormat::Text) {
                    println!("Breeder stop requested (graceful shutdown): {}", id);
                    println!("Workers will finish current trial before stopping.");
                } else if let Some(data) = response.data {
                    format_output(&data, output);
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        BreederCommands::Start { id } => {
            let response = client.start_breeder(&id).await;
            if response.success {
                if matches!(output, OutputFormat::Text) {
                    println!("Breeder started/resumed: {}", id);
                } else if let Some(data) = response.data {
                    format_output(&data, output);
                }
            } else {
                write_error(response.error.as_deref().unwrap_or("Unknown error"));
            }
        }

        BreederCommands::Purge { id, force } => {
            let response = client.delete_breeder(&id, force).await;
            if response.success {
                if matches!(output, OutputFormat::Text) {
                    if force {
                        println!("Breeder force deleted (workers cancelled): {}", id);
                    } else {
                        println!("Breeder deleted: {}", id);
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
