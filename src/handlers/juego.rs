//! `/api/juegos` (JuegoResource.java + JuegoService.java).

use crate::{
    error::{parse_json, ApiResult, AppError},
    handlers::json_or_no_content,
    models::{
        collect,
        juego::{Juego, JuegoIn},
    },
    AppState,
};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

#[derive(Deserialize)]
pub struct LimiteQuery {
    pub limite: Option<i64>,
}

#[derive(Deserialize)]
pub struct RankingQuery {
    #[serde(rename = "idPartida")]
    pub id_partida: Option<String>,
    pub limite: Option<i64>,
}

fn no_encontrado() -> AppError {
    AppError::NotFound("Juego no encontrado".to_string())
}

pub async fn guardar(State(st): State<AppState>, body: Bytes) -> ApiResult<Json<Juego>> {
    let dto: JuegoIn = parse_json(&body)?;
    let res = sqlx::query(
        "INSERT INTO juego (puntaje, fecha_juego, preguntas_correctas, total_preguntas, \
         idPartida, id_usuario, id_nivel) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(dto.puntaje.unwrap_or(0))
    .bind(dto.fecha_juego)
    .bind(dto.preguntas_correctas.unwrap_or(0))
    .bind(dto.total_preguntas.unwrap_or(0))
    .bind(dto.id_partida)
    .bind(dto.usuario.as_ref().and_then(|u| u.id()))
    .bind(dto.nivel.as_ref().and_then(|n| n.id()))
    .execute(&st.pool)
    .await?;
    let creado = Juego::find(&st.pool, res.last_insert_id() as i32)
        .await?
        .ok_or_else(|| AppError::Internal("No se pudo leer la partida creada".to_string()))?;
    Ok(Json(creado))
}

/// Reemplazo completo (era un `merge` en Java).
pub async fn actualizar(
    State(st): State<AppState>,
    Path(id): Path<i32>,
    body: Bytes,
) -> ApiResult<Json<Juego>> {
    let dto: JuegoIn = parse_json(&body)?;
    Juego::find(&st.pool, id).await?.ok_or_else(no_encontrado)?;
    sqlx::query(
        "UPDATE juego SET puntaje = ?, fecha_juego = ?, preguntas_correctas = ?, \
         total_preguntas = ?, idPartida = ?, id_usuario = ?, id_nivel = ? WHERE id_juego = ?",
    )
    .bind(dto.puntaje.unwrap_or(0))
    .bind(dto.fecha_juego)
    .bind(dto.preguntas_correctas.unwrap_or(0))
    .bind(dto.total_preguntas.unwrap_or(0))
    .bind(dto.id_partida)
    .bind(dto.usuario.as_ref().and_then(|u| u.id()))
    .bind(dto.nivel.as_ref().and_then(|n| n.id()))
    .bind(id)
    .execute(&st.pool)
    .await?;
    let nuevo = Juego::find(&st.pool, id).await?.ok_or_else(no_encontrado)?;
    Ok(Json(nuevo))
}

pub async fn historial(
    State(st): State<AppState>,
    Path(id_usuario): Path<i32>,
) -> ApiResult<Json<Vec<Juego>>> {
    let sql = format!(
        "{} WHERE j.id_usuario = ? ORDER BY j.fecha_juego DESC",
        Juego::select()
    );
    let rows = sqlx::query(&sql).bind(id_usuario).fetch_all(&st.pool).await?;
    Ok(Json(collect(&rows, Juego::from_row)?))
}

pub async fn mejores(
    State(st): State<AppState>,
    Path(id_nivel): Path<i32>,
    Query(q): Query<LimiteQuery>,
) -> ApiResult<Json<Vec<Juego>>> {
    let sql = format!(
        "{} WHERE j.id_nivel = ? ORDER BY j.puntaje DESC, j.fecha_juego DESC LIMIT ?",
        Juego::select()
    );
    let rows = sqlx::query(&sql)
        .bind(id_nivel)
        .bind(q.limite.unwrap_or(10).max(0))
        .fetch_all(&st.pool)
        .await?;
    Ok(Json(collect(&rows, Juego::from_row)?))
}

/// 204 sin cuerpo si el usuario nunca jugó ese nivel (en Java devolvía `null`).
pub async fn mejor(
    State(st): State<AppState>,
    Path((id_usuario, id_nivel)): Path<(i32, i32)>,
) -> ApiResult<Response> {
    let sql = format!(
        "{} WHERE j.id_usuario = ? AND j.id_nivel = ? ORDER BY j.puntaje DESC LIMIT 1",
        Juego::select()
    );
    let row = sqlx::query(&sql)
        .bind(id_usuario)
        .bind(id_nivel)
        .fetch_optional(&st.pool)
        .await?;
    let juego = match row {
        Some(r) => Juego::from_row(&r)?,
        None => None,
    };
    Ok(json_or_no_content(juego))
}

/// Devuelve un número JSON (puntaje acumulado; 0 si no hay partidas).
pub async fn total(
    State(st): State<AppState>,
    Path(id_usuario): Path<i32>,
) -> ApiResult<Json<i64>> {
    let total: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(puntaje), 0) AS SIGNED) FROM juego WHERE id_usuario = ?",
    )
    .bind(id_usuario)
    .fetch_one(&st.pool)
    .await?;
    Ok(Json(total))
}

/// Igual que Java (`List<Object[]>`): arreglos posicionales, no objetos.
/// - por partida: `[idUsuario, nombre, totalPuntaje]`
/// - global:      `[idUsuario, nombre, totalPuntaje, idPartida]`
pub async fn ranking(
    State(st): State<AppState>,
    Query(q): Query<RankingQuery>,
) -> ApiResult<Json<Vec<Value>>> {
    if let Some(partida) = q.id_partida.filter(|p| !p.is_empty()) {
        let rows = sqlx::query(
            "SELECT u.id_usuario AS id, u.nombre AS nombre, \
             CAST(SUM(j.puntaje) AS SIGNED) AS total \
             FROM juego j JOIN usuarios u ON u.id_usuario = j.id_usuario \
             WHERE j.idPartida = ? GROUP BY u.id_usuario, u.nombre ORDER BY total DESC",
        )
        .bind(partida)
        .fetch_all(&st.pool)
        .await?;
        let out = rows
            .iter()
            .map(|r| {
                Ok(json!([
                    r.try_get::<i32, _>("id")?,
                    r.try_get::<String, _>("nombre")?,
                    r.try_get::<i64, _>("total")?
                ]))
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;
        return Ok(Json(out));
    }

    let rows = sqlx::query(
        "SELECT u.id_usuario AS id, u.nombre AS nombre, \
         CAST(SUM(j.puntaje) AS SIGNED) AS total, j.idPartida AS partida \
         FROM juego j JOIN usuarios u ON u.id_usuario = j.id_usuario \
         WHERE j.idPartida IS NOT NULL \
         GROUP BY u.id_usuario, u.nombre, j.idPartida ORDER BY total DESC LIMIT ?",
    )
    .bind(q.limite.unwrap_or(10).max(0))
    .fetch_all(&st.pool)
    .await?;
    let out = rows
        .iter()
        .map(|r| {
            Ok(json!([
                r.try_get::<i32, _>("id")?,
                r.try_get::<String, _>("nombre")?,
                r.try_get::<i64, _>("total")?,
                r.try_get::<String, _>("partida")?
            ]))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    Ok(Json(out))
}
