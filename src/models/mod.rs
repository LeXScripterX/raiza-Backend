//! Modelos y mapeo fila -> struct.
//!
//! Las consultas con relaciones usan LEFT JOIN y columnas con prefijo
//! (`u_`, `n_`, `pa_`, `ca_`) para armar los objetos anidados, igual que el JSON
//! que devolvía JPA. Convención de JSON (la de JSON-B en GlassFish): camelCase
//! según los getters Java y propiedades `null` omitidas.

pub mod categoria;
pub mod favorito;
pub mod juego;
pub mod nivel_juego;
pub mod palabra;
pub mod pregunta_juego;
pub mod usuario;

use sqlx::{mysql::MySqlRow, Row};

/// `a.col AS prefijocol, ...` para cada columna.
pub(crate) fn cols(alias: &str, prefix: &str, names: &[&str]) -> String {
    names
        .iter()
        .map(|c| format!("{alias}.{c} AS {prefix}{c}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn opt_str(row: &MySqlRow, key: &str) -> sqlx::Result<Option<String>> {
    row.try_get(key)
}

pub(crate) fn opt_i32(row: &MySqlRow, key: &str) -> sqlx::Result<Option<i32>> {
    row.try_get(key)
}

/// Entero que en Java es `int` primitivo: NULL cuenta como 0.
pub(crate) fn int(row: &MySqlRow, key: &str) -> sqlx::Result<i32> {
    Ok(row.try_get::<Option<i32>, _>(key)?.unwrap_or(0))
}

/// Convierte filas en objetos, descartando las que el mapeo marque como `None`.
pub(crate) fn collect<T>(
    rows: &[MySqlRow],
    f: impl Fn(&MySqlRow) -> sqlx::Result<Option<T>>,
) -> sqlx::Result<Vec<T>> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(v) = f(row)? {
            out.push(v);
        }
    }
    Ok(out)
}
