use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bytes::Bytes;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Offer {
    pub ID: Uuid,
    #[serde(with = "base64_standard")]
    pub data: Vec<u8>,
    pub mostSpecificRegionID: i32,
    pub startDate: i64,
    pub endDate: i64,
    pub numberSeats: u8,
    pub price: u16,
    pub carType: String,
    pub hasVollkasko: bool,
    pub freeKilometers: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResultOffer {
    pub ID: Uuid,
    #[serde(with = "base64_standard")]
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PriceRange {
    pub start: u16,
    pub end: u16,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CarTypeCount {
    pub small: u32,
    pub sports: u32,
    pub luxury: u32,
    pub family: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VollkaskoCount {
    pub trueCount: u32,
    pub falseCount: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SeatsCount {
    pub numberSeats: u8,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FreeKilometerRange {
    pub start: u16,
    pub end: u16,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub offers: Vec<SearchResultOffer>,
    pub priceRanges: Vec<PriceRange>,
    pub carTypeCounts: CarTypeCount,
    pub seatsCount: Vec<SeatsCount>,
    pub freeKilometerRange: Vec<FreeKilometerRange>,
    pub vollkaskoCount: VollkaskoCount,
}

mod base64_standard {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = base64::encode(bytes);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        base64::decode(&s).map_err(serde::de::Error::custom)
    }
}