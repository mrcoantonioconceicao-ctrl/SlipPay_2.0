# Build stage
FROM rust:1.82-slim as builder

WORKDIR /app

# Dependências do sistema
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copia arquivos
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Compila em release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Dependências runtime
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copia binário compilado
COPY --from=builder /app/target/release/slippay_2_0 .

# Expõe porta
EXPOSE 3000

# Inicia servidor
CMD ["./slippay_2_0"]
