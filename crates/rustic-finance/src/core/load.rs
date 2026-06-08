use anyhow::Result;
use rustic_ml::EmbeddingClient;
use rustic_providers::finance::service::ProviderService;
use tracing::info;
use std::{collections::HashMap, sync::Arc};

use crate::{core::tickers::update::{update_all_ticker_overview_embeddings, update_all_tickers}, domain::{Ticker, TickerControl, dto::ticker_seed::TickerSeed}, storage::{FinanceMongoStorageReader, mongo::writer::FinanceMongoStorageWriter, reader::TickerControlStorageReader}};

pub async fn load_tickers(
    reader: Arc<FinanceMongoStorageReader>,
    writer: Arc<FinanceMongoStorageWriter>,
    provider_service: Arc<ProviderService>,
    embedding_client: Arc<dyn EmbeddingClient>,
    ticker_seeds: &[TickerSeed],
    update: bool,
) -> Result<()> {

    info!("Loading tickers: {} update: {}", ticker_seeds.len(), update);

    let all_controls = reader.get_ticker_controls().await?;
    let mut control_map: HashMap<String, TickerControl> = all_controls
        .into_iter()
        .map(|c| (c.symbol.clone(), c))
        .collect();

    let mut all_tickers = Vec::new();
    let mut all_new_controls = Vec::new();
    for seed in ticker_seeds.iter() {
        let tc = control_map
            .remove(&seed.symbol)
            .unwrap_or_else(|| TickerControl::new(seed.clone()));
        let ticker = Ticker::new(seed.clone());
        all_tickers.push(ticker);
        all_new_controls.push(tc);
    }

    update_all_tickers(
        reader.clone(),
        writer.clone(),
        provider_service,
        embedding_client.clone(),
        all_new_controls,
        all_tickers.clone(),
        update,
    )
    .await?;

    // update ticker overview embeddings
    update_all_ticker_overview_embeddings(
        writer.clone(),
        embedding_client.clone(),
        all_tickers,
    )
    .await?;

    Ok(())
}
