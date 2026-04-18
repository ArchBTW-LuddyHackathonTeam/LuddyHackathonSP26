.PHONY: run down reset-password

COMPOSE := $(shell command -v docker-compose >/dev/null 2>&1 && echo docker-compose || echo "docker compose")

run:
	$(COMPOSE) up --build

down:
	$(COMPOSE) down

reset-password:
	$(COMPOSE) run --rm api cargo run --release -- --reset-password