FROM rust:alpine3.22 AS builder

# Install necessary build tools and headers
RUN apk add --no-cache \
    musl-dev \
    gcc \
    make \
    libc-dev \
    linux-headers \
    openssl-dev \
    pkgconf

WORKDIR /app
RUN cargo init --bin
COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN cargo build --release
RUN rm src/*.rs
COPY ./src ./src
RUN cargo build --release


FROM alpine:3.22

RUN apk add --no-cache \
    chromium \
    nss \
    freetype \
    harfbuzz \
    ttf-freefont \
    dumb-init

ENV CHROME_BIN=/usr/bin/chromium-browser

WORKDIR /app
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
COPY --from=builder /app/target/release/rebot .
RUN chown -R appuser:appgroup /app

USER appuser

ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/app/rebot"]
