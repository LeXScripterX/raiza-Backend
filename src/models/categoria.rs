use super::{cols, int, opt_i32, opt_str};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, MySqlPool};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Categoria {
    pub id_categoria: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nombre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imagen_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descripcion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estado: Option<String>,
}

/// Cuerpo de POST/PUT. `estado` e `idCategoria` del cliente se ignoran, como en Java.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CategoriaIn {
    pub nombre: Option<String>,
    pub imagen_url: Option<String>,
    pub descripcion: Option<String>,
}

const COLS: &[&str] = &["id_categoria", "nombre", "imagen_url", "descripcion", "estado"];

impl Categoria {
    pub fn cols(alias: &str, prefix: &str) -> String {
        cols(alias, prefix, COLS)
    }

    /// `None` si la columna id es NULL (LEFT JOIN sin coincidencia).
    pub fn from_row(row: &MySqlRow, p: &str) -> sqlx::Result<Option<Self>> {
        if opt_i32(row, &format!("{p}id_categoria"))?.is_none() {
            return Ok(None);
        }
        Ok(Some(Self {
            id_categoria: int(row, &format!("{p}id_categoria"))?,
            nombre: opt_str(row, &format!("{p}nombre"))?,
            imagen_url: opt_str(row, &format!("{p}imagen_url"))?,
            descripcion: opt_str(row, &format!("{p}descripcion"))?,
            estado: opt_str(row, &format!("{p}estado"))?,
        }))
    }

    pub fn select() -> String {
        format!("SELECT {} FROM categorias c", Self::cols("c", ""))
    }

    pub async fn find(pool: &MySqlPool, id: i32) -> sqlx::Result<Option<Self>> {
        let sql = format!("{} WHERE c.id_categoria = ?", Self::select());
        match sqlx::query(&sql).bind(id).fetch_optional(pool).await? {
            Some(row) => Self::from_row(&row, ""),
            None => Ok(None),
        }
    }
}
