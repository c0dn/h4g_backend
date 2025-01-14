use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}
