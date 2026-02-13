# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.92.0

FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION} AS build
RUN apt-get update \
  && apt-get install --no-install-recommends -y \
  nettle-dev \
  libclang-dev \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN --mount=type=bind,source=crates,target=crates \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  cargo build --locked --release && \
  cp ./target/release/baza /bin/baza

FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION} AS web
WORKDIR /app
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown
RUN --mount=type=bind,source=crates,target=crates,rw \
  --mount=type=bind,source=.cargo,target=.cargo \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Trunk.toml,target=Trunk.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  trunk build --release --config Trunk.toml --dist /usr/share/baza

FROM caddy:2-alpine AS caddy

FROM debian:stable AS final
RUN apt-get update \
  && apt-get install --no-install-recommends -y libssl3 ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=caddy /usr/bin/caddy /usr/bin/caddy

COPY --from=build /bin/baza /bin/
COPY --from=web /usr/share/baza /usr/share/baza

COPY Caddyfile /etc/caddy/Caddyfile
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD []
