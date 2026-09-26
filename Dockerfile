# ————— Build stage —————
FROM rust:1.98-slim AS builder

WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Build
COPY src ./src
COPY templates ./templates
COPY content ./content
COPY static ./static
COPY build.rs ./

RUN touch src/main.rs && cargo build --release

# ————— Runtime stage —————
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/portfolio /app/portfolio
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/content /app/content
COPY --from=builder /app/static /app/static

EXPOSE 3000

ENV PORT=3000

CMD ["/app/portfolio"]
