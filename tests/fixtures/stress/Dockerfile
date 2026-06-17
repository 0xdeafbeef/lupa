# syntax=docker/dockerfile:1.7
FROM rust:1.82 AS planner
ARG PROFILE=release
ENV CARGO_TERM_COLOR=always
RUN --mount=type=cache,target=/usr/local/cargo/registry <<'SH'
set -eu
cargo --version
SH
FROM scratch AS export
COPY --from=planner /usr/local/cargo/bin/cargo /cargo
ENTRYPOINT ["/cargo"]
