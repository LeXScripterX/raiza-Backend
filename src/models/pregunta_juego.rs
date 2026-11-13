use super::categoria::Categoria;
use super::nivel_juego::{NivelJuego, NivelRef};
use super::palabra::{Palabra, PalabraRef, CAT_PREFIX};
use super::{cols, int, opt_i32, opt_str};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, MySqlPool};

/// Nota: los nombres `respuestaCoreccta` (con la errata) y `audio_url` (con guion
/// bajo) vienen de los getters del backend Java y se conservan para no romper
/// al frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreguntaJuego {
    pub id_pregunta: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pregunta_texto: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opcion1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opcion2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opcion3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opcion4: Option<String>,
    #[serde(rename = "respuestaCoreccta", skip_serializing_if = "Option::is_none")]
    pub respuesta_correcta: Option<String>,
    pub xp_valor: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imagen_url: Option<String>,
    #[serde(rename = "audio_url", skip_serializing_if = "Option::is_none")]
    pub audio_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palabra: Option<Palabra>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nivel: Option<NivelJuego>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct PreguntaIn {
    pub pregunta_texto: Option<String>,
    pub opcion1: Option<String>,
    pub opcion2: Option<String>,
    pub opcion3: Option<String>,
    pub opcion4: Option<String>,
    #[serde(rename = "respuestaCoreccta")]
    pub respuesta_correcta: Option<String>,
    pub xp_valor: Option<i32>,
    pub imagen_url: Option<String>,
    #[serde(rename = "audio_url")]
    pub audio_url: Option<String>,
    pub palabra: Option<PalabraRef>,
    pub nivel: Option<NivelRef>,
}

const COLS: &[&str] = &[
    "id_pregunta",
    "pregunta_texto",
    "opcion1",
    "opcion2",
    "opcion3",
    "opcion4",
    "respuesta_correcta",
    "xp_valor",
    "imagen_url",
    "audio_url",
];

impl PreguntaJuego {
    pub fn from_row(row: &MySqlRow) -> sqlx::Result<Option<Self>> {
        if opt_i32(row, "id_pregunta")?.is_none() {
            return Ok(None);
        }
        Ok(Some(Self {
            id_pregunta: int(row, "id_pregunta")?,
            pregunta_texto: opt_str(row, "pregunta_texto")?,
            opcion1: opt_str(row, "opcion1")?,
            opcion2: opt_str(row, "opcion2")?,
            opcion3: opt_str(row, "opcion3")?,
            opcion4: opt_str(row, "opcion4")?,
            respuesta_correcta: opt_str(row, "respuesta_correcta")?,
            xp_valor: int(row, "xp_valor")?,
            imagen_url: opt_str(row, "imagen_url")?,
            audio_url: opt_str(row, "audio_url")?,
            palabra: Palabra::from_row(row, "pa_")?,
            nivel: NivelJuego::from_row(row, "n_")?,
        }))
    }

    /// SELECT de preguntas con palabra (+ su categoría) y nivel. Alias `q`.
    pub fn select() -> String {
        format!(
            "SELECT {}, {}, {}, {} FROM preguntas_juego q \
             LEFT JOIN palabras pa ON pa.id_palabra = q.id_palabra \
             LEFT JOIN categorias c ON c.id_categoria = pa.id_categoria \
             LEFT JOIN niveles_juego n ON n.id_nivel = q.id_nivel",
            cols("q", "", COLS),
            Palabra::cols("pa", "pa_"),
            Categoria::cols("c", CAT_PREFIX),
            NivelJuego::cols("n", "n_")
        )
    }

    pub async fn find(pool: &MySqlPool, id: i32) -> sqlx::Result<Option<Self>> {
        let sql = format!("{} WHERE q.id_pregunta = ?", Self::select());
        match sqlx::query(&sql).bind(id).fetch_optional(pool).await? {
            Some(row) => Self::from_row(&row),
            None => Ok(None),
        }
    }
}
