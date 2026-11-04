# Runtime image only: CI (nos-github-actions' rust-ci) compiles the binaries and
# copies them in. See "Container Image" in ARCHITECTURE.md.

FROM cgr.dev/chainguard/glibc-dynamic:latest@sha256:671a621016fb33812136dea6182b1db957996d3c8c618187aaf86abd9b14af53

COPY my_compiled_binary /

EXPOSE 8080

ENTRYPOINT ["/my_compiled_binary"]
