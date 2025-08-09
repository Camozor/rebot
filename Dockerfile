FROM rust:alpine3.22 AS builder

RUN apk add --no-cache \
    musl-dev \
    gcc \
    make \
	cmake \
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
	opus \
    nss \
    freetype \
    harfbuzz \
    ttf-freefont \
    dumb-init \
	su-exec

ENV CHROME_BIN=/usr/bin/chromium-browser

WORKDIR /app
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
COPY --from=builder /app/target/release/rebot .
RUN chown -R appuser:appgroup /app

COPY audio audio

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh


ENTRYPOINT ["/usr/bin/dumb-init", "--", "/entrypoint.sh"]
CMD ["/app/rebot"]
