mod crypto;
mod db;
mod riot;
mod ui;

#[cfg(windows)]
fn ensure_console() {
    extern "system" {
        fn AttachConsole(dwProcessId: u32) -> i32;
        fn AllocConsole() -> i32;
    }
    unsafe {
        if AttachConsole(0xFFFFFFFF) == 0 {
            AllocConsole();
        }
    }
}

fn main() {
    #[cfg(windows)]
    ensure_console();

    if let Err(e) = db::Database::open().and_then(ui::run) {
        eprintln!("Fatal error: {e}");
        std::process::exit(1);
    }
}
