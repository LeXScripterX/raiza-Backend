//! `/api/palabras` (PalabraResource.java).

use crate::{
    error::{mensaje, parse_json, ApiResult, AppError},
    handlers::{like, now_datetime},
    models::{
        categoria::Categoria,
        collect,
        palabra::{Palabra, PalabraIn},
    },
    AppState,
};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CategoriaQuery {
    #[serde(rename = "idCategoria")]
    pub id_categoria: Option<i32>,
}

#[derive(Deserialize)]
pub struct BusquedaQuery {
    #[serde(rename = "palabraNasa")]
    pub palabra_nasa: Option<String>,
    pub traduccion: Option<String>,
}

fn no_encontrada() -> AppError {
    AppError::NotFound("Palabra no encontrada".to_string())
}

pub async fn listar(
    State(st): State<AppState>,
    Query(q): Query<BusquedaQuery>,
) -> ApiResult<Json<Vec<Palabra>>> {
    let rows = if let Some(t) = q.palabra_nasa.filter(|t| !t.is_empty()) {
        let sql = format!("{} WHERE LOWER(p.palabra_nasa) LIKE LOWER(?)", Palabra::select());
        sqlx::query(&sql).bind(like(&t)).fetch_all(&st.pool).await?
    } else if let Some(t) = q.traduccion.filter(|t| !t.is_empty()) {
        let sql = format!("{} WHERE LOWER(p.traduccion) LIKE LOWER(?)", Palabra::select());
        sqlx::query(&sql).bind(like(&t)).fetch_all(&st.pool).await?
    } else {
        sqlx::query(&Palabra::select()).fetch_all(&st.pool).await?
    };
    Ok(Json(collect(&rows, |r| Palabra::from_row(r, ""))?))
}

pub async fn obtener(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<Json<Palabra>> {
    Palabra::find(&st.pool, id)
        .await?
        .map(Json)
        .ok_or_else(no_encontrada)
}

pub async fn por_categoria(
    State(st): State<AppState>,
    Path(id_categoria): Path<i32>,
) -> ApiResult<Json<Vec<Palabra>>> {
    let sql = format!("{} WHERE p.id_categoria = ?", Palabra::select());
    let rows = sqlx::query(&sql).bind(id_categoria).fetch_all(&st.pool).await?;
    let lista = collect(&rows, |r| Palabra::from_row(r, ""))?;
    if lista.is_empty() {
        return Err(AppError::NotFound(
            "No hay palabras para esta categoría".to_string(),
        ));
    }
    Ok(Json(lista))
}

pub async fn crear(
    State(st): State<AppState>,
    Query(q): Query<CategoriaQuery>,
    body: Bytes,
) -> ApiResult<Response> {
    let dto: PalabraIn = parse_json(&body)?;
    let categoria = Categoria::find(&st.pool, q.id_categoria.unwrap_or(0))
        .await?
        .ok_or_else(|| AppError::NotFound("Categoría no encontrada".to_string()))?;
    let palabra_nasa = dto
        .palabra_nasa
        .ok_or_else(|| AppError::BadRequest("El campo palabraNasa es obligatorio".to_string()))?;

    sqlx::query(
        "INSERT INTO palabras (palabra_nasa, traduccion, frase_ejemplo, imagen_url, audio_url, \
         fecha_creacion, id_categoria) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(palabra_nasa)
    .bind(dto.traduccion)
    .bind(dto.frase_ejemplo)
    .bind(dto.imagen_url)
    .bind(dto.audio_url)
    .bind(now_datetime())
    .bind(categoria.id_categoria)
    .execute(&st.pool)
    .await?;
    Ok(mensaje(StatusCode::CREATED, "Palabra creada correctamente"))
}

pub async fn actualizar(
    State(st): State<AppState>,
    Path(id): Path<i32>,
    Query(q): Query<CategoriaQuery>,
    body: Bytes,
) -> ApiResult<Response> {
    let dto: PalabraIn = parse_json(&body)?;
    let existente = Palabra::find(&st.pool, id).await?.ok_or_else(no_encontrada)?;

    // Categoría nueva solo si se indicó y existe; si no, se conserva la actual.
    let mut id_categoria = existente.categoria.as_ref().map(|c| c.id_categoria);
    if let Some(nueva) = q.id_categoria.filter(|c| *c > 0) {
        if let Some(c) = Categoria::find(&st.pool, nueva).await? {
            id_categoria = Some(c.id_categoria);
        }
    }

    // Solo cambian los campos presentes en el JSON (como los `if != null` de Java).
    sqlx::query(
        "UPDATE palabras SET palabra_nasa = COALESCE(?, palabra_nasa), \
         traduccion = COALESCE(?, traduccion), frase_ejemplo = COALESCE(?, frase_ejemplo), \
         imagen_url = COALESCE(?, imagen_url), audio_url = COALESCE(?, audio_url), \
         id_categoria = ? WHERE id_palabra = ?",
    )
    .bind(dto.palabra_nasa)
    .bind(dto.traduccion)
    .bind(dto.frase_ejemplo)
    .bind(dto.imagen_url)
    .bind(dto.audio_url)
    .bind(id_categoria)
    .bind(id)
    .execute(&st.pool)
    .await?;
    Ok(mensaje(StatusCode::OK, "Palabra actualizada correctamente"))
}

pub async fn eliminar(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<Response> {
    Palabra::find(&st.pool, id).await?.ok_or_else(no_encontrada)?;
    sqlx::query("DELETE FROM palabras WHERE id_palabra = ?")
        .bind(id)
        .execute(&st.pool)
        .await?;
    Ok(mensaje(StatusCode::OK, "Palabra eliminada correctamente"))
}
