use agentic_core::agent::service::AgentService;
use anyhow::Result;
use axum::{
    Router,
    http::{HeaderValue, Method},
    middleware::from_fn_with_state,
    routing::{delete, get, post},
};
use bin_shared::{
    config::loader::load_app_config,
    logger,
    services::{get_storage_service, load_agent_config},
    templates::loader::load_templates,
};
use rustic_ai_api::{
    handlers::{
        chats::{
            chat_completion_handler, chat_completion_streaming_handler, create_chat_handler,
            delete_chat_handler, get_all_chats_handler, get_chat_handler,
        },
        rustic::{get_llm_providers_handler, get_templates_handler},
    },
    middleware::firebase_auth::firebase_auth_middleware,
    state::AppState,
};
use rustic_ai_services::{chat::ChatsService, rustic::RusticService};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::debug;

#[tokio::main]

async fn main() -> Result<()> {
    logger::set_logger();

    // // initialize storage and the services
    // let storage = ChatStorage::new(
    //     "agenticdb".to_string(),
    //     "data/agenticdb".to_string(),
    //     "chats".to_string(),
    // )
    // .await?;
    let app_config = match load_app_config().await {
        Ok(c) => c,
        Err(e) => return Err(anyhow::anyhow!(format!("Loading app Config error: {}", e)))
    };
    let agent_config = load_agent_config(&app_config).await;
    let templates = match load_templates().await {
        Ok(c) => c,
        Err(e) => return Err(anyhow::anyhow!(format!("Loading templates error: {}", e)))
    };

    let agent_service = AgentService::with_config(agent_config);
    let rustic_service = RusticService::new(Arc::new(agent_service));

    let storage_service = get_storage_service().await?;
    let chats_service = ChatsService::new(storage_service.clone(), rustic_service.clone());

    let app_state = AppState::new(
        Arc::new(chats_service),
        Arc::new(rustic_service),
        Some(templates),
    )
    .await?;

    let origins = [
        "http://localhost:4200".parse::<HeaderValue>().unwrap(),
        "http://localhost:4201".parse::<HeaderValue>().unwrap(),
        "http://localhost:4202".parse::<HeaderValue>().unwrap(),
        "https://rustic-ai-rkapps.web.app"
            .parse::<HeaderValue>()
            .unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let chats_routes = Router::new()
        .route("/", get(get_all_chats_handler))
        .route("/{id}", get(get_chat_handler))
        .route("/{id}", delete(delete_chat_handler))
        .route("/create", post(create_chat_handler))
        .route("/completion", post(chat_completion_handler))
        .route(
            "/completion_streaming",
            post(chat_completion_streaming_handler),
        )
        .route_layer(from_fn_with_state(
            app_state.clone(),
            firebase_auth_middleware, // 👈 all chats routes protected
        ));

    let app = Router::new()
        .route("/llm/providers", get(get_llm_providers_handler))
        .route("/templates", get(get_templates_handler))
        .nest("/chats", chats_routes)      // ✅ all protected

        // .route("/chats", get(get_all_chats_handler))
        // .route("/chats/{id}", get(get_chat_handler))
        // .route("/chats/create", post(create_chat_handler))
        // .route("/chats/completion", post(chat_completion_handler))
        // .route("/chats/completion_streaming", post(chat_completion_streaming_handler))
        // .route("/chats/save_streaming_message", post(save_streaming_message_handler))
        .layer(cors)
        .with_state(app_state) // Shared state
        ;

    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("🚀 Listening on {:?}", listener);

    axum::serve(listener, app)
        .with_graceful_shutdown(handle_shutdown())
        .await
        .unwrap();

    Ok(())
}

async fn handle_shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    debug!("handled shutdown");
}
