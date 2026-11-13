//! Equivalente a `security/SecurityFilter.java` (la parte de autenticación;
//! CORS lo aplica `tower-http` en `main.rs`).
//!
//! Reglas replicadas tal cual:
//! 1. `OPTIONS` pasa sin token (preflight).
//! 2. Rutas públicas: `login`, GET de categorias/palabras/niveles-juego/
//!    preguntas-juego/juegos y POST de usuarios (registro).
//! 3. El resto exige `Authorization: Bearer <jwt>` HS256 con claim `rol`
//!    igual a `ADMIN` o `USUARIO` (`INVITADO` => 403).
//!
//! Igual que en Java, solo se comprueba que el rol sea válido: no hay permisos
//! distintos para ADMIN y USUARIO en cada endpoint.

use crate::{
    error::{ApiResult, AppError},
    AppState,
};
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, Method},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Claims {
    rol: Option<String>,
}

pub async fn require_auth(State(state): State<AppState>, req: Request, next: Next) -> Response {
    match authorize(&state, &req) {
        Ok(()) => next.run(req).await,
        Err(e) => e.into_response(),
    }
}

fn is_public(method: &Method, path: &str) -> bool {
    let get = method == Method::GET;
    path.ends_with("login")
        || (path.contains("categorias") && get)
        || (path.contains("palabras") && get)
        || (path.contains("niveles-juego") && get)
        || (path.contains("preguntas-juego") && get)
        || (path.contains("juegos") && get)
        || (path.contains("usuarios") && method == Method::POST)
}

fn authorize(state: &AppState, req: &Request) -> ApiResult<()> {
    if req.method() == Method::OPTIONS {
        return Ok(());
    }

    // Dentro del router anidado el prefijo `/api` ya fue retirado, como en
    // `UriInfo.getPath()` de JAX-RS (ruta relativa a la aplicación).
    let path = req.uri().path().trim_start_matches('/');
    if is_public(req.method(), path) {
        return Ok(());
    }

    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Token no proporcionado".to_string()))?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0; // jjwt no aplica margen de reloj por defecto

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized("Token inválido o expirado".to_string()))?;

    match data.claims.rol.as_deref() {
        Some("ADMIN") | Some("USUARIO") => Ok(()),
        _ => Err(AppError::Forbidden("Rol no autorizado".to_string())),
    }
}
