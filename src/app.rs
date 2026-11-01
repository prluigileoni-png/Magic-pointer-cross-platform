use anyhow::Result;
use std::io::{self, Write};

use crate::{config::Config, platform, providers};

pub struct MagicPointer {
    config: Config,
}

impl MagicPointer {
    pub fn load() -> Result<Self> {
        Ok(Self {
            config: Config::load()?,
        })
    }

    pub fn run(self) -> Result<()> {
        println!("Magic Pointer native Rust host");
        println!(
            "Provider: {} | model: {}",
            self.config.provider.as_str(),
            self.config.model
        );
        println!("Config: {}", Config::path()?.display());
        println!(
            "Type a question, `:capture <question>` to ask about the screen, or `:quit` to exit."
        );
        platform::run_platform_loop()?;
        let mut line = String::new();
        loop {
            print!("> ");
            io::stdout().flush()?;
            line.clear();
            if io::stdin().read_line(&mut line)? == 0 {
                break;
            }
            let input = line.trim();
            if input == ":quit" {
                break;
            }
            if input == ":capture" {
                match platform::capture_screen() {
                    Ok(bytes) => println!("Captured {} bytes.", bytes.len()),
                    Err(error) => eprintln!("Capture unavailable: {error:#}"),
                }
                continue;
            }
            if let Some(question) = input.strip_prefix(":capture ") {
                if question.trim().is_empty() {
                    eprintln!("Usage: :capture <question>");
                    continue;
                }
                match platform::capture_screen()
                    .and_then(|image| providers::ask(&self.config, question.trim(), Some(&image)))
                {
                    Ok(answer) => println!("{answer}"),
                    Err(error) => eprintln!("Capture request failed: {error:#}"),
                }
                continue;
            }
            if input.is_empty() {
                continue;
            }
            match providers::ask(&self.config, input, None) {
                Ok(answer) => println!("{answer}"),
                Err(error) => eprintln!("Request failed: {error:#}"),
            }
        }
        Ok(())
    }
}
