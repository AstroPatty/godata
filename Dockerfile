FROM rust:latest

# Build the server
COPY src /app/src
COPY Cargo.toml /app/Cargo.toml
RUN cd /app && cargo build --release


FROM python:3.10
ENV PATH="~/.local/bin:${PATH}"

# copy the server binary into the container
COPY --from=0 /app/target/release/godata_server /bin

# copy the python code into the container
COPY godata /app/godata
COPY pyproject.toml /app/pyproject.toml
COPY poetry.lock /app/poetry.lock
COPY README.md /app/README.md

#install poetry and dependencies
RUN curl -sSL https://install.python-poetry.org | python3 -
WORKDIR /app
RUN ~/.local/bin/poetry install

COPY ./tests /app/tests
RUN mv /app/tests/run_tests.sh /app/run_tests.sh
RUN chmod +x run_tests.sh
CMD ["./run_tests.sh"]



