use crate::helper::save_product_image;
use crate::models::products::Product;
use crate::req_res::inventory::{NewProduct, NewProductReq, UpdateProduct, UpdateProductReq};
use crate::req_res::AppError;
use crate::schema::private;
use crate::AppState;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{patch, post};
use axum::{Json, Router};
use bytes::Bytes;
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel_async::{RunQueryDsl};
use std::sync::Arc;
use diesel::{QueryDsl};
use uuid::Uuid;

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/inventory/",
        Router::new()
            .route("/", post(create_product))
            .route("/{uid}", patch(update_product).delete(delete_product))
            .route("/{uid}/image", patch(update_product_image)),
    )
}

async fn create_product(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;

    let mut product_data: Option<NewProductReq> = None;
    let mut image_data: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(AppError::from)? {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "product" => {
                let data = field.text().await.map_err(AppError::from)?;
                product_data =
                    Some(serde_json::from_str(&data).map_err(|e| AppError::bad_request(None))?);
            }
            "image" => {
                image_data = Some(field.bytes().await.map_err(AppError::from)?);
            }
            _ => continue,
        }
    }

    let payload = product_data.ok_or_else(|| AppError::bad_request(None))?;
    let mut req: NewProduct = payload.try_into()?;

    let image_data = image_data.ok_or_else(|| AppError::bad_request(None))?;
    let img_path = save_product_image(image_data.iter().as_slice(), &req.title).await?;
    req.image_path = format!("uploads/products/{}", img_path);

    let new_product = diesel::insert_into(private::products::table)
        .values(&req)
        .returning((
            private::products::uuid,
            private::products::title,
            private::products::image_path,
            private::products::description,
            private::products::stock,
            private::products::cost,
        ))
        .get_result::<(Uuid, String, String, String, i32, i32)>(&mut con)
        .await
        .map_err(AppError::from)?;

    let new_product = Product {
        uuid: new_product.0,
        title: new_product.1,
        image_path: new_product.2,
        description: new_product.3,
        stock: new_product.4,
        cost: new_product.5,
    };

    Ok((StatusCode::CREATED, Json(new_product)))
}

async fn update_product(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    Json(update_req): Json<UpdateProductReq>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;

    let update_product: UpdateProduct = update_req.try_into()?;

    let updated_product = diesel::update(private::products::table)
        .filter(private::products::uuid.eq(uid))
        .set(&update_product)
        .returning((
            private::products::uuid,
            private::products::title,
            private::products::image_path,
            private::products::description,
            private::products::stock,
            private::products::cost,
        ))
        .get_result::<(Uuid, String, String, String, i32, i32)>(&mut con)
        .await
        .optional()
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found())?;

    let product = Product {
        uuid: updated_product.0,
        title: updated_product.1,
        image_path: updated_product.2,
        description: updated_product.3,
        stock: updated_product.4,
        cost: updated_product.5,
    };

    Ok((StatusCode::OK, Json(product)))
}

async fn update_product_image(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;

    let product = private::products::table
        .filter(private::products::uuid.eq(uid))
        .select(private::products::title)
        .first::<String>(&mut con)
        .await
        .optional()?
        .ok_or_else(|| AppError::not_found())?;

    let mut image_data: Option<Bytes> = None;
    while let Some(field) = multipart.next_field().await.map_err(AppError::from)? {
        if field.name().unwrap_or_default() == "image" {
            image_data = Some(field.bytes().await.map_err(AppError::from)?);
            break;
        }
    }

    let image_data = image_data.ok_or_else(|| AppError::bad_request(None))?;

    let img_path = save_product_image(&*image_data, &product).await?;
    let img_path = format!("uploads/products/{}", img_path);

    diesel::update(private::products::table)
        .filter(private::products::uuid.eq(uid))
        .set(private::products::image_path.eq(&img_path))
        .execute(&mut con)
        .await
        .map_err(AppError::from)?;

    Ok((StatusCode::OK, ()))
}

async fn delete_product(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;

    let deleted_count = diesel::delete(private::products::table)
        .filter(private::products::uuid.eq(uid))
        .execute(&mut con)
        .await
        .map_err(AppError::from)?;

    if deleted_count == 0 {
        return Err(AppError::not_found());
    }

    Ok(StatusCode::NO_CONTENT)
}
