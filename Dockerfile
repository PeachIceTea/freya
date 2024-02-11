FROM debian:12.4-slim AS build
WORKDIR /app
SHELL [ "/bin/bash", "-c" ]
RUN apt-get update && apt-get install -y nodejs npm curl pkg-config && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs/ | sh -s -- --default-toolchain=1.70.0 -y

COPY . .
RUN source $HOME/.cargo/env && cargo build --release

FROM debian:12.4-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y libssl3 ffmpeg
COPY --from=build /app/target/release/freya .
ENV PORT=80
ENV DEFAULT_DIRECTORY=/media
ENV DATABASE_PATH=/db/freya.db
CMD [ "./freya" ]