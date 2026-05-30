// rustic-economic/src/service/impl.rs

use anyhow::Result;
use chrono::{Duration, Utc};
use std::sync::Arc;

use rustic_providers::{BeaClient, CensusClient, DataPoint, FredClient};

use crate::{
    domain::{BeaNipaData, BeaRegionalData, CensusData, EconomicSeries},
    storage::EconomicStorageManager,
};

#[derive(Debug, Clone)]
pub struct EconomicDataService {
    storage: Arc<EconomicStorageManager>,
    fred: Arc<FredClient>,
    bea: Arc<BeaClient>,
    census: Arc<CensusClient>,
}

impl EconomicDataService {
    pub fn new(
        storage: Arc<EconomicStorageManager>,
        fred: Arc<FredClient>,
        bea: Arc<BeaClient>,
        census: Arc<CensusClient>,
    ) -> Self {
        Self {
            storage,
            fred,
            bea,
            census,
        }
    }

    fn next_refresh(frequency: &str) -> chrono::DateTime<Utc> {
        let now = Utc::now();
        match frequency {
            "m" => now + Duration::days(1),
            "q" => now + Duration::days(7),
            "a" => now + Duration::days(30),
            _ => now + Duration::days(1),
        }
    }

    // ── READ ──────────────────────────────────────────────────────────────

    pub async fn get_fred_series(
        &self,
        series_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<DataPoint>> {
        let stored = self.storage.get_series(series_id).await?.ok_or_else(|| {
            anyhow::anyhow!("Series {} not found — run pipeline first", series_id)
        })?;

        let obs = match limit {
            Some(n) => stored.observations.into_iter().take(n).collect(),
            None => stored.observations,
        };
        Ok(obs)
    }

    pub async fn get_bea_nipa(&self, table_name: &str, year: &str) -> Result<Vec<BeaNipaData>> {
        let years: Vec<String> = if year == "LAST5" {
            vec![
                "2025".to_string(),
                "2024".to_string(),
                "2023".to_string(),
                "2022".to_string(),
                "2021".to_string(),
            ]
        } else {
            year.split(',').map(|y| y.trim().to_string()).collect()
        };
        let mut result = Vec::new();

        for y in &years {
            let stored = self.storage.get_bea_nipa_by_table(table_name, y).await?;

            if stored.is_empty() {
                return Err(anyhow::anyhow!(
                    "BEA NIPA {} {} not found — run pipeline first",
                    table_name,
                    year
                ));
            }
            result.extend(stored);
        }

        Ok(result)
    }

    pub async fn get_bea_regional(
        &self,
        table_name: &str,
        geo_fips: &str,
        year: &str,
    ) -> Result<Vec<BeaRegionalData>> {
        // expand LAST5 to actual years
        let years: Vec<String> = if year == "LAST5" {
            vec![
                "2025".to_string(),
                "2024".to_string(),
                "2023".to_string(),
                "2022".to_string(),
                "2021".to_string(),
            ]
        } else {
            year.split(',').map(|y| y.trim().to_string()).collect()
        };

        let mut result = Vec::new();

        for y in &years {
            let stored = self
                .storage
                .get_bea_regional_by_table(table_name, y)
                .await?;

            if stored.is_empty() {
                return Err(anyhow::anyhow!(
                    "BEA Regional {} {} not found — run pipeline first",
                    table_name,
                    y
                ));
            }

            // filter by geo_fips
            let filtered = if geo_fips == "STATE" {
                stored
            } else {
                stored
                    .into_iter()
                    .filter(|r| r.geo_fips == geo_fips)
                    .collect()
            };

            result.extend(filtered);
        }
        Ok(result)
    }

    pub async fn get_census_data(
        &self,
        variables: &[&str],
        dataset: &str,
        year: &str,
        geo_fips: &str,
    ) -> Result<Vec<CensusData>> {
        // expand LAST5 to actual years
        let years: Vec<String> = if year == "LAST5" {
            vec![
                "2025".to_string(),
                "2024".to_string(),
                "2023".to_string(),
                "2022".to_string(),
                "2021".to_string(),
            ]
        } else {
            year.split(',').map(|y| y.trim().to_string()).collect()
        };

        let mut result = Vec::new();

        for y in &years {
            for variable in variables {
                let stored = self
                    .storage
                    .get_census_by_variable(dataset, y, variable)
                    .await?;

                // filter by geo_fips
                let filtered = if geo_fips == "STATE" {
                    stored
                } else {
                    stored
                        .into_iter()
                        .filter(|r| r.geo_fips == geo_fips)
                        .collect()
                };

                result.extend(filtered);
            }
        }
        Ok(result)
    }

    // ── UPDATE (pipeline only) ────────────────────────────────────────────

    pub async fn update_fred_series(
        &self,
        series_id: &str,
        frequency: &str,
        limit: usize,
    ) -> Result<()> {
        let data = self
            .fred
            .get_series(series_id, Some(frequency), Some(limit))
            .await?;

        let series = EconomicSeries {
            id: series_id.to_string(),
            series_id: series_id.to_string(),
            observations: data.data_points,
            last_refreshed: Some(Utc::now()),
            next_refresh: Some(Self::next_refresh(frequency)),
            ..Default::default()
        };
        self.storage.upsert_series(series).await
    }

    pub async fn update_bea_nipa(
        &self,
        table_name: &str,
        frequency: &str,
        year: &str,
    ) -> Result<()> {
        let rows = self.bea.get_nipa(table_name, frequency, year).await?;

        for row in &rows {
            let id = format!(
                "bea_nipa_{}_{}_{}",
                table_name, row.series_code, row.time_period
            );
            self.storage
                .upsert_bea_nipa(BeaNipaData {
                    id,
                    table_name: row.table_name.clone(),
                    series_code: row.series_code.clone(),
                    line_number: row.line_number.clone(),
                    line_description: row.line_description.clone(),
                    time_period: row.time_period.clone(),
                    metric_name: row.metric_name.clone(),
                    cl_unit: row.cl_unit.clone(),
                    unit_mult: row.unit_mult.clone(),
                    data_value: row.data_value.clone(),
                    last_refreshed: Utc::now(),
                    next_refresh: Self::next_refresh("m"),
                })
                .await?;
        }
        Ok(())
    }

    pub async fn update_bea_regional(
        &self,
        table_name: &str,
        line_code: &str,
        geo_fips: &str,
        years: &str,
    ) -> Result<()> {
        let rows = self
            .bea
            .get_regional(table_name, line_code, geo_fips, years)
            .await?;

        for row in &rows {
            let id = format!(
                "bea_regional_{}_{}_{}",
                table_name, row.geo_fips, row.time_period
            );
            self.storage
                .upsert_bea_regional(BeaRegionalData {
                    id,
                    code: row.code.clone(),
                    geo_fips: row.geo_fips.clone(),
                    geo_name: row.geo_name.clone(),
                    time_period: row.time_period.clone(),
                    data_value: row.data_value.clone(),
                    cl_unit: row.cl_unit.clone(),
                    unit_mult: row.unit_mult.clone(),
                    last_refreshed: Utc::now(),
                    next_refresh: Self::next_refresh("a"),
                })
                .await?;
        }
        Ok(())
    }

    pub async fn update_census_data(
        &self,
        variables: &[&str],
        geo: &str,
        dataset: &str,
        year: &str,
    ) -> Result<()> {
        let records = self.census.get_acs(year, dataset, variables, geo).await?;

        for record in &records {
            let id = format!(
                "census_{}_{}_{}_{}",
                dataset, year, record.variable, record.geo_fips
            );
            self.storage
                .upsert_census(CensusData {
                    id,
                    dataset: dataset.to_string(),
                    year: year.to_string(),
                    variable: record.variable.clone(),
                    value: record.value.clone(),
                    geo_name: record.geo_name.clone(),
                    geo_fips: record.geo_fips.clone(),
                    geo_type: record.geo_type.clone(),
                    last_refreshed: Utc::now(),
                    next_refresh: Self::next_refresh("a"),
                })
                .await?;
        }
        Ok(())
    }
}
