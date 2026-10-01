mod bench;

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
    /// Print the resolved runtime data directory and store location (REQ-30).
    Env(JsonFlag),
    /// Error code registry (REQ-04)
    Errors {
        #[command(subcommand)]
        action: ErrorsAction,
    },
    /// Resource budget harness (REQ-02, T2-BOOT-06).
    Bench {
        #[command(subcommand)]
        action: BenchAction,
    },
    Host {
        #[command(subcommand)]
        action: HostAction,
    },
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
    Connect {
        host_id: String,
    },
    Tunnel {
        #[command(subcommand)]
        action: TunnelAction,
    },
    Monitor {
        host_id: String,
    },
    Key {
        #[command(subcommand)]
        action: KeyAction,
    },
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },
}

#[derive(Subcommand, Debug)]
enum HostAction {
    List,
    Add {
        #[arg(long)]
        label: String,
        #[arg(long)]
        address: String,
        #[arg(long)]
        port: u16,
        #[arg(long)]
        username: String,
        #[arg(long, help = "password, key-path=..., or key-id=...")]
        auth_method: String,
        #[arg(long, default_value = "")]
        secret: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    Rm {
        id: String,
    },
    Connect {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum VaultAction {
    Unlock { password: String },
    Lock,
    Status,
}

#[derive(Subcommand, Debug)]
enum TunnelAction {
    Start { id: String },
    Stop { id: String },
    List,
}

#[derive(Subcommand, Debug)]
enum KeyAction {
    List,
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        algorithm: String,
    },
    Deploy {
        #[arg(long)]
        host_id: String,
        #[arg(long)]
        key_id: String,
    },
}

#[derive(Subcommand, Debug)]
enum AuditAction {
    Tail {
        #[arg(long)]
        host_id: Option<String>,
        #[arg(long)]
        search: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum BenchAction {
    /// Build a full report: binary size, cold start, RSS (main + descendant WebView procs).
    Run {
        /// Path to the caf-app release binary. Defaults to the sibling binary next to
        /// this caframeworkctl executable (cargo places all workspace bins in the same dir).
        #[arg(long)]
        exe: Option<PathBuf>,
        /// Seconds to wait after cold-start before snapshotting RSS, so WebView child
        /// processes finish spawning.
        #[arg(long, default_value_t = 3)]
        settle_secs: u64,
        /// Seconds to wait for the CATERM_COLD_START_MS line before giving up.
        #[arg(long, default_value_t = 15)]
        timeout_secs: u64,
        /// Emit machine-readable JSON instead of a human table.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args, Debug)]
struct JsonFlag {
    /// Emit machine-readable JSON (pretty-printed, one field per line) instead of plain text.
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum ErrorsAction {
    /// List every currently-defined error code, domain, and message.
    List,
}

#[derive(Serialize)]
struct EnvInfo {
    data_dir: String,
    source: String,
    platform: String,
    version: &'static str,
    db_path: String,
    db_exists: bool,
    db_size_bytes: u64,
}

fn parse_auth_method(val: &str) -> Result<caf_core::store::AuthMethod, String> {
    if val == "password" {
        Ok(caf_core::store::AuthMethod::Password)
    } else if let Some(path) = val.strip_prefix("key-path=") {
        Ok(caf_core::store::AuthMethod::Key {
            path: path.to_string(),
        })
    } else if let Some(id) = val.strip_prefix("key-id=") {
        Ok(caf_core::store::AuthMethod::KeyId { id: id.to_string() })
    } else {
        Err(format!("invalid auth method: {val}"))
    }
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match cli.command {
        Some(Commands::Env(flags)) => {
            print_env(flags.json);
            0
        }
        Some(Commands::Errors {
            action: ErrorsAction::List,
        }) => {
            print_errors_list();
            0
        }
        Some(Commands::Bench {
            action:
                BenchAction::Run {
                    exe,
                    settle_secs,
                    timeout_secs,
                    json,
                },
        }) => run_bench(exe, settle_secs, timeout_secs, json),
        Some(Commands::Host { action }) => handle_host(action),
        Some(Commands::Vault { action }) => handle_vault(action),
        Some(Commands::Connect { host_id }) => handle_connect(&host_id),
        Some(Commands::Tunnel { action }) => handle_tunnel(action),
        Some(Commands::Monitor { host_id }) => handle_monitor(&host_id),
        Some(Commands::Key { action }) => handle_key(action),
        Some(Commands::Audit { action }) => handle_audit(action),
        None => 0,
    };

    #[allow(clippy::disallowed_methods)]
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

fn run_bench(exe: Option<PathBuf>, settle_secs: u64, timeout_secs: u64, as_json: bool) -> i32 {
    let exe_path = match exe.or_else(|| bench::default_exe_path().ok()) {
        Some(p) => p,
        None => {
            eprintln!("gagal menentukan path exe caf-app; pakai --exe");
            return 1;
        }
    };

    match bench::run(&exe_path, settle_secs, timeout_secs) {
        Ok(report) => {
            if as_json {
                match serde_json::to_string_pretty(&report) {
                    Ok(j) => println!("{j}"),
                    Err(e) => eprintln!("gagal serialize bench report: {e}"),
                }
            } else {
                println!("binary_size_bytes={}", report.binary_size_bytes);
                println!(
                    "binary_size_mb={:.2}",
                    report.binary_size_bytes as f64 / 1_048_576.0
                );
                println!("cold_start_ms={}", report.cold_start_ms);
                match report.private_bytes {
                    Some(b) => {
                        println!("private_bytes={b}   <-- ANGKA BUDGET REQ-02");
                        println!("private_mb={:.2}", b as f64 / 1_048_576.0);
                    }
                    None => {
                        println!("private_bytes=null  <-- platform ini belum didukung");
                        println!("private_mb=null");
                    }
                }
                println!(
                    "working_set_sum_bytes={}  (transparansi saja - double-count halaman bersama)",
                    report.working_set_sum_bytes
                );
                println!(
                    "working_set_sum_mb={:.2}",
                    report.working_set_sum_bytes as f64 / 1_048_576.0
                );
                println!("process_count={}", report.process_count);
                println!("db_query_ms=null");
                println!("db_query_note={}", report.db_query_note);
                println!("memory_note={}", report.memory_note);
            }
            0
        }
        Err(e) => {
            eprintln!("bench gagal: {e}");
            1
        }
    }
}

fn print_env(as_json: bool) {
    let Ok(dir_info) = caf_core::paths::resolve_data_dir() else {
        eprintln!("gagal menentukan direktori data");
        return;
    };
    let db_path = caf_core::paths::db_path(&dir_info.path);
    let metadata = std::fs::metadata(&db_path);

    let info = EnvInfo {
        data_dir: dir_info.path.display().to_string(),
        source: dir_info.source.as_str().to_string(),
        platform: std::env::consts::OS.to_string(),
        version: caf_core::CORE_VERSION,
        db_path: db_path.display().to_string(),
        db_exists: metadata.is_ok(),
        db_size_bytes: metadata.map(|m| m.len()).unwrap_or(0),
    };

    if as_json {
        match serde_json::to_string_pretty(&info) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("gagal serialize env info: {e}"),
        }
        return;
    }

    println!("data_dir={}", info.data_dir);
    println!("source={}", info.source);
    println!("platform={}", info.platform);
    println!("version={}", info.version);
    println!("db_path={}", info.db_path);
    println!("db_exists={}", info.db_exists);
    println!("db_size_bytes={}", info.db_size_bytes);
}

fn print_errors_list() {
    for err in caf_core::CatermError::all_known_for_registry() {
        println!("{}\t{}\t{}", err.code(), err.domain(), err.user_message());
    }
}

fn handle_host(action: HostAction) -> i32 {
    match action {
        HostAction::List => match caf_core::store::list_hosts() {
            Ok(hosts) => {
                for h in hosts {
                    println!("{} | {} | {}@{}", h.id, h.label, h.username, h.address);
                }
                0
            }
            Err(e) => {
                eprintln!("Error: {}", e.user_message());
                1
            }
        },
        HostAction::Add {
            label,
            address,
            port,
            username,
            auth_method,
            secret,
            tags,
        } => {
            let auth = match parse_auth_method(&auth_method) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return 1;
                }
            };
            let input = caf_core::store::HostInput {
                id: None,
                label,
                address,
                port,
                username,
                auth_method: auth,
                tags,
                os: None,
                protocol: Some(caf_core::store::ConnectionProtocol::Ssh),
                secret: if secret.is_empty() {
                    None
                } else {
                    Some(secret)
                },
            };
            match caf_core::store::save_host(input) {
                Ok(h) => {
                    println!("Host saved: {}", h.id);
                    0
                }
                Err(e) => {
                    eprintln!("Error: {}", e.user_message());
                    1
                }
            }
        }
        HostAction::Rm { id } => match caf_core::store::delete_host(&id) {
            Ok(_) => {
                println!("Host deleted");
                0
            }
            Err(e) => {
                eprintln!("Error: {}", e.user_message());
                1
            }
        },
        HostAction::Connect { id } => handle_connect(&id),
    }
}

fn handle_vault(action: VaultAction) -> i32 {
    match action {
        VaultAction::Unlock { password } => match caf_core::vault::unlock_vault(&password) {
            Ok(_) => {
                println!("Vault unlocked");
                0
            }
            Err(e) => {
                eprintln!("Error: {}", e.user_message());
                1
            }
        },
        VaultAction::Lock => match caf_core::vault::lock_vault() {
            Ok(_) => {
                println!("Vault locked");
                0
            }
            Err(e) => {
                eprintln!("Error: {}", e.user_message());
                1
            }
        },
        VaultAction::Status => match caf_core::vault::is_unlocked() {
            Ok(true) => {
                println!("Vault is UNLOCKED");
                0
            }
            Ok(false) => {
                println!("Vault is LOCKED");
                0
            }
            Err(e) => {
                eprintln!("Error: {}", e.user_message());
                1
            }
        },
    }
}

fn handle_connect(host_id: &str) -> i32 {
    match caf_core::ssh::connect(host_id) {
        Ok(session) => {
            println!(
                "Connected to {}, session id: {}",
                host_id, session.session_id
            );
            let _ = caf_core::ssh::disconnect(&session.session_id);
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e.user_message());
            1
        }
    }
}

fn handle_tunnel(action: TunnelAction) -> i32 {
    match action {
        TunnelAction::Start { id } => match caf_core::tunnels::start_tunnel(&id) {
            Ok(_) => {
                println!("Tunnel {} started", id);
                0
            }
            Err(e) => {
                eprintln!("Error: {}", e.user_message());
                1
            }
        },
        TunnelAction::Stop { id } => match caf_core::tunnels::stop_tunnel(&id) {
            Ok(_) => {
                println!("Tunnel {} stopped", id);
                0
            }
            Err(e) => {
                eprintln!("Error: {}", e.user_message());
                1
            }
        },
        TunnelAction::List => match caf_core::tunnels::list_tunnels() {
            Ok(tunnels) => {
                for t in tunnels {
                    println!(
                        "{} | {} | {:?} | bind:{}:{} | target:{}:{} | active:{}",
                        t.id,
                        t.host_id,
                        t.forward_type,
                        t.bind_addr,
                        t.bind_port,
                        t.target_addr,
                        t.target_port,
                        t.is_active
                    );
                }
                0
            }
            Err(e) => {
                eprintln!("Error: {}", e.user_message());
                1
            }
        },
    }
}

fn handle_monitor(host_id: &str) -> i32 {
    match caf_core::monitor::fetch_metrics_for_host(host_id) {
        Ok(metrics) => {
            println!("CPU: {:.2}%", metrics.cpu_usage);
            println!(
                "Memory: {:.2} MB / {:.2} MB",
                metrics.mem_used_mb, metrics.mem_total_mb
            );
            println!(
                "Disk: {:.2} GB / {:.2} GB",
                metrics.disk_used_gb, metrics.disk_total_gb
            );
            println!("Uptime: {}", metrics.uptime);
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e.user_message());
            1
        }
    }
}

fn handle_key(action: KeyAction) -> i32 {
    match action {
        KeyAction::List => match caf_core::keys::list_keys() {
            Ok(keys) => {
                for k in keys {
                    println!("{} | {} | {}", k.id, k.name, k.algorithm);
                }
                0
            }
            Err(e) => {
                eprintln!("Error: {}", e.user_message());
                1
            }
        },
        KeyAction::Add { name, algorithm } => {
            let input = caf_core::keys::KeyInput { name, algorithm };
            match caf_core::keys::generate_key(input) {
                Ok(k) => {
                    println!("Key generated: {}", k.id);
                    0
                }
                Err(e) => {
                    eprintln!("Error: {}", e.user_message());
                    1
                }
            }
        }
        KeyAction::Deploy { host_id, key_id } => {
            match caf_core::keys::deploy_public_key(&host_id, &key_id) {
                Ok(_) => {
                    println!("Key deployed to host {}", host_id);
                    0
                }
                Err(e) => {
                    eprintln!("Error: {}", e.user_message());
                    1
                }
            }
        }
    }
}

fn handle_audit(action: AuditAction) -> i32 {
    match action {
        AuditAction::Tail { host_id, search } => {
            match caf_core::audit::get_logs(host_id.as_deref(), search.as_deref(), None) {
                Ok(logs) => {
                    for l in logs {
                        println!(
                            "[{}] {} - {}: {}",
                            l.timestamp,
                            l.host_id.unwrap_or_default(),
                            l.event_type,
                            l.details
                        );
                    }
                    0
                }
                Err(e) => {
                    eprintln!("Error: {}", e.user_message());
                    1
                }
            }
        }
    }
}
