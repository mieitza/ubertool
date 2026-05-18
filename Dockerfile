# syntax=docker/dockerfile:1.7
# Build stage: compile static musl binary.
FROM rust:1.81-alpine AS build
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Pre-fetch deps for cache reuse.
RUN cargo fetch

# Static musl build with full release profile from Cargo.toml.
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage: minimal alpine.
FROM alpine:3.20 AS runtime
RUN apk add --no-cache ca-certificates && adduser -D -u 1000 app
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/ubertool /usr/local/bin/ubertool
USER app
ENTRYPOINT ["ubertool"]
CMD ["--help"]
