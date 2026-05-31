use rustic_providers::economic::bea::model::BeaParamValue;


pub fn fips_to_census_geo(geo_fips: &str) -> String {
    if geo_fips == "00000" {
        "us:1".to_string()
    } else if geo_fips.ends_with("000") {
        // state — 06000 → state:06
        format!("state:{}", &geo_fips[..2])
    } else {
        // county — 06075 → county:075&in=state:06
        format!("county:{}&in=state:{}", &geo_fips[2..], &geo_fips[..2])
    }
}

pub fn geo_type(geo_fip: &BeaParamValue) -> &'static str {
    let key = geo_fip.key.as_str();
    let name = geo_fip.description.as_str();

    if key == "00000" {
        "national"
    } else if key >= "91000" && key <= "98000" {
        "region"
    } else if key.ends_with("000") {
        "state"
    } else if name.contains("Metropolitan") || name.contains("Nonmetropolitan") {
        "metro"
    } else if name.contains("Division") {
        "division"
    } else {
        "county"
    }
}

pub fn resolve_years(year: &str) -> Vec<String> {
    let current_year = 2026; // latest available BEA year
    
    match year {
        "LAST5" => (0..5)
            .map(|i| (current_year - i).to_string())
            .collect(),
        "LAST3" => (0..3)
            .map(|i| (current_year - i).to_string())
            .collect(),
        "LAST2" => (0..2)
            .map(|i| (current_year - i).to_string())
            .collect(),
        "LATEST" | "LAST1" => vec![current_year.to_string()],
        _ => year.split(',')
            .map(|y| y.trim().to_string())
            .collect(),
    }
}