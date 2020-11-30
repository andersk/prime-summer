FROM rust:slim as builder
RUN apt-get -q update && apt-get -qy install libprimesieve-dev m4 make && rm -rf /var/lib/apt/lists/*
WORKDIR /prime-summer
COPY Cargo.lock Cargo.toml ./
COPY src/ src/
RUN cargo install --path .
RUN mkdir /deps
RUN ldd /usr/local/cargo/bin/prime-summer | sed -n 's,.* => \(/\S*\).*,\1,p' | xargs cp --parents -t /deps

FROM busybox:glibc
COPY --from=builder /usr/local/cargo/bin/prime-summer /usr/local/bin/prime-summer
COPY --from=builder /deps /
CMD ["prime-summer"]
