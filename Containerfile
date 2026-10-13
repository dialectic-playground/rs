# Toolchain image for dialectic-playground/rs: Rust (container-first; see tools/dp-run).
#
# Rust 1.99; build and package tool: cargo.
# Official base image pinned by multi-architecture index digest, plus make and python3 (for the
# curriculum's black-box runner). Dependabot (docker) refreshes the digest; moving to a new
# release line is a deliberate change (keep `make toolchain` and the native CI in step).
FROM docker.io/library/rust:1.99-slim-trixie@sha256:2752b332db73fdbb7dc576f06c82ed1f312005784ef913d7e04a28f5f55dc581

RUN apt-get update \
    && apt-get install -y --no-install-recommends make python3 \
    && rm -rf /var/lib/apt/lists/*

# Cargo's registry and build cache in the mounted cache (the image's CARGO_HOME is read-only).
ENV CARGO_HOME=/dp-cache/cargo \
    DP_IN_CONTAINER=1
