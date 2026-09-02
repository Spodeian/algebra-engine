//! Main entry point for `urae-notebook` cross-platform GUI application.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    urae_notebook::log_info(&format!(
        "Main binary started [Profile: {}, LogLevel: {}]. PID={}, OS={}, CWD={:?}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        urae_notebook::LogLevel::current().label(),
        std::process::id(),
        std::env::consts::OS,
        std::env::current_dir().unwrap_or_default()
    ));

    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::capture();
        let err_msg =
            format!("FATAL APPLICATION PANIC: {panic_info}\nStack Backtrace:\n{backtrace}");
        urae_notebook::log_error(&err_msg);
    }));

    if std::env::var("ANTIGRAVITY").is_ok() || std::env::var("VSCODE_PID").is_ok() {
        urae_notebook::log_info("Tip: Launching from external PowerShell / Windows Terminal or Windows Explorer enables full native desktop window interaction.");
    }

    urae_notebook::run_gui()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
