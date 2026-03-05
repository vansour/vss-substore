use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub client: reqwest::Client,
}
