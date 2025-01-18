FROM rust:1.84-bookworm AS build

RUN apt-get install -y pkg-config libssl-dev libfontconfig libfontconfig1-dev
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y libssl3
COPY --from=build /usr/local/cargo/bin/promegraph /usr/local/bin/promegraph

ENTRYPOINT [ "/usr/local/bin/promegraph" ]
