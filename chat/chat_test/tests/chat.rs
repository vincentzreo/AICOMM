use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use chat_core::{Chat, ChatAgent, ChatType, Message};
use futures::StreamExt;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use serde_json::json;
use tokio::{net::TcpListener, time::sleep};

#[derive(Debug, Deserialize)]
struct AuthToken {
    token: String,
}

struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}

struct NotifyServer;

const WILD_ADDR: &str = "0.0.0.0:0";

#[tokio::test]
async fn chat_server_should_work() -> Result<()> {
    let (tdb, state) = chat_server::AppState::new_for_test().await?;
    let chat_server = ChatServer::new(state).await?;
    let db_url = tdb.url();
    NotifyServer::new(&db_url, &chat_server.token).await?;
    let chat = chat_server.create_chat().await?;
    let _agent = chat_server.create_agent(chat.id as u64).await?;
    let _msg = chat_server.create_message(chat.id as u64).await?;
    sleep(Duration::from_secs(1)).await;
    Ok(())
}

impl NotifyServer {
    async fn new(db_url: &str, token: &str) -> Result<Self> {
        let mut config = notify_server::AppConfig::load()?;
        config.server.db_url = db_url.to_string();
        let app = notify_server::get_router(config).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });
        let mut es = EventSource::get(format!("http://{}/events?token={}", addr, token));
        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => println!("Connection Open!"),
                    Ok(Event::Message(message)) => match message.event.as_ref() {
                        "NewChat" => {
                            let chat: Chat = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(chat.name.as_ref().unwrap(), "test");
                            assert_eq!(chat.members, vec![1, 2]);
                            assert_eq!(chat.r#type, ChatType::PrivateChannel);
                        }
                        "NewMessage" => {
                            let msg: Message = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(msg.content, "hello");
                            assert_eq!(msg.files.len(), 1);
                            assert_eq!(msg.sender_id, 1);
                        }
                        _ => {
                            panic!("unexpected event: {:?}", message);
                        }
                    },
                    Err(err) => {
                        println!("Error: {}", err);
                        es.close();
                    }
                }
            }
        });

        Ok(Self)
    }
}

impl ChatServer {
    async fn new(state: chat_server::AppState) -> Result<Self> {
        let app = chat_server::get_router(state).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });
        let client = reqwest::Client::new();
        let mut ret = Self {
            addr,
            token: "".to_string(),
            client,
        };
        /* ret.signup().await?; */
        ret.token = ret.signin().await?;
        Ok(ret)
    }
    #[allow(dead_code)]
    async fn signup(&self) -> Result<()> {
        self.client
            .post(format!("http://{}/api/signup", self.addr))
            .header("Content-Type", "application/json")
            .body(
                r#"{
                "workspace": "acme",
                "fullname": "zzq",
                "email": "zzq@163.com",
                "password": "123456"
            }"#,
            )
            .send()
            .await?;
        Ok(())
    }
    async fn signin(&self) -> Result<String> {
        let res = self
            .client
            .post(format!("http://{}/api/signin", self.addr))
            .header("Content-Type", "application/json")
            .body(r#"{"email": "zzq@zzq.com","password":"123456"}"#)
            .send()
            .await?;
        assert_eq!(res.status(), 200);
        let ret: AuthToken = res.json().await?;
        Ok(ret.token)
    }
    async fn create_chat(&self) -> Result<Chat> {
        let res = self
            .client
            .post(format!("http://{}/api/chats", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(
                r#"{
                "name": "test",
                "members": [1,2],
                "public": false
            }"#,
            )
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let ret: Chat = res.json().await?;
        assert_eq!(ret.name.as_ref().unwrap(), "test");
        assert_eq!(ret.members, vec![1, 2]);
        assert_eq!(ret.r#type, ChatType::PrivateChannel);
        Ok(ret)
    }

    async fn create_agent(&self, chat_id: u64) -> Result<ChatAgent> {
        let res = self
            .client
            .post(format!("http://{}/api/chats/{}/agents", self.addr, chat_id))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(
                r#"{
                "name": "translation222",
                "type": "proxy",
                "prompt": "if content is in english, translate to chinese.if language is chinese, translate to english"
            }"#,
            )
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let ret: ChatAgent = res.json().await?;
        Ok(ret)
    }
    async fn create_message(&self, chat_id: u64) -> Result<Message> {
        let data = include_bytes!("../Cargo.toml");
        let file = Part::bytes(data)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;
        let form = Form::new().part("file", file);
        let res = self
            .client
            .post(format!("http://{}/api/upload", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::OK);
        let ret: Vec<String> = res.json().await?;
        let body = serde_json::to_string(&json!(
            {
                "content": "hello",
                "files": ret}
        ))?;
        let res = self
            .client
            .post(format!("http://{}/api/chats/{}", self.addr, chat_id))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let message: Message = res.json().await?;
        assert_eq!(message.content, "hello");
        assert_eq!(message.files, ret);
        assert_eq!(message.sender_id, 1);
        assert_eq!(message.chat_id, chat_id as i64);
        Ok(message)
    }
}
