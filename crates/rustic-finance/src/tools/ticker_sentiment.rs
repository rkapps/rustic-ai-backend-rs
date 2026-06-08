use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use rustic_core::Tool;
use rustic_ml::EmbeddingClient;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::info;

use crate::{core::tickers::sentiments::search_ticker_sentiments, storage::reader::StorageReader};

#[derive(Debug)]
pub struct TickerSentimentTool {
    storage_service: Arc<dyn StorageReader>,
    embedding_client: Arc<dyn EmbeddingClient>,
}
impl TickerSentimentTool {
    pub fn new(
        storage_service: Arc<dyn StorageReader>,
        embedding_client: Arc<dyn EmbeddingClient>,
    ) -> TickerSentimentTool {
        Self {
            storage_service,
            embedding_client,
        }
    }
}

#[async_trait]
impl Tool for TickerSentimentTool {
    fn name(&self) -> String {
        "ticker_sentiment".to_string()
    }

    fn description(&self) -> String {
        "Returns relevant sentiment analysis and news for a stock ticker, \
 filtered by the user's query context. Use this to understand market \
 narrative, news-driven momentum, and investor sentiment."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "symbols": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of stock ticker symbols to find peers for"
                },
                "query": {
                "type": "string",
                "description": "The user's query or context used to find relevant sentiment. e.g. 'AI chip demand', 'earnings outlook', 'competitive positioning'"
            }
            },
            "required": ["symbols", "query"]
        })
    }

    async fn execute(&self, value: serde_json::Value) -> Result<Value> {
        #[derive(Debug, Deserialize)]
        struct Params {
            symbols: Vec<String>,
            query: String,
            #[serde(default = "default_limit")]
            limit: usize,
        }
        fn default_limit() -> usize {
            10
        }

        let start = std::time::Instant::now();

        let params: Params = serde_json::from_value(value.clone())
            .map_err(|e| anyhow::anyhow!("Failed to deserialize params: {:?} — {:?}", value, e))?;

        info!(
            "Ticker sentiment symbols {:?} query {:#?} ",
            params.symbols, params.query
        );

        let sentiments = search_ticker_sentiments(
            self.storage_service.clone(),
            self.embedding_client.clone(),
            params.symbols,
            params.query,
            params.limit,
        )
        .await?;

        let elapsed = start.elapsed();
        info!(
            "Sentiments: {:?}  {:.1}s",
            sentiments.len(),
            elapsed.as_secs_f32()
        );

        Ok(json!({ "sentiments": sentiments }))
    }
}
