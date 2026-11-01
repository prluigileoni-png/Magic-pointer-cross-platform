use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::time::Duration;

use crate::config::{Config, Provider};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
}

fn client(config: &Config) -> Result<Client> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds.max(1)))
        .build()?)
}

pub fn ask(config: &Config, prompt: &str, image: Option<&[u8]>) -> Result<String> {
    if config.api_key.trim().is_empty() {
        bail!("no API key configured for {}", config.provider.as_str());
    }
    let client = client(config)?;
    match config.provider {
        Provider::Gemini => ask_gemini(&client, config, prompt, image),
        Provider::Openai => ask_openai(&client, config, prompt, image),
        Provider::Anthropic => ask_anthropic(&client, config, prompt, image),
        Provider::Nvidia => ask_openai_compatible(&client, config, prompt, image),
    }
}

pub fn list_models(config: &Config) -> Result<Vec<ModelInfo>> {
    if config.api_key.trim().is_empty() {
        bail!("enter an API key before loading models");
    }
    let client = client(config)?;
    let mut models = match config.provider {
        Provider::Gemini => list_gemini_models(&client, config)?,
        Provider::Openai => {
            list_openai_compatible_models(&client, config, "https://api.openai.com/v1/models")?
        }
        Provider::Anthropic => list_anthropic_models(&client, config)?,
        Provider::Nvidia => list_openai_compatible_models(
            &client,
            config,
            "https://integrate.api.nvidia.com/v1/models",
        )?,
    };
    models.sort_by(|left, right| left.label.to_lowercase().cmp(&right.label.to_lowercase()));
    models.dedup_by(|left, right| left.id == right.id);
    Ok(models)
}

fn list_gemini_models(client: &Client, c: &Config) -> Result<Vec<ModelInfo>> {
    let url = if c.base_url.is_empty() {
        "https://generativelanguage.googleapis.com/v1beta/models".to_owned()
    } else {
        format!("{}/models", c.base_url.trim_end_matches('/'))
    };
    let value: Value = client
        .get(url)
        .query(&[("key", c.api_key.as_str())])
        .send()?
        .error_for_status()?
        .json()?;
    Ok(parse_gemini_models(&value))
}

fn parse_gemini_models(value: &Value) -> Vec<ModelInfo> {
    value["models"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|model| {
            model["supportedGenerationMethods"]
                .as_array()
                .map(|methods| {
                    methods
                        .iter()
                        .any(|method| method.as_str() == Some("generateContent"))
                })
                .unwrap_or(false)
        })
        .filter_map(|model| {
            let name = model["name"].as_str()?.to_owned();
            Some(ModelInfo {
                id: name.strip_prefix("models/").unwrap_or(&name).to_owned(),
                label: model["displayName"].as_str().unwrap_or(&name).to_owned(),
            })
        })
        .collect()
}

fn list_anthropic_models(client: &Client, c: &Config) -> Result<Vec<ModelInfo>> {
    let url = if c.base_url.is_empty() {
        "https://api.anthropic.com/v1/models".to_owned()
    } else {
        format!("{}/models", c.base_url.trim_end_matches('/'))
    };
    let value: Value = client
        .get(url)
        .header("x-api-key", &c.api_key)
        .header("anthropic-version", "2023-06-01")
        .send()?
        .error_for_status()?
        .json()?;
    Ok(parse_openai_style_models(&value))
}

fn list_openai_compatible_models(
    client: &Client,
    c: &Config,
    default_url: &str,
) -> Result<Vec<ModelInfo>> {
    let url = if c.base_url.is_empty() {
        default_url.to_owned()
    } else if c.base_url.ends_with("/chat/completions") {
        c.base_url.replace("/chat/completions", "/models")
    } else {
        format!("{}/models", c.base_url.trim_end_matches('/'))
    };
    let value: Value = client
        .get(url)
        .header("Authorization", format!("Bearer {}", c.api_key))
        .send()?
        .error_for_status()?
        .json()?;
    Ok(parse_openai_style_models(&value))
}

fn parse_openai_style_models(value: &Value) -> Vec<ModelInfo> {
    value["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|model| {
            let id = model["id"].as_str()?.to_owned();
            Some(ModelInfo {
                label: id.clone(),
                id,
            })
        })
        .collect()
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
    ask_openai_compatible_at(client, c, prompt, image, url)
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
    ask_openai_compatible_at(client, c, prompt, image, url)
}

fn ask_openai_compatible_at(
    client: &Client,
    c: &Config,
    prompt: &str,
    image: Option<&[u8]>,
    url: &str,
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
        .header("Authorization", format!("Bearer {}", c.api_key))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openai_style_models() {
        let value = serde_json::json!({"data": [{"id": "model-a"}, {"id": "model-b"}]});
        assert_eq!(
            parse_openai_style_models(&value),
            vec![
                ModelInfo {
                    id: "model-a".into(),
                    label: "model-a".into()
                },
                ModelInfo {
                    id: "model-b".into(),
                    label: "model-b".into()
                },
            ]
        );
    }

    #[test]
    fn filters_gemini_models_without_generation_support() {
        let value = serde_json::json!({"models": [
            {"name": "models/vision", "displayName": "Vision", "supportedGenerationMethods": ["generateContent"]},
            {"name": "models/embed", "displayName": "Embed", "supportedGenerationMethods": ["embedContent"]}
        ]});
        assert_eq!(
            parse_gemini_models(&value),
            vec![ModelInfo {
                id: "vision".into(),
                label: "Vision".into()
            }]
        );
    }
}
