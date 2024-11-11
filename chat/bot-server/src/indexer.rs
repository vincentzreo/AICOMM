use anyhow::Result;
use bot_server::{AppConfig, VECTOR_SIZE};
use sqlx::postgres::PgPoolOptions;
use swiftide::{
    indexing::{
        self,
        loaders::FileLoader,
        transformers::{ChunkCode, Embed, MetadataQACode},
    },
    integrations,
};
use swiftide_pgvector::PgVector;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;

    let pool = PgPoolOptions::new().connect(&config.server.db_url).await?;

    let ollama_client = integrations::ollama::Ollama::default()
        .with_default_embed_model("nomic-embed-text")
        .with_default_prompt_model("llama3.2")
        .to_owned();

    let store = PgVector::try_new(pool, VECTOR_SIZE as _).await?;

    indexing::Pipeline::from_loader(FileLoader::new("./src").with_extensions(&["rs"]))
        .then(MetadataQACode::new(ollama_client.clone()))
        .then_chunk(ChunkCode::try_for_language_and_chunk_size(
            "rust",
            10..2048,
        )?)
        .then_in_batch(Embed::new(ollama_client.clone()).with_batch_size(10))
        .then_store_with(store)
        .run()
        .await?;
    Ok(())
}
