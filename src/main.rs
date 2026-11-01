mod app;
mod config;
mod platform;
mod providers;

use anyhow::Result;
use app::MagicPointer;

fn main() -> Result<()> {
    let setup = std::env::args().skip(1).any(|arg| arg == "--setup");
    let app = MagicPointer::load()?;
    if setup {
        app.setup()?;
        return Ok(());
    }
    app.run()
}
