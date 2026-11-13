mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;

use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    middleware::from_fn_with_state,
    routing::{get, post, put},
    Router,
};
use config::Config;
use handlers::{auth, categoria, favorito, juego, nivel_juego, palabra, pregunta_juego, usuario};
use sqlx::MySqlPool;
use std::{sync::Arc, time::Duration};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() {
    // Config::from_env() carga también el archivo .env
    let config = Config::from_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .init();

    if config.jwt_secret.len() < 32 {
        tracing::warn!(
            "JWT_SECRET tiene solo {} caracteres (HS256 pide 32 o más). Si lo copiaste de \
             SecurityConstants.java, usa comillas simples en el .env: JWT_SECRET='...' \
             (sin ellas, `$` y `#` se interpretan y el valor queda recortado).",
            config.jwt_secret.len()
        );
    }

    let pool = db::init_db_pool(&config.database_url)
        .await
        .expect("No se pudo conectar a la base de datos (revisa DATABASE_URL)");

    let addr = format!("{}:{}", config.host, config.port);
    let api_prefix = config.api_prefix.clone();
    let state = AppState {
        pool,
        config: Arc::new(config),
    };

    let api = Router::new()
        // Auth
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        // Usuarios
        .route(
            "/usuarios",
            post(usuario::crear).get(usuario::listar).put(usuario::actualizar),
        )
        .route(
            "/usuarios/:id",
            get(usuario::obtener)
                .put(usuario::actualizar_por_id)
                .delete(usuario::eliminar),
        )
        // Categorías
        .route("/categorias", get(categoria::listar).post(categoria::crear))
        .route(
            "/categorias/:id",
            get(categoria::obtener)
                .put(categoria::actualizar)
                .delete(categoria::eliminar),
        )
        // Palabras
        .route("/palabras", get(palabra::listar).post(palabra::crear))
        .route(
            "/palabras/:id",
            get(palabra::obtener)
                .put(palabra::actualizar)
                .delete(palabra::eliminar),
        )
        .route("/palabras/categoria/:id", get(palabra::por_categoria))
        // Favoritos
        .route("/favoritos", get(favorito::listar).post(favorito::agregar))
        .route(
            "/favoritos/:id",
            get(favorito::obtener).delete(favorito::eliminar),
        )
        .route("/favoritos/usuario/:id", get(favorito::por_usuario))
        // Niveles
        .route(
            "/niveles-juego",
            get(nivel_juego::listar).post(nivel_juego::crear),
        )
        .route(
            "/niveles-juego/:id",
            get(nivel_juego::obtener)
                .put(nivel_juego::actualizar)
                .delete(nivel_juego::eliminar),
        )
        // Preguntas
        .route(
            "/preguntas-juego",
            get(pregunta_juego::listar).post(pregunta_juego::crear),
        )
        .route(
            "/preguntas-juego/:id",
            get(pregunta_juego::obtener)
                .put(pregunta_juego::actualizar)
                .delete(pregunta_juego::eliminar),
        )
        .route("/preguntas-juego/nivel/:id", get(pregunta_juego::por_nivel))
        // Juegos
        .route("/juegos", post(juego::guardar))
        .route("/juegos/:id", put(juego::actualizar))
        .route("/juegos/historial/:id", get(juego::historial))
        .route("/juegos/mejores/:id", get(juego::mejores))
        .route("/juegos/mejor/:id_usuario/:id_nivel", get(juego::mejor))
        .route("/juegos/total/:id", get(juego::total))
        .route("/juegos/ranking", get(juego::ranking))
        .layer(from_fn_with_state(state.clone(), middleware::auth::require_auth))
        .with_state(state);

    // Mismos headers y política que SecurityFilter.java: se refleja el Origin.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])
        .expose_headers([AUTHORIZATION])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));

    let app = Router::new()
        .nest(&api_prefix, api)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("No se pudo abrir {addr}: {e}"));
    tracing::info!("YuweLongo backend escuchando en http://{addr}{api_prefix}");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("Apagando servidor...");
        })
        .await
        .expect("Error en el servidor");
}
