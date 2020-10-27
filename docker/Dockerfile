FROM rust
RUN apt-get update -yq \
    && apt-get install -yqq --no-install-recommends \
        libgfortran4 \
        libblas3 \
        liblapack3 \
        libgcc1 \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-tarpaulin -f \
    && cargo install cargo-suity -f \
    && rm -rf /usr/local/cargo/registry/cache/*

