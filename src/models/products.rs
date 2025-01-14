use diesel::Queryable;
use diesel_full_text_search::TsVector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable)]
#[diesel(table_name = private::products)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductDB {
    pub uuid: Uuid,
    pub title: String,
    pub image_path: String,
    pub description: String,
    pub stock: i32,
    pub cost: i32,
    #[diesel(sql_type = TsVector)]
    pub search_vector: TsVector,
}

#[derive(Queryable, Serialize, Deserialize, Clone, Debug)]
pub struct Product {
    pub uuid: Uuid,
    pub title: String,
    pub image_path: String,
    pub description: String,
    pub stock: i32,
    pub cost: i32,
}
