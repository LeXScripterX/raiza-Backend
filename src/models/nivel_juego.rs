use super::{cols, int, opt_i32, opt_str};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, MySqlPool};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NivelJuego {
    pub id_nivel: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nombre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descripcion: Option<String>,
    pub orden: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estado: Option<String>,
}

/// En Java `estado` valía "Activo" y `orden` 0 si el JSON no los traía.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NivelIn {
    pub nombre: Option<String>,
    pub descripcion: Option<String>,
    pub orden: Option<i32>,
    pub estado: Option<String>,
}

/// Referencia anidada `{"idNivel": 3}` dentro de preguntas y juegos.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct NivelRef {
    pub id_nivel: Option<i32>,
}

impl NivelRef {
    /// Id utilizable (Java trataba 0 como "sin id").
    pub fn id(&self) -> Option<i32> {
        self.id_nivel.filter(|i| *i != 0)
    }
}

const COLS: &[&str] = &["id_nivel", "nombre", "descripcion", "orden", "estado"];

impl NivelJuego {
    pub fn cols(alias: &str, prefix: &str) -> String {
        cols(alias, prefix, COLS)
    }

    pub fn from_row(row: &MySqlRow, p: &str) -> sqlx::Result<Option<Self>> {
        if opt_i32(row, &format!("{p}id_nivel"))?.is_none() {
            return Ok(None);
        }
        Ok(Some(Self {
            id_nivel: int(row, &format!("{p}id_nivel"))?,
            nombre: opt_str(row, &format!("{p}nombre"))?,
            descripcion: opt_str(row, &format!("{p}descripcion"))?,
            orden: int(row, &format!("{p}orden"))?,
            estado: opt_str(row, &format!("{p}estado"))?,
        }))
    }

    pub fn select() -> String {
        format!("SELECT {} FROM niveles_juego n", Self::cols("n", ""))
    }

    pub async fn find(pool: &MySqlPool, id: i32) -> sqlx::Result<Option<Self>> {
        let sql = format!("{} WHERE n.id_nivel = ?", Self::select());
        match sqlx::query(&sql).bind(id).fetch_optional(pool).await? {
            Some(row) => Self::from_row(&row, ""),
            None => Ok(None),
        }
    }
}
