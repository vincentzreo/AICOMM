use ai_sdk::{AiService as _, Message, OllamaAdapter, Role};

#[tokio::main]
async fn main() {
    let adapter = OllamaAdapter::default();
    let response = adapter
        .complete(&[Message {
            role: Role::User,
            content: "世界上最高的山峰是?".to_string(),
        }])
        .await
        .unwrap();
    println!("{}", response);
}
