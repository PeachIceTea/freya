# Justfile for Fela development
# Run `just` for help on available commands


# Default task: run backend and frontend in parallel
[parallel]
dev: server web

# Backend: watch Rust source and run cargo
server:
    watchexec -r -e rs,toml,env,sql -i web -i target -- cargo run

# Frontend: run npm dev server (no watching needed - server handles it)
web: web-install
    npm start

# Install npm dependencies
web-install:
    npm install

# Build the frontend in release mode
build-web:
    npm run build

# Full release build (includes frontend)
release:
    cargo build --release

# Database management
db-reset:
    rm -f fela.db
    cargo run

# Format
fmt:
    cargo fmt
    npm run prettier

# Lint
lint:
    cargo clippy
    npm run lint

# Test all
test:
    cargo test
    npm run test

# Clean up
clean:
    rm -rf target
    rm -rf node_modules
