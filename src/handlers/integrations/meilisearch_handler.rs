use actix_web::{get, web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::handlers::ApiResponse;
use crate::services::integrations::meilisearch_service::{MeilisearchService, SearchableListing, SearchableProduct};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub game: Option<String>,
    pub set: Option<String>,
    pub condition: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse<T> {
    pub hits: Vec<T>,
    pub query: String,
    pub processing_time_ms: u64,
    pub hits_count: usize,
    pub offset: usize,
    pub limit: usize,
    pub estimated_total_hits: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct QuickSearchResponse {
    pub products: Vec<SearchableProduct>,
    pub listings: Vec<SearchableListing>,
    pub total_hits: usize,
    pub processing_time_ms: u64,
}

#[get("/quick")]
pub async fn quick_search(
    state: web::Data<AppState>,
    query: web::Query<SearchQuery>,
) -> Result<impl Responder> {
    let search_service = MeilisearchService::new(state.as_ref().clone());
    let query_params = query.into_inner();

    if query_params.q.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            message: "Search query cannot be empty".to_string(),
            data: None,
        }));
    }

    if query_params.q.len() < 2 {
        return Ok(HttpResponse::Ok().json(QuickSearchResponse {
            products: vec![],
            listings: vec![],
            total_hits: 0,
            processing_time_ms: 0,
        }));
    }

    let limit = query_params.limit.unwrap_or(5).min(10); // Limit quick search results
    let filters = build_filters(&query_params);

    let sort_params = parse_sort_param(&query_params.sort);
    let start_time = std::time::Instant::now();

    let (products_result, listings_result) = tokio::join!(

        search_service.search_products_paginated(
            &query_params.q,
            filters.as_deref(),
            0,
            limit,
            None
        ),
        search_service.search_listings_paginated(
            &query_params.q,
            filters.as_deref(),
            0,
            limit,
            sort_params.as_deref()
        )
    );

    let processing_time = start_time.elapsed().as_millis() as u64;

    match (products_result, listings_result) {
        (Ok(products), Ok(listings)) => {
            let product_hits: Vec<_> = products.hits.into_iter().map(|hit| hit.result).collect();
            let listing_hits: Vec<_> = listings.hits.into_iter().map(|hit| hit.result).collect();
            let total_hits = product_hits.len() + listing_hits.len();

            Ok(HttpResponse::Ok().json(QuickSearchResponse {
                products: product_hits,
                listings: listing_hits,
                total_hits,
                processing_time_ms: processing_time,
            }))
        }
        (Err(e), _) | (_, Err(e)) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: format!("Search failed: {}", e),
                data: None,
            }))
        }
    }
}


#[get("/products")]
pub async fn search_products(
    state: web::Data<AppState>,
    query: web::Query<SearchQuery>,
) -> Result<impl Responder> {
    let search_service = MeilisearchService::new(state.as_ref().clone());
    let query_params = query.into_inner();

    /*
    if query_params.q.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            message: "Search query cannot be empty".to_string(),
            data: None,
        }));
    }

    if query_params.q.len() < 2 {
        return Ok(HttpResponse::Ok().json(QuickSearchResponse {
            products: vec![],
            listings: vec![],
            total_hits: 0,
            processing_time_ms: 0,
        }));
    }
     */

    let limit = query_params.limit.unwrap_or(20).min(100);
    let offset = query_params.offset.unwrap_or(0);
    let filters = build_filters(&query_params);

    let start_time = std::time::Instant::now();

    match search_service.search_products_paginated(
        &query_params.q,
        filters.as_deref(),
        offset,
        limit,
        None
    ).await {
        Ok(results) => {
            let processing_time = start_time.elapsed().as_millis() as u64;

            let hits_count = results.hits.len();
            let estimated_total_hits = results.estimated_total_hits;

            let hits: Vec<_> = results.hits.into_iter().map(|hit| hit.result).collect();

            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "Products found".to_string(),
                data: Some(SearchResponse {
                    hits,
                    query: query_params.q,
                    processing_time_ms: processing_time,
                    hits_count,
                    offset,
                    limit,
                    estimated_total_hits,
                }),
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: format!("Product search failed: {}", e),
                data: None,
            }))
        }
    }
}

#[get("/products/trending")]
pub async fn get_trending_products(
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let search_service = MeilisearchService::new(state.as_ref().clone());

    match search_service.search_products_trending(
        0,
        10
    ).await {
        Ok(products) => {
            let hits: Vec<_> = products.hits.into_iter().map(|hit| hit.result).collect();
            let hits_count = hits.len();
            let estimated_total_hits = products.estimated_total_hits.unwrap_or(hits.len());

            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "Trending products retrieved successfully".to_string(),
                data: Some(SearchResponse {
                    hits,
                    query: "trending".to_string(),
                    processing_time_ms: 0,
                    hits_count,
                    offset: 0,
                    limit: 10,
                    estimated_total_hits: Some(estimated_total_hits),
                }),
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: format!("Failed to retrieve trending products: {}", e),
                data: None,
            }))
        }
    }
}

fn build_filters(query: &SearchQuery) -> Option<String> {
    let mut filters = Vec::new();

    if let Some(game) = &query.game {
        filters.push(format!("game = \"{}\"", game));
    }

    if let Some(set) = &query.set {
        filters.push(format!("set = \"{}\"", set));
    }

    if filters.is_empty() {
        None
    } else {
        Some(filters.join(" AND "))
    }
}

fn build_listing_filters(query: &SearchQuery) -> Option<String> {
    let mut filters = Vec::new();

    if let Some(game) = &query.game {
        filters.push(format!("game = \"{}\"", game));
    }

    if let Some(set) = &query.set {
        filters.push(format!("set = \"{}\"", set));
    }

    if let Some(condition) = &query.condition {
        filters.push(format!("condition = \"{}\"", condition));
    }

    if let Some(min_price) = query.min_price {
        filters.push(format!("price >= {}", min_price));
    }

    if let Some(max_price) = query.max_price {
        filters.push(format!("price <= {}", max_price));
    }

    if filters.is_empty() {
        None
    } else {
        Some(filters.join(" AND "))
    }
}

fn parse_sort_param(sort: &Option<String>) -> Option<&[&str]> {
    sort.as_ref().and_then(|s| {
        match s.as_str() {
            "price_asc" => Some(&["price:asc"][..]),
            "price_desc" => Some(&["price:desc"][..]),
            "name_asc" => Some(&["product_name:asc"][..]),
            "name_desc" => Some(&["product_name:desc"][..]),
            _ => None
        }
    })
}