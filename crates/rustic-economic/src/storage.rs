use std::sync::Arc;

use crate::domain::{
    BEA_NIPA_COLLECTION, BEA_REGIONAL_COLLECTION, BeaNipaData, BeaRegionalData, CENSUS_COLLECTION,
    CensusData, ECONOMIC_SERIES_COLLECTION, EconomicSeries,
};
use anyhow::Result;
use rustic_storage::{
    MongoDatabase, Repository, SearchCriteria, mongo::repository::MongoRepository,
};
use tokio::sync::Mutex;
use tracing::debug;

#[derive(Debug)]
pub struct EconomicStorageManager {
    db: MongoDatabase,
}

impl EconomicStorageManager {
    pub async fn new(uri: &str, name: &str) -> Result<Self> {
        let mut mdb = MongoDatabase::new(uri, name).await?;
        mdb.register_collection::<String, EconomicSeries>(ECONOMIC_SERIES_COLLECTION.to_owned())
            .await?;
        mdb.register_collection::<String, BeaNipaData>(BEA_NIPA_COLLECTION.to_owned())
            .await?;
        mdb.register_collection::<String, BeaRegionalData>(BEA_REGIONAL_COLLECTION.to_owned())
            .await?;
        mdb.register_collection::<String, CensusData>(CENSUS_COLLECTION.to_owned())
            .await?;
        Ok(Self { db: mdb })
    }

    // private collection accessors
    async fn economic_series(&self) -> Result<Arc<Mutex<MongoRepository<String, EconomicSeries>>>> {
        self.db
            .collection::<String, EconomicSeries>(ECONOMIC_SERIES_COLLECTION.to_string())
            .await
    }

    async fn bea_nipa(&self) -> Result<Arc<Mutex<MongoRepository<String, BeaNipaData>>>> {
        self.db
            .collection::<String, BeaNipaData>(BEA_NIPA_COLLECTION.to_string())
            .await
    }

    async fn bea_regional(&self) -> Result<Arc<Mutex<MongoRepository<String, BeaRegionalData>>>> {
        self.db
            .collection::<String, BeaRegionalData>(BEA_REGIONAL_COLLECTION.to_string())
            .await
    }

    async fn census(&self) -> Result<Arc<Mutex<MongoRepository<String, CensusData>>>> {
        self.db
            .collection::<String, CensusData>(CENSUS_COLLECTION.to_string())
            .await
    }

    // FRED
    pub async fn delete_all_fred_series(&self) -> Result<()> {
        let Ok(repo) = self.economic_series().await else {
            return Err(anyhow::anyhow!("Error getting EconomicSeries Repository"));
        };
        let mut repo = repo.lock().await;
        repo.delete_many(Some(SearchCriteria::new())).await?;
        Ok(())
    }

    pub async fn get_series(&self, series_id: &str) -> Result<Option<EconomicSeries>> {
        let Ok(repo) = self.economic_series().await else {
            return Err(anyhow::anyhow!("Error getting EconomicSeries Repository"));
        };
        let mut repo = repo.lock().await;
        Ok(repo.find_by_id(series_id.to_owned()).await.ok())
    }

    pub async fn upsert_series(&self, series: EconomicSeries) -> Result<()> {
        let Ok(repo) = self.economic_series().await else {
            return Err(anyhow::anyhow!("Error getting EconomicSeries Repository"));
        };
        let mut repo = repo.lock().await;
        repo.update(series).await
    }

    pub async fn list_active(&self) -> Result<Vec<EconomicSeries>> {
        let Ok(repo) = self.economic_series().await else {
            return Err(anyhow::anyhow!("Error getting EconomicSeries Repository"));
        };
        let mut repo = repo.lock().await;
        let criteria = SearchCriteria::new().eq("active", true);

        repo.find(Some(criteria)).await
    }

    // BEA NIPA
    pub async fn delete_all_bea_nipa(&self) -> Result<()> {
        let Ok(repo) = self.bea_nipa().await else {
            return Err(anyhow::anyhow!("Error getting EconomicSeries Repository"));
        };
        let mut repo = repo.lock().await;
        repo.delete_many(Some(SearchCriteria::new())).await?;
        Ok(())
    }

    pub async fn get_bea_nipa(&self, id: &str) -> Result<Option<BeaNipaData>> {
        let Ok(repo) = self.bea_nipa().await else {
            return Err(anyhow::anyhow!("Error getting BeaNipa Repository"));
        };
        let mut repo = repo.lock().await;
        Ok(repo.find_by_id(id.to_owned()).await.ok())
    }

    pub async fn upsert_bea_nipa(&self, data: BeaNipaData) -> Result<()> {
        let Ok(repo) = self.bea_nipa().await else {
            return Err(anyhow::anyhow!("Error getting BeaNipa Repository"));
        };
        let mut repo = repo.lock().await;
        repo.update(data).await
    }

    pub async fn upsert_bea_nipa_bulk(&self, datas: Vec<BeaNipaData>) -> Result<()> {
        let Ok(repo) = self.bea_nipa().await else {
            return Err(anyhow::anyhow!("Error getting BeaNipa Repository"));
        };
        let mut repo = repo.lock().await;
        repo.bulk_update(datas).await
    }

    // BEA Regional
    pub async fn delete_all_bea_regional(&self) -> Result<()> {
        let Ok(repo) = self.bea_regional().await else {
            return Err(anyhow::anyhow!("Error getting EconomicSeries Repository"));
        };
        let mut repo = repo.lock().await;
        repo.delete_many(Some(SearchCriteria::new())).await?;
        Ok(())
    }

    pub async fn get_bea_regional(&self, id: &str) -> Result<Option<BeaRegionalData>> {
        let Ok(repo) = self.bea_regional().await else {
            return Err(anyhow::anyhow!("Error getting BeaRegional Repository"));
        };
        let mut repo = repo.lock().await;
        Ok(repo.find_by_id(id.to_owned()).await.ok())
    }

    // pub async fn upsert_bea_regional(&self, data: BeaRegionalData) -> Result<()> {
    //     let Ok(repo) = self.bea_regional().await else {
    //         return Err(anyhow::anyhow!("Error getting BeaRegional Repository"));
    //     };
    //     let mut repo = repo.lock().await;
    //     repo.update(data).await
    // }
    pub async fn upsert_bea_regional_bulk(&self, datas: Vec<BeaRegionalData>) -> Result<()> {
        let Ok(repo) = self.bea_regional().await else {
            return Err(anyhow::anyhow!("Error getting BeaRegional Repository"));
        };
        let mut repo = repo.lock().await;
        repo.bulk_update(datas).await
    }

    pub async fn get_bea_nipa_by_table(
        &self,
        table_name: &str,
        year: &str,
    ) -> Result<Vec<BeaNipaData>> {
        let Ok(repo) = self.bea_nipa().await else {
            return Err(anyhow::anyhow!("Error getting BeaNipa Repository"));
        };
        let mut repo = repo.lock().await;

        let criteria = SearchCriteria::new()
            .eq("table_name", table_name)
            .eq("time_period", year);
        repo.find(Some(criteria)).await
    }

    pub async fn get_bea_regional_by_table(
        &self,
        table_name: &str,
        year: &str,
    ) -> Result<Vec<BeaRegionalData>> {
        let Ok(repo) = self.bea_regional().await else {
            return Err(anyhow::anyhow!("Error getting BeaRegional Repository"));
        };
        let mut repo = repo.lock().await;

        let criteria = SearchCriteria::new()
            .eq("code", table_name)
            .eq("time_period", year);

        repo.find(Some(criteria)).await
    }

    pub async fn get_bea_regional_filtered(
        &self,
        table_name: &str,
        geo_fips: Option<&str>,
        geo_type: Option<&str>,
        state_prefix: Option<&str>,
        year: &str,
    ) -> Result<Vec<BeaRegionalData>> {
        let Ok(repo) = self.bea_regional().await else {
            return Err(anyhow::anyhow!("Error getting BeaRegional Repository"));
        };
        let mut repo = repo.lock().await;

        // use contains till the table name feld is added. code has tablename + linecode
        let mut criteria = SearchCriteria::new()
            .contains("code", table_name)
            .eq("time_period", year);

        if let Some(fips) = geo_fips {
            criteria = criteria.eq("geo_fips", fips);
        }
        if let Some(gt) = geo_type {
            criteria = criteria.eq("geo_type", gt);
        }
        if let Some(prefix) = state_prefix {
            criteria = criteria.starts_with("geo_fips", prefix);
        }
        debug!("get_bea_regional_filtered SearchCriteria: {:#?}", criteria);
        repo.find(Some(criteria)).await
    }

    // Census
    pub async fn delete_all_census(&self) -> Result<()> {
        let Ok(repo) = self.census().await else {
            return Err(anyhow::anyhow!("Error getting EconomicSeries Repository"));
        };
        let mut repo = repo.lock().await;
        repo.delete_many(Some(SearchCriteria::new())).await?;
        Ok(())
    }

    pub async fn get_census(&self, id: &str) -> Result<Option<CensusData>> {
        let Ok(repo) = self.census().await else {
            return Err(anyhow::anyhow!("Error getting Census Repository"));
        };
        let mut repo = repo.lock().await;
        Ok(repo.find_by_id(id.to_owned()).await.ok())
    }

    pub async fn upsert_census_bulk(&self, datas: Vec<CensusData>) -> Result<()> {
        let Ok(repo) = self.census().await else {
            return Err(anyhow::anyhow!("Error getting Census Repository"));
        };
        let mut repo = repo.lock().await;
        repo.bulk_update(datas).await
    }

    pub async fn upsert_census(&self, data: CensusData) -> Result<()> {
        let Ok(repo) = self.census().await else {
            return Err(anyhow::anyhow!("Error getting Census Repository"));
        };
        let mut repo = repo.lock().await;
        repo.update(data).await
    }

    pub async fn get_census_filtered(
        &self,
        dataset: &str,
        variable: &str,
        geo_fips: Option<&str>,
        geo_type: Option<&str>,
        state_prefix: Option<&str>,
        year: &str,
    ) -> Result<Vec<CensusData>> {
        let Ok(repo) = self.census().await else {
            return Err(anyhow::anyhow!("Error getting Census Repository"));
        };
        let mut repo = repo.lock().await;
        let mut criteria = SearchCriteria::new()
            .eq("dataset", dataset)
            .eq("year", year)
            .eq("variable", variable);

        if let Some(fips) = geo_fips {
            criteria = criteria.eq("geo_fips", fips);
        }
        if let Some(gt) = geo_type {
            criteria = criteria.eq("geo_type", gt);
        }
        if let Some(prefix) = state_prefix {
            criteria = criteria.starts_with("geo_fips", prefix);
        }
        debug!("get_census_filtered SearchCriteria: {:#?}", criteria);

        repo.find(Some(criteria)).await
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    async fn storage_manager() -> Result<EconomicStorageManager> {
        // Find these again for rusticai
        let mongo_db =
            env::var("RUSTIC_AI_DB_NAME").expect("RUSTIC_AI_DB_NAME envrionment variable not set");
        let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI envrionment variable not set");

        EconomicStorageManager::new(&mongo_uri, &mongo_db).await
    }

    #[tokio::test]
    async fn test_get_bea_regional_by_state_prefix() -> Result<()> {
        let manager = storage_manager().await?;

        // query all CA counties
        let rows = manager
            .get_bea_regional_filtered("CAINC1", None, Some("county"), Some("06"), "2024")
            .await
            .unwrap();

        println!("Rows: {}", rows.len());
        assert!(!rows.is_empty(), "Should return CA county rows");

        // verify geo_fips starts with 06
        for row in &rows {
            assert!(
                row.geo_fips.starts_with("06"),
                "geo_fips should start with 06, got {}",
                row.geo_fips
            );
            assert_eq!(row.geo_type, "county");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_bea_regional_by_geo_fips() -> Result<()> {
        let manager = storage_manager().await?;

        // query Sacramento County directly
        let rows = manager
            .get_bea_regional_filtered("CAINC1", Some("06067"), None, None, "2024")
            .await
            .unwrap();

        println!("Rows: {}", rows.len());
        assert!(!rows.is_empty(), "Should return Sacramento County row");
        assert_eq!(rows[0].geo_fips, "06067");
        assert_eq!(rows[0].geo_name, "Sacramento");

        Ok(())
    }

    #[tokio::test]
    async fn test_bea_regional_by_geo_type_state() -> Result<()> {
        let manager = storage_manager().await?;

        let rows = manager
            .get_bea_regional_filtered("CAINC1", None, Some("state"), None, "2024")
            .await
            .unwrap();

        println!("State rows: {}", rows.len());
        assert!(!rows.is_empty(), "Should return state rows");
        for row in &rows {
            assert_eq!(row.geo_type, "state");
            assert!(
                row.geo_fips.ends_with("000"),
                "State FIPS should end with 000, got {}",
                row.geo_fips
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_bea_regional_multiple_years() -> Result<()> {
        let manager = storage_manager().await?;

        let rows = manager
            .get_bea_regional_filtered("CAINC1", Some("48000"), None, None, "2024")
            .await
            .unwrap();

        println!("Texas rows: {}", rows.len());
        assert!(!rows.is_empty(), "Should return Texas rows");
        assert_eq!(rows[0].geo_fips, "48000");

        Ok(())
    }

    #[tokio::test]
    async fn test_bea_regional_by_region() -> Result<()> {
        let manager = storage_manager().await?;

        let rows = manager
            .get_bea_regional_filtered("CAINC1", None, Some("region"), None, "2024")
            .await
            .unwrap();

        println!("Region rows: {}", rows.len());
        assert!(!rows.is_empty(), "Should return region rows");
        for row in &rows {
            assert_eq!(row.geo_type, "region");
        }
        Ok(())
    }

    // ── Census ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_census_by_variable() -> Result<()> {
        let manager = storage_manager().await?;

        let records = manager
            .get_census_filtered("acs5", "B19013_001E", None, None, None, "2024")
            .await
            .unwrap();

        println!("Census median income records: {}", records.len());
        assert!(!records.is_empty(), "Should return median income records");
        for record in &records {
            assert_eq!(record.variable, "B19013_001E");
            assert!(!record.value.is_empty());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_census_by_geo_fips() -> Result<()> {
        let manager = storage_manager().await?;

        let records = manager
            .get_census_filtered("acs5", "B19013_001E", Some("06075"), None, None, "2024")
            .await
            .unwrap();

        println!("SF Census records: {}", records.len());
        assert!(!records.is_empty(), "Should return San Francisco record");
        assert_eq!(records[0].geo_fips, "06075");
        assert_eq!(records[0].geo_name, "San Francisco");

        Ok(())
    }

    #[tokio::test]
    async fn test_census_by_state_prefix() -> Result<()> {
        let manager = storage_manager().await?;

        let records = manager
            .get_census_filtered(
                "acs5",
                "B19013_001E",
                None,
                Some("county"),
                Some("06"),
                "2024",
            )
            .await
            .unwrap();

        println!("CA county census records: {}", records.len());
        assert!(!records.is_empty(), "Should return CA county records");
        for record in &records {
            assert!(record.geo_fips.starts_with("06"), "got {}", record.geo_fips);
            assert_eq!(record.geo_type, Some("county".to_string()));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_census_by_geo_type_state() -> Result<()> {
        let manager = storage_manager().await?;

        let records = manager
            .get_census_filtered("acs5", "B19013_001E", None, Some("state"), None, "2024")
            .await
            .unwrap();

        println!("State census records: {}", records.len());
        assert!(!records.is_empty());
        assert!(
            records.len() >= 50,
            "Should have at least 50 states, got {}",
            records.len()
        );
        for record in &records {
            assert_eq!(record.geo_type, Some("state".to_string()));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_census_maricopa_multiple_variables() -> Result<()> {
        let manager = storage_manager().await?;
        let variables = vec!["B19013_001E", "B25077_001E", "B01003_001E"];
        let mut all_records = Vec::new();

        for variable in &variables {
            let records = manager
                .get_census_filtered("acs5", variable, Some("04013"), None, None, "2024")
                .await
                .unwrap();
            all_records.extend(records);
        }

        println!("Maricopa multi-variable records: {}", all_records.len());
        assert_eq!(
            all_records.len(),
            3,
            "Should return one record per variable"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_census_national() -> Result<()> {
        let manager = storage_manager().await?;

        let records = manager
            .get_census_filtered("acs5", "B19013_001E", Some("00000"), None, None, "2024")
            .await
            .unwrap();

        println!("National census records: {}", records.len());
        assert!(!records.is_empty(), "Should return national record");
        assert_eq!(records[0].geo_fips, "00000");
        Ok(())
    }
}
