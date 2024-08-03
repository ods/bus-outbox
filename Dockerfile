FROM rust:1.80.0-alpine AS builder
RUN apk add --no-cache musl-dev bash g++ make

WORKDIR /build
ENV CARGO_HOME=${HOME}/.cargo

RUN --mount=type=bind,source=src,target=/build/src \
    --mount=type=bind,source=Cargo.toml,target=/build/Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=/build/Cargo.lock \
    --mount=type=bind,source=migrations,target=/build/migrations \
    --mount=type=bind,source=.sqlx,target=/build/.sqlx \
    --mount=type=cache,target=${HOME}/.cargo \
    --mount=type=cache,target=/build/target \
    cargo build --release \
    && cp target/release/bus-outbox /bus-outbox


FROM scratch

COPY --from=builder /bus-outbox /bus-outbox
COPY migrations /migrations

ENTRYPOINT [ "/bus-outbox" ]
