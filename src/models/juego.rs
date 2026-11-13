use super::nivel_juego::{NivelJuego, NivelRef};
use super::usuario::{Usuario, UsuarioRef};
use super::{cols, int, opt_i32, opt_str};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, MySqlPool};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Juego {
    pub id_juego: i32,
    pub puntaje: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fecha_juego: Option<String>,
    pub preguntas_correctas: i32,
    pub total_preguntas: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_partida: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usuario: Option<Usuario>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nivel: Option<NivelJuego>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct JuegoIn {
    pub puntaje: Option<i32>,
    pub fecha_juego: Option<String>,
    pub preguntas_correctas: Option<i32>,
    pub total_preguntas: Option<i32>,
    pub id_partida: Option<String>,
    pub usuario: Option<UsuarioRef>,
    pub nivel: Option<NivelRef>,
}

// `idPartida` es el nombre real de la columna (campo `idPartida` en Java).
const COLS: &[&str] = &[
    "id_juego",
    "puntaje",
    "fecha_juego",
    "preguntas_correctas",
    "total_preguntas",
    "idPartida",
];

impl Juego {
    pub fn from_row(row: &MySqlRow) -> sqlx::Result<Option<Self>> {
        if opt_i32(row, "id_juego")?.is_none() {
            return Ok(None);
        }
        Ok(Some(Self {
            id_juego: int(row, "id_juego")?,
            puntaje: int(row, "puntaje")?,
            fecha_juego: opt_str(row, "fecha_juego")?,
            preguntas_correctas: int(row, "preguntas_correctas")?,
            total_preguntas: int(row, "total_preguntas")?,
            id_partida: opt_str(row, "idPartida")?,
            usuario: Usuario::from_row(row, "u_")?,
            nivel: NivelJuego::from_row(row, "n_")?,
        }))
    }

    /// SELECT de juegos con usuario y nivel. Alias `j`.
    pub fn select() -> String {
        format!(
            "SELECT {}, {}, {} FROM juego j \
             LEFT JOIN usuarios u ON u.id_usuario = j.id_usuario \
             LEFT JOIN niveles_juego n ON n.id_nivel = j.id_nivel",
            cols("j", "", COLS),
            Usuario::cols("u", "u_"),
            NivelJuego::cols("n", "n_")
        )
    }

    pub async fn find(pool: &MySqlPool, id: i32) -> sqlx::Result<Option<Self>> {
        let sql = format!("{} WHERE j.id_juego = ?", Self::select());
        match sqlx::query(&sql).bind(id).fetch_optional(pool).await? {
            Some(row) => Self::from_row(&row),
            None => Ok(None),
        }
    }
}
