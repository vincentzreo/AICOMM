use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
pub mod middlewares;
mod utils;

pub use middlewares::*;
use thiserror::Error;
pub use utils::*;
use utoipa::ToSchema;

#[allow(async_fn_in_trait)]
pub trait Agent {
    async fn process(
        &self,
        message: Message,
        ctx: &AgentContext,
    ) -> Result<AgentDecision, AgentError>;
}

#[derive(Debug, Clone)]
pub struct AgentContext {}

#[derive(Debug, Clone)]
pub enum AgentDecision {
    Modify(String),
    Reply(String),
    Delete,
    None,
}

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Network error: {0}")]
    Network(String),
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    #[sqlx(default)]
    pub ws_name: String,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, ToSchema)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, ToSchema)]
pub struct ChatUser {
    pub id: i64,
    pub fullname: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, sqlx::Type, ToSchema)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all(serialize = "camelCase"))]
pub enum ChatType {
    #[serde(alias = "single", alias = "Single")]
    Single,
    #[serde(alias = "group", alias = "Group")]
    Group,
    #[serde(alias = "private_channel", alias = "privateChannel")]
    PrivateChannel,
    #[serde(alias = "public_channel", alias = "publicChannel")]
    PublicChannel,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub id: i64,
    #[serde(alias = "wsId")]
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    pub agents: Vec<i64>,
    #[serde(alias = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, ToSchema)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Message {
    pub id: i64,
    #[serde(alias = "chatId")]
    pub chat_id: i64,
    #[serde(alias = "senderId")]
    pub sender_id: i64,
    pub content: String,
    pub modified_content: Option<String>,
    pub files: Vec<String>,
    #[serde(alias = "createdAt")]
    pub created_at: DateTime<Utc>,
}

/*
-- create agent_type type
CREATE TYPE agent_type AS ENUM ('proxy', 'reply', 'tap');

-- add chat_agents table
CREATE TABLE chat_agents (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    name TEXT NOT NULL UNIQUE,
    type agent_type NOT NULL DEFAULT 'reply',
    prompt TEXT NOT NULL,
    args JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (chat_id, name)
);
*/

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, sqlx::Type, ToSchema)]
#[sqlx(type_name = "agent_type", rename_all = "snake_case")]
#[serde(rename_all(serialize = "camelCase"))]
pub enum AgentType {
    #[serde(alias = "proxy", alias = "Proxy")]
    Proxy,
    #[serde(alias = "reply", alias = "Reply")]
    Reply,
    #[serde(alias = "tap", alias = "Tap")]
    Tap,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChatAgent {
    pub id: i64,
    pub chat_id: i64,
    pub name: String,
    pub r#type: AgentType,
    pub prompt: String,
    pub args: sqlx::types::Json<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            ws_name: "".to_string(),
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}
