FROM rust:latest

# Build the server
COPY src /app/src
COPY Cargo.toml /app/Cargo.toml
RUN cd /app && cargo build --release


FROM python:3.10
ENV PATH="~/.local/bin:${PATH}"

# copy the server binary into the container
COPY --from=0 /app/target/release/godata_server /root/.local/bin/
#install poetry
RUN curl -sSL https://install.python-poetry.org | python3 -

# copy the python code into the container
COPY pyproject.toml /app/pyproject.toml
COPY poetry.lock /app/poetry.lock
COPY README.md /app/README.md
COPY godata /app/godata


WORKDIR /app
#install dependencies
RUN ~/.local/bin/poetry install --with test

COPY ./tests /app/tests
RUN mv /app/tests/run_tests.sh /app/run_tests.sh
RUN chmod +x run_tests.sh
CMD ["./run_tests.sh"]



