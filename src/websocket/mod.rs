use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WSAuthToken {
    pub token: String,
}
