pub mod config;
pub mod error;
mod events;
mod extractors;
pub mod handler;
pub mod openapi;
pub mod pb;

use clickhouse::Client;
pub use config::*;
pub use error::*;

use anyhow::Context;
use chat_core::{extract_user, set_layer, DecodingKey, TokenVerify, User};
use handler::create_event_handler;
use openapi::OpenApiRouter;

use std::{fmt, ops::Deref, sync::Arc};
use tokio::fs;
use tower_http::cors::{Any, CorsLayer};

use axum::{http::Method, middleware::from_fn_with_state, routing::post, Router};
pub use config::AppConfig;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) client: Client,
}

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_headers(Any)
        .allow_origin(Any);

    let api = Router::new()
        .route("/event", post(create_event_handler))
        .layer(from_fn_with_state(state.clone(), extract_user::<AppState>))
        .layer(cors);
    let app = Router::new().openapi().nest("/api", api).with_state(state);
    Ok(set_layer(app))
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir)
            .await
            .context("create base dir failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pd failed")?;
        let mut client = Client::default()
            .with_url(&config.server.db_url)
            .with_database(&config.server.db_name);
        if let Some(user) = &config.server.db_user {
            client = client.with_user(user);
        }
        if let Some(password) = &config.server.db_password {
            client = client.with_password(password);
        }
        Ok(Self {
            inner: Arc::new(AppStateInner { config, dk, client }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}
