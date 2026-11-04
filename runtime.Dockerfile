# Runtime image for repositories on GitHub Actions. CI (nos-github-actions' rust-ci
# workflow) compiles the binaries in nos-common-infra's Wolfi rust-builder image and
# puts them in the build context next to a copy of this file, so building the image
# takes seconds and needs no Rust toolchain or credentials. To build it locally, put
# the release binaries next to a copy of this file and build that directory.
#
# Chainguard's distroless glibc image: glibc, libgcc and CA certificates, but no
# shell, no package manager, and a non-root user. It is built from Wolfi, like the
# CI image, so the binaries match it. Pinned by digest; let Dependabot propose
# updates (see SETUP.md).
#
# Need a C library at runtime (libvips, say)? Install it from Wolfi in an extra stage
# and copy its libraries over. The copied apk database (usr/lib/apk) lists the base
# image's packages plus the new ones, so image scanners still see everything:
#
#   FROM cgr.dev/chainguard/wolfi-base:latest@sha256:<digest> AS libs
#   RUN apk add --no-cache --initdb --root /rootfs \
#       --keys-dir /etc/apk/keys --repositories-file /etc/apk/repositories \
#       libvips
#
#   FROM cgr.dev/chainguard/glibc-dynamic:latest@sha256:<digest>
#   COPY --from=libs /rootfs/usr/lib/ /usr/lib/
#
# glibc makes a malloc arena per thread, up to 8 per core. If memory grows under
# many threads (tokio plus a C library's own threads), cap it:
#
#   ENV MALLOC_ARENA_MAX=2

FROM cgr.dev/chainguard/glibc-dynamic:latest@sha256:671a621016fb33812136dea6182b1db957996d3c8c618187aaf86abd9b14af53

COPY my_compiled_binary /

EXPOSE 8080

ENTRYPOINT ["/my_compiled_binary"]
