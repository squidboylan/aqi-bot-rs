use serde::{Deserialize, Deserializer};

use std::fmt;

use serde::de::{self, Visitor};

const CONVERSION: [(f64, f64); 8] = [
    (0.0, 0.0),
    (12.1, 51.0),
    (35.5, 101.0),
    (55.5, 151.0),
    (150.5, 201.0),
    (250.5, 301.0),
    (350.5, 401.0),
    (500.5, 500.0),
];

fn create_url(id: u64) -> reqwest::Url {
    let url = format!("https://www.purpleair.com/json?show={}", id,);
    reqwest::Url::parse(&url).unwrap()
}

/// Convert a raw pm2.5 ug/m^3 value to AQI
pub fn raw_to_aqi(raw: f64) -> f64 {
    if raw >= 500.5 {
        return 500.0;
    }
    let mut low = 0;
    let mut high = 1;
    while CONVERSION[high].0 < raw {
        low += 1;
        high += 1;
    }

    ((CONVERSION[high].1 - CONVERSION[low].1) / (CONVERSION[high].0 - CONVERSION[low].0))
        * (raw - CONVERSION[low].0)
        + CONVERSION[low].1
}

pub async fn get_sensor_data(id: u64) -> reqwest::Result<Response> {
    let url = create_url(id);
    let res = reqwest::get(url).await?.json::<Response>().await;
    return res;
}

#[derive(Deserialize)]
pub struct Response {
    pub results: Vec<Sensor>,
}

#[derive(Deserialize)]
pub struct Sensor {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(rename = "Label")]
    pub label: String,
    #[serde(rename = "Stats")]
    pub stats: Stats,
}

pub struct Stats {
    pub v: f64,
    pub v1: f64,
    pub v2: f64,
    pub v3: f64,
    pub v4: f64,
    pub v5: f64,
    pub v6: f64,
}

#[derive(Deserialize)]
struct StatsTmp {
    v: f64,
    v1: f64,
    v2: f64,
    v3: f64,
    v4: f64,
    v5: f64,
    v6: f64,
}

struct StatsVisitor;

impl<'de> Visitor<'de> for StatsVisitor {
    type Value = Stats;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A string of stats data")
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let stats_tmp = serde_json::from_str::<StatsTmp>(value).unwrap();
        Ok(Stats {
            v: stats_tmp.v,
            v1: stats_tmp.v1,
            v2: stats_tmp.v2,
            v3: stats_tmp.v3,
            v4: stats_tmp.v4,
            v5: stats_tmp.v5,
            v6: stats_tmp.v6,
        })
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let stats_tmp = serde_json::from_str::<StatsTmp>(value).unwrap();
        Ok(Stats {
            v: stats_tmp.v,
            v1: stats_tmp.v1,
            v2: stats_tmp.v2,
            v3: stats_tmp.v3,
            v4: stats_tmp.v4,
            v5: stats_tmp.v5,
            v6: stats_tmp.v6,
        })
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let stats_tmp = serde_json::from_str::<StatsTmp>(&value).unwrap();
        Ok(Stats {
            v: stats_tmp.v,
            v1: stats_tmp.v1,
            v2: stats_tmp.v2,
            v3: stats_tmp.v3,
            v4: stats_tmp.v4,
            v5: stats_tmp.v5,
            v6: stats_tmp.v6,
        })
    }
}

impl<'de> Deserialize<'de> for Stats {
    fn deserialize<D>(deserializer: D) -> Result<Stats, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(StatsVisitor)
    }
}
