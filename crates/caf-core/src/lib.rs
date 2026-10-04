//! CAFramework v2 core library. Pure Rust — no Tauri, no GUI toolkit, no WebView.
//! Every public function here must be callable and testable without a display server.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::todo,
        clippy::unimplemented
    )
)]

pub mod ai;
pub mod api;
pub mod audit;
pub mod backup;
pub mod crash;
pub mod db;
pub mod error;
pub mod feedback;
pub mod http;
pub mod keyring;
pub mod migrations;
pub mod notes;
pub mod paths;
pub mod prefs;
pub mod pro;
pub mod profiles;
pub mod rbac;
pub mod secret;
pub mod session;
pub mod vault;
pub mod visibility;

pub use error::{CafError, CatermError};

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Test-only helpers shared across modules.
#[cfg(test)]
pub(crate) mod test_support {
    use parking_lot::{Mutex, MutexGuard};
    use std::ffi::OsString;
    use std::path::PathBuf;

    /// The `CAF_DATA_DIR` env var and the unlocked-vault key are process-wide, and cargo
    /// runs tests in parallel threads. Tests touching either hold this lock, so one test can't
    /// swap the data directory or lock the vault underneath another.
    static GLOBAL_STATE: Mutex<()> = Mutex::new(());

    /// A throwaway data directory that `CAF_DATA_DIR` points at for the lifetime of the
    /// value. Dropping it locks the vault, restores the previous env value and deletes the dir,
    /// so no test ever reads or writes the developer's real vault.
    pub(crate) struct IsolatedDataDir {
        pub(crate) path: PathBuf,
        previous: Option<OsString>,
        _guard: MutexGuard<'static, ()>,
    }

    pub(crate) fn isolated_data_dir(label: &str) -> IsolatedDataDir {
        let guard = GLOBAL_STATE.lock();
        let path =
            std::env::temp_dir().join(format!("caframework_test_{label}_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).expect("create isolated test data dir");
        let previous = std::env::var_os("CAF_DATA_DIR");
        // SAFETY: tests that depend on CAF_DATA_DIR hold GLOBAL_STATE (taken above), so no
        // other such test reads or writes the variable while it changes.
        #[allow(clippy::disallowed_methods)]
        unsafe {
            std::env::set_var("CAF_DATA_DIR", &path);
        }
        IsolatedDataDir {
            path,
            previous,
            _guard: guard,
        }
    }

    impl Drop for IsolatedDataDir {
        fn drop(&mut self) {
            crate::vault::lock();
            let _ = std::fs::remove_dir_all(&self.path);
            if let Some(prev) = self.previous.take() {
                #[allow(clippy::disallowed_methods)]
                unsafe {
                    std::env::set_var("CAF_DATA_DIR", prev);
                }
            } else {
                #[allow(clippy::disallowed_methods)]
                unsafe {
                    std::env::remove_var("CAF_DATA_DIR");
                }
            }
        }
    }
}
pub mod pii;
pub mod sync;
