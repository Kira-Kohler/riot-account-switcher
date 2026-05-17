mod crypto;
mod db;
mod riot;
mod ui;

fn main() {
    if let Err(e) = db::Database::open().and_then(ui::run) {
        eprintln!("Fatal error: {e}");
        std::process::exit(1);
    }
}
