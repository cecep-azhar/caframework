//! Main window creation and WebView2 memory governance.
//!
//! The window is built here instead of by Tauri (`"create": false` in tauri.conf.json)
//! because whether the WebView gets GPU acceleration is a user preference that has to be
//! known before the WebView exists — Chromium switches cannot change afterwards.

use caf_core::prefs;
use tauri::WebviewWindow;

/// Appended to configured browser args when user turned GPU acceleration off in
/// Settings -> Performance: software rendering, less memory, more CPU.
const NO_GPU_ARGS: &str = " --disable-gpu --disable-gpu-compositing";

pub fn create_main_window(app: &tauri::App) -> tauri::Result<WebviewWindow> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .cloned()
        .ok_or(tauri::Error::WindowNotFound)?;

    let mut builder = tauri::WebviewWindowBuilder::from_config(app.handle(), &config)?;
    if !prefs::load_performance_prefs()
        .map(|p| p.gpu_acceleration)
        .unwrap_or(false)
    {
        let base = config.additional_browser_args.clone().unwrap_or_default();
        builder = builder.additional_browser_args(&format!("{base} {NO_GPU_ARGS}"));
    }

    let window = builder.build()?;
    memory::install(&window);
    disable_browser_accelerators(&window);
    Ok(window)
}

/// Release builds turn off WebView2's *browser* shortcuts: F5/Ctrl+R would reload the whole
/// app (dropping every SSH session and locking the vault), Ctrl+P opens a print dialog, Ctrl+F
/// Edge's find bar, Ctrl+plus/minus zoom the whole UI. Editing keys (copy, paste, undo) keep
/// working. Debug builds keep them so reload and DevTools stay available while developing.
#[cfg(all(windows, not(debug_assertions)))]
fn disable_browser_accelerators(window: &WebviewWindow) {
    let _ = window.with_webview(|webview| {
        use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3;
        use windows_core::Interface;

        // SAFETY: plain COM calls on the live controller, on the UI thread (see set_low_memory).
        unsafe {
            let Ok(core) = webview.controller().CoreWebView2() else {
                return;
            };
            let Ok(settings) = core.Settings() else {
                return;
            };
            if let Ok(settings3) = settings.cast::<ICoreWebView2Settings3>() {
                let _ = settings3.SetAreBrowserAcceleratorKeysEnabled(false);
            }
        }
    });
}

#[cfg(not(all(windows, not(debug_assertions))))]
fn disable_browser_accelerators(_window: &WebviewWindow) {}

/// Tells WebView2 to shrink (drop caches, compact the heap) while CAFramework is in the
/// background, and to go back to normal the moment it is in front again. This is the
/// supported way to lower WebView2's footprint; it reaches the renderer and GPU processes,
/// which is where most of the memory is — unlike trimming working sets from the outside.
#[cfg(windows)]
mod memory {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    use tauri::{WebviewWindow, WindowEvent};

    /// Visible but unfocused for this long counts as "in the background". Minimizing is
    /// immediate. SSH sessions keep running either way — this only trims caches.
    const BACKGROUND_AFTER: Duration = Duration::from_secs(30);

    fn set_low_memory(window: &WebviewWindow, low: bool) {
        let _ = window.with_webview(move |webview| {
            use webview2_com::Microsoft::Web::WebView2::Win32::{
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL, ICoreWebView2_19,
            };
            use windows_core::Interface;

            // SAFETY: plain COM calls on the live controller Tauri hands us, on the thread
            // Tauri runs `with_webview` closures on (the UI thread, as WebView2 requires).
            unsafe {
                let Ok(core) = webview.controller().CoreWebView2() else {
                    return;
                };
                // ICoreWebView2_19 needs WebView2 Runtime 1.0.2210+. Older runtimes simply
                // keep the default level — nothing to do, nothing to report.
                let Ok(core19) = core.cast::<ICoreWebView2_19>() else {
                    return;
                };
                let level = if low {
                    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
                } else {
                    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
                };
                let _ = core19.SetMemoryUsageTargetLevel(level);
            }
        });
    }

    pub fn install(window: &WebviewWindow) {
        // Every focus change bumps the generation, so a pending "go low" timer from an
        // earlier blur is ignored once the user has come back.
        let generation = Arc::new(AtomicU64::new(0));
        let handle = window.clone();
        window.on_window_event(move |event| match event {
            WindowEvent::Focused(true) => {
                generation.fetch_add(1, Ordering::SeqCst);
                set_low_memory(&handle, false);
            }
            WindowEvent::Focused(false) => {
                let ticket = generation.fetch_add(1, Ordering::SeqCst) + 1;
                let (generation, window) = (generation.clone(), handle.clone());
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(BACKGROUND_AFTER).await;
                    if generation.load(Ordering::SeqCst) == ticket {
                        set_low_memory(&window, true);
                    }
                });
            }
            WindowEvent::Resized(_) if handle.is_minimized().unwrap_or(false) => {
                generation.fetch_add(1, Ordering::SeqCst);
                set_low_memory(&handle, true);
            }
            _ => {}
        });
    }
}

#[cfg(target_os = "linux")]
mod memory {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    use tauri::{WebviewWindow, WindowEvent};
    use webkit2gtk::{WebContextExt, WebViewExt, WebsiteDataManagerExtManual};

    /// Visible but unfocused for this long counts as "in background". Minimizing is
    /// immediate. SSH sessions keep running either way; this only trims caches.
    const BACKGROUND_AFTER: Duration = Duration::from_secs(15);

    fn set_low_memory(window: &WebviewWindow, low: bool) {
        let _ = window.with_webview(move |webview| {
            let inner = webview.inner();
            if let Some(context) = inner.context() {
                if low {
                    context.set_cache_model(webkit2gtk::CacheModel::DocumentViewer);
                    context.clear_cache();
                    if let Some(dm) = context.website_data_manager() {
                        dm.clear(
                            webkit2gtk::WebsiteDataTypes::MEMORY_CACHE
                                | webkit2gtk::WebsiteDataTypes::DISK_CACHE,
                            webkit2gtk::glib::TimeSpan::from_seconds(0),
                            webkit2gtk::gio::Cancellable::NONE,
                            |_| {},
                        );
                    }
                } else {
                    context.set_cache_model(webkit2gtk::CacheModel::WebBrowser);
                }
            }
        });
    }

    pub fn install(window: &WebviewWindow) {
        let generation = Arc::new(AtomicU64::new(0));
        let handle = window.clone();
        window.on_window_event(move |event| match event {
            WindowEvent::Focused(true) => {
                generation.fetch_add(1, Ordering::SeqCst);
                set_low_memory(&handle, false);
            }
            WindowEvent::Focused(false) => {
                let ticket = generation.fetch_add(1, Ordering::SeqCst) + 1;
                let (generation, window) = (generation.clone(), handle.clone());
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(BACKGROUND_AFTER).await;
                    if generation.load(Ordering::SeqCst) == ticket {
                        set_low_memory(&window, true);
                    }
                });
            }
            WindowEvent::Resized(_) if handle.is_minimized().unwrap_or(false) => {
                generation.fetch_add(1, Ordering::SeqCst);
                set_low_memory(&handle, true);
            }
            _ => {}
        });
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod memory {
    pub fn install(_window: &tauri::WebviewWindow) {}
}
