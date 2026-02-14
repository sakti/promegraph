FROM rust:1.93-trixie AS build

RUN apt-get install -y pkg-config libssl-dev libfontconfig libfontconfig1-dev
COPY . .
RUN cargo install --path .

FROM gcr.io/distroless/cc-debian13 AS runtime
# RUN apt-get update && apt-get install -y libssl3
LABEL org.opencontainers.image.description promegraph
COPY --from=build /usr/local/cargo/bin/promegraph /usr/local/bin/promegraph

ENTRYPOINT [ "/usr/local/bin/promegraph" ]
