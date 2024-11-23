use axum::{
    extract::{Query, Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::collections::HashMap;
use crate::models::*;
use crate::db::Database;
use uuid::Uuid;
use rocksdb::{DB, WriteBatch};

pub async fn get_offers(
    State(db): State<Database>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Extract and parse query parameters
    let region_id: i32 = match params.get("regionID") {
        Some(val) => val.parse().unwrap_or_default(),
        None => return (StatusCode::BAD_REQUEST, "Missing regionID").into_response(),
    };
    let time_range_start: i64 = match params.get("timeRangeStart") {
        Some(val) => val.parse().unwrap_or_default(),
        None => return (StatusCode::BAD_REQUEST, "Missing timeRangeStart").into_response(),
    };
    let time_range_end: i64 = match params.get("timeRangeEnd") {
        Some(val) => val.parse().unwrap_or_default(),
        None => return (StatusCode::BAD_REQUEST, "Missing timeRangeEnd").into_response(),
    };
    let number_days: i32 = match params.get("numberDays") {
        Some(val) => val.parse().unwrap_or_default(),
        None => return (StatusCode::BAD_REQUEST, "Missing numberDays").into_response(),
    };
    let sort_order = match params.get("sortOrder") {
        Some(val) => val.as_str(),
        None => return (StatusCode::BAD_REQUEST, "Missing sortOrder").into_response(),
    };
    let page: u32 = match params.get("page") {
        Some(val) => val.parse().unwrap_or_default(),
        None => return (StatusCode::BAD_REQUEST, "Missing page").into_response(),
    };
    let page_size: u32 = match params.get("pageSize") {
        Some(val) => val.parse().unwrap_or_default(),
        None => return (StatusCode::BAD_REQUEST, "Missing pageSize").into_response(),
    };
    let price_range_width: u32 = match params.get("priceRangeWidth") {
        Some(val) => val.parse().unwrap_or_default(),
        None => return (StatusCode::BAD_REQUEST, "Missing priceRangeWidth").into_response(),
    };
    let min_free_kilometer_width: u32 = match params.get("minFreeKilometerWidth") {
        Some(val) => val.parse().unwrap_or_default(),
        None => return (StatusCode::BAD_REQUEST, "Missing minFreeKilometerWidth").into_response(),
    };

    // Optional parameters
    let min_number_seats: Option<u8> = params.get("minNumberSeats").and_then(|v| v.parse().ok());
    let min_price: Option<u16> = params.get("minPrice").and_then(|v| v.parse().ok());
    let max_price: Option<u16> = params.get("maxPrice").and_then(|v| v.parse().ok());
    let car_type: Option<String> = params.get("carType").cloned();
    let only_vollkasko: Option<bool> = params.get("onlyVollkasko").and_then(|v| v.parse().ok());
    let min_free_kilometer: Option<u16> = params.get("minFreeKilometer").and_then(|v| v.parse().ok());

    // Build and execute the query
    let offers = query_offers(
        &db,
        region_id,
        time_range_start,
        time_range_end,
        min_number_seats,
        min_price,
        max_price,
        car_type,
        only_vollkasko,
        min_free_kilometer,
        sort_order,
    );

    // Paginate results
    let start_index = (page - 1) * page_size;
    let end_index = start_index + page_size;
    let paginated_offers = offers.get(start_index as usize..end_index as usize)
        .unwrap_or(&[])
        .to_vec();

    // Perform aggregations
    let price_ranges = compute_price_ranges(&offers, price_range_width);
    let car_type_counts = compute_car_type_counts(&offers);
    let seats_count = compute_seats_count(&offers);
    let free_kilometer_range = compute_free_kilometer_ranges(&offers, min_free_kilometer_width);
    let vollkasko_count = compute_vollkasko_count(&offers);

    let result = SearchResult {
        offers: paginated_offers,
        priceRanges: price_ranges,
        carTypeCounts: car_type_counts,
        seatsCount: seats_count,
        freeKilometerRange: free_kilometer_range,
        vollkaskoCount: vollkasko_count,
    };

    Json(result).into_response()
}

pub async fn create_offers(
    State(db): State<Database>,
    Json(payload): Json<HashMap<String, Vec<Offer>>>,
) -> impl IntoResponse {
    let offers = match payload.get("offers") {
        Some(offers) if !offers.is_empty() => offers,
        _ => return (StatusCode::BAD_REQUEST, "Offers list is empty").into_response(),
    };

    // Batch insert offers
    for offer in offers {
        if let Err(e) = insert_offer(&db, offer.clone()) {
            eprintln!("Failed to insert offer {}: {}", offer.ID, e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to insert offers").into_response();
        }
    }

    (StatusCode::OK, "Offers were created").into_response()
}

// Helper functions for querying and aggregations
fn query_offers(
    db: &Database,
    region_id: i32,
    time_range_start: i64,
    time_range_end: i64,
    min_number_seats: Option<u8>,
    min_price: Option<u16>,
    max_price: Option<u16>,
    car_type: Option<String>,
    only_vollkasko: Option<bool>,
    min_free_kilometer: Option<u16>,
    sort_order: &str,
) -> Vec<SearchResultOffer> {
    // Implement the query logic here
    let mut offers = Vec::new();

    let iter = db.iterator(rocksdb::IteratorMode::Start);
    for item in iter {
        let (_, value) = item.unwrap();
        let offer: Offer = serde_json::from_slice(&value).unwrap();

        // Apply filters
        if offer.mostSpecificRegionID != region_id {
            continue;
        }
        if offer.startDate > time_range_end || offer.endDate < time_range_start {
            continue;
        }
        if let Some(min_seats) = min_number_seats {
            if offer.numberSeats < min_seats {
                continue;
            }
        }
        if let Some(min_p) = min_price {
            if offer.price < min_p {
                continue;
            }
        }
        if let Some(max_p) = max_price {
            if offer.price >= max_p {
                continue;
            }
        }
        if let Some(ref ct) = car_type {
            if &offer.carType != ct {
                continue;
            }
        }
        if let Some(only_vk) = only_vollkasko {
            if offer.hasVollkasko != only_vk {
                continue;
            }
        }
        if let Some(min_fk) = min_free_kilometer {
            if offer.freeKilometers < min_fk {
                continue;
            }
        }

        // Add to results
        offers.push(SearchResultOffer {
            ID: offer.ID,
            data: offer.data.clone(),
        });
    }


    // Sort offers
    offers.sort_by(|a, b| {
        match sort_order {
            "price-asc" => a.ID.cmp(&b.ID),
            "price-desc" => b.ID.cmp(&a.ID),
            _ => a.ID.cmp(&b.ID),
        }
    });

    offers
}

pub async fn cleanup_data(State(db): State<Database>) -> impl IntoResponse {
    // Get the default column family handle
    let cf_handle = match db.cf_handle("default") {
        Some(handle) => handle,
        None => {
            eprintln!("Failed to get default column family");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to clean up data").into_response();
        }
    };

    // Delete all keys in the default column family
    if let Err(e) = db.delete_range_cf(&cf_handle, &b""[..], &b"\xFF"[..]) {
        eprintln!("Failed to delete range: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to clean up data").into_response();
    }

    // Optionally, force compaction to reclaim disk space
    db.compact_range_cf(&cf_handle, None::<&[u8]>, None::<&[u8]>);

    (StatusCode::OK, "Data was cleaned up").into_response()
}

// TODO implement batch insert
fn insert_offers(db: &Database, offers: &[Offer]) -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = WriteBatch::default();
    for offer in offers {
        let key = offer.ID.as_bytes();
        let value = serde_json::to_vec(offer)?;
        batch.put(key, value);
    }
    db.write(batch)?;
    Ok(())
}

fn insert_offer(db: &Database, offer: Offer) -> Result<(), Box<dyn std::error::Error>> {
    let key = offer.ID.as_bytes();
    let value = serde_json::to_vec(&offer)?;
    db.put(key, value)?;
    Ok(())
}

// Placeholder aggregation functions
fn compute_price_ranges(offers: &Vec<SearchResultOffer>, width: u32) -> Vec<PriceRange> {
    // Implement aggregation logic
    Vec::new()
}

fn compute_car_type_counts(offers: &Vec<SearchResultOffer>) -> CarTypeCount {
    // Implement aggregation logic
    CarTypeCount {
        small: 0,
        sports: 0,
        luxury: 0,
        family: 0,
    }
}

fn compute_seats_count(offers: &Vec<SearchResultOffer>) -> Vec<SeatsCount> {
    // Implement aggregation logic
    Vec::new()
}

fn compute_free_kilometer_ranges(offers: &Vec<SearchResultOffer>, width: u32) -> Vec<FreeKilometerRange> {
    // Implement aggregation logic
    Vec::new()
}

fn compute_vollkasko_count(offers: &Vec<SearchResultOffer>) -> VollkaskoCount {
    // Implement aggregation logic
    VollkaskoCount {
        trueCount: 0,
        falseCount: 0,
    }
}