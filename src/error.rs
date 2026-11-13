use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

/// Número de error de MySQL/MariaDB, si el fallo vino de la base de datos.
fn mysql_code(err: &sqlx::Error) -> Option<u16> {
    err.as_database_error()
        .and_then(|d| d.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
        .map(|d| d.number())
}

/// `true` si el error es una violación de UNIQUE (1062).
pub fn is_duplicate(err: &sqlx::Error) -> bool {
    mysql_code(err) == Some(1062)
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match mysql_code(&err) {
            Some(1062) => AppError::Conflict("Ya existe un registro con ese valor".to_string()),
            // Borrar un padre que todavía tiene hijos (FK sin cascada).
            Some(1451) => AppError::Conflict(
                "No se puede eliminar: hay registros que dependen de este".to_string(),
            ),
            // Insertar/actualizar apuntando a un padre que no existe.
            Some(1452) => AppError::BadRequest(
                "Referencia inválida: el usuario, nivel, palabra o categoría indicado no existe"
                    .to_string(),
            ),
            _ => {
                // El detalle va al log, no al cliente.
                tracing::error!("Database error: {:?}", err);
                AppError::Internal("Error interno del servidor".to_string())
            }
        }
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        tracing::error!("Bcrypt error: {:?}", err);
        AppError::Internal("Error en procesamiento de contraseña".to_string())
    }
}

pub type ApiResult<T> = Result<T, AppError>;

/// Respuesta `{"mensaje":"..."}` con el código indicado (formato del backend Java).
pub fn mensaje(status: StatusCode, texto: &str) -> Response {
    (status, Json(json!({ "mensaje": texto }))).into_response()
}

/// Deserializa el cuerpo JSON; 400 si está vacío o es inválido.
pub fn parse_json<T: serde::de::DeserializeOwned>(body: &[u8]) -> ApiResult<T> {
    if body.iter().all(|b| b.is_ascii_whitespace()) {
        return Err(AppError::BadRequest("Se requiere un cuerpo JSON".to_string()));
    }
    serde_json::from_slice(body)
        .map_err(|e| AppError::BadRequest(format!("JSON inválido: {e}")))
}

/// Como `parse_json`, pero un cuerpo vacío es `None` (en JAX-RS llegaba `null`).
pub fn parse_json_opt<T: serde::de::DeserializeOwned>(body: &[u8]) -> ApiResult<Option<T>> {
    if body.iter().all(|b| b.is_ascii_whitespace()) {
        return Ok(None);
    }
    parse_json(body).map(Some)
}
