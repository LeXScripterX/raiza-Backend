//! `/api/favoritos` (FavoritoResource.java).

use crate::{
    error::{mensaje, ApiResult, AppError},
    handlers::now_datetime,
    models::{collect, favorito::Favorito, palabra::Palabra, usuario::Usuario},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AgregarQuery {
    #[serde(rename = "idUsuario")]
    pub id_usuario: Option<i32>,
    #[serde(rename = "idPalabra")]
    pub id_palabra: Option<i32>,
}

fn no_encontrado() -> AppError {
    AppError::NotFound("Favorito no encontrado".to_string())
}

pub async fn agregar(
    State(st): State<AppState>,
    Query(q): Query<AgregarQuery>,
) -> ApiResult<Response> {
    let id_usuario = q.id_usuario.unwrap_or(0);
    let id_palabra = q.id_palabra.unwrap_or(0);

    let usuario = Usuario::find(&st.pool, id_usuario).await?;
    let palabra = Palabra::find(&st.pool, id_palabra).await?;
    if usuario.is_none() || palabra.is_none() {
        return Err(AppError::NotFound(
            "Usuario o Palabra no encontrados".to_string(),
        ));
    }

    let existe: Option<i32> =
        sqlx::query_scalar("SELECT id_favorito FROM favoritos WHERE id_usuario = ? AND id_palabra = ? LIMIT 1")
            .bind(id_usuario)
            .bind(id_palabra)
            .fetch_optional(&st.pool)
            .await?;
    if existe.is_some() {
        return Err(AppError::Conflict(
            "La palabra ya está en favoritos".to_string(),
        ));
    }

    sqlx::query("INSERT INTO favoritos (fecha_guardado, id_usuario, id_palabra) VALUES (?, ?, ?)")
        .bind(now_datetime())
        .bind(id_usuario)
        .bind(id_palabra)
        .execute(&st.pool)
        .await?;
    Ok(mensaje(StatusCode::CREATED, "Favorito agregado correctamente"))
}

pub async fn listar(State(st): State<AppState>) -> ApiResult<Json<Vec<Favorito>>> {
    let rows = sqlx::query(&Favorito::select()).fetch_all(&st.pool).await?;
    Ok(Json(collect(&rows, Favorito::from_row)?))
}

pub async fn obtener(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<Json<Favorito>> {
    Favorito::find(&st.pool, id)
        .await?
        .map(Json)
        .ok_or_else(no_encontrado)
}

pub async fn por_usuario(
    State(st): State<AppState>,
    Path(id_usuario): Path<i32>,
) -> ApiResult<Json<Vec<Favorito>>> {
    let sql = format!("{} WHERE f.id_usuario = ?", Favorito::select());
    let rows = sqlx::query(&sql).bind(id_usuario).fetch_all(&st.pool).await?;
    let lista = collect(&rows, Favorito::from_row)?;
    if lista.is_empty() {
        return Err(AppError::NotFound(
            "El usuario no tiene favoritos".to_string(),
        ));
    }
    Ok(Json(lista))
}

pub async fn eliminar(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<Response> {
    Favorito::find(&st.pool, id).await?.ok_or_else(no_encontrado)?;
    sqlx::query("DELETE FROM favoritos WHERE id_favorito = ?")
        .bind(id)
        .execute(&st.pool)
        .await?;
    Ok(mensaje(StatusCode::OK, "Favorito eliminado correctamente"))
}
