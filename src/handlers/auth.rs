//! `/api/login` y `/api/logout` (LoginEndpoint.java / LogoutEndpoint.java).

use crate::{
    error::{mensaje, parse_json_opt, ApiResult, AppError},
    models::usuario::{LoginDto, LoginResponse, Usuario},
    AppState,
};
use axum::{body::Bytes, extract::State, http::StatusCode, response::Response, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;

#[derive(Serialize)]
struct TokenClaims {
    sub: String,
    rol: String,
    nombre: String,
    iat: i64,
    exp: i64,
}

pub async fn login(State(st): State<AppState>, body: Bytes) -> ApiResult<Json<LoginResponse>> {
    let (correo, contrasena) = match parse_json_opt::<LoginDto>(&body)? {
        Some(LoginDto {
            correo: Some(c),
            contrasena: Some(p),
        }) => (c, p),
        _ => return Err(AppError::BadRequest("Faltan credenciales".to_string())),
    };

    let invalidas = || AppError::Unauthorized("Credenciales inválidas".to_string());

    let usuario = Usuario::find_by_correo(&st.pool, &correo)
        .await?
        .ok_or_else(invalidas)?;

    if !verificar(contrasena, usuario.contrasena.clone()).await {
        return Err(invalidas());
    }

    let ahora = chrono::Utc::now().timestamp();
    let claims = TokenClaims {
        sub: usuario.correo.clone(),
        rol: usuario.rol.clone(),
        nombre: usuario.nombre.clone(),
        iat: ahora,
        exp: ahora + st.config.jwt_expiration_hours * 3600,
    };
    let access_token = encode(
        &Header::default(), // HS256, como jjwt con una clave de 256 bits
        &claims,
        &EncodingKey::from_secret(st.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("No se pudo firmar el JWT: {e}");
        AppError::Internal("Error generando el token".to_string())
    })?;

    Ok(Json(LoginResponse {
        access_token,
        rol: usuario.rol,
        nombre: usuario.nombre,
    }))
}

pub async fn logout() -> Response {
    mensaje(StatusCode::OK, "Sesión cerrada correctamente")
}

/// BCrypt es CPU-intensivo: se ejecuta fuera del hilo asíncrono.
async fn verificar(contrasena: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || bcrypt::verify(contrasena, &hash).unwrap_or(false))
        .await
        .unwrap_or(false)
}
