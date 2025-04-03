ARG RUST_VERSION=1.86.0
ARG ALPINE_VERSION=3.21

FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS base

RUN apk add --no-cache \
  # musl-dev is needed so the rust compiler can link against musl's libc implementation
  musl-dev

# Adding this compiler feature disables static linking to libgcc. This is
# required, as static linking against the musl lib has poorer performance.
ENV RUSTFLAGS="-C target-feature=-crt-static"

FROM base AS builder

WORKDIR /app
COPY ./ /app

RUN cargo build --release --locked

FROM alpine:${ALPINE_VERSION} AS dist

# We dynamic linked libgcc, so install it.
RUN apk add --no-cache libgcc

COPY --from=builder /app/target/release/my_compiled_binary .

EXPOSE 8080

ENTRYPOINT ["/my_compiled_binary"]
