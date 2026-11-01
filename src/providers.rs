use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::time::Duration;

use crate::config::{Config, Provider};

pub fn ask(config: &Config, prompt: &str, image: Option<&[u8]>) -> Result<String> {
    if config.api_key.trim().is_empty() {
        bail!("no API key configured for {}", config.provider.as_str());
    }
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds.max(1)))
        .build()?;
    match config.provider {
        Provider::Gemini => ask_gemini(&client, config, prompt, image),
        Provider::Openai => ask_openai(&client, config, prompt, image),
        Provider::Anthropic => ask_anthropic(&client, config, prompt, image),
        Provider::Nvidia => ask_openai_compatible(&client, config, prompt, image),
    }
}

fn ask_gemini(client: &Client, c: &Config, prompt: &str, image: Option<&[u8]>) -> Result<String> {
    let mut parts = vec![json!({"text": prompt})];
    if let Some(bytes) = image {
        parts.push(
            json!({"inline_data": {"mime_type": "image/png", "data": STANDARD.encode(bytes)}}),
        );
    }
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        c.model, c.api_key
    );
    let body = json!({"contents": [{"parts": parts}]});
    let value: Value = client
        .post(url)
        .json(&body)
        .send()?
        .error_for_status()?
        .json()?;
    value["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .map(str::to_owned)
        .context("Gemini returned no text")
}

fn ask_openai(client: &Client, c: &Config, prompt: &str, image: Option<&[u8]>) -> Result<String> {
    let url = if c.base_url.is_empty() {
        "https://api.openai.com/v1/chat/completions"
    } else {
        &c.base_url
    };
    ask_openai_compatible_at(client, c, prompt, image, url, "Bearer")
}

fn ask_openai_compatible(
    client: &Client,
    c: &Config,
    prompt: &str,
    image: Option<&[u8]>,
) -> Result<String> {
    let url = if c.base_url.is_empty() {
        "https://integrate.api.nvidia.com/v1/chat/completions"
    } else {
        &c.base_url
    };
    ask_openai_compatible_at(client, c, prompt, image, url, "Bearer")
}

fn ask_openai_compatible_at(
    client: &Client,
    c: &Config,
    prompt: &str,
    image: Option<&[u8]>,
    url: &str,
    auth: &str,
) -> Result<String> {
    let mut content = vec![json!({"type": "text", "text": prompt})];
    if let Some(bytes) = image {
        content.push(json!({"type": "image_url", "image_url": {
            "url": format!("data:image/png;base64,{}", STANDARD.encode(bytes))
        }}));
    }
    let body = json!({"model": c.model, "messages": [{"role": "user", "content": content}]});
    let value: Value = client
        .post(url)
        .header("Authorization", format!("{} {}", auth, c.api_key))
        .json(&body)
        .send()?
        .error_for_status()?
        .json()?;
    value["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_owned)
        .context("provider returned no text")
}

fn ask_anthropic(
    client: &Client,
    c: &Config,
    prompt: &str,
    image: Option<&[u8]>,
) -> Result<String> {
    let mut content = vec![json!({"type": "text", "text": prompt})];
    if let Some(bytes) = image {
        content.insert(
            0,
            json!({"type": "image", "source": {
                "type": "base64", "media_type": "image/png", "data": STANDARD.encode(bytes)
            }}),
        );
    }
    let url = if c.base_url.is_empty() {
        "https://api.anthropic.com/v1/messages"
    } else {
        &c.base_url
    };
    let body = json!({"model": c.model, "max_tokens": 2048, "messages": [{"role": "user", "content": content}]});
    let value: Value = client
        .post(url)
        .header("x-api-key", &c.api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()?
        .error_for_status()?
        .json()?;
    value["content"][0]["text"]
        .as_str()
        .map(str::to_owned)
        .context("Anthropic returned no text")
}
