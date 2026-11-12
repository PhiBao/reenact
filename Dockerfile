# Case-file server + web UI in one small image.
FROM node:22-slim AS web
RUN corepack enable
WORKDIR /w
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY web ./
RUN pnpm build

FROM rust:1-slim-bookworm AS build
WORKDIR /b
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN cargo build --release -p reenact-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /b/target/release/reenact-server /usr/local/bin/reenact-server
COPY --from=web /w/dist /app/web
ENV REENACT_WEB_DIR=/app/web REENACT_CASES_DIR=/data/cases REENACT_BENCHMARK_FILE=/data/benchmark.json PORT=8080
EXPOSE 8080
CMD ["reenact-server"]
