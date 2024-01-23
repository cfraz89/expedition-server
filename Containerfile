FROM rustlang/rust:nightly as builder

WORKDIR /usr/src/expedition
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
RUN cargo install --path .

# FROM debian:bookworm-slim
# RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/expedition-server /usr/local/bin/expedition-server

CMD ["expedition-server"]