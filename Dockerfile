FROM lukemathwalker/cargo-chef:latest-rust-1.91.0 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

# Builder Stage
# We use the latest Rust stable release as base image
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the saame,
# all layers should be cached.
COPY . .
ENV SQLX_OFFLINE=true

# Build binary!
# We'll use the release profile to make it fast!
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  # Clean Up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/list/*

# Copy the compiled binary from the builder environment
COPY --from=builder /app/target/release/zero2prod zero2prod
# We need the configuration file at runtime!
COPY configuration configuration
ENV APP_ENVIRONMENT=production

# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./zero2prod"]
