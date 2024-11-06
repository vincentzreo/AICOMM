use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tracing::warn;
use utoipa::{IntoParams, ToSchema};

use crate::{agent::AgentVariant, AppError, AppState, ChatFile};
use chat_core::{Agent as _, AgentContext, AgentDecision, ChatType, Message};
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMessage {
    pub content: String,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ListMessages {
    #[serde(default)]
    pub last_id: Option<u64>,
    #[serde(default)]
    pub limit: u64,
}

#[allow(dead_code)]
impl AppState {
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: u64,
        user_id: u64,
    ) -> Result<Message, AppError> {
        let base_dir = &self.config.server.base_dir;
        // verify content - not empty
        if input.content.is_empty() {
            return Err(AppError::MessageCreateError("content is empty".to_string()));
        }
        // verify files - all files should exist
        for s in &input.files {
            let file = ChatFile::from_str(s)?;
            if !file.path(base_dir).exists() {
                return Err(AppError::MessageCreateError(format!(
                    "File does not exist: {}",
                    s
                )));
            }
        }

        let mut agents = self.list_agents(chat_id).await?;
        let decision = if let Some(agent) = agents.pop() {
            AgentVariant::from(agent)
                .process(&input.content, &AgentContext::default())
                .await?
        } else {
            AgentDecision::None
        };

        let modified_content = match decision {
            AgentDecision::Modify(ref content) => Some(content),
            _ => None,
        };

        let message: Message = sqlx::query_as(
            r#"
        INSERT INTO messages (chat_id, sender_id, content, modified_content, files)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .bind(input.content)
        .bind(modified_content)
        .bind(&input.files)
        .fetch_one(&self.pool)
        .await?;

        if let AgentDecision::Reply(reply) = decision {
            let chat = self
                .get_chat_by_id(chat_id)
                .await?
                .expect("chat should exist");
            if chat.r#type != ChatType::Single {
                warn!(
                    "agent reply is only allowed in single chat, but the chat {} is a not single chat",
                    chat_id
                );
            }
            let other_user_id = chat
                .members
                .into_iter()
                .find(|u| *u != (user_id as i64))
                .expect("other user should exist");
            let _: (i64,) = sqlx::query_as(
                r#"
                INSERT INTO messages (chat_id, sender_id, content)
                VALUES ($1, $2, $3)
                RETURNING id
                "#,
            )
            .bind(chat_id as i64)
            .bind(other_user_id)
            .bind(reply)
            .fetch_one(&self.pool)
            .await?;
        }
        Ok(message)
    }

    pub async fn list_messages(
        &self,
        input: ListMessages,
        chat_id: u64,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX as _);
        let limit = match input.limit {
            0 => i64::MAX,
            _ => input.limit as i64,
        };
        let messages = sqlx::query_as(
            r#"
        SELECT id, chat_id, sender_id, content, modified_content, files, created_at
        FROM messages
        WHERE chat_id = $1
        AND id < $2
        ORDER BY id DESC
        LIMIT $3
        "#,
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {

    use crate::CreateChat;

    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn create_message_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state
            .create_chat(CreateChat::new(None, &[1, 2], false), 1, 1)
            .await?;
        let input = CreateMessage {
            content: "Hello".to_string(),
            files: vec![],
        };
        let message = state.create_message(input, chat.id as _, 1).await?;
        assert_eq!(message.content, "Hello");

        // invalid files should fail
        let input = CreateMessage {
            content: "Hello".to_string(),
            files: vec!["1".to_string()],
        };
        let err = state
            .create_message(input, chat.id as _, 1)
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), "File does not exist: 1");

        // valid files should work
        let url = upload_dummy_file(&state)?;
        let input = CreateMessage {
            content: "Hello".to_string(),
            files: vec![url],
        };
        let message = state.create_message(input, chat.id as _, 1).await?;
        assert_eq!(message.content, "Hello");
        assert_eq!(message.files.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn list_messages_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = ListMessages {
            last_id: None,
            limit: 6,
        };
        let messages = state.list_messages(input, 1).await?;
        assert_eq!(messages.len(), 6);

        let last_id = messages.last().unwrap().id;
        let input = ListMessages {
            last_id: Some(last_id as _),
            limit: 6,
        };
        let messages = state.list_messages(input, 1).await?;
        assert_eq!(messages.len(), 4);
        Ok(())
    }

    fn upload_dummy_file(state: &AppState) -> Result<String> {
        let file = ChatFile::new(1, "test.txt", b"hello world");
        let path = file.path(&state.config.server.base_dir);
        std::fs::create_dir_all(path.parent().expect("file path parent should exists"))?;
        std::fs::write(&path, b"hello world")?;
        Ok(file.url())
    }
}
