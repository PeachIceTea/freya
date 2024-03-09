#!/bin/bash

# Function to handle SIGINT (Ctrl+C)
handle_sigint() {
    kill $CARGO_PID $NPM_PID
}

# Register the function to handle SIGINT
trap handle_sigint SIGINT

# Change to the script's directory
cd "$(dirname "$0")"

# Run cargo watch -x run
cargo watch -x run &
CARGO_PID=$!

# Change to the web subdirectory
cd web

# Run npm start
npm start &
NPM_PID=$!

# Wait for both processes to finish
wait $CARGO_PID $NPM_PID