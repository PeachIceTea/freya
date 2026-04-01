FROM alpine:3.23.3 as builder
RUN apk add --no-cache build-base openssl-dev nodejs npm rustup && \
    rustup-init -y --default-toolchain 1.94.1-x86_64-unknown-linux-musl

# Setup environment.
WORKDIR /fela
ENV PATH="/root/.cargo/bin:${PATH}" \
    OPENSSL_LIB_DIR=/usr/lib \
    OPENSSL_INCLUDE_DIR=/usr/include

# Prebuild dependencies to cache them
RUN mkdir -p server && echo 'fn main() {}' > server/main.rs
COPY Cargo.toml Cargo.lock ./
RUN cargo build --target x86_64-unknown-linux-musl --release && \
    rm server/*.rs

# Install npm dependencies (cached until lockfile changes)
COPY package.json package-lock.json ./
RUN npm install

# Build actual project
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.23.3 as runtime
RUN apk add --no-cache ffmpeg

WORKDIR /app
COPY --from=builder /fela/target/x86_64-unknown-linux-musl/release/fela /app/fela

RUN mkdir -p /app/data/media && \
    addgroup -S fela -g 8000 && \
    adduser -S fela -u 8000 -G fela && \
    chown -R fela:fela /app

USER fela

VOLUME /app/data
ENV FELA_DATA_DIRECTORY=/app/data/media
ENV FELA_DATABASE_PATH=/app/freya.db

CMD /app/fela
