//! `/api/usuarios` (UsuarioEndpoint.java + UsuarioResource.java).

use crate::{
    error::{is_duplicate, mensaje, parse_json, ApiResult, AppError},
    models::{
        collect,
        usuario::{CreateUsuarioDto, Rol, UpdateUsuarioDto, Usuario},
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
pub struct UsuarioQuery {
    pub correo: Option<String>,
}

fn no_encontrado() -> AppError {
    AppError::NotFound("Usuario no encontrado".to_string())
}

/// Hash BCrypt (coste 10, como `BCrypt.gensalt()`) con prefijo `$2a$`: es el único
/// que entiende jBCrypt 0.4, así Java y Rust pueden compartir la misma base de datos.
pub async fn hash_password(contrasena: String) -> ApiResult<String> {
    let hash = tokio::task::spawn_blocking(move || {
        bcrypt::hash_with_result(contrasena, 10).map(|h| h.format_for_version(bcrypt::Version::TwoA))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Error en tarea de hash: {e}")))??;
    Ok(hash)
}

fn parse_rol(r: &str) -> String {
    r.parse::<Rol>().unwrap_or_default().to_string()
}

pub async fn crear(State(st): State<AppState>, body: Bytes) -> ApiResult<Response> {
    let dto: CreateUsuarioDto = parse_json(&body)?;
    let ya_existe = || AppError::Conflict("El correo ya existe".to_string());

    if Usuario::find_by_correo(&st.pool, &dto.correo).await?.is_some() {
        return Err(ya_existe());
    }

    let hash = hash_password(dto.contrasena).await?;
    let rol = dto.rol.as_deref().map(parse_rol).unwrap_or_else(|| Rol::INVITADO.to_string());
    let fecha = chrono::Local::now().format("%Y-%m-%d").to_string();

    sqlx::query(
        "INSERT INTO usuarios (nombre, correo, contrasena, rol, estado, fecha_registro) \
         VALUES (?, ?, ?, ?, 'Activo', ?)",
    )
    .bind(&dto.nombre)
    .bind(&dto.correo)
    .bind(&hash)
    .bind(&rol)
    .bind(&fecha)
    .execute(&st.pool)
    .await
    .map_err(|e| if is_duplicate(&e) { ya_existe() } else { e.into() })?;

    Ok(mensaje(StatusCode::OK, "Usuario creado correctamente"))
}

pub async fn listar(
    State(st): State<AppState>,
    Query(q): Query<UsuarioQuery>,
) -> ApiResult<Response> {
    use axum::response::IntoResponse;
    if let Some(correo) = q.correo {
        let u = Usuario::find_by_correo(&st.pool, &correo)
            .await?
            .ok_or_else(no_encontrado)?;
        return Ok(Json(u).into_response());
    }
    let rows = sqlx::query(&Usuario::select()).fetch_all(&st.pool).await?;
    let lista = collect(&rows, |r| Usuario::from_row(r, ""))?;
    Ok(Json(lista).into_response())
}

pub async fn obtener(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<Json<Usuario>> {
    Usuario::find(&st.pool, id)
        .await?
        .map(Json)
        .ok_or_else(no_encontrado)
}

/// `PUT /api/usuarios` con `idUsuario` en el cuerpo (así era en Java).
pub async fn actualizar(State(st): State<AppState>, body: Bytes) -> ApiResult<Response> {
    let dto: UpdateUsuarioDto = parse_json(&body)?;
    let id = dto.id_usuario.unwrap_or(0);
    actualizar_usuario(&st, id, dto).await
}

/// `PUT /api/usuarios/{id}` (variante del plan de migración).
pub async fn actualizar_por_id(
    State(st): State<AppState>,
    Path(id): Path<i32>,
    body: Bytes,
) -> ApiResult<Response> {
    let dto: UpdateUsuarioDto = parse_json(&body)?;
    actualizar_usuario(&st, id, dto).await
}

/// Actualización parcial: solo cambian los campos presentes en el JSON.
/// La contraseña solo se re-hashea si viene y no está vacía (igual que en Java).
async fn actualizar_usuario(st: &AppState, id: i32, dto: UpdateUsuarioDto) -> ApiResult<Response> {
    Usuario::find(&st.pool, id).await?.ok_or_else(no_encontrado)?;

    let hash = match dto.contrasena.filter(|p| !p.is_empty()) {
        Some(p) => Some(hash_password(p).await?),
        None => None,
    };
    let rol = dto.rol.as_deref().map(parse_rol);

    sqlx::query(
        "UPDATE usuarios SET nombre = COALESCE(?, nombre), correo = COALESCE(?, correo), \
         rol = COALESCE(?, rol), estado = COALESCE(?, estado), \
         contrasena = COALESCE(?, contrasena) WHERE id_usuario = ?",
    )
    .bind(dto.nombre)
    .bind(dto.correo)
    .bind(rol)
    .bind(dto.estado)
    .bind(hash)
    .bind(id)
    .execute(&st.pool)
    .await
    .map_err(|e| {
        if is_duplicate(&e) {
            AppError::Conflict("El correo ya existe".to_string())
        } else {
            e.into()
        }
    })?;

    Ok(mensaje(StatusCode::OK, "Usuario actualizado"))
}

pub async fn eliminar(State(st): State<AppState>, Path(id): Path<i32>) -> ApiResult<Response> {
    Usuario::find(&st.pool, id).await?.ok_or_else(no_encontrado)?;
    sqlx::query("DELETE FROM usuarios WHERE id_usuario = ?")
        .bind(id)
        .execute(&st.pool)
        .await?;
    Ok(mensaje(StatusCode::OK, "Usuario eliminado"))
}
