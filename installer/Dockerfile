# syntax=docker/dockerfile:1.7

FROM rust:1.95-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p django-bakery

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    git ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /bin/bash baker
COPY --from=builder /src/target/release/django-bakery /usr/local/bin/django-bakery
USER baker
WORKDIR /work
ENTRYPOINT ["django-bakery"]
CMD ["new"]
