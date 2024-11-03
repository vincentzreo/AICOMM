use ai_sdk::{AiAdapter, AiService, OllamaAdapter};
use chat_core::{AdapterType, Agent, AgentType, ChatAgent};

pub enum AgentVariant {
    Proxy(ProxyAgent),
    Reply(ReplyAgent),
    Tap(TapAgent),
}

#[allow(unused)]
pub struct ProxyAgent {
    pub name: String,
    pub adapter: AiAdapter,
    pub prompt: String,
    pub args: serde_json::Value,
}

#[allow(unused)]
pub struct ReplyAgent {
    pub name: String,
    pub adapter: AiAdapter,
    pub prompt: String,
    pub args: serde_json::Value,
}

#[allow(unused)]
pub struct TapAgent {
    pub name: String,
    pub adapter: AiAdapter,
    pub prompt: String,
    pub args: serde_json::Value,
}

impl Agent for ProxyAgent {
    async fn process(
        &self,
        message: &str,
        _ctx: &chat_core::AgentContext,
    ) -> Result<chat_core::AgentDecision, chat_core::AgentError> {
        let prompt = format!("{} {}", self.prompt, message);
        let messages = vec![ai_sdk::Message::user(&prompt)];
        let response = self.adapter.complete(&messages).await?;
        Ok(chat_core::AgentDecision::Modify(response))
    }
}

impl Agent for ReplyAgent {
    async fn process(
        &self,
        message: &str,
        _ctx: &chat_core::AgentContext,
    ) -> Result<chat_core::AgentDecision, chat_core::AgentError> {
        let prompt = format!("{} {}", self.prompt, message);
        let messages = vec![ai_sdk::Message::user(&prompt)];
        let response = self.adapter.complete(&messages).await?;
        Ok(chat_core::AgentDecision::Reply(response))
    }
}

impl Agent for TapAgent {
    async fn process(
        &self,
        _message: &str,
        _ctx: &chat_core::AgentContext,
    ) -> Result<chat_core::AgentDecision, chat_core::AgentError> {
        Ok(chat_core::AgentDecision::None)
    }
}

impl Agent for AgentVariant {
    async fn process(
        &self,
        message: &str,
        ctx: &chat_core::AgentContext,
    ) -> Result<chat_core::AgentDecision, chat_core::AgentError> {
        match self {
            AgentVariant::Proxy(agent) => agent.process(message, ctx).await,
            AgentVariant::Reply(agent) => agent.process(message, ctx).await,
            AgentVariant::Tap(agent) => agent.process(message, ctx).await,
        }
    }
}

impl From<ChatAgent> for AgentVariant {
    fn from(mut agent: ChatAgent) -> Self {
        let adapter: AiAdapter = match agent.adapter {
            AdapterType::Ollama => OllamaAdapter::new_local(agent.model).into(),
        };
        match agent.r#type {
            AgentType::Proxy => Self::Proxy(ProxyAgent {
                name: agent.name,
                adapter,
                prompt: agent.prompt,
                args: agent.args.take(),
            }),
            AgentType::Reply => Self::Reply(ReplyAgent {
                name: agent.name,
                adapter,
                prompt: agent.prompt,
                args: agent.args.take(),
            }),
            AgentType::Tap => Self::Tap(TapAgent {
                name: agent.name,
                adapter,
                prompt: agent.prompt,
                args: agent.args.take(),
            }),
        }
    }
}

impl From<ProxyAgent> for AgentVariant {
    fn from(agent: ProxyAgent) -> Self {
        Self::Proxy(agent)
    }
}

impl From<ReplyAgent> for AgentVariant {
    fn from(agent: ReplyAgent) -> Self {
        Self::Reply(agent)
    }
}

impl From<TapAgent> for AgentVariant {
    fn from(agent: TapAgent) -> Self {
        Self::Tap(agent)
    }
}

#[cfg(test)]
mod tests {
    use chat_core::AgentContext;

    use crate::AppState;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn agent_variant_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let agents = state.list_agents(1).await.expect("list agents failed");
        let agent = agents[0].clone();
        let agent_variant = AgentVariant::from(agent);
        let decision = agent_variant
            .process("hello", &AgentContext::default())
            .await?;
        if let chat_core::AgentDecision::Modify(_content) = decision {
        } else {
            panic!("decision is not modify");
        }
        Ok(())
    }
}
