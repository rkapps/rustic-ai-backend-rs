use anyhow::Result;
use futures::StreamExt;
use reqwest::StatusCode;
use std::{convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use axum::{
    Extension, Json,
    extract::{Path, State},
    response::{IntoResponse, Sse, sse::Event},
};
use rustic_ai_domain::{
    chats::{chat::Chat, config::ChatConfig},
    dto::chat::{ChatRequest, ChatResponse},
};
use rustic_ai_services::chat::ChatsService;

use crate::middleware::firebase_auth::FirebaseClaims;

pub async fn create_chat_handler(
    State(chat_service): State<Arc<ChatsService>>,
    Extension(user): Extension<FirebaseClaims>, // 👈 opt-in
    Json(payload): Json<ChatConfig>,
) -> Result<Json<Chat>, (StatusCode, String)> {
    debug!("config: {:?}", payload);
    let chat = chat_service
        .create_chat(user.sub, payload)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Save Chat error: {}", e)))?;
    Ok(Json(chat))
}

pub async fn chat_completion_handler(
    State(chats_service): State<Arc<ChatsService>>,
    Extension(user): Extension<FirebaseClaims>, // 👈 opt-in
    Json(payload): Json<ChatRequest>,
// ) {
    ) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    //get chat
    debug!("started chat_completion_streaming_handler {:?}", payload.id);

    let response = chats_service
        .chat_completion(user.sub, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Chat Completion error: {}", e),
            )
        })?;
    Ok(Json(response))
}

pub async fn chat_completion_streaming_handler(
    State(chats_service): State<Arc<ChatsService>>,
    Extension(user): Extension<FirebaseClaims>, // 👈 opt-in
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    debug!("started chat_completion_streaming_handler: {}", payload.id);

    let stream = match chats_service
        .chat_completion_streaming(user.sub.clone(), payload.clone())
        .await
    {
        Ok(stream) => stream,
        Err(e) => {
            error!("{:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let final_content = Arc::new(Mutex::new(String::new()));

    let event_stream = stream.then(move |chunk_result| {
        // ✅ Clone handles into the async block
        let chat_service = chats_service.clone();
        let payload = payload.clone();
        let final_content = final_content.clone();
        let uid = user.sub.clone();

        async move {
            match chunk_result {
                Ok(chunk) => {
                    // ✅ Always accumulate content once (was being doubled before)
                    {
                        let mut fc = final_content.lock().await;
                        fc.push_str(&chunk.content);
                    }

                    // ✅ Save only on the final chunk
                    if chunk.is_final {
                        let fc = final_content.lock().await;
                        info!("final_content: {:?}", *fc);

                        // ✅ .await now works inside .then()
                        if let Err(e) = chat_service
                            .save_streaming_message(
                                uid,
                                &payload.id,
                                &payload.prompt,
                                &fc,
                                &chunk.response_id,
                            )
                            .await
                        {
                            // ❌ Don't silently swallow errors
                            error!("Failed to save streaming message: {}", e);
                        }
                    }

                    match serde_json::to_string(&chunk) {
                        Ok(c) => Ok::<Event, Infallible>(Event::default().data(c).event("message")),
                        Err(e) => Ok::<Event, Infallible>(
                            Event::default().data(format!("{}", e)).event("error"),
                        ),
                    }
                }
                Err(e) => {
                    error!("error: {:?}", e);
                    Ok::<Event, Infallible>(Event::default().data(format!("{}", e)).event("error"))
                }
            }
        }
    });

    Sse::new(event_stream).into_response()
}


pub async fn delete_chat_handler(
    State(chats_service): State<Arc<ChatsService>>,
    Extension(user): Extension<FirebaseClaims>, // 👈 opt-in
    Path(id): Path<String>,
) -> Result<(), (StatusCode, String)> {
    debug!("get chat handler {:?}", id);

    chats_service
        .delete_chat(user.sub, id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Get Chat error: {}", e)))?;

    Ok(())
}

pub async fn get_all_chats_handler(
    State(chats_service): State<Arc<ChatsService>>,
    Extension(user): Extension<FirebaseClaims>, // 👈 opt-in
) -> Result<Json<Vec<Chat>>, (StatusCode, String)> {
    debug!("User sub: {:?}", user.sub);
    let chats = chats_service.get_all_chats(user.sub).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Get All Chats error: {}", e),
        )
    })?;
    Ok(Json(chats))
}

pub async fn get_chat_handler(
    State(chats_service): State<Arc<ChatsService>>,
    Extension(user): Extension<FirebaseClaims>, // 👈 opt-in
    Path(id): Path<String>,
) -> Result<Json<Chat>, (StatusCode, String)> {
    debug!("get chat handler {:?}", id);

    let chat = chats_service
        .get_chat(user.sub, id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Get Chat error: {}", e)))?;

    Ok(Json(chat))
}
