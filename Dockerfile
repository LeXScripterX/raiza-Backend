# Etapa 1: Compilación
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /app

# Copiar manifiestos primero para aprovechar caché de dependencias
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copiar el código fuente real y compilar
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Etapa 2: Imagen final ultra ligera
FROM alpine:3.20
RUN apk add --no-cache ca-certificates libgcc
WORKDIR /app

# REEMPLAZA "backend-rust" por el valor de name en tu Cargo.toml
COPY --from=builder /app/target/release/backend-rust /app/server

EXPOSE 8080
CMD ["/app/server"]
