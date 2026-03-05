use axum::{
    extract::{Path, State},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_sessions::Session;
use std::sync::Arc;

use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use crate::utils::is_valid_username;

// 认证检查 - 从 session 获取用户
async fn require_auth(session: Session) -> Result<(), AppError> {
    let user_id: Option<String> = session.get("user_id").await?;
    user_id.ok_or(AppError::Unauthorized)?;
    Ok(())
}

#[derive(Deserialize)]
pub struct CreateUserPayload {
    pub username: String,
}

#[derive(Deserialize)]
pub struct LinksPayload {
    pub links: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct OrderPayload {
    pub order: Vec<String>,
}

pub async fn list_users(
    _session: Session,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<String>>> {
    require_auth(_session).await?;

    let rows = sqlx::query("SELECT username FROM users ORDER BY rank ASC")
        .fetch_all(&state.db)
        .await?;

    let list: Vec<String> = rows.iter().map(|r| r.get("username")).collect();
    Ok(Json(list))
}

pub async fn create_user(
    _session: Session,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserPayload>,
) -> AppResult<(StatusCode, Json<String>)> {
    require_auth(_session).await?;

    let username = payload.username.trim().to_string();

    if !is_valid_username(&username) {
        return Err(AppError::BadRequest("Invalid username".into()));
    }

    let max_rank_res = sqlx::query("SELECT MAX(rank) FROM users")
        .fetch_one(&state.db)
        .await;

    let next_rank: i64 = match max_rank_res {
        Ok(row) => row.try_get::<i64, _>(0).unwrap_or(0) + 1,
        Err(_) => 1,
    };

    let result = sqlx::query("INSERT INTO users (username, links, rank) VALUES ($1, '[]', $2)")
        .bind(&username)
        .bind(next_rank)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            tracing::info!(%username, "user created");
            Ok((StatusCode::CREATED, Json(username)))
        }
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            if msg.contains("duplicate key value") || msg.contains("unique constraint") {
                Err(AppError::BadRequest("user exists".into()))
            } else {
                Err(AppError::DbError(e))
            }
        }
    }
}

pub async fn delete_user(
    _session: Session,
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> AppResult<Json<&'static str>> {
    require_auth(_session).await?;

    let res = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(&state.db)
        .await?;

    if res.rows_affected() > 0 {
        tracing::info!(%username, "user deleted");
        Ok(Json("deleted"))
    } else {
        Err(AppError::NotFound("not found".into()))
    }
}

pub async fn get_links(
    _session: Session,
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    require_auth(_session).await?;

    let row = sqlx::query("SELECT links FROM users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&state.db)
        .await?;

    match row {
        Some(r) => {
            let links: serde_json::Value = r.get("links");
            Ok(Json(links))
        }
        None => Err(AppError::NotFound("user not found".into())),
    }
}

pub async fn set_links(
    _session: Session,
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
    Json(payload): Json<LinksPayload>,
) -> AppResult<Json<Vec<String>>> {
    require_auth(_session).await?;

    for link in &payload.links {
        if !link.starts_with("http://") && !link.starts_with("https://") {
            return Err(AppError::BadRequest(format!("Invalid URL: {}", link)));
        }
    }

    let links_value = serde_json::to_value(&payload.links).unwrap_or(serde_json::json!([]));

    let res = sqlx::query("UPDATE users SET links = $1 WHERE username = $2")
        .bind(&links_value)
        .bind(&username)
        .execute(&state.db)
        .await?;

    if res.rows_affected() > 0 {
        Ok(Json(payload.links))
    } else {
        Err(AppError::NotFound("user not found".into()))
    }
}

pub async fn set_user_order(
    _session: Session,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrderPayload>,
) -> AppResult<Json<Vec<String>>> {
    require_auth(_session).await?;

    let order = &payload.order;
    if order.is_empty() {
        return Err(AppError::BadRequest("order must not be empty".into()));
    }

    let mut tx = state.db.begin().await?;

    for (i, username) in order.iter().enumerate() {
        sqlx::query("UPDATE users SET rank = $1 WHERE username = $2")
            .bind(i as i64)
            .bind(username)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    tracing::info!("User order updated");
    Ok(Json(order.clone()))
}
