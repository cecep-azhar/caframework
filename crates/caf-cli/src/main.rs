use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

/// caframeworkctl — headless verification CLI for CAFramework v2.
#[derive(Parser, Debug)]
#[command(name = "caframeworkctl", version = caf_core::CORE_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print the resolved runtime data directory and store location.
    Env(JsonFlag),
    /// Error code registry.
    Errors {
        #[command(subcommand)]
        action: ErrorsAction,
    },
    /// Vault management.
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
    /// Profiles management.
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Notes sample slice management.
    Notes {
        #[command(subcommand)]
        action: NotesAction,
    },
}

#[derive(Subcommand, Debug)]
enum ErrorsAction {
    List,
}

#[derive(Subcommand, Debug)]
enum VaultAction {
    Unlock { password: String },
    Lock,
    Status,
}

#[derive(Subcommand, Debug)]
enum ProfileAction {
    List,
    Add {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "owner")]
        role: String,
        #[arg(long)]
        pin: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum NotesAction {
    List,
    Add {
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        content: String,
        #[arg(long, default_value = "shared")]
        visibility: String,
    },
}

#[derive(Args, Debug, Default)]
struct JsonFlag {
    #[arg(long)]
    json: bool,
}

#[derive(Serialize)]
struct EnvOutput {
    data_dir: PathBuf,
    db_path: PathBuf,
    source: String,
    device_id: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Env(flag)) => {
            let info = caf_core::paths::resolve_data_dir()?;
            let db = caf_core::paths::db_path(&info.path);
            let device_id = caf_core::paths::device_id().unwrap_or_else(|_| "unknown".into());

            if flag.json {
                let out = EnvOutput {
                    data_dir: info.path,
                    db_path: db,
                    source: info.source.as_str().to_string(),
                    device_id,
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("Data Directory: {}", info.path.display());
                println!("Database Path:  {}", db.display());
                println!("Source:         {}", info.source.as_str());
                println!("Device ID:      {}", device_id);
            }
        }
        Some(Commands::Errors {
            action: ErrorsAction::List,
        }) => {
            println!("CAF-001: General Error");
            println!("CAF-101: Vault Authentication Failed");
            println!("CAF-102: Vault Not Initialized");
            println!("CAF-201: Profile Not Found");
            println!("CAF-301: Storage Encryption Error");
        }
        Some(Commands::Vault { action }) => match action {
            VaultAction::Status => {
                let initialized = caf_core::vault::is_vault_initialized()?;
                let is_unlocked = caf_core::vault::is_unlocked()?;
                println!("Vault Initialized: {initialized}");
                println!("Vault Unlocked:    {is_unlocked}");
            }
            VaultAction::Unlock { password } => {
                let ok = caf_core::vault::validate_password(&password)?;
                if ok {
                    println!("Vault unlocked successfully.");
                } else {
                    eprintln!("Invalid master password.");
                    std::process::exit(1);
                }
            }
            VaultAction::Lock => {
                caf_core::vault::lock();
                println!("Vault locked.");
            }
        },
        Some(Commands::Profile { action }) => match action {
            ProfileAction::List => {
                let list = caf_core::profiles::list_profiles()?;
                for p in list {
                    println!("- [{}] {} ({}) [PIN: {}]", p.id, p.name, p.role, p.has_pin);
                }
            }
            ProfileAction::Add { name, role, pin } => {
                let profile = caf_core::profiles::save_profile(
                    caf_core::profiles::ProfileInput {
                        id: None,
                        name,
                        role,
                        avatar: None,
                        pin,
                    },
                    "cli-root",
                )?;
                println!("Created profile: {} ({})", profile.name, profile.id);
            }
        },
        Some(Commands::Notes { action }) => match action {
            NotesAction::List => {
                let notes = caf_core::notes::list_notes("cli-root", true)?;
                for n in notes {
                    println!("- [{}] {} (visibility: {})", n.id, n.title, n.visibility);
                }
            }
            NotesAction::Add {
                title,
                content,
                visibility,
            } => {
                let note = caf_core::notes::save_note(
                    caf_core::notes::NoteInput {
                        id: None,
                        title,
                        content,
                        tags: vec![],
                        visibility: Some(visibility),
                        owner_profile_id: "cli-root".to_string(),
                    },
                    "cli-root",
                )?;
                println!("Created note: {} ({})", note.title, note.id);
            }
        },
        None => {
            println!("caframeworkctl v{}", caf_core::CORE_VERSION);
            println!("Use --help for available subcommands.");
        }
    }

    Ok(())
}
