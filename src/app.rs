use anyhow::Result;
use std::io::{self, Write};

use crate::{
    config::{Config, Provider},
    platform, providers,
};

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
        let mut config = self.config;
        if config.api_key.trim().is_empty() {
            config = Self::setup_config(config)?;
        }
        println!("Magic Pointer native Rust host");
        println!(
            "Provider: {} | model: {}",
            config.provider.as_str(),
            config.model
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
                    .and_then(|image| providers::ask(&config, question.trim(), Some(&image)))
                {
                    Ok(answer) => println!("{answer}"),
                    Err(error) => eprintln!("Capture request failed: {error:#}"),
                }
                continue;
            }
            if input.is_empty() {
                continue;
            }
            match providers::ask(&config, input, None) {
                Ok(answer) => println!("{answer}"),
                Err(error) => eprintln!("Request failed: {error:#}"),
            }
        }
        Ok(())
    }

    pub fn setup(self) -> Result<()> {
        let config = Self::setup_config(self.config)?;
        println!("Configuration saved to {}", Config::path()?.display());
        println!(
            "Provider: {} | model: {}",
            config.provider.as_str(),
            config.model
        );
        Ok(())
    }

    fn setup_config(mut config: Config) -> Result<Config> {
        println!("First run setup - press Enter to keep the default in brackets.");
        println!("API keys are stored only in the user configuration file.");
        config.api_key = prompt("API key", &config.api_key)?;
        if config.api_key.trim().is_empty() {
            anyhow::bail!("an API key is required; run `magic-pointer.exe --setup` to try again");
        }

        let provider = prompt(
            "Provider (gemini/openai/anthropic/nvidia)",
            config.provider.as_str(),
        )?;
        config.provider = match provider.trim().to_ascii_lowercase().as_str() {
            "gemini" => Provider::Gemini,
            "openai" => Provider::Openai,
            "anthropic" => Provider::Anthropic,
            "nvidia" => Provider::Nvidia,
            _ => anyhow::bail!("unsupported provider `{provider}`"),
        };
        config.model = prompt("Model", &config.model)?;
        config.save()?;
        Ok(config)
    }
}

fn prompt(label: &str, default: &str) -> Result<String> {
    use std::io::{self, Write};
    print!("{label} [{default}]: ");
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    let value = value.trim();
    Ok(if value.is_empty() {
        default.to_owned()
    } else {
        value.to_owned()
    })
}
