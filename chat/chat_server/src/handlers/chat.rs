use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{AppError, AppState, CreateChat, UpdateChat};
use chat_core::User;

#[utoipa::path(
        get,
        path = "/api/chats",
        responses(
            (status = 200, description = "List of chats", body = Vec<Chat>),
        ),
        security(
            ("token" = [])
        )
    )]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.fetch_chats(user.id as _, user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(chat)))
}

#[utoipa::path(
        post,
        path = "/api/chats",
        responses(
            (status = 201, description = "Chat Created", body = Chat),
        ),
        security(
            ("token" = [])
        )
    )]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state
        .create_chat(input, user.id as _, user.ws_id as _)
        .await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

#[utoipa::path(
        get,
        path = "/api/chats/{id}",
        params(("id"=u64, Path, description="Chat ID")),
        responses(
            (status = 201, description = "Chat Created", body = Chat),
            (status = 404, description = "Chat not found", body = ErrorOutput)
        ),
        security(
            ("token" = [])
        )
    )]
pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id as _).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("chat with id {} not found", id))),
    }
}

pub(crate) async fn update_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.update_chat_by_id(input, id as _).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("chat with id {} not found", id))),
    }
}

pub(crate) async fn delete_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.delete_chat_by_id(id as _).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(format!("chat with id {} not found", id))),
    }
}
