use super::categoria::Categoria;
use super::{cols, int, opt_i32, opt_str};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, MySqlPool};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Palabra {
    pub id_palabra: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palabra_nasa: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traduccion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frase_ejemplo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imagen_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fecha_creacion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categoria: Option<Categoria>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PalabraIn {
    pub palabra_nasa: Option<String>,
    pub traduccion: Option<String>,
    pub frase_ejemplo: Option<String>,
    pub imagen_url: Option<String>,
    pub audio_url: Option<String>,
}

/// Referencia anidada `{"idPalabra": 7}` dentro de preguntas.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PalabraRef {
    pub id_palabra: Option<i32>,
}

impl PalabraRef {
    pub fn id(&self) -> Option<i32> {
        self.id_palabra.filter(|i| *i != 0)
    }
}

const COLS: &[&str] = &[
    "id_palabra",
    "palabra_nasa",
    "traduccion",
    "frase_ejemplo",
    "imagen_url",
    "audio_url",
    "fecha_creacion",
];

/// Prefijo fijo con el que siempre se selecciona la categoría de una palabra.
pub const CAT_PREFIX: &str = "ca_";

impl Palabra {
    pub fn cols(alias: &str, prefix: &str) -> String {
        cols(alias, prefix, COLS)
    }

    pub fn from_row(row: &MySqlRow, p: &str) -> sqlx::Result<Option<Self>> {
        if opt_i32(row, &format!("{p}id_palabra"))?.is_none() {
            return Ok(None);
        }
        Ok(Some(Self {
            id_palabra: int(row, &format!("{p}id_palabra"))?,
            palabra_nasa: opt_str(row, &format!("{p}palabra_nasa"))?,
            traduccion: opt_str(row, &format!("{p}traduccion"))?,
            frase_ejemplo: opt_str(row, &format!("{p}frase_ejemplo"))?,
            imagen_url: opt_str(row, &format!("{p}imagen_url"))?,
            audio_url: opt_str(row, &format!("{p}audio_url"))?,
            fecha_creacion: opt_str(row, &format!("{p}fecha_creacion"))?,
            categoria: Categoria::from_row(row, CAT_PREFIX)?,
        }))
    }

    /// SELECT de palabras con su categoría (alias `p` y `c`).
    pub fn select() -> String {
        format!(
            "SELECT {}, {} FROM palabras p \
             LEFT JOIN categorias c ON c.id_categoria = p.id_categoria",
            Self::cols("p", ""),
            Categoria::cols("c", CAT_PREFIX)
        )
    }

    pub async fn find(pool: &MySqlPool, id: i32) -> sqlx::Result<Option<Self>> {
        let sql = format!("{} WHERE p.id_palabra = ?", Self::select());
        match sqlx::query(&sql).bind(id).fetch_optional(pool).await? {
            Some(row) => Self::from_row(&row, ""),
            None => Ok(None),
        }
    }
}
