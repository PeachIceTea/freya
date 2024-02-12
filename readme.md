# Freya - Audiobook server

## Build

There is a Dockerfile available to build the project in a container. This is the recommended way to build the project.

1. To build the project, you will need the following:

   - [Node.js](https://nodejs.org/en/) (tested with `v21.6.1`)
   - [Rust](https://www.rust-lang.org/tools/install) (tested with `rustc 1.76.0 (07dca489a 2024-02-04)`)

2. Clone the repository and navigate to the project directory.

3. Run the following command to build:

   ```bash
   cargo build --release
   ```

   This should install all the required dependencies and build both the frontend and backend.

## Run

1. To run the project, you will need the following dependencies:

   - [FFmpeg](https://ffmpeg.org/download.html) (tested with `n6.1.1`)

2. Set the following environment variables:
   - `DATABASE_PATH`: The URL to the database, e.g. `sqlite://db`. (default: `./freya.db`)
   - `NO_MIGRATE`: Set to not run migrations on startup. (default: `false`)
   - `PORT`: The port to run the server on. (default: `3000`)
   - `SESSION_LIFETIME`: The lifetime of a session in hours. (default: `720` which equates to 30 days)
   - `DEFAULT_DIRECTORY`: The path that the file select dialog will open to. (default: `C:\` on Windows, `/` on Unix)

This can either be done by creating a `.env` file in the root of the project or by setting the environment variables manually.

3. Then simply run the binary you built. The frontend is bundled into the binary and will be served unless the client hits an API endpoint.
