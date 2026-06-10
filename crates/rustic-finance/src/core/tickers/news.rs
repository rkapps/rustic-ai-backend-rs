use crate::{
    domain::{Ticker, TickerNews, TickerNewsEntity},
    storage::{
        mongo::writer::FinanceMongoStorageWriter, reader::StorageReader,
        writer::TickerNewsStorageWriter,
    },
};
use anyhow::Result;
use rustic_providers::finance::service::ProviderService;
use std::sync::Arc;
use tracing::{debug, error, info};

pub async fn get_ticker_news(
    reader: Arc<dyn StorageReader>,
    symbol: &str,
) -> Result<Vec<TickerNewsEntity>> {
    let news = reader
        .get_ticker_news(symbol)
        .await
        .map_err(|e| anyhow::anyhow!(format!("Get Ticker Groups error: {}", e)))?;

    debug!("Ticker {} news: {}", symbol, news.len());
    let news_entity: Vec<TickerNewsEntity> = news
        .iter()
        .map(|n| {
            let entity = n.clone();
            TickerNewsEntity {
                date: entity.date,
                description: entity.description,
                source: entity.source,
                symbol: entity.symbol,
                title: entity.title,
                url: entity.url,
            }
        })
        .collect();
    Ok(news_entity)
}

pub async fn update_tickers_news(
    writer: Arc<FinanceMongoStorageWriter>,
    provider_service: Arc<ProviderService>,
    all_tickers: Vec<Ticker>,
) -> Result<()> {
    let length = all_tickers.len();

    for (i, ticker) in all_tickers.into_iter().enumerate() {
        let mut updated_news: Vec<TickerNews> = Vec::new();

        if i % 20 == 0 {
            info!(
                "Updating Ticker News: {} {}/{}",
                ticker.symbol,
                i + 1,
                length
            );
        }
        match provider_service.get_ticker_news(&ticker.symbol).await {
            Ok(c) => {
                let news = TickerNews::from_tiingo_batch(&ticker.symbol, c)?;
                updated_news.extend(news)
            }
            Err(e) => error!("Ticker News error {}: {}", ticker.symbol, e),
        }

        // bulk write at the end
        if !updated_news.is_empty() {
            writer
                .save_ticker_news(&ticker.symbol, updated_news)
                .await?;
        }
    }

    Ok(())
}
