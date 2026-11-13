pub mod auth;
pub mod categoria;
pub mod favorito;
pub mod juego;
pub mod nivel_juego;
pub mod palabra;
pub mod pregunta_juego;
pub mod usuario;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Patrón `%texto%` para `LIKE` (las búsquedas de Java eran "contiene").
pub fn like(texto: &str) -> String {
    format!("%{texto}%")
}

/// `yyyy-MM-dd HH:mm:ss`, el formato de fecha que guardaba el backend Java.
pub fn now_datetime() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 200 con el JSON, o 204 si no hay nada (JAX-RS devolvía 204 para `null`).
pub fn json_or_no_content<T: Serialize>(v: Option<T>) -> Response {
    match v {
        Some(v) => Json(v).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}
