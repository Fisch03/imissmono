FROM rustlang/rust:nightly-slim AS rust

RUN cargo install cargo-chef 
WORKDIR /usr/src/imissmono

FROM rust AS plan
# prepare deps
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust AS build-server
# compile/cache deps
COPY --from=plan /usr/src/imissmono/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
# compile server
RUN cargo build --bin imissmono --release

FROM debian:bookworm-slim AS runtime
WORKDIR /imissmono
COPY --from=build-server /usr/src/imissmono/target/release/imissmono imissmono
COPY --from=build-server /usr/src/imissmono/static static

EXPOSE 8080
CMD ["./imissmono"]


