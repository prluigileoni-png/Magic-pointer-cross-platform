mod app;
mod config;
mod platform;
mod providers;

use anyhow::Result;
use app::MagicPointer;

fn main() -> Result<()> {
    let app = MagicPointer::load()?;
    app.run()
}
