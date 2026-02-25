# We use the latest Rust stable release as base image
FROM rust:1.91.1

# Let's switch out working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does 
# not exist already
WORKDIR /app

# Install the required system dependencies of our linking configuration
RUN apt update && apt install lld clang -y

# Copy all file from our working environment to out Docker image
COPY . .

ENV SQLX_OFFLINE true

# Let's build out binary!
# We'll use the release profile to make it fast!
RUN cargo build --release

# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./target/release/zero2prod"]
