//! AI integration: daimon client, hoosh inference
//!
//! Provides the AGNOS integration layer for the Kiran game engine,
//! registering as a daimon agent and routing LLM requests through hoosh.

use serde::{Deserialize, Serialize};

pub use hoosh::HooshClient;
pub use hoosh::inference::{InferenceRequest, InferenceResponse, Message, Role};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Client for interacting with the AGNOS daimon and hoosh services.
pub struct DaimonClient {
    config: DaimonConfig,
    hoosh: HooshClient,
    agent_id: Option<String>,
}

impl DaimonClient {
    /// Create a new client with the given configuration.
    pub fn new(config: DaimonConfig) -> Self {
        let hoosh = HooshClient::new(&config.hoosh_url);
        Self {
            config,
            hoosh,
            agent_id: None,
        }
    }

    /// Register Kiran as an agent with daimon.
    pub async fn register(&mut self) -> anyhow::Result<String> {
        // Daimon registration still uses direct HTTP — hoosh is for inference only
        let client = reqwest::Client::new();
        let url = format!("{}/v1/agents/register", self.config.daimon_url);
        let body = serde_json::json!({
            "name": self.config.agent_name,
            "capabilities": ["game-engine", "scene-management", "rendering"],
            "version": env!("CARGO_PKG_VERSION"),
        });

        let resp = client.post(&url).json(&body).send().await?;
        let data: serde_json::Value = resp.json().await?;
        let id = data["id"].as_str().unwrap_or("unknown").to_string();
        self.agent_id = Some(id.clone());
        tracing::info!(agent_id = %id, "registered with daimon");
        Ok(id)
    }

    /// Send a heartbeat to daimon.
    pub async fn heartbeat(&self) -> anyhow::Result<()> {
        let Some(ref id) = self.agent_id else {
            anyhow::bail!("not registered");
        };
        let client = reqwest::Client::new();
        let url = format!("{}/v1/agents/{}/heartbeat", self.config.daimon_url, id);
        client.post(&url).send().await?;
        Ok(())
    }

    /// Request LLM inference through hoosh.
    pub async fn infer(&self, prompt: &str, model: Option<&str>) -> anyhow::Result<String> {
        let request = InferenceRequest {
            model: model.unwrap_or("default").to_string(),
            prompt: prompt.to_string(),
            ..Default::default()
        };
        let response = self.hoosh.infer(&request).await?;
        Ok(response.text)
    }

    /// Request LLM inference with full control over the request.
    pub async fn infer_full(
        &self,
        request: &InferenceRequest,
    ) -> anyhow::Result<InferenceResponse> {
        Ok(self.hoosh.infer(request).await?)
    }

    /// Access the underlying hoosh client.
    #[must_use]
    pub fn hoosh(&self) -> &HooshClient {
        &self.hoosh
    }

    /// The registered agent ID, if any.
    #[must_use]
    pub fn agent_id(&self) -> Option<&str> {
        self.agent_id.as_deref()
    }

    /// The underlying configuration.
    #[must_use]
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

    #[test]
    fn client_hoosh_accessible() {
        let client = DaimonClient::new(DaimonConfig::default());
        let _ = client.hoosh();
    }

    #[test]
    fn inference_request_reexport() {
        let req = InferenceRequest {
            model: "test".into(),
            prompt: "hello".into(),
            ..Default::default()
        };
        assert_eq!(req.model, "test");
    }

    #[test]
    fn config_serde_roundtrip() {
        let cfg = DaimonConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: DaimonConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.daimon_url, cfg.daimon_url);
    }
}
