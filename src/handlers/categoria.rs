//! `/api/categorias` (CategoriaResource.java).

use crate::{
    error::{mensaje, parse_json, ApiResult, AppError},
    handlers::{like, now_datetime},
    models::{
        categoria::{Categoria, CategoriaIn},
        collect,
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
pub struct NombreQuery {
    pub nombre: Option<String>,
}

fn no_encontrada() -> AppError {
    AppError::NotFound("Categoría no encontrada".to_string())
}

pub async fn listar(
    State(st): State<AppState>,
    Query(q): Query<NombreQuery>,
) -> ApiResult<Json<Vec<Categoria>>> {
    let rows = match q.nombre.filter(|n| !n.is_empty()) {
        Some(n) => {
            let sql = format!("{} WHERE LOWER(c.nombre) LIKE LOWER(?)", Categoria::select());
            sqlx::query(&sql).bind(like(&n)).fetch_all(&st.pool).await?
        }
        None => sqlx::query(&Categoria::select()).fetch_all(&st.pool).await?,
    };
    Ok(Json(collect(&rows, |r| Categoria::from_row(r, ""))?))
}

pub async fn obtener(
    State(st): State<AppState>,
    Path(id): Path<i32>,
) -> ApiResult<Json<Categoria>> {
    Categoria::find(&st.pool, id)
        .await?
        .map(Json)
        .ok_or_else(no_encontrada)
}

pub async fn crear(State(st): State<AppState>, body: Bytes) -> ApiResult<Response> {
    let dto: CategoriaIn = parse_json(&body)?;
    // Java siempre fuerza estado "Activa" y añade la fecha al final de la descripción.
    let descripcion = format!(
        "{} (Creado: {})",
        dto.descripcion.unwrap_or_default(),
        now_datetime()
    );
    sqlx::query(
        "INSERT INTO categorias (nombre, imagen_url, descripcion, estado) VALUES (?, ?, ?, 'Activa')",
    )
    .bind(dto.nombre)
    .bind(dto.imagen_url)
    .bind(descripcion)
    .execute(&st.pool)
    .await?;
    Ok(mensaje(StatusCode::CREATED, "Categoría creada correctamente"))
}

pub async fn actualizar(
    State(st): State<AppState>,
    Path(id): Path<i32>,
    body: Bytes,
) -> ApiResult<Response> {
    let dto: CategoriaIn = parse_json(&body)?;
    Categoria::find(&st.pool, id).await?.ok_or_else(no_encontrada)?;
    // Como en Java: nombre, descripción e imagen se reemplazan con lo recibido.
    sqlx::query("UPDATE categorias SET nombre = ?, descripcion = ?, imagen_url = ? WHERE id_categoria = ?")
        .bind(dto.nombre)
        .bind(dto.descripcion)
        .bind(dto.imagen_url)
        .bind(id)
        .execute(&st.pool)
        .await?;
    Ok(mensaje(StatusCode::OK, "Categoría actualizada"))
}

pub async fn eliminar(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<Response> {
    Categoria::find(&st.pool, id).await?.ok_or_else(no_encontrada)?;
    sqlx::query("DELETE FROM categorias WHERE id_categoria = ?")
        .bind(id)
        .execute(&st.pool)
        .await?;
    Ok(mensaje(StatusCode::OK, "Categoría eliminada"))
}
