//! `/api/preguntas-juego` (PreguntaJuegoResource.java).

use crate::{
    error::{parse_json, ApiResult, AppError},
    models::{
        collect,
        nivel_juego::NivelJuego,
        palabra::Palabra,
        pregunta_juego::{PreguntaIn, PreguntaJuego},
    },
    AppState,
};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NivelQuery {
    pub aleatorias: Option<String>,
    pub cantidad: Option<i64>,
}

fn no_encontrada() -> AppError {
    AppError::NotFound("Pregunta no encontrada".to_string())
}

pub async fn listar(State(st): State<AppState>) -> ApiResult<Json<Vec<PreguntaJuego>>> {
    let sql = format!("{} ORDER BY q.id_pregunta", PreguntaJuego::select());
    let rows = sqlx::query(&sql).fetch_all(&st.pool).await?;
    Ok(Json(collect(&rows, PreguntaJuego::from_row)?))
}

pub async fn obtener(
    State(st): State<AppState>,
    Path(id): Path<i32>,
) -> ApiResult<Json<PreguntaJuego>> {
    PreguntaJuego::find(&st.pool, id)
        .await?
        .map(Json)
        .ok_or_else(no_encontrada)
}

/// `?aleatorias=true&cantidad=10` devuelve preguntas al azar (por defecto 10).
pub async fn por_nivel(
    State(st): State<AppState>,
    Path(id_nivel): Path<i32>,
    Query(q): Query<NivelQuery>,
) -> ApiResult<Json<Vec<PreguntaJuego>>> {
    let aleatorias = q
        .aleatorias
        .map(|a| a.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let rows = if aleatorias {
        let sql = format!(
            "{} WHERE q.id_nivel = ? ORDER BY RAND() LIMIT ?",
            PreguntaJuego::select()
        );
        sqlx::query(&sql)
            .bind(id_nivel)
            .bind(q.cantidad.unwrap_or(10).max(0))
            .fetch_all(&st.pool)
            .await?
    } else {
        let sql = format!(
            "{} WHERE q.id_nivel = ? ORDER BY q.id_pregunta",
            PreguntaJuego::select()
        );
        sqlx::query(&sql).bind(id_nivel).fetch_all(&st.pool).await?
    };
    Ok(Json(collect(&rows, PreguntaJuego::from_row)?))
}

pub async fn crear(State(st): State<AppState>, body: Bytes) -> ApiResult<Json<PreguntaJuego>> {
    let dto: PreguntaIn = parse_json(&body)?;

    let id_nivel = dto
        .nivel
        .as_ref()
        .and_then(|n| n.id())
        .ok_or_else(|| AppError::BadRequest("Debe incluir un nivel válido".to_string()))?;
    NivelJuego::find(&st.pool, id_nivel)
        .await?
        .ok_or_else(|| AppError::NotFound("Nivel no encontrado".to_string()))?;

    let id_palabra = match dto.palabra.as_ref().and_then(|p| p.id()) {
        Some(id) => {
            Palabra::find(&st.pool, id)
                .await?
                .ok_or_else(|| AppError::NotFound("Palabra no encontrada".to_string()))?;
            Some(id)
        }
        None => None,
    };

    let res = sqlx::query(
        "INSERT INTO preguntas_juego (pregunta_texto, opcion1, opcion2, opcion3, opcion4, \
         respuesta_correcta, xp_valor, imagen_url, audio_url, id_palabra, id_nivel) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(dto.pregunta_texto)
    .bind(dto.opcion1)
    .bind(dto.opcion2)
    .bind(dto.opcion3)
    .bind(dto.opcion4)
    .bind(dto.respuesta_correcta)
    .bind(dto.xp_valor.unwrap_or(0))
    .bind(dto.imagen_url)
    .bind(dto.audio_url)
    .bind(id_palabra)
    .bind(id_nivel)
    .execute(&st.pool)
    .await?;

    let creada = PreguntaJuego::find(&st.pool, res.last_insert_id() as i32)
        .await?
        .ok_or_else(|| AppError::Internal("No se pudo leer la pregunta creada".to_string()))?;
    Ok(Json(creada))
}

/// Actualización parcial: solo cambian los campos presentes (y `xpValor` si es distinto de 0).
pub async fn actualizar(
    State(st): State<AppState>,
    Path(id): Path<i32>,
    body: Bytes,
) -> ApiResult<Json<PreguntaJuego>> {
    let dto: PreguntaIn = parse_json(&body)?;
    let existente = PreguntaJuego::find(&st.pool, id).await?.ok_or_else(no_encontrada)?;

    let mut id_palabra = existente.palabra.as_ref().map(|p| p.id_palabra);
    if let Some(nueva) = dto.palabra.as_ref().and_then(|p| p.id()) {
        if let Some(p) = Palabra::find(&st.pool, nueva).await? {
            id_palabra = Some(p.id_palabra);
        }
    }
    let mut id_nivel = existente.nivel.as_ref().map(|n| n.id_nivel);
    if let Some(nuevo) = dto.nivel.as_ref().and_then(|n| n.id()) {
        if let Some(n) = NivelJuego::find(&st.pool, nuevo).await? {
            id_nivel = Some(n.id_nivel);
        }
    }
    let xp = dto.xp_valor.filter(|x| *x != 0).unwrap_or(existente.xp_valor);

    sqlx::query(
        "UPDATE preguntas_juego SET pregunta_texto = COALESCE(?, pregunta_texto), \
         opcion1 = COALESCE(?, opcion1), opcion2 = COALESCE(?, opcion2), \
         opcion3 = COALESCE(?, opcion3), opcion4 = COALESCE(?, opcion4), \
         respuesta_correcta = COALESCE(?, respuesta_correcta), xp_valor = ?, \
         imagen_url = COALESCE(?, imagen_url), audio_url = COALESCE(?, audio_url), \
         id_palabra = ?, id_nivel = ? WHERE id_pregunta = ?",
    )
    .bind(dto.pregunta_texto)
    .bind(dto.opcion1)
    .bind(dto.opcion2)
    .bind(dto.opcion3)
    .bind(dto.opcion4)
    .bind(dto.respuesta_correcta)
    .bind(xp)
    .bind(dto.imagen_url)
    .bind(dto.audio_url)
    .bind(id_palabra)
    .bind(id_nivel)
    .bind(id)
    .execute(&st.pool)
    .await?;

    let actualizada = PreguntaJuego::find(&st.pool, id).await?.ok_or_else(no_encontrada)?;
    Ok(Json(actualizada))
}

pub async fn eliminar(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<StatusCode> {
    PreguntaJuego::find(&st.pool, id).await?.ok_or_else(no_encontrada)?;
    sqlx::query("DELETE FROM preguntas_juego WHERE id_pregunta = ?")
        .bind(id)
        .execute(&st.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
