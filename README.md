# YuweLongo – Backend en Rust

Migración del backend de YuweLongo (Java / Jakarta EE 10 / GlassFish) a **Rust**
(Axum 0.7 + Tokio + SQLx). Mantiene las rutas `/api/...`, los códigos de estado, el formato
JSON, los hashes BCrypt y los JWT del backend Java, y usa la misma base de datos MariaDB/MySQL
(`yuwelongo`). El backend Java de la raíz del repositorio no se toca.

## Requisitos
- Rust estable (`rustup` o el paquete `rust` de tu distro)
- MariaDB/MySQL con la base `yuwelongo` y el usuario `userYuwe` (ver README del backend Java)

## Configuración
```bash
cp .env.example .env     # y ajusta los valores
```
| Variable | Descripción |
|---|---|
| `DATABASE_URL` | `mysql://userYuwe:123456@localhost:3306/yuwelongo` |
| `PORT` / `HOST` | `8080` / `0.0.0.0` |
| `JWT_SECRET` | Debe ser idéntico al de `SecurityConstants.java` si quieres que los tokens de Java sigan valiendo. **Va entre comillas simples** (`JWT_SECRET='...'`): sin ellas `dotenvy` interpreta `$Yx7uT` como variable y el secreto queda de 26 caracteres. |
| `JWT_EXPIRATION_HOURS` | `1` (igual que Java) |
| `CONTEXT_PATH` | Opcional. `/YuweLongo-Backend` mantiene las URLs `/YuweLongo-Backend/api/...` del WAR de GlassFish. Vacío = `/api/...` |

## Ejecutar
```bash
cargo run                      # desarrollo (crea las tablas que falten al arrancar)
cargo build --release          # producción: target/release/backend-rust
RUST_LOG=debug cargo run       # más logs
```
Al arrancar se ejecuta `CREATE TABLE IF NOT EXISTS` para las 7 tablas; sobre una base ya creada
por Java no cambia nada.

## Probar
```bash
./scripts/smoke_test.sh                                   # requiere curl y jq
BASE=http://localhost:8080/YuweLongo-Backend/api ./scripts/smoke_test.sh
```
Cubre login, rutas públicas/protegidas, y todos los CRUD, ranking y errores (401/403/404/409).

## Estructura
```
src/main.rs            rutas, CORS (tower-http), estado compartido
src/config.rs          variables de entorno
src/error.rs           AppError -> {"error": "..."} y mapeo de errores MySQL (1062/1451/1452)
src/db.rs              pool SQLx y creación de tablas
src/middleware/auth.rs equivalente a SecurityFilter.java (JWT + rutas públicas)
src/models/            structs + mapeo fila->JSON (anidados con LEFT JOIN)
src/handlers/          un archivo por recurso REST
```

## Equivalencias y diferencias con el backend Java
Se conserva a propósito: nombres JSON de los getters Java (`respuestaCoreccta`, `audio_url`,
`imagenUrl`…), propiedades `null` omitidas, 204 cuando Java devolvía `null`/`void`, ranking como
arreglos posicionales, regla de rutas públicas (`path.contains(...)`), rol `INVITADO` => 403,
BCrypt coste 10 con prefijo `$2a$` (el único que entiende jBCrypt 0.4).

Cambios deliberados (todos en rutas de error o por seguridad):
- El JSON de usuario **ya no incluye el hash `contrasena`** (Java lo exponía, también anidado en favoritos y juegos).
- `PUT /api/usuarios` (id en el cuerpo, como Java) y también `PUT /api/usuarios/{id}`; la actualización es **parcial**.
- FK/UNIQUE: 409 o 400 con mensaje claro en vez de 500; los errores de BD no se filtran al cliente.
- Cuerpos inválidos o con campos obligatorios ausentes: 400 con `{"error": ...}` en vez de 500.
- Las excepciones `WebApplicationException` de Java (cuerpo vacío) ahora llevan `{"error": ...}`.
- `PUT` de niveles/juegos sobre un id inexistente da 404 (Java, vía `merge`, insertaba un registro nuevo).

## Pendiente de seguridad (heredado de Java)
- `POST /api/usuarios` es público y acepta `rol` en el cuerpo: cualquiera puede registrarse como `ADMIN`.
- Cualquier `USUARIO` válido puede crear/editar/borrar usuarios, categorías, etc. (solo se valida que el rol sea ADMIN o USUARIO).
- El secreto JWT está en el repositorio Java (`SecurityConstants.java`): conviene rotarlo en producción.
