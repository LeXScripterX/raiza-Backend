use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

pub async fn init_db_pool(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    init_schema(&pool).await?;

    Ok(pool)
}

async fn init_schema(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS usuarios (
            id_usuario INT AUTO_INCREMENT PRIMARY KEY,
            nombre VARCHAR(255) NOT NULL,
            correo VARCHAR(255) NOT NULL UNIQUE,
            contrasena VARCHAR(255) NOT NULL,
            rol VARCHAR(50) NOT NULL DEFAULT 'INVITADO',
            estado VARCHAR(50) NOT NULL DEFAULT 'Activo',
            fecha_registro VARCHAR(50)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS categorias (
            id_categoria INT AUTO_INCREMENT PRIMARY KEY,
            nombre VARCHAR(255),
            imagen_url VARCHAR(500),
            descripcion TEXT,
            estado VARCHAR(50) NOT NULL DEFAULT 'Activa'
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS niveles_juego (
            id_nivel INT AUTO_INCREMENT PRIMARY KEY,
            nombre VARCHAR(255),
            descripcion TEXT,
            orden INT NOT NULL DEFAULT 1,
            estado VARCHAR(50) NOT NULL DEFAULT 'Activo'
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS palabras (
            id_palabra INT AUTO_INCREMENT PRIMARY KEY,
            palabra_nasa VARCHAR(255) NOT NULL,
            traduccion VARCHAR(255),
            frase_ejemplo TEXT,
            imagen_url VARCHAR(500),
            audio_url VARCHAR(500),
            fecha_creacion VARCHAR(50),
            id_categoria INT,
            CONSTRAINT fk_palabras_categoria FOREIGN KEY (id_categoria) REFERENCES categorias(id_categoria) ON DELETE SET NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS favoritos (
            id_favorito INT AUTO_INCREMENT PRIMARY KEY,
            fecha_guardado VARCHAR(50),
            id_usuario INT,
            id_palabra INT,
            CONSTRAINT fk_favoritos_usuario FOREIGN KEY (id_usuario) REFERENCES usuarios(id_usuario) ON DELETE CASCADE,
            CONSTRAINT fk_favoritos_palabra FOREIGN KEY (id_palabra) REFERENCES palabras(id_palabra) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS preguntas_juego (
            id_pregunta INT AUTO_INCREMENT PRIMARY KEY,
            pregunta_texto TEXT,
            opcion1 VARCHAR(255),
            opcion2 VARCHAR(255),
            opcion3 VARCHAR(255),
            opcion4 VARCHAR(255),
            respuesta_correcta VARCHAR(255),
            xp_valor INT NOT NULL DEFAULT 10,
            imagen_url VARCHAR(500),
            audio_url VARCHAR(500),
            id_palabra INT,
            id_nivel INT,
            CONSTRAINT fk_preguntas_palabra FOREIGN KEY (id_palabra) REFERENCES palabras(id_palabra) ON DELETE SET NULL,
            CONSTRAINT fk_preguntas_nivel FOREIGN KEY (id_nivel) REFERENCES niveles_juego(id_nivel) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS juego (
            id_juego INT AUTO_INCREMENT PRIMARY KEY,
            puntaje INT NOT NULL DEFAULT 0,
            fecha_juego VARCHAR(50),
            preguntas_correctas INT NOT NULL DEFAULT 0,
            total_Preguntas INT NOT NULL DEFAULT 0,
            idPartida VARCHAR(255),
            id_usuario INT,
            id_nivel INT,
            CONSTRAINT fk_juego_usuario FOREIGN KEY (id_usuario) REFERENCES usuarios(id_usuario) ON DELETE CASCADE,
            CONSTRAINT fk_juego_nivel FOREIGN KEY (id_nivel) REFERENCES niveles_juego(id_nivel) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#
    )
    .execute(pool)
    .await?;

    tracing::info!("Esquema de base de datos verificado y listo.");
    Ok(())
}
