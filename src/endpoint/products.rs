use crate::models::products::Product;
use crate::req_res::products::SearchParams;
use crate::req_res::AppError;
use crate::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_full_text_search::{websearch_to_tsquery, TsVectorExtensions};
use std::sync::Arc;
use uuid::Uuid as UuidType;

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new().nest("/products/", Router::new().route("/", get(get_products)))
}

async fn get_products(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    use crate::schema::private::products::dsl::*;
    let product_vec = products
        .select((uuid, title, description, stock, cost))
        .filter(search_vector.matches(websearch_to_tsquery(&params.q)))
        .get_results::<(UuidType, String, String, i32, i32)>(&mut con)
        .await?
        .into_iter()
        .map(|(uid, p_title, p_desc, p_stock, p_cost)| Product {
            uuid: uid,
            title: p_title,
            description: p_desc,
            stock: p_stock,
            cost: p_cost,
        })
        .collect::<Vec<Product>>();

    Ok((StatusCode::OK, Json(product_vec)))
}
