//! Port Forwarding / Tunneling Manager (`T2-TOOL-02`).
//! Supports Local, Remote, and Dynamic (SOCKS5 proxy) port forwarding over SSH.
//! Safe lifecycle management: guarantees clean closing upon exit or vault lock.

use crate::error::{CatermError, ValidationError};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ForwardType {
    Local,
    Remote,
    Dynamic, // SOCKS5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelRecord {
    pub id: String,
    pub host_id: String,
    pub forward_type: ForwardType,
    pub bind_addr: String,
    pub bind_port: u16,
    pub target_addr: String,
    pub target_port: u16,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInput {
    pub host_id: String,
    pub forward_type: ForwardType,
    pub bind_addr: Option<String>,
    pub bind_port: u16,
    pub target_addr: String,
    pub target_port: u16,
}

struct ActiveTunnel {
    shutdown_signal: Arc<AtomicBool>,
}

static ACTIVE_TUNNELS: Lazy<Arc<Mutex<HashMap<String, ActiveTunnel>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

fn generate_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("tun-{nanos:x}")
}

/// Create the `tunnels` database table if it doesn't already exist.
pub fn init_table(conn: &rusqlite::Connection) -> Result<(), CatermError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tunnels (
            id TEXT PRIMARY KEY,
            host_id TEXT NOT NULL,
            forward_type TEXT NOT NULL,
            bind_addr TEXT NOT NULL,
            bind_port INTEGER NOT NULL,
            target_addr TEXT NOT NULL,
            target_port INTEGER NOT NULL
        );",
    )
    .map_err(|e| {
        CatermError::Validation(ValidationError::Generic(format!(
            "Failed to create tunnels table: {e}"
        )))
    })?;
    Ok(())
}

/// List all configured tunnels along with active live state.
pub fn list_tunnels() -> Result<Vec<TunnelRecord>, CatermError> {
    let conn = crate::db::open()?;
    init_table(&conn)?;

    let mut stmt = conn
        .prepare("SELECT id, host_id, forward_type, bind_addr, bind_port, target_addr, target_port FROM tunnels")
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let active_map = ACTIVE_TUNNELS.lock();

    let rows = stmt
        .query_map([], |row| {
            let ftype_str: String = row.get(2)?;
            let forward_type = match ftype_str.as_str() {
                "Remote" => ForwardType::Remote,
                "Dynamic" => ForwardType::Dynamic,
                _ => ForwardType::Local,
            };

            let id: String = row.get(0)?;
            let is_active = active_map.contains_key(&id);

            Ok(TunnelRecord {
                id,
                host_id: row.get(1)?,
                forward_type,
                bind_addr: row.get(3)?,
                bind_port: row.get(4)?,
                target_addr: row.get(5)?,
                target_port: row.get(6)?,
                is_active,
            })
        })
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    let mut result = Vec::new();
    for r in rows {
        result
            .push(r.map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?);
    }
    Ok(result)
}

/// Save a tunnel configuration in the vault.
pub fn save_tunnel(input: TunnelInput) -> Result<TunnelRecord, CatermError> {
    let bind_addr = input.bind_addr.unwrap_or_else(|| "127.0.0.1".to_string());
    let forward_type_str = match input.forward_type {
        ForwardType::Local => "Local",
        ForwardType::Remote => "Remote",
        ForwardType::Dynamic => "Dynamic",
    };

    let id = generate_id();

    let conn = crate::db::open()?;
    init_table(&conn)?;

    conn.execute(
        "INSERT INTO tunnels (id, host_id, forward_type, bind_addr, bind_port, target_addr, target_port)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            input.host_id,
            forward_type_str,
            bind_addr,
            input.bind_port,
            input.target_addr,
            input.target_port
        ],
    )
    .map_err(|e| CatermError::Validation(ValidationError::Generic(format!("Failed to save tunnel: {e}"))))?;

    Ok(TunnelRecord {
        id,
        host_id: input.host_id,
        forward_type: input.forward_type,
        bind_addr,
        bind_port: input.bind_port,
        target_addr: input.target_addr,
        target_port: input.target_port,
        is_active: false,
    })
}

/// Delete a tunnel configuration and close if currently active.
pub fn delete_tunnel(id: &str) -> Result<(), CatermError> {
    let _ = stop_tunnel(id);

    let conn = crate::db::open()?;
    init_table(&conn)?;

    let affected = conn
        .execute("DELETE FROM tunnels WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| CatermError::Validation(ValidationError::Generic(e.to_string())))?;

    if affected == 0 {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "Tunnel '{id}' not found"
        ))));
    }

    Ok(())
}

/// A dedicated blocking session for a tunnel listener. Delegates to
/// [`crate::ssh::open_authenticated_session`] so a tunnel goes through the same TOFU host key
/// verification as every other connection — this used to be a hand-rolled copy that skipped it.
fn connect_host_session(host_id: &str) -> Result<ssh2::Session, CatermError> {
    let (sess, _host) = crate::ssh::open_authenticated_session(host_id)?;
    Ok(sess)
}

/// Start an active port forward tunnel listener.
pub fn start_tunnel(id: &str) -> Result<(), CatermError> {
    if ACTIVE_TUNNELS.lock().contains_key(id) {
        return Ok(()); // Already running
    }

    let tunnels = list_tunnels()?;
    let tun = tunnels
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| {
            CatermError::Validation(ValidationError::Generic(format!("Tunnel '{id}' not found")))
        })?
        .clone();

    let _ = crate::audit::log_event(
        "TUNNEL_START",
        Some(&tun.host_id),
        &format!(
            "Started tunnel ({:?}) on port {}",
            tun.forward_type, tun.bind_port
        ),
    );

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown_signal);

    match tun.forward_type {
        ForwardType::Local => {
            let bind_str = format!("{}:{}", tun.bind_addr, tun.bind_port);
            let listener = TcpListener::bind(&bind_str).map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Failed to bind local port {bind_str}: {e}"
                )))
            })?;

            listener.set_nonblocking(true).ok();

            let host_id = tun.host_id.clone();
            let target_addr = tun.target_addr.clone();
            let target_port = tun.target_port;

            let sess = connect_host_session(&host_id)?;
            let sess_arc = Arc::new(Mutex::new(sess));

            thread::spawn(move || {
                while !shutdown_clone.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut local_stream, _)) => {
                            let sess_inner = Arc::clone(&sess_arc);
                            let t_addr = target_addr.clone();
                            let shutdown_worker = Arc::clone(&shutdown_clone);

                            thread::spawn(move || {
                                let mut channel = {
                                    let guard = sess_inner.lock();
                                    match guard.channel_direct_tcpip(&t_addr, target_port, None) {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    }
                                };

                                local_stream.set_nonblocking(true).ok();
                                let mut buf = [0u8; 8192];

                                while !shutdown_worker.load(Ordering::Relaxed) {
                                    let mut active = false;

                                    // Local -> Remote
                                    match local_stream.read(&mut buf) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            active = true;
                                            if channel
                                                .write_all(buf.get(..n).unwrap_or(&[]))
                                                .is_err()
                                            {
                                                break;
                                            }
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                        Err(_) => break,
                                    }

                                    // Remote -> Local
                                    match channel.read(&mut buf) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            active = true;
                                            if local_stream
                                                .write_all(buf.get(..n).unwrap_or(&[]))
                                                .is_err()
                                            {
                                                break;
                                            }
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                        Err(_) => break,
                                    }

                                    if !active {
                                        thread::sleep(Duration::from_millis(5));
                                    }
                                }
                            });
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(_) => break,
                    }
                }
            });
        }
        ForwardType::Remote => {
            let sess = connect_host_session(&tun.host_id)?;
            let mut listener_channel = sess
                .channel_forward_listen(tun.bind_port, Some(&tun.bind_addr), None)
                .map_err(|e| {
                    CatermError::Validation(ValidationError::Generic(format!(
                        "Failed to remote bind: {e}"
                    )))
                })?;

            let target_addr = tun.target_addr.clone();
            let target_port = tun.target_port;

            thread::spawn(move || {
                while !shutdown_clone.load(Ordering::Relaxed) {
                    if let Ok(mut remote_stream) = listener_channel.0.accept() {
                        let t_addr = target_addr.clone();
                        let shutdown_worker = Arc::clone(&shutdown_clone);
                        thread::spawn(move || {
                            let mut local_stream =
                                match TcpStream::connect(format!("{}:{}", t_addr, target_port)) {
                                    Ok(s) => s,
                                    Err(_) => return,
                                };
                            local_stream.set_nonblocking(true).ok();
                            let mut buf = [0u8; 8192];
                            while !shutdown_worker.load(Ordering::Relaxed) {
                                let mut active = false;
                                match local_stream.read(&mut buf) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        active = true;
                                        if remote_stream
                                            .write_all(buf.get(..n).unwrap_or(&[]))
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                    Err(_) => break,
                                }
                                match remote_stream.read(&mut buf) {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        active = true;
                                        if local_stream
                                            .write_all(buf.get(..n).unwrap_or(&[]))
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                    Err(_) => break,
                                }
                                if !active {
                                    thread::sleep(Duration::from_millis(5));
                                }
                            }
                        });
                    } else {
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            });
        }
        ForwardType::Dynamic => {
            let bind_str = format!("{}:{}", tun.bind_addr, tun.bind_port);
            let listener = TcpListener::bind(&bind_str).map_err(|e| {
                CatermError::Validation(ValidationError::Generic(format!(
                    "Failed to bind local port {bind_str}: {e}"
                )))
            })?;
            listener.set_nonblocking(true).ok();

            let sess = connect_host_session(&tun.host_id)?;
            let sess_arc = Arc::new(Mutex::new(sess));

            thread::spawn(move || {
                while !shutdown_clone.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut local_stream, _)) => {
                            let sess_inner = Arc::clone(&sess_arc);
                            let shutdown_worker = Arc::clone(&shutdown_clone);

                            thread::spawn(move || {
                                local_stream.set_nonblocking(false).ok();
                                let mut buf = [0u8; 512];
                                let Some(head) = buf.get_mut(0..2) else {
                                    return;
                                };
                                if local_stream.read_exact(head).is_err()
                                    || head.first() != Some(&5)
                                {
                                    return;
                                }
                                let nmethods = head.get(1).copied().unwrap_or(0) as usize;
                                let Some(methods_buf) = buf.get_mut(0..nmethods) else {
                                    return;
                                };
                                if local_stream.read_exact(methods_buf).is_err() {
                                    return;
                                }
                                if local_stream.write_all(&[5, 0]).is_err() {
                                    return;
                                }

                                let Some(req_head) = buf.get_mut(0..4) else {
                                    return;
                                };
                                if local_stream.read_exact(req_head).is_err()
                                    || req_head.first() != Some(&5)
                                    || req_head.get(1) != Some(&1)
                                {
                                    return;
                                }
                                let atyp = req_head.get(3).copied().unwrap_or(0);
                                let target_host = if atyp == 1 {
                                    let Some(ip_buf) = buf.get_mut(0..4) else {
                                        return;
                                    };
                                    if local_stream.read_exact(ip_buf).is_err() {
                                        return;
                                    }
                                    format!(
                                        "{}.{}.{}.{}",
                                        ip_buf.first().copied().unwrap_or(0),
                                        ip_buf.get(1).copied().unwrap_or(0),
                                        ip_buf.get(2).copied().unwrap_or(0),
                                        ip_buf.get(3).copied().unwrap_or(0)
                                    )
                                } else if atyp == 3 {
                                    let Some(len_buf) = buf.get_mut(0..1) else {
                                        return;
                                    };
                                    if local_stream.read_exact(len_buf).is_err() {
                                        return;
                                    }
                                    let len = len_buf.first().copied().unwrap_or(0) as usize;
                                    let Some(domain_buf) = buf.get_mut(0..len) else {
                                        return;
                                    };
                                    if local_stream.read_exact(domain_buf).is_err() {
                                        return;
                                    }
                                    String::from_utf8_lossy(domain_buf).to_string()
                                } else {
                                    return;
                                };

                                let Some(port_buf) = buf.get_mut(0..2) else {
                                    return;
                                };
                                if local_stream.read_exact(port_buf).is_err() {
                                    return;
                                }
                                let target_port = u16::from_be_bytes([
                                    port_buf.first().copied().unwrap_or(0),
                                    port_buf.get(1).copied().unwrap_or(0),
                                ]);

                                let mut channel = {
                                    let guard = sess_inner.lock();
                                    match guard.channel_direct_tcpip(
                                        &target_host,
                                        target_port,
                                        None,
                                    ) {
                                        Ok(c) => c,
                                        Err(_) => return,
                                    }
                                };

                                if local_stream
                                    .write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0])
                                    .is_err()
                                {
                                    return;
                                }

                                local_stream.set_nonblocking(true).ok();
                                let mut buf2 = [0u8; 8192];
                                while !shutdown_worker.load(Ordering::Relaxed) {
                                    let mut active = false;
                                    match local_stream.read(&mut buf2) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            active = true;
                                            if channel
                                                .write_all(buf2.get(..n).unwrap_or(&[]))
                                                .is_err()
                                            {
                                                break;
                                            }
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                        Err(_) => break,
                                    }
                                    match channel.read(&mut buf2) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            active = true;
                                            if local_stream
                                                .write_all(buf2.get(..n).unwrap_or(&[]))
                                                .is_err()
                                            {
                                                break;
                                            }
                                        }
                                        Err(ref e)
                                            if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                        Err(_) => break,
                                    }
                                    if !active {
                                        thread::sleep(Duration::from_millis(5));
                                    }
                                }
                            });
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    }

    ACTIVE_TUNNELS
        .lock()
        .insert(id.to_string(), ActiveTunnel { shutdown_signal });
    Ok(())
}

/// Stop an active port forward tunnel listener.
pub fn stop_tunnel(id: &str) -> Result<(), CatermError> {
    if let Some(active) = ACTIVE_TUNNELS.lock().remove(id) {
        active.shutdown_signal.store(true, Ordering::Relaxed);

        if let Some(tun) = list_tunnels()
            .ok()
            .as_ref()
            .and_then(|tunnels| tunnels.iter().find(|t| t.id == id))
        {
            let _ = crate::audit::log_event(
                "TUNNEL_STOP",
                Some(&tun.host_id),
                &format!(
                    "Stopped tunnel ({:?}) on port {}",
                    tun.forward_type, tun.bind_port
                ),
            );
        }
    }
    Ok(())
}

/// Stop all active tunnels cleanly (e.g. called on vault lock or app exit).
pub fn stop_all_tunnels() -> Result<(), CatermError> {
    let mut tunnels = ACTIVE_TUNNELS.lock();
    for (_, tunnel) in tunnels.drain() {
        tunnel.shutdown_signal.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression for C-03: save_tunnel/list_tunnels/delete_tunnel used to open the database
    // with `vault::load_or_create_local_key` (a plaintext key file used nowhere else) instead
    // of the vault DEK every other module uses via `db::open()`. Against a database created by
    // any other module the mismatch failed with "file is not a database". Unlocking the vault
    // first, as the real app does before any of these calls, reproduces that setup.
    #[test]
    fn save_list_and_delete_tunnel_use_the_same_key_as_the_rest_of_the_app() {
        let _data = crate::test_support::isolated_data_dir("tunnels_roundtrip");
        crate::vault::unlock_vault("12345678").expect("unlock vault");

        let saved = save_tunnel(TunnelInput {
            host_id: "host-1".into(),
            forward_type: ForwardType::Local,
            bind_addr: None,
            bind_port: 2222,
            target_addr: "127.0.0.1".into(),
            target_port: 22,
        })
        .expect("save_tunnel should use the vault DEK, not a separate local key");

        let all = list_tunnels().expect("list_tunnels should open the same database");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, saved.id);
        assert_eq!(all[0].bind_port, 2222);

        delete_tunnel(&saved.id).expect("delete_tunnel should open the same database");
        assert!(
            list_tunnels()
                .expect("list_tunnels after delete")
                .is_empty()
        );
    }
}
