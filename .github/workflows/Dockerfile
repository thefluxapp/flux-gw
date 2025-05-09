FROM rust:alpine AS builder

WORKDIR /app

COPY . .
RUN apk --no-cache add musl-dev protoc protobuf-dev
RUN cargo build --release -p flux-gw

FROM alpine

WORKDIR /app

COPY --from=builder /app/target/release/flux-gw /app/
COPY --from=builder /app/settings/default.toml /app/
