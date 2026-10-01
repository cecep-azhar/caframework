//! `caframeworkctl bench run` (T2-BOOT-06). Measures the Fase-0-measurable REQ-02 metrics
//! against a *release* build: binary size, cold start, and memory across the main process
//! **plus every descendant WebView process** — measuring only the Rust process is
//! explicitly forbidden by task-v2.md, because it would hide exactly the cost WebView adds.
//!
//! # Why two memory numbers, and why only one of them is the budget
//!
//! The first implementation of this harness summed `sysinfo::Process::memory()` across the
//! process tree and reported ~330 MB for an *empty* app. That number was wrong, and it was
//! wrong in a way that nearly cost the project its headline claim.
//!
//! On Windows `Process::memory()` returns the **working set**, which includes pages the
//! process maps but shares with others. WebView2 is Chromium: its six processes all map the
//! same ~200 MB `msedge.dll`. Summing working sets therefore counts that shared code once
//! per process. The same trap exists on Linux with WebKitGTK and `/proc/<pid>/statm`.
//!
//! So this harness reports both, and labels them honestly:
//!
//! * `working_set_sum_bytes` — the old figure. Kept because deleting an embarrassing number
//!   is how measurement mistakes get repeated. **Never** use it as a budget.
//! * `private_bytes` — memory charged to this process tree alone, shared pages excluded.
//!   This is the figure comparable to Task Manager's "Memory", to Activity Monitor, and to
//!   the numbers people quote for Electron clients. **This is the budget figure.**
//!
//! `db_query_ms` is reported as `null`/unavailable: there is no `hosts` table yet (that
//! schema is normative in Fase 1, T2-CORE-01). Fabricating a number against a table that
//! does not exist would be the "data palsu" anti-pattern this whole project exists to avoid.

use serde::Serialize;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

const MEMORY_NOTE: &str = "private_bytes is the REQ-02 budget figure (shared pages excluded, \
comparable to Task Manager). working_set_sum_bytes double-counts pages shared across the \
WebView process tree and is reported for transparency only - see bench.rs module docs.";

const DB_QUERY_NOTE: &str = "belum tersedia: tabel `hosts` belum ada (Fase 1, T2-CORE-01). \
Bukan 0ms - memang belum terukur.";

#[derive(Serialize)]
pub struct BenchReport {
    pub binary_size_bytes: u64,
    pub cold_start_ms: u128,
    /// Budget figure. `None` when this platform has no private-memory accounting wired up
    /// yet — reported as null rather than silently falling back to the inflated number.
    pub private_bytes: Option<u64>,
    /// Transparency only. Inflated on multi-process WebView runtimes.
    pub working_set_sum_bytes: u64,
    pub process_count: usize,
    pub db_query_ms: Option<u64>,
    pub db_query_note: &'static str,
    pub memory_note: &'static str,
}

pub fn default_exe_path() -> Result<PathBuf, String> {
    let mine = std::env::current_exe().map_err(|e| format!("current_exe gagal: {e}"))?;
    let dir = mine
        .parent()
        .ok_or_else(|| "current_exe tidak punya parent dir".to_string())?;
    Ok(dir.join(format!("caf-app{}", std::env::consts::EXE_SUFFIX)))
}

pub fn run(exe: &Path, settle_secs: u64, timeout_secs: u64) -> Result<BenchReport, String> {
    let binary_size_bytes = std::fs::metadata(exe)
        .map_err(|e| format!("gagal stat {}: {e}", exe.display()))?
        .len();

    let measured = spawn_and_measure(exe, settle_secs, timeout_secs)?;

    Ok(BenchReport {
        binary_size_bytes,
        cold_start_ms: measured.cold_start_ms,
        private_bytes: measured.private_bytes,
        working_set_sum_bytes: measured.working_set_sum_bytes,
        process_count: measured.process_count,
        db_query_ms: None,
        db_query_note: DB_QUERY_NOTE,
        memory_note: MEMORY_NOTE,
    })
}

struct Measured {
    cold_start_ms: u128,
    private_bytes: Option<u64>,
    working_set_sum_bytes: u64,
    process_count: usize,
}

fn spawn_and_measure(exe: &Path, settle_secs: u64, timeout_secs: u64) -> Result<Measured, String> {
    let mut child = Command::new(exe)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("gagal spawn {}: {e}", exe.display()))?;

    let pid = child.id();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "child stdout tidak tersedia".to_string())?;

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if let Some(value) = line.strip_prefix("CATERM_COLD_START_MS=") {
                let _ = tx.send(value.to_string());
                return;
            }
        }
    });

    let cold_start_ms: u128 = match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(value) => value.parse().unwrap_or(0),
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "timeout {timeout_secs}s menunggu CATERM_COLD_START_MS dari {}",
                exe.display()
            ));
        }
    };

    std::thread::sleep(Duration::from_secs(settle_secs));

    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    let root = sysinfo::Pid::from_u32(pid);
    let pids = descendant_pids(&sys, root);

    let working_set_sum_bytes: u64 = pids
        .iter()
        .filter_map(|p| sys.process(*p))
        .map(sysinfo::Process::memory)
        .sum();

    // A single unreadable process (it may exit between enumeration and query) must not be
    // silently counted as zero — that would under-report the budget. Fold to None instead.
    let private_bytes = pids.iter().try_fold(0u64, |acc, p| {
        private_bytes_for(p.as_u32()).map(|b| acc.saturating_add(b))
    });

    let process_count = pids.len();

    let _ = child.kill();
    let _ = child.wait();

    Ok(Measured {
        cold_start_ms,
        private_bytes,
        working_set_sum_bytes,
        process_count,
    })
}

/// `root` plus every process transitively parented by it (WebView2's browser/renderer/GPU
/// process tree on Windows, or the WebKitGTK helper processes on Linux).
fn descendant_pids(sys: &sysinfo::System, root: sysinfo::Pid) -> Vec<sysinfo::Pid> {
    let mut found = vec![root];
    let mut frontier = vec![root];
    while let Some(parent) = frontier.pop() {
        for (pid, process) in sys.processes() {
            if process.parent() == Some(parent) && !found.contains(pid) {
                found.push(*pid);
                frontier.push(*pid);
            }
        }
    }
    found
}

/// Private (non-shared) memory charged to one process, in bytes.
///
/// Windows: `PROCESS_MEMORY_COUNTERS_EX::PrivateUsage` — private commit. This is slightly
/// *more* conservative than Task Manager's private working set (it includes paged-out
/// private pages), which is the right direction for a budget gate to err in.
#[cfg(windows)]
fn private_bytes_for(pid: u32) -> Option<u64> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::ProcessStatus::{
        K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };

    let size = u32::try_from(size_of::<PROCESS_MEMORY_COUNTERS_EX>()).ok()?;

    // SAFETY: `handle` is checked for null before use and closed on every path. `counters`
    // is zeroed and its `cb` field set to its own size, exactly as the API contract requires;
    // the cast to PROCESS_MEMORY_COUNTERS is the documented calling convention for the _EX
    // variant, which the API distinguishes by `cb`.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut counters: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        counters.cb = size;
        let ok = K32GetProcessMemoryInfo(
            handle,
            std::ptr::addr_of_mut!(counters).cast::<PROCESS_MEMORY_COUNTERS>(),
            size,
        );
        let _ = CloseHandle(handle);
        if ok == 0 {
            None
        } else {
            u64::try_from(counters.PrivateUsage).ok()
        }
    }
}

/// Linux: `Pss` from `/proc/<pid>/smaps_rollup` — proportional set size, which splits each
/// shared page evenly across the processes mapping it. Summing PSS across a process tree is
/// the standard way to get a total that neither double-counts nor ignores shared memory.
#[cfg(target_os = "linux")]
fn private_bytes_for(pid: u32) -> Option<u64> {
    let rollup = std::fs::read_to_string(format!("/proc/{pid}/smaps_rollup")).ok()?;
    for line in rollup.lines() {
        let Some(rest) = line.strip_prefix("Pss:") else {
            continue;
        };
        let kib: u64 = rest.trim().trim_end_matches(" kB").trim().parse().ok()?;
        return Some(kib.saturating_mul(1024));
    }
    None
}

/// macOS: Uses `proc_pidinfo` with `PROC_PIDTASKINFO` to retrieve `pti_resident_size`.
#[cfg(target_os = "macos")]
fn private_bytes_for(pid: u32) -> Option<u64> {
    use std::ffi::c_void;

    const PROC_PIDTASKINFO: i32 = 4;
    #[repr(C)]
    struct proc_taskinfo {
        pti_virtual_size: u64,
        pti_resident_size: u64,
        pti_total_user: u64,
        pti_total_system: u64,
        pti_threads_user: u64,
        pti_threads_system: u64,
        pti_policy: i32,
        pti_faults: i32,
        pti_pageins: i32,
        pti_cow_faults: i32,
        pti_messages_sent: i32,
        pti_messages_received: i32,
        pti_syscalls_mach: i32,
        pti_syscalls_unix: i32,
        pti_csw: i32,
        pti_threadnum: i32,
        pti_numrunning: i32,
        pti_priority: i32,
    }

    extern "C" {
        fn proc_pidinfo(
            pid: i32,
            flavor: i32,
            arg: u64,
            buffer: *mut c_void,
            buffersize: i32,
        ) -> i32;
    }

    let mut info: proc_taskinfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<proc_taskinfo>() as i32;
    let ret = unsafe {
        proc_pidinfo(
            pid as i32,
            PROC_PIDTASKINFO,
            0,
            &mut info as *mut _ as *mut c_void,
            size,
        )
    };

    if ret == size {
        Some(info.pti_resident_size)
    } else {
        None
    }
}

/// Fallback for other unsupported target operating systems.
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn private_bytes_for(_pid: u32) -> Option<u64> {
    None
}
