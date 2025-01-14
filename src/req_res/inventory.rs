use crate::req_res::{AppError, ClientErrorMessages, DataValidationError};
use crate::schema::private;
use diesel::{AsChangeset, Insertable};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct NewProductReq {
    pub title: String,
    pub description: String,
    pub stock: i32,
    pub cost: i32,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = private::products)]
pub struct NewProduct {
    pub title: String,
    pub image_path: String,
    pub description: String,
    pub stock: i32,
    pub cost: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductReq {
    pub title: Option<String>,
    pub description: Option<String>,
    pub stock: Option<i32>,
    pub cost: Option<i32>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = private::products)]
pub struct UpdateProduct {
    pub title: Option<String>,
    pub description: Option<String>,
    pub stock: Option<i32>,
    pub cost: Option<i32>,
}

impl TryInto<UpdateProduct> for UpdateProductReq {
    type Error = AppError;

    fn try_into(self) -> Result<UpdateProduct, Self::Error> {
        let mut errors = vec![];

        if let Some(stock) = self.stock {
            if stock < 0 {
                errors.push("Stock cannot be negative".to_string());
            }
        }

        if let Some(cost) = self.cost {
            if cost < 0 {
                errors.push("Cost cannot be negative".to_string());
            }
        }

        if errors.is_empty() {
            Ok(UpdateProduct {
                title: self.title,
                description: self.description,
                stock: self.stock,
                cost: self.cost,
            })
        } else {
            Err(AppError::bad_request::<ClientErrorMessages>(
                DataValidationError { errors }.into(),
            ))
        }
    }
}

impl TryInto<NewProduct> for NewProductReq {
    type Error = AppError;

    fn try_into(self) -> Result<NewProduct, Self::Error> {
        let mut errors = vec![];

        if self.stock < 0 {
            errors.push("Stock cannot be negative".to_string());
        }
        if self.cost < 0 {
            errors.push("Cost cannot be negative".to_string());
        }

        if errors.is_empty() {
            Ok(NewProduct {
                title: self.title,
                image_path: "".to_string(),
                description: self.description,
                stock: self.stock,
                cost: self.cost,
            })
        } else {
            Err(AppError::bad_request::<ClientErrorMessages>(
                DataValidationError { errors }.into(),
            ))
        }
    }
}
