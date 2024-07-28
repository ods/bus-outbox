FROM rust:1.80.0-alpine AS builder
RUN apk add --no-cache musl-dev

WORKDIR /build

RUN --mount=type=bind,source=src,target=/build/src \
    --mount=type=bind,source=Cargo.toml,target=/build/Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=/build/Cargo.lock \
    --mount=type=bind,source=migrations,target=/build/migrations \
    --mount=type=cache,target=${HOME}/.cargo \
    --mount=type=cache,target=/build/target \
    cargo build --release \
    && cp target/release/bus-producer /bus-producer


FROM scratch

COPY --from=builder /bus-producer /bus-producer
COPY migrations /migrations

ENTRYPOINT [ "/bus-producer" ]
