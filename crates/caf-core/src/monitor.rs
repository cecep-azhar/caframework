//! Server Monitoring Manager (`T2-TOOL-07`).
//! Polls resource usage (CPU, RAM, Disk, Uptime) across connected active hosts via SSH commands without installing agents.
//! Results are returned as JSON to be rendered by the frontend.
//!
//! Polling runs on a pooled non-interactive session ([`crate::ssh::with_exec_session`]) rather
//! than on a terminal pane's session. Reusing the PTY session here used to switch it to
//! blocking mode behind the reader thread's back, which froze every open terminal for that
//! host on the first poll tick.

use crate::error::CatermError;
use crate::ssh::SESSIONS;
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetrics {
    pub host_id: String,
    pub hostname: String,
    pub os_name: String,
    pub uptime: String,
    pub cpu_usage: f64,
    pub mem_total_mb: f64,
    pub mem_used_mb: f64,
    pub disk_total_gb: f64,
    pub disk_used_gb: f64,
}

fn require_non_empty(field: &str, value: &str) -> Result<(), CatermError> {
    if value.trim().is_empty() {
        return Err(CatermError::Validation(
            crate::error::ValidationError::Generic(format!("Field {} cannot be empty", field)),
        ));
    }
    Ok(())
}

fn ssh_err(message: String) -> CatermError {
    CatermError::Ssh(crate::error::SshError::Generic(message))
}

/// One-liner bash script to extract metrics safely without remote dependencies.
const METRICS_SCRIPT: &str = r#"
        set -e
        OS=$(uname -s)
        if [ "$OS" = "Linux" ]; then
            UPTIME=$(uptime -p | sed 's/up //')
            CPU_IDLE=$(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1}')
            MEM_INFO=$(free -m | grep Mem)
            MEM_TOTAL=$(echo "$MEM_INFO" | awk '{print $2}')
            MEM_USED=$(echo "$MEM_INFO" | awk '{print $3}')
            DISK_INFO=$(df -h / | tail -1)
            DISK_TOTAL=$(echo "$DISK_INFO" | awk '{print $2}' | sed 's/G//')
            DISK_USED=$(echo "$DISK_INFO" | awk '{print $3}' | sed 's/G//')
            HOSTNAME=$(hostname)
            OS_NAME=$(cat /etc/os-release | grep "^PRETTY_NAME=" | cut -d'"' -f2 || echo "Linux")
            echo "$HOSTNAME|$OS_NAME|$UPTIME|$CPU_IDLE|$MEM_TOTAL|$MEM_USED|$DISK_TOTAL|$DISK_USED"
        elif [ "$OS" = "Darwin" ]; then
            UPTIME=$(uptime | awk -F'( |,|:)+' '{print $6 " hrs " $7 " mins"}')
            CPU_IDLE=$(top -l 1 | grep -E "^CPU" | awk '{print $7}' | sed 's/%//')
            MEM_INFO=$(vm_stat)
            MEM_TOTAL=$(sysctl -n hw.memsize | awk '{print $1 / 1024 / 1024}')
            MEM_USED=$(echo "$MEM_INFO" | grep "Pages active" | awk '{print $3}' | sed 's/\.//' | awk '{print $1 * 4096 / 1024 / 1024}')
            DISK_INFO=$(df -h / | tail -1)
            DISK_TOTAL=$(echo "$DISK_INFO" | awk '{print $2}' | sed 's/Gi//')
            DISK_USED=$(echo "$DISK_INFO" | awk '{print $3}' | sed 's/Gi//')
            HOSTNAME=$(hostname)
            OS_NAME="macOS $(sw_vers -productVersion)"
            echo "$HOSTNAME|$OS_NAME|$UPTIME|$CPU_IDLE|$MEM_TOTAL|$MEM_USED|$DISK_TOTAL|$DISK_USED"
        else
            echo "unknown|unknown|0|0|0|0|0|0"
        fi
    "#;

/// Parses the single pipe-delimited line `METRICS_SCRIPT` prints.
fn parse_metrics(host_id: &str, raw: &str) -> Result<HostMetrics, CatermError> {
    let output = raw.trim();
    let parts: Vec<&str> = output.split('|').collect();

    let [
        hostname,
        os_name,
        uptime,
        cpu,
        mem_total,
        mem_used,
        disk_total,
        disk_used,
    ] = parts.as_slice()
    else {
        return Err(ssh_err(format!("Invalid metrics response: {output}")));
    };

    Ok(HostMetrics {
        host_id: host_id.to_string(),
        hostname: hostname.to_string(),
        os_name: os_name.to_string(),
        uptime: uptime.to_string(),
        cpu_usage: cpu.parse::<f64>().unwrap_or(0.0),
        mem_total_mb: mem_total.parse::<f64>().unwrap_or(0.0),
        mem_used_mb: mem_used.parse::<f64>().unwrap_or(0.0),
        disk_total_gb: disk_total.parse::<f64>().unwrap_or(0.0),
        disk_used_gb: disk_used.parse::<f64>().unwrap_or(0.0),
    })
}

pub fn fetch_metrics_for_host(host_id: &str) -> Result<HostMetrics, CatermError> {
    require_non_empty("host_id", host_id)?;

    let output = crate::ssh::with_exec_session(host_id, |sess| {
        let mut channel = sess
            .channel_session()
            .map_err(|e| ssh_err(format!("Failed to open SSH monitoring channel: {e}")))?;

        channel
            .exec(METRICS_SCRIPT)
            .map_err(|e| ssh_err(format!("Failed to execute monitoring script: {e}")))?;

        let mut raw = String::new();
        channel.read_to_string(&mut raw).unwrap_or_default();
        channel.wait_close().unwrap_or_default();
        Ok(raw)
    })?;

    parse_metrics(host_id, &output)
}

/// Polls every host that currently has an open terminal pane. The metrics themselves travel
/// over that host's pooled exec session, not the pane's.
pub fn poll_active_metrics() -> Result<Vec<HostMetrics>, CatermError> {
    let active_host_ids: Vec<String> = {
        let sessions = SESSIONS
            .lock()
            .map_err(|_| ssh_err("Lock poisoned".to_string()))?;
        let mut ids: Vec<String> = sessions.values().map(|m| m.host_id.clone()).collect();
        ids.sort();
        ids.dedup();
        ids
    };

    let mut results = Vec::new();
    for hid in active_host_ids.into_iter() {
        if hid == "local" || hid == "__local__" {
            continue;
        }
        if let Ok(metrics) = fetch_metrics_for_host(&hid) {
            results.push(metrics);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_metrics_rejects_empty_host_id() {
        assert!(fetch_metrics_for_host("").is_err());
    }

    #[test]
    fn parse_metrics_reads_a_well_formed_line() {
        let metrics = parse_metrics(
            "h1",
            "  ypc|Debian GNU/Linux 12|3 days|12.5|7861|2210|98.4|41.2
",
        )
        .expect("well-formed line should parse");
        assert_eq!(metrics.hostname, "ypc");
        assert_eq!(metrics.os_name, "Debian GNU/Linux 12");
        assert_eq!(metrics.cpu_usage, 12.5);
        assert_eq!(metrics.disk_used_gb, 41.2);
    }

    #[test]
    fn parse_metrics_rejects_a_truncated_line() {
        assert!(parse_metrics("h1", "ypc|Debian|3 days").is_err());
    }
}
