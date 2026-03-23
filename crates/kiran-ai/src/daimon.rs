//! Daimon (agent-runtime) integration for the Kiran game engine.
//!
//! Registers Kiran as an AGNOS agent, sends heartbeats, and provides
//! an inference helper that routes through hoosh (LLM gateway).

use serde::{Deserialize, Serialize};

/// Configuration for connecting to the AGNOS daimon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaimonConfig {
    /// Base URL for the daimon agent-runtime API.
    pub daimon_url: String,
    /// Base URL for the hoosh LLM gateway.
    pub hoosh_url: String,
    /// Agent name to register as.
    pub agent_name: String,
}

impl Default for DaimonConfig {
    fn default() -> Self {
        Self {
            daimon_url: "http://localhost:8090".into(),
            hoosh_url: "http://localhost:8088".into(),
            agent_name: "kiran".into(),
        }
    }
}

/// Client for interacting with the AGNOS daimon and hoosh services.
pub struct DaimonClient {
    config: DaimonConfig,
    http: reqwest::Client,
    agent_id: Option<String>,
}

impl DaimonClient {
    /// Create a new client with the given configuration.
    pub fn new(config: DaimonConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
            agent_id: None,
        }
    }

    /// Register Kiran as an agent with daimon.
    pub async fn register(&mut self) -> anyhow::Result<String> {
        let url = format!("{}/v1/agents/register", self.config.daimon_url);
        let body = serde_json::json!({
            "name": self.config.agent_name,
            "capabilities": ["game-engine", "scene-management", "rendering"],
            "version": env!("CARGO_PKG_VERSION"),
        });

        let resp = self.http.post(&url).json(&body).send().await?;
        let data: serde_json::Value = resp.json().await?;
        let id = data["id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        self.agent_id = Some(id.clone());
        tracing::info!(agent_id = %id, "registered with daimon");
        Ok(id)
    }

    /// Send a heartbeat to daimon.
    pub async fn heartbeat(&self) -> anyhow::Result<()> {
        let Some(ref id) = self.agent_id else {
            anyhow::bail!("not registered");
        };
        let url = format!("{}/v1/agents/{}/heartbeat", self.config.daimon_url, id);
        self.http.post(&url).send().await?;
        Ok(())
    }

    /// Request LLM inference through hoosh.
    pub async fn infer(&self, prompt: &str, model: Option<&str>) -> anyhow::Result<String> {
        let url = format!("{}/v1/chat/completions", self.config.hoosh_url);
        let body = serde_json::json!({
            "model": model.unwrap_or("default"),
            "messages": [{"role": "user", "content": prompt}],
        });

        let resp = self.http.post(&url).json(&body).send().await?;
        let data: serde_json::Value = resp.json().await?;
        let text = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(text)
    }

    /// The registered agent ID, if any.
    pub fn agent_id(&self) -> Option<&str> {
        self.agent_id.as_deref()
    }

    /// The underlying configuration.
    pub fn config(&self) -> &DaimonConfig {
        &self.config
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = DaimonConfig::default();
        assert_eq!(cfg.daimon_url, "http://localhost:8090");
        assert_eq!(cfg.hoosh_url, "http://localhost:8088");
        assert_eq!(cfg.agent_name, "kiran");
    }

    #[test]
    fn client_not_registered() {
        let client = DaimonClient::new(DaimonConfig::default());
        assert!(client.agent_id().is_none());
    }
}
