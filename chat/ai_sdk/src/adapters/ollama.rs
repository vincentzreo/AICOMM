use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{AiAdapter, AiService, Message};

pub struct OllamaAdapter {
    pub host: String,
    pub model: String,
    pub client: Client,
}

#[derive(Serialize)]
pub struct OllamaChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    pub stream: bool,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct OllamaChatCompletionResponse {
    pub model: String,
    pub created_at: String,
    pub message: OllamaMessage,
    pub done: bool,
    pub total_duration: u64,
    pub load_duration: u64,
    pub prompt_eval_count: u64,
    pub prompt_eval_duration: u64,
    pub eval_count: u64,
    pub eval_duration: u64,
}

impl From<OllamaAdapter> for AiAdapter {
    fn from(adapter: OllamaAdapter) -> Self {
        Self::Ollama(adapter)
    }
}

impl OllamaAdapter {
    pub fn new(host: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            model: model.into(),
            client: Client::new(),
        }
    }
    pub fn new_local(model: impl Into<String>) -> Self {
        Self::new("http://localhost:11434", model)
    }
}

impl Default for OllamaAdapter {
    fn default() -> Self {
        Self::new_local("llama3.2")
    }
}

impl AiService for OllamaAdapter {
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<String> {
        let request = OllamaChatCompletionRequest {
            model: self.model.clone(),
            messages: messages.iter().map(|msg| msg.into()).collect(),
            stream: false,
        };
        let url = format!("{}/api/chat", self.host);
        let response = self.client.post(url).json(&request).send().await?;
        let response = response.json::<OllamaChatCompletionResponse>().await?;
        Ok(response.message.content)
    }
}

impl From<Message> for OllamaMessage {
    fn from(msg: Message) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content,
        }
    }
}
impl From<&Message> for OllamaMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Role;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn ollama_complete_should_work() {
        let adapter = OllamaAdapter::new_local("llama3.2");
        let response = adapter
            .complete(&[Message {
                role: Role::User,
                content: "Hello".to_string(),
            }])
            .await
            .unwrap();
        println!("{}", response);
    }
}
