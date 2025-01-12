use crate::models::user::AccountType;
use crate::schema::private;
use diesel::AsChangeset;

#[derive(Debug, AsChangeset)]
#[diesel(table_name = private::users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub role: Option<AccountType>,
}
