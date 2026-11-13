use super::palabra::{Palabra, CAT_PREFIX};
use super::usuario::Usuario;
use super::{cols, int, opt_i32, opt_str};
use super::categoria::Categoria;
use serde::Serialize;
use sqlx::{mysql::MySqlRow, MySqlPool};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorito {
    pub id_favorito: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fecha_guardado: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usuario: Option<Usuario>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palabra: Option<Palabra>,
}

const COLS: &[&str] = &["id_favorito", "fecha_guardado"];

impl Favorito {
    pub fn from_row(row: &MySqlRow) -> sqlx::Result<Option<Self>> {
        if opt_i32(row, "id_favorito")?.is_none() {
            return Ok(None);
        }
        Ok(Some(Self {
            id_favorito: int(row, "id_favorito")?,
            fecha_guardado: opt_str(row, "fecha_guardado")?,
            usuario: Usuario::from_row(row, "u_")?,
            palabra: Palabra::from_row(row, "pa_")?,
        }))
    }

    /// SELECT de favoritos con usuario, palabra y la categoría de la palabra.
    pub fn select() -> String {
        format!(
            "SELECT {}, {}, {}, {} FROM favoritos f \
             LEFT JOIN usuarios u ON u.id_usuario = f.id_usuario \
             LEFT JOIN palabras pa ON pa.id_palabra = f.id_palabra \
             LEFT JOIN categorias c ON c.id_categoria = pa.id_categoria",
            cols("f", "", COLS),
            Usuario::cols("u", "u_"),
            Palabra::cols("pa", "pa_"),
            Categoria::cols("c", CAT_PREFIX)
        )
    }

    pub async fn find(pool: &MySqlPool, id: i32) -> sqlx::Result<Option<Self>> {
        let sql = format!("{} WHERE f.id_favorito = ?", Self::select());
        match sqlx::query(&sql).bind(id).fetch_optional(pool).await? {
            Some(row) => Self::from_row(&row),
            None => Ok(None),
        }
    }
}
