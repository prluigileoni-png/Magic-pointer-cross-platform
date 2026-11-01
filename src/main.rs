#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod config;
mod gui;
mod platform;
mod providers;

use anyhow::Result;

fn main() -> Result<()> {
    gui::run()
}
