ENV_FILE := .env.dev
COMPOSE  := docker compose --env-file $(ENV_FILE) -f docker-compose.dev.yml

.PHONY: up down restart ps logs psql

up:
	$(COMPOSE) up -d

down:
	$(COMPOSE) down

restart:
	$(COMPOSE) restart

ps:
	$(COMPOSE) ps

logs:
	$(COMPOSE) logs -f $(s)

psql:
	docker exec -it velora-postgres-1 psql -U velora -d $(or $(db),velora_auth)
