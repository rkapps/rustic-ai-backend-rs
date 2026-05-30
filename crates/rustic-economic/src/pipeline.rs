use anyhow::Result;
use std::sync::Arc;

use crate::service::EconomicDataService;

pub struct EconomicDataPipeline {
    service: Arc<EconomicDataService>,
}

impl EconomicDataPipeline {
    pub async fn run(&self) -> Result<()> {
        tokio::try_join!(self.run_fred(), self.run_bea(), self.run_census(),)?;
        Ok(())
    }

    async fn run_fred(&self) -> Result<()> {
        let series = vec![
            ("CPIAUCSL", "m", 12),
            ("UMCSENT", "m", 12),
            ("UNRATE", "m", 12),
            ("DSPIC96", "m", 12),
            ("PCE", "m", 12),
            ("HOUST", "m", 12),
            ("PERMIT", "m", 12),
            ("RSFSXMV", "m", 12),
            ("DFFFRC1A027NBEA", "a", 5),
            ("DFDHRC1Q027SBEA", "q", 8),
            ("DCAFRC1A027NBEA", "a", 5),
            ("DREQRC1Q027SBEA", "q", 8),
        ];

        for (series_id, frequency, limit) in series {
            self.service
                .update_fred_series(series_id, frequency, limit)
                .await?;
        }
        Ok(())
    }

    async fn run_bea(&self) -> Result<()> {
        let years = "2026,2025,2024,2023,2022,2021,2020";

        // NIPA
        self.service.update_bea_nipa("T20100", "A", years).await?;

        // Regional — all states
        self.service
            .update_bea_regional("CAINC1", "1", "STATE", years)
            .await?;
        self.service
            .update_bea_regional(
                "SASUMMARY",
                "1",
                "STATE",
                "2026,2025,2024,2023,2022,2021,2020",
            )
            .await?;

        // Regional — key counties
        let counties = "04013,48453,48491,06037,36061,12086";
        self.service
            .update_bea_regional(
                "CAINC1",
                "1",
                counties,
                "2026,2025,2024,2023,2022,2021,2020",
            )
            .await?;

        Ok(())
    }

    async fn run_census(&self) -> Result<()> {
        let variables = vec![
            "B19013_001E", // median income
            "B01002_001E", // median age
            "B01003_001E", // population
            "B25003_002E", // owner occupied
            "B25077_001E", // median home value
            "B17001_002E", // below poverty
            "B23025_005E", // unemployed
        ];

        let vars: Vec<&str> = variables.iter().map(|s| *s).collect();

        let years = vec!["2026", "2025", "2024", "2023", "2022", "2021", "2020"];
        // all counties for all states
        let states = vec![
            "01", "02", "04", "05", "06", "08", "09", "10", "11", "12", "13", "15", "16", "17",
            "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
            "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "44", "45", "46",
            "47", "48", "49", "50", "51", "53", "54", "55", "56",
        ];

        // all states
        for year in years {
            self.service
                .update_census_data(
                    &variables.iter().map(|s| *s).collect::<Vec<_>>(),
                    "state:*",
                    "acs5",
                    year,
                )
                .await?;

            for state in &states {
                self.service
                    .update_census_data(
                        &vars,
                        &format!("county:*&in=state:{}", state),
                        "acs5",
                        year,
                    )
                    .await?;
            }
        }

        Ok(())
    }
}
