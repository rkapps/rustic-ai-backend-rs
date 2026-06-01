use anyhow::Result;
use chrono::{Duration, Utc};
use std::{str::FromStr, sync::Arc};
use tracing::{error, info};

use rustic_providers::{
    BeaClient, CensusClient, DataPoint, FredClient, economic::bea::model::BeaParamValue,
};

use crate::{
    domain::{BeaNipaData, BeaRegionalData, CensusData, EconomicSeries, EconomicSource, Frequency},
    helper::{fips_to_census_geo, geo_type, resolve_years},
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

    // ── CLEAN ──────────────────────────────────────────────────────────────
    pub async fn clean_fred(&self) -> Result<()> {
        self.storage.delete_all_fred_series().await
    }

    pub async fn clean_bea(&self) -> Result<()> {
        self.storage.delete_all_bea_nipa().await?;
        self.storage.delete_all_bea_regional().await?;
        Ok(())
    }

    pub async fn clean_census(&self) -> Result<()> {
        self.storage.delete_all_census().await
    }

    // ── READ ──────────────────────────────────────────────────────────────

    pub async fn get_fred_series(
        &self,
        series_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<DataPoint>> {
        let stored = self.storage.get_series(series_id).await?;
        if stored.is_none() {
            return Ok(Vec::new());
        }
        let stored = stored.unwrap();

        let obs = match limit {
            Some(n) => stored.observations.into_iter().take(n).collect(),
            None => stored.observations,
        };
        Ok(obs)
    }

    pub async fn get_geo_fips(&self) -> Result<Vec<BeaParamValue>> {
        self.bea.get_geo_fips().await
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

            // if stored.is_empty() {
            //     return Err(anyhow::anyhow!(
            //         "BEA NIPA {} {} not found — run pipeline first",
            //         table_name,
            //         year
            //     ));
            // }
            result.extend(stored);
        }

        Ok(result)
    }

    pub async fn get_bea_regional(
        &self,
        table_name: &str,
        geo_fips: Option<&str>,
        geo_type: Option<&str>,
        state_prefix: Option<&str>,
        year: &str,
    ) -> Result<Vec<BeaRegionalData>> {
        let years = resolve_years(year);
        let mut result = Vec::new();

        for y in &years {
            let stored = self
                .storage
                .get_bea_regional_filtered(table_name, geo_fips, geo_type, state_prefix, y)
                .await?;
            result.extend(stored);
        }

        Ok(result)
    }

    pub async fn get_census_data(
        &self,
        variables: &[&str],
        dataset: &str,
        geo_fips: Option<&str>,
        geo_type: Option<&str>,
        state_prefix: Option<&str>,
        year: &str,
    ) -> Result<Vec<CensusData>> {
        // expand LAST5 to actual years
        let years = resolve_years(year);

        let mut result = Vec::new();

        for y in &years {
            for variable in variables {
                let stored = self
                    .storage
                    .get_census_filtered(dataset,  variable,
                        geo_fips, geo_type, state_prefix, y
                        )
                    .await?;

                result.extend(stored);
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
        name: &str,
        category: &str,
        sectors: &[String],
    ) -> Result<()> {
        let data = self
            .fred
            .get_series(series_id, Some(frequency), Some(limit))
            .await?;

        let series = EconomicSeries {
            id: series_id.to_string(),
            series_id: series_id.to_string(),
            source: EconomicSource::Fred,
            name: name.to_string(),
            frequency: Frequency::from_str(frequency)?,
            category: category.to_string(),
            sectors: sectors.to_vec(),
            active: true,
            observations: data.data_points,
            last_refreshed: Some(Utc::now()),
            next_refresh: Some(Self::next_refresh(frequency)),
        };
        info!(
            "Series: {} observations: {:?}",
            series_id,
            series.observations.len()
        );
        self.storage.upsert_series(series).await
    }

    pub async fn update_bea_nipa(
        &self,
        table_name: &str,
        frequency: &str,
        year: &str,
    ) -> Result<()> {
        info!(
            "Bea nipa table_name: {} frequency: {} years: {}",
            table_name, frequency, year,
        );

        let rows = self.bea.get_nipa(table_name, frequency, year).await?;
        let mut all_records = Vec::new();
        for row in &rows {
            let id = format!(
                "bea_nipa_{}_{}_{}",
                table_name, row.series_code, row.time_period
            );

            let new_record = BeaNipaData {
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
            };
            all_records.push(new_record);
        }
        match self.storage.upsert_bea_nipa_bulk(all_records).await {
            Ok(c) => c,
            Err(e) => error!("Update bea_nipa bulk error: {}", e),
        };
        Ok(())
    }

    pub async fn update_bea_regional(
        &self,
        tables: Vec<(&str, &str)>,
        geo_fips: &[BeaParamValue],
        years: &[&str],
    ) -> Result<()> {
        // loop through the years
        for year in years {
            // loop through the tables
            for table in &tables {
                let mut all_rows = Vec::new();

                // loop through the geo-fips
                for (i, geo_fip) in geo_fips.iter().enumerate() {
                    let rows = match self
                        .bea
                        .get_regional(table.0, table.1, &geo_fip.key, year)
                        .await
                    {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::warn!(
                                "BEA Regional for year: {} table: {} geo_flip {:?} failed: {}",
                                year,
                                table.0,
                                geo_fip.key,
                                e
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;

                            match self
                                .bea
                                .get_regional(table.0, table.1, &geo_fip.key, year)
                                .await
                            {
                                Ok(c) => c,
                                Err(e) => {
                                    tracing::warn!(
                                        "BEA Regional for year: {} table: {} geo_flip {:?} failed: {}",
                                        year,
                                        table.0,
                                        geo_fip.key,
                                        e
                                    );
                                    Vec::new()
                                }
                            }
                        }
                    };

                    for row in rows {
                        let id = format!(
                            "bea_regional_{}_{}_{}",
                            table.0, row.geo_fips, row.time_period
                        );
                        let new_row = BeaRegionalData {
                            id,
                            code: row.code.clone(),
                            geo_fips: row.geo_fips.clone(),
                            geo_name: row.geo_name.clone(),
                            geo_type: geo_type(geo_fip).to_owned(),
                            time_period: row.time_period.clone(),
                            data_value: row.data_value.clone(),
                            cl_unit: row.cl_unit.clone(),
                            unit_mult: row.unit_mult.clone(),
                            last_refreshed: Utc::now(),
                            next_refresh: Self::next_refresh("a"),
                        };
                        all_rows.push(new_row);
                    }

                    if i % 20 == 0 {
                        info!("i: {} geo_fip: {}", i, geo_fip.key);
                        tokio::time::sleep(tokio::time::Duration::from_millis(10000)).await;
                    }
                }

                info!(
                    "all records for year: {} table: {} - {}",
                    year,
                    table.0,
                    all_rows.len()
                );

                match self.storage.upsert_bea_regional_bulk(all_rows).await {
                    Ok(c) => c,
                    Err(e) => error!("Update census_bulk error: {}", e),
                };
            }
        }

        Ok(())
    }

    pub async fn update_census(
        &self,
        dataset: &str,
        variables: &[&str],
        years: Vec<&str>,
        geo_fips: Vec<BeaParamValue>,
    ) -> Result<()> {
        for year in &years {
            let mut vars = vec!["NAME"];
            vars.extend_from_slice(variables);

            let mut all_records = Vec::new();

            for (i, geo_fip) in geo_fips.iter().enumerate() {
                // skip divisions and metro portions
                if geo_fip.key == "division" || geo_fip.key == "metro" {
                    continue;
                }

                let census_geo = fips_to_census_geo(&geo_fip.key);
                if i % 40 == 0 {
                    info!(
                        "i: {} Census dataset: {} variables: {:?} geo: {} years: {}",
                        i, dataset, vars, geo_fip.key, year,
                    );
                }

                let records = match self.census.get_acs(year, "acs5", &vars, &census_geo).await {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                for record in &records {
                    let id = format!(
                        "census_{}_{}_{}_{}",
                        dataset, year, record.variable, geo_fip.key
                    );
                    let geo_name = record.geo_name.clone();
                    // "San Francisco County, California" → "San Francisco"
                    let geo_name = geo_name
                        .split(',')
                        .next()
                        .unwrap_or(&geo_name)
                        .trim()
                        .replace(" County", "")
                        .replace(" Parish", "") // Louisiana uses Parish
                        .replace(" Borough", "") // Alaska uses Borough
                        .replace(" Census Area", "") // Alaska
                        .trim()
                        .to_string();
                    let new_record = CensusData {
                        id,
                        dataset: dataset.to_string(),
                        year: year.to_string(),
                        variable: record.variable.clone(),
                        value: record.value.clone(),
                        geo_name,
                        geo_fips: geo_fip.key.clone(),
                        geo_type: record.geo_type.clone(),
                        last_refreshed: Utc::now(),
                        next_refresh: Self::next_refresh("a"),
                    };
                    all_records.push(new_record);
                }
            }

            info!("all records for year: {} - {}", year, all_records.len());
            match self.storage.upsert_census_bulk(all_records).await {
                Ok(c) => c,
                Err(e) => error!("Update census_bulk error: {}", e),
            };
        }

        Ok(())
    }
}
