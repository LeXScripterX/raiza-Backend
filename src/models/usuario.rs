use super::{cols, int, opt_i32, opt_str};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, FromRow, MySqlPool};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
pub enum Rol {
    ADMIN,
    USUARIO,
    INVITADO,
}

impl Default for Rol {
    fn default() -> Self {
        Rol::INVITADO
    }
}

impl std::fmt::Display for Rol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rol::ADMIN => write!(f, "ADMIN"),
            Rol::USUARIO => write!(f, "USUARIO"),
            Rol::INVITADO => write!(f, "INVITADO"),
        }
    }
}

impl std::str::FromStr for Rol {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ADMIN" => Ok(Rol::ADMIN),
            "USUARIO" => Ok(Rol::USUARIO),
            "INVITADO" => Ok(Rol::INVITADO),
            _ => Ok(Rol::INVITADO),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Usuario {
    pub id_usuario: i32,
    pub nombre: String,
    pub correo: String,
    #[serde(skip_serializing)]
    pub contrasena: String,
    pub rol: String,
    pub estado: String,
    pub fecha_registro: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUsuarioDto {
    pub nombre: String,
    pub correo: String,
    pub contrasena: String,
    pub rol: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUsuarioDto {
    /// Java actualizaba con `PUT /api/usuarios` e id en el cuerpo; se conserva.
    pub id_usuario: Option<i32>,
    pub nombre: Option<String>,
    pub correo: Option<String>,
    pub contrasena: Option<String>,
    pub rol: Option<String>,
    pub estado: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginDto {
    pub correo: Option<String>,
    pub contrasena: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub rol: String,
    pub nombre: String,
}

/// Referencia anidada `{"idUsuario": 5}` dentro de juegos.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UsuarioRef {
    pub id_usuario: Option<i32>,
}

impl UsuarioRef {
    /// Java trataba 0 como "sin id".
    pub fn id(&self) -> Option<i32> {
        self.id_usuario.filter(|i| *i != 0)
    }
}

const COLS: &[&str] = &[
    "id_usuario",
    "nombre",
    "correo",
    "contrasena",
    "rol",
    "estado",
    "fecha_registro",
];

// Mapeo manual con prefijo de columnas: permite usar el mismo código para un
// usuario suelto y para el usuario anidado dentro de favoritos/juegos.
impl Usuario {
    pub fn cols(alias: &str, prefix: &str) -> String {
        cols(alias, prefix, COLS)
    }

    /// `None` si la columna id es NULL (LEFT JOIN sin coincidencia).
    pub fn from_row(row: &MySqlRow, p: &str) -> sqlx::Result<Option<Self>> {
        if opt_i32(row, &format!("{p}id_usuario"))?.is_none() {
            return Ok(None);
        }
        Ok(Some(Self {
            id_usuario: int(row, &format!("{p}id_usuario"))?,
            nombre: opt_str(row, &format!("{p}nombre"))?.unwrap_or_default(),
            correo: opt_str(row, &format!("{p}correo"))?.unwrap_or_default(),
            contrasena: opt_str(row, &format!("{p}contrasena"))?.unwrap_or_default(),
            rol: opt_str(row, &format!("{p}rol"))?.unwrap_or_else(|| "INVITADO".to_string()),
            estado: opt_str(row, &format!("{p}estado"))?.unwrap_or_else(|| "Activo".to_string()),
            fecha_registro: opt_str(row, &format!("{p}fecha_registro"))?,
        }))
    }

    pub fn select() -> String {
        format!("SELECT {} FROM usuarios u", Self::cols("u", ""))
    }

    pub async fn find(pool: &MySqlPool, id: i32) -> sqlx::Result<Option<Self>> {
        let sql = format!("{} WHERE u.id_usuario = ?", Self::select());
        match sqlx::query(&sql).bind(id).fetch_optional(pool).await? {
            Some(row) => Self::from_row(&row, ""),
            None => Ok(None),
        }
    }

    pub async fn find_by_correo(pool: &MySqlPool, correo: &str) -> sqlx::Result<Option<Self>> {
        let sql = format!("{} WHERE u.correo = ?", Self::select());
        match sqlx::query(&sql).bind(correo).fetch_optional(pool).await? {
            Some(row) => Self::from_row(&row, ""),
            None => Ok(None),
        }
    }
}
