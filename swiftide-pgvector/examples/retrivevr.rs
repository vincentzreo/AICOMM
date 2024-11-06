use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use swiftide::{
    integrations,
    query::{self, answers, query_transformers, response_transformers},
};
use swiftide_pgvector::PgVector;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

const VECTOR_SIZE: usize = 768;

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let pool = PgPoolOptions::new()
        .connect("postgres://postgres:postgres@localhost:5432/swiftide_rag")
        .await?;

    let ollama_client = integrations::ollama::Ollama::default()
        .with_default_embed_model("nomic-embed-text")
        .with_default_prompt_model("llama3.2")
        .to_owned();

    let store = PgVector::try_new(pool, VECTOR_SIZE as _).await?;

    let pipeline = query::Pipeline::default()
        .then_transform_query(query_transformers::GenerateSubquestions::from_client(
            ollama_client.clone(),
        ))
        .then_transform_query(query_transformers::Embed::from_client(
            ollama_client.clone(),
        ))
        .then_retrieve(store)
        .then_transform_response(response_transformers::Summary::from_client(
            ollama_client.clone(),
        ))
        .then_answer(answers::Simple::from_client(ollama_client.clone()));

    let result = pipeline.query("这段代码是做什么的？").await?;

    println!("{}", result.answer());
    todo!()
}
