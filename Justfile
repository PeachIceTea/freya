# Justfile for Freya development
# Run `just` for help on available commands

# Default task: run backend and frontend in parallel
[parallel]
dev: backend frontend

# Backend: watch Rust source and run cargo
backend:
    watchexec -r -e rs,toml,env,sql -i web -i target -- cargo run

# Frontend: run npm dev server (no watching needed - server handles it)
frontend:
    cd web && npm start

# Build the frontend in release mode
build-web:
    cd web && npm run build

# Full release build (includes frontend)
release: build-web
    cargo build --release

# Database management
db-reset:
    rm -f freya.db
    cargo run

# Lint and format
fmt:
    cargo fmt
    cd web && npm run prettier

lint:
    cargo clippy
    cd web && npm run lint

# Test all
test:
    cargo test

# Clean up
clean:
    rm -rf target
    cd web && rm -rf node_modules dist
