FROM alpine:3.19.1 as builder
RUN apk add --no-cache build-base openssl-dev nodejs npm rustup && \
    rustup-init -y --default-toolchain 1.76.0-x86_64-unknown-linux-musl

# Setup environment.
WORKDIR /freya
ENV PATH="/root/.cargo/bin:${PATH}" \
    OPENSSL_LIB_DIR=/usr/lib \
    OPENSSL_INCLUDE_DIR=/usr/include

# Prebuild dependencies to cache them
RUN cargo init .
COPY Cargo.toml Cargo.lock ./
COPY migrate ./migrate
RUN cargo build --target x86_64-unknown-linux-musl --release && \
    rm src/*.rs

# Build actual project
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.19.1 as runtime
RUN apk add --no-cache ffmpeg

WORKDIR /freya
COPY --from=builder /freya/target/x86_64-unknown-linux-musl/release/freya /freya/freya

RUN mkdir -p /media /data && \
    addgroup -S freya -g 8000 && \
    adduser -S freya -u 8000 -G freya && \
    chown -R freya:freya /freya /media /data

USER freya

VOLUME /media
VOLUME /data

ENV PORT=80
ENV DEFAULT_DIRECTORY=/media
ENV DATABASE_PATH=/data/freya.db
ENV COOKIE_ONLY_OVER_HTTPS=true

EXPOSE $PORT
CMD /freya/freya