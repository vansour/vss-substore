use axum::{
    extract::State,
    response::{IntoResponse, Json},
    Json as ResponseJson,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use tower_sessions::Session;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use std::sync::Arc;

use crate::error::AppError;
use crate::error::AppResult;
use crate::state::AppState;
use crate::utils::is_valid_username;

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdateAccountPayload {
    pub new_username: String,
    pub new_password: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    session: Session,
    Json(payload): Json<LoginPayload>,
) -> AppResult<impl IntoResponse> {
    let row = sqlx::query("SELECT password_hash FROM admins WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await?;

    if let Some(r) = row {
        let hash_str: String = r.get(0);
        let parsed_hash = PasswordHash::new(&hash_str).map_err(|e| {
            tracing::error!("Invalid password hash stored in DB: {}", e);
            AppError::InternalError("Auth error".into())
        })?;

        if Argon2::default()
            .verify_password(payload.password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            session.insert("user_id", payload.username.clone()).await.map_err(|e| {
                tracing::error!("Failed to attach identity: {}", e);
                AppError::InternalError("Login session error".into())
            })?;
            return Ok(ResponseJson(json!({ "message": "Logged in" })));
        }
    }

    Err(AppError::Unauthorized)
}

pub async fn logout(session: Session) -> impl IntoResponse {
    let _ = session.flush().await;
    ResponseJson("Logged out")
}

pub async fn get_me(session: Session) -> AppResult<impl IntoResponse> {
    let username: Option<String> = session.get("user_id").await?;
    match username {
        Some(u) => Ok(ResponseJson(json!({ "username": u }))),
        None => Err(AppError::Unauthorized),
    }
}

pub async fn update_account(
    State(state): State<Arc<AppState>>,
    session: Session,
    Json(payload): Json<UpdateAccountPayload>,
) -> AppResult<impl IntoResponse> {
    let current_user: Option<String> = session.get("user_id").await?;
    let current_user = current_user.ok_or(AppError::Unauthorized)?;

    let new_username = payload.new_username.trim().to_string();
    let new_password = payload.new_password.trim();

    if !is_valid_username(&new_username) {
        return Err(AppError::BadRequest("Invalid username format".into()));
    }
    if new_password.is_empty() {
        return Err(AppError::BadRequest("Password cannot be empty".into()));
    }

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalError(format!("Hash error: {}", e)))?
        .to_string();

    sqlx::query("UPDATE admins SET username = $1, password_hash = $2 WHERE username = $3")
        .bind(&new_username)
        .bind(&password_hash)
        .bind(&current_user)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Update account error: {}", e);
            AppError::InternalError("Failed to update account (username might exist)".into())
        })?;

    let _ = session.flush().await;
    Ok(ResponseJson("Account updated, please login again"))
}
