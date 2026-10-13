# Toolchain image for dialectic-playground/rs: Rust (container-first; see tools/dp-run).
#
# Rust 1.99; build and package tool: cargo; lint: clippy; format: rustfmt.
# Official base image pinned by multi-architecture index digest, plus make and python3 (for the
# curriculum's black-box runner). Dependabot (docker) refreshes the digest; moving to a new
# release line is a deliberate change (keep `make toolchain` and the native CI in step).
FROM docker.io/library/rust:1.99-slim-trixie@sha256:2752b332db73fdbb7dc576f06c82ed1f312005784ef913d7e04a28f5f55dc581

RUN apt-get update \
    && apt-get install -y --no-install-recommends make python3 \
    && rm -rf /var/lib/apt/lists/*

# Lint and format tools (the slim image ships the minimal rustup profile). The components go
# to the image's RUSTUP_HOME (/usr/local/rustup), and their rustup proxies (cargo-clippy,
# cargo-fmt, rustfmt) already sit in /usr/local/cargo/bin on PATH, so they keep working after
# CARGO_HOME is relocated to the cache below.
RUN rustup component add clippy rustfmt \
    && cargo clippy --version && cargo fmt --version

# Cargo's registry and build cache in the mounted cache (the image's CARGO_HOME is read-only).
ENV CARGO_HOME=/dp-cache/cargo \
    DP_IN_CONTAINER=1
