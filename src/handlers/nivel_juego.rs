//! `/api/niveles-juego` (NivelJuegoResource.java).

use crate::{
    error::{parse_json, ApiResult, AppError},
    handlers::json_or_no_content,
    models::{
        collect,
        nivel_juego::{NivelIn, NivelJuego},
    },
    AppState,
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
    Json,
};

/// Solo niveles activos, ordenados por `orden`. (`'activo'` coincide con
/// `'Activo'` porque la colación por defecto de MySQL/MariaDB no distingue mayúsculas.)
pub async fn listar(State(st): State<AppState>) -> ApiResult<Json<Vec<NivelJuego>>> {
    let sql = format!("{} WHERE n.estado = 'activo' ORDER BY n.orden", NivelJuego::select());
    let rows = sqlx::query(&sql).fetch_all(&st.pool).await?;
    Ok(Json(collect(&rows, |r| NivelJuego::from_row(r, ""))?))
}

/// Si no existe responde 204 sin cuerpo (en Java el método devolvía `null`).
pub async fn obtener(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<Response> {
    Ok(json_or_no_content(NivelJuego::find(&st.pool, id).await?))
}

pub async fn crear(State(st): State<AppState>, body: Bytes) -> ApiResult<Json<NivelJuego>> {
    let dto: NivelIn = parse_json(&body)?;
    let res = sqlx::query(
        "INSERT INTO niveles_juego (nombre, descripcion, orden, estado) VALUES (?, ?, ?, ?)",
    )
    .bind(dto.nombre)
    .bind(dto.descripcion)
    .bind(dto.orden.unwrap_or(0))
    .bind(dto.estado.unwrap_or_else(|| "Activo".to_string()))
    .execute(&st.pool)
    .await?;
    let creado = NivelJuego::find(&st.pool, res.last_insert_id() as i32)
        .await?
        .ok_or_else(|| AppError::Internal("No se pudo leer el nivel creado".to_string()))?;
    Ok(Json(creado))
}

/// Reemplazo completo (era un `merge` en Java): lo que no venga toma el valor por defecto.
pub async fn actualizar(
    State(st): State<AppState>,
    Path(id): Path<i32>,
    body: Bytes,
) -> ApiResult<Json<NivelJuego>> {
    let dto: NivelIn = parse_json(&body)?;
    NivelJuego::find(&st.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Nivel no encontrado".to_string()))?;
    sqlx::query(
        "UPDATE niveles_juego SET nombre = ?, descripcion = ?, orden = ?, estado = ? WHERE id_nivel = ?",
    )
    .bind(dto.nombre)
    .bind(dto.descripcion)
    .bind(dto.orden.unwrap_or(0))
    .bind(dto.estado.unwrap_or_else(|| "Activo".to_string()))
    .bind(id)
    .execute(&st.pool)
    .await?;
    let nuevo = NivelJuego::find(&st.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Nivel no encontrado".to_string()))?;
    Ok(Json(nuevo))
}

/// 204 sin cuerpo, exista o no (igual que el `void` de Java).
pub async fn eliminar(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<StatusCode> {
    sqlx::query("DELETE FROM niveles_juego WHERE id_nivel = ?")
        .bind(id)
        .execute(&st.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
