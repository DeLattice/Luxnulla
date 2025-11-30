use serde::Deserialize;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: usize,
    pub limit: usize,
}
