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

FROM rust:latest

# Build the server

FROM python:3.10
ENV PATH="~/.local/bin:${PATH}"

# copy the server binary into the container
COPY --from=builder /app/target/release/godata_server /root/.local/bin/
#install poetry
RUN echo "INSTALLING POETRY"
RUN curl -sSL https://install.python-poetry.org | python3 -

# copy the python code into the container
COPY pyproject.toml /app/pyproject.toml
COPY poetry.lock /app/poetry.lock
COPY README.rst /app/README.rst
COPY godata /app/godata


WORKDIR /app
#install dependencies
RUN ~/.local/bin/poetry install --with test

COPY ./tests /app/tests
RUN mv /app/tests/run_tests.sh /app/run_tests.sh
RUN chmod +x run_tests.sh
CMD ["./run_tests.sh"]
