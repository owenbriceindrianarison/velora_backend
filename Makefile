ENV_FILE := .env.dev
COMPOSE  := docker compose --env-file $(ENV_FILE) -f docker-compose.dev.yml
include $(ENV_FILE)

# Microservices with their own database (services/<name> -> velora_<name>)
SERVICES := auth user catalog media playback billing

# Auto-detect whether `make` runs inside the devcontainer (reach postgres via
# the compose network) or on the host (reach it via the published port).
DB_HOST := $(shell [ -f /.dockerenv ] && echo postgres || echo localhost)
DB_PORT := $(shell [ -f /.dockerenv ] && echo 5432 || echo $(POSTGRES_PORT))

# service -> database name (the "user" service's database is plural: velora_users)
ifeq ($(service),user)
DB_NAME := velora_users
else
DB_NAME := velora_$(service)
endif

.PHONY: up down restart ps logs psql \
	migrate-add migrate-up migrate-down migrate-info migrate-up-all

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

# --- Migrations (per service) -----------------------------------
# Usage:
#   make migrate-add  service=auth name=create_users
#   make migrate-up    service=auth
#   make migrate-down  service=auth
#   make migrate-info  service=auth
#   make migrate-up-all   (runs pending migrations for every service)

migrate-add:
	@test -n "$(service)" -a -n "$(name)" || { echo "usage: make migrate-add service=<name> name=<migration_name>"; exit 1; }
	cd services/$(service) && sqlx migrate add -r $(name)

migrate-up:
	@test -n "$(service)" || { echo "usage: make migrate-up service=<name>"; exit 1; }
	cd services/$(service) && sqlx migrate run --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)"

migrate-down:
	@test -n "$(service)" || { echo "usage: make migrate-down service=<name>"; exit 1; }
	cd services/$(service) && sqlx migrate revert --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)"

migrate-info:
	@test -n "$(service)" || { echo "usage: make migrate-info service=<name>"; exit 1; }
	cd services/$(service) && sqlx migrate info --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)"

migrate-up-all:
	@for s in $(SERVICES); do \
		if [ -d services/$$s/migrations ]; then \
			db=velora_$$s; [ "$$s" = "user" ] && db=velora_users; \
			echo ">>> migrating $$s ($$db)"; \
			(cd services/$$s && sqlx migrate run --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/$$db") || exit 1; \
		fi; \
	done
