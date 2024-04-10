FROM lukemathwalker/cargo-chef:latest-rust-1 as chef
WORKDIR /app

from chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


from chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin godata_server

FROM debian:bookworm-slim AS runtime 
WORKDIR /app

RUN apt-get update -y && apt-get install -y --no-install-recommends openssl ca-certificates

COPY --from=builder /app/target/release/godata_server /app/godata_server

CMD ["/app/godata_server", "--port", "8080"]

