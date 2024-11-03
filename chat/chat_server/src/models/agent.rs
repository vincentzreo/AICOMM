use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

use crate::{AppError, AppState};

use chat_core::{AdapterType, AgentType, ChatAgent};

#[derive(Debug, Serialize, Deserialize, Default, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgent {
    pub name: String,
    pub r#type: AgentType,
    pub prompt: String,
    pub adapter: AdapterType,
    pub model: String,
    #[serde(default = "default_map")]
    pub args: serde_json::Value,
}

fn default_map() -> serde_json::Value {
    serde_json::json!({})
}

impl CreateAgent {
    pub fn new(
        name: impl Into<String>,
        r#type: AgentType,
        adapter: AdapterType,
        model: impl Into<String>,
        prompt: impl Into<String>,
        args: impl Serialize,
    ) -> Self {
        Self {
            name: name.into(),
            r#type,
            adapter,
            model: model.into(),
            prompt: prompt.into(),
            args: serde_json::to_value(args).unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgent {
    pub id: u64,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

impl UpdateAgent {
    pub fn new(id: u64, prompt: impl Into<String>, args: impl Serialize) -> Self {
        Self {
            id,
            prompt: prompt.into(),
            args: serde_json::to_value(args).unwrap(),
        }
    }
}

#[allow(dead_code)]
impl AppState {
    pub async fn create_agent(
        &self,
        input: CreateAgent,
        chat_id: u64,
    ) -> Result<ChatAgent, AppError> {
        // check if the agent already exists
        if self.agent_name_exists(chat_id, &input.name).await? {
            info!("agent {} already exists in chat {}", input.name, chat_id);
            return Err(AppError::CreateAgentError(format!(
                "agent {} already exists",
                input.name
            )));
        }

        let agent = sqlx::query_as(
            r#"insert into chat_agents (chat_id, name, type, adapter, model, prompt, args) values ($1, $2, $3, $4, $5, $6, $7) returning *"#,
        )
        .bind(chat_id as i64)
        .bind(&input.name)
        .bind(input.r#type)
        .bind(input.adapter)
        .bind(&input.model)
        .bind(&input.prompt)
        .bind(input.args)
        .fetch_one(&self.pool)
        .await?;
        Ok(agent)
    }
    /// check if an agent name exists
    pub async fn agent_name_exists(&self, chat_id: u64, name: &str) -> Result<bool, AppError> {
        let agent = sqlx::query_scalar(
            r#"select exists (select 1 from chat_agents where chat_id = $1 and name = $2)"#,
        )
        .bind(chat_id as i64)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;
        Ok(agent)
    }
    /// check if an agent id exists
    pub async fn agent_id_exists(&self, chat_id: u64, agent_id: u64) -> Result<bool, AppError> {
        let agent = sqlx::query_scalar(
            r#"select exists (select 1 from chat_agents where chat_id = $1 and id = $2)"#,
        )
        .bind(chat_id as i64)
        .bind(agent_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(agent)
    }
    pub async fn list_agents(&self, chat_id: u64) -> Result<Vec<ChatAgent>, AppError> {
        let agents =
            sqlx::query_as(r#"select * from chat_agents where chat_id = $1 order by id asc"#)
                .bind(chat_id as i64)
                .fetch_all(&self.pool)
                .await?;
        Ok(agents)
    }

    pub async fn update_agent(
        &self,
        input: UpdateAgent,
        chat_id: u64,
    ) -> Result<ChatAgent, AppError> {
        let agent_id = input.id;
        // check if the agent exists
        if !self.agent_id_exists(chat_id, agent_id).await? {
            info!("agent {agent_id} not found in chat {chat_id}");
            return Err(AppError::UpdateAgentError(format!(
                "agent {} not found",
                agent_id
            )));
        }

        let prompt = input.prompt;
        let args = input.args;
        let agent = match (prompt.as_str(), &args) {
            ("", _) => {
                sqlx::query_as(r#"update chat_agents set args = $1 where chat_id = $2 and id = $3 returning *"#)
                    .bind(args)
                    .bind(chat_id as i64)
                    .bind(agent_id as i64)
                    .fetch_one(&self.pool)
                    .await?
            }
            _ => sqlx::query_as(
                r#"update chat_agents set prompt = $1, args = $2 where chat_id = $3 and id = $4 returning *"#,
            )
            .bind(prompt)
            .bind(args)
            .bind(chat_id as i64)
            .bind(agent_id as i64)
            .fetch_one(&self.pool)
            .await?,
        };
        Ok(agent)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn create_agent_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateAgent::new(
            "agent1",
            AgentType::Proxy,
            AdapterType::Ollama,
            "llama3.2",
            "You are a helpful assistant.",
            HashMap::<String, String>::new(),
        );
        let agent = state
            .create_agent(input, 1)
            .await
            .expect("create agent failed");
        assert_eq!(agent.name, "agent1");
        assert_eq!(agent.r#type, AgentType::Proxy);
        assert_eq!(agent.adapter, AdapterType::Ollama);
        assert_eq!(agent.model, "llama3.2");
        assert_eq!(agent.prompt, "You are a helpful assistant.");
        assert_eq!(agent.args, sqlx::types::Json(serde_json::json!({})));
        Ok(())
    }

    #[tokio::test]
    async fn list_agents_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let agents = state.list_agents(1).await.expect("list agents failed");
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "translation");
        assert_eq!(agents[0].r#type, AgentType::Proxy);
        assert_eq!(agents[0].prompt, "if content is in english, translate to chinese.if language is chinese, translate to english");
        assert_eq!(agents[0].args, sqlx::types::Json(serde_json::json!({})));
        Ok(())
    }

    #[tokio::test]
    async fn update_agent_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = UpdateAgent::new(
            1,
            "You are a helpful assistant.",
            HashMap::<String, String>::new(),
        );
        let agent = state
            .update_agent(input, 1)
            .await
            .expect("update agent failed");
        assert_eq!(agent.name, "translation");
        assert_eq!(agent.r#type, AgentType::Proxy);
        assert_eq!(agent.prompt, "You are a helpful assistant.");
        assert_eq!(agent.args, sqlx::types::Json(serde_json::json!({})));
        Ok(())
    }
}
