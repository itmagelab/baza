# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.83.0

# BUILD
FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION} AS build
RUN apt-get update \
  && apt-get install --no-install-recommends -y \
  nettle-dev \
  libclang-dev \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN --mount=type=bind,source=src,target=src \
  --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
  --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
  --mount=type=cache,target=/app/target/ \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  cargo build --locked --release && \
  cp ./target/release/baza /bin/baza

# RUN
FROM debian:stable AS final
RUN apt-get update \
  && apt-get install --no-install-recommends -y libssl3=3.0.* ca-certificates=20230311 \
  vim=* \
  && rm -rf /var/lib/apt/lists/*
ARG UID=10001
RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/usr/share/baza" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "${UID}" \
  itmage
USER itmage
COPY --from=build /bin/baza /bin/
WORKDIR /usr/share/baza
CMD ["/bin/baza"]
