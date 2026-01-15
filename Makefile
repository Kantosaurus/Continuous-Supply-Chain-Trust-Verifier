# =============================================================================
# SCTV Makefile
# =============================================================================
# Convenience commands for development, building, and deployment.
#
# Usage:
#   make help           - Show available commands
#   make dev            - Start development environment
#   make build          - Build Docker images
#   make prod           - Start production environment
# =============================================================================

.PHONY: help dev dev-down build build-api build-worker build-cli build-dashboard \
        prod prod-down logs clean test migrate db-shell

# Default target
.DEFAULT_GOAL := help

# Colors for output
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RESET := \033[0m

# =============================================================================
# Help
# =============================================================================

help: ## Show this help message
	@echo ""
	@echo "$(CYAN)SCTV - Supply Chain Trust Verifier$(RESET)"
	@echo ""
	@echo "$(GREEN)Available commands:$(RESET)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'
	@echo ""

# =============================================================================
# Development
# =============================================================================

dev: ## Start development environment
	docker-compose up -d
	@echo "$(GREEN)Development environment started$(RESET)"
	@echo "  API:       http://localhost:3000"
	@echo "  GraphQL:   http://localhost:3000/graphql"
	@echo "  Postgres:  localhost:5432"

dev-down: ## Stop development environment
	docker-compose down
	@echo "$(GREEN)Development environment stopped$(RESET)"

dev-logs: ## Show development logs (follow mode)
	docker-compose logs -f

dev-restart: dev-down dev ## Restart development environment

# =============================================================================
# Building
# =============================================================================

build: build-api build-worker ## Build all Docker images

build-api: ## Build API server image
	docker build -f docker/api.Dockerfile -t sctv-api:latest .

build-worker: ## Build worker image
	docker build -f docker/worker.Dockerfile -t sctv-worker:latest .

build-cli: ## Build CLI image
	docker build -f docker/Dockerfile --target sctv-cli -t sctv-cli:latest .

build-dashboard: ## Build dashboard image
	docker build -f docker/Dockerfile --target sctv-dashboard -t sctv-dashboard:latest .

build-all: build-api build-worker build-cli build-dashboard ## Build all images including CLI and dashboard

build-no-cache: ## Build all images without cache
	docker build -f docker/api.Dockerfile --no-cache -t sctv-api:latest .
	docker build -f docker/worker.Dockerfile --no-cache -t sctv-worker:latest .

# =============================================================================
# Production
# =============================================================================

prod: ## Start production environment
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
	@echo "$(GREEN)Production environment started$(RESET)"

prod-down: ## Stop production environment
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml down

prod-logs: ## Show production logs (follow mode)
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml logs -f

prod-scale-workers: ## Scale workers (usage: make prod-scale-workers N=3)
	docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --scale worker=$(N)

# =============================================================================
# Database
# =============================================================================

db-shell: ## Open PostgreSQL shell
	docker-compose exec postgres psql -U sctv -d sctv

db-migrate: ## Run database migrations
	docker-compose exec api sqlx migrate run

db-reset: ## Reset database (WARNING: destroys all data)
	@echo "$(YELLOW)WARNING: This will destroy all data!$(RESET)"
	@read -p "Are you sure? [y/N] " confirm && [ "$$confirm" = "y" ]
	docker-compose down -v
	docker-compose up -d postgres
	@sleep 5
	docker-compose up -d api

# =============================================================================
# Logs and Debugging
# =============================================================================

logs: ## Show all logs
	docker-compose logs

logs-api: ## Show API logs
	docker-compose logs -f api

logs-worker: ## Show worker logs
	docker-compose logs -f worker

logs-db: ## Show database logs
	docker-compose logs -f postgres

ps: ## Show running containers
	docker-compose ps

# =============================================================================
# Testing and Validation
# =============================================================================

test: ## Run tests locally
	cargo test --workspace

test-docker: ## Run tests in Docker
	docker-compose run --rm api cargo test --workspace

lint: ## Run linter
	cargo clippy --workspace -- -D warnings

fmt: ## Format code
	cargo fmt --all

check: fmt lint test ## Run all checks (format, lint, test)

# =============================================================================
# Cleanup
# =============================================================================

clean: ## Remove all containers and volumes
	docker-compose down -v --remove-orphans
	docker system prune -f

clean-images: ## Remove built images
	docker rmi sctv-api:latest sctv-worker:latest sctv-cli:latest sctv-dashboard:latest 2>/dev/null || true

clean-all: clean clean-images ## Remove everything (containers, volumes, images)
	@echo "$(GREEN)Cleanup complete$(RESET)"

# =============================================================================
# Local Development (without Docker)
# =============================================================================

run-api: ## Run API server locally
	RUST_LOG=debug cargo run --bin sctv-api

run-worker: ## Run worker locally
	RUST_LOG=debug cargo run --bin sctv-worker

run-cli: ## Run CLI locally
	cargo run --bin sctv -- --help

# =============================================================================
# Release
# =============================================================================

release-build: ## Build release binaries
	cargo build --release --workspace

release-docker: ## Build and tag release Docker images
	docker build -f docker/api.Dockerfile -t sctv-api:$(VERSION) .
	docker build -f docker/worker.Dockerfile -t sctv-worker:$(VERSION) .
	docker tag sctv-api:$(VERSION) sctv-api:latest
	docker tag sctv-worker:$(VERSION) sctv-worker:latest
