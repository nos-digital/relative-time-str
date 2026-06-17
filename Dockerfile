ARG RUST_VERSION=1.96.0
ARG ALPINE_VERSION=3.23

FROM default-route-openshift-image-registry.apps.cluster.chp5-prod.npocloud.nl/nos/rust-builder:${RUST_VERSION}-alpine${ALPINE_VERSION} AS base

ARG AWS_ROLE_ARN
ARG AWS_WEB_IDENTITY_TOKEN_FILE
ARG RUSTC_WRAPPER
ARG SCCACHE_BUCKET
ARG SCCACHE_REGION

ENV AWS_ROLE_ARN="${AWS_ROLE_ARN}"
ENV AWS_WEB_IDENTITY_TOKEN_FILE="${AWS_WEB_IDENTITY_TOKEN_FILE}"
ENV RUSTC_WRAPPER="${RUSTC_WRAPPER}"
ENV SCCACHE_BUCKET="${SCCACHE_BUCKET}"
ENV SCCACHE_REGION="${SCCACHE_REGION}"

FROM base AS builder

WORKDIR /app
COPY ./ /app

RUN cargo build --bins --release --locked

FROM alpine:${ALPINE_VERSION} AS dist

# We dynamic linked libgcc, so install it.
RUN apk add --no-cache libgcc

COPY --from=builder /app/target/release/my_compiled_binary .

EXPOSE 8080

ENTRYPOINT ["/my_compiled_binary"]
