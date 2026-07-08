ENV_FILE := .env.dev
COMPOSE  := docker compose --env-file $(ENV_FILE) -f docker-compose.dev.yml
include $(ENV_FILE)

# Microservices with their own database (services/<name> -> velora_<name>)
SERVICES := auth user catalog media playback billing

# Auto-detect whether `make` runs inside the devcontainer or on the host.
# Inside: reach postgres via the compose network, run sqlx directly.
# Outside: reach postgres via the published port, run sqlx via the workspace container.
# Auto-detect whether `make` runs inside the devcontainer or on the host.
IN_DOCKER := $(shell [ -f /.dockerenv ] && echo yes || echo no)

# DB_HOST/DB_PORT for direct sqlx calls (inside container uses compose network,
# host uses the published port).
DB_HOST := $(shell [ -f /.dockerenv ] && echo postgres || echo localhost)
DB_PORT := $(shell [ -f /.dockerenv ] && echo 5432 || echo $(POSTGRES_PORT))

# When running via `docker exec` from the host, the command executes inside
# the workspace container, so it must always reach postgres via the compose network.
EXEC_DB_HOST := postgres
EXEC_DB_PORT := 5432

# service -> database name (the "user" service's database is plural: velora_users)
ifeq ($(service),user)
DB_NAME := velora_users
else
DB_NAME := velora_$(service)
endif

.PHONY: up down restart ps logs psql \
	migrate-add migrate-up migrate-down migrate-info migrate-up-all \
	test test-all \
	redis nats-streams nats-info nats-view nats-get nats-consumers nats-sub nats-pub

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

redis:
	docker exec -it velora-redis-1 redis-cli

# --- NATS JetStream -------------------------------------------
# Usage:
#   make nats-streams                       (liste tous les streams)
#   make nats-info    stream=<name>         (détails d'un stream)
#   make nats-view    stream=<name>         (parcourir les messages)
#   make nats-get     stream=<name> seq=<n> (message par numéro de séquence)
#   make nats-consumers stream=<name>       (liste les consumers)
#   make nats-sub     subject=<subject>     (écouter un subject en live)
#   make nats-pub     subject=<subject> msg=<json> (publier un message test)

NATS := nats --server nats://localhost:$(NATS_CLIENT_PORT)

nats-streams:
	$(NATS) stream ls

nats-info:
	@test -n "$(stream)" || { echo "usage: make nats-info stream=<name>"; exit 1; }
	$(NATS) stream info $(stream)

nats-view:
	@test -n "$(stream)" || { echo "usage: make nats-view stream=<name>"; exit 1; }
	$(NATS) stream view $(stream)

nats-get:
	@test -n "$(stream)" -a -n "$(seq)" || { echo "usage: make nats-get stream=<name> seq=<n>"; exit 1; }
	$(NATS) stream get $(stream) $(seq)

nats-consumers:
	@test -n "$(stream)" || { echo "usage: make nats-consumers stream=<name>"; exit 1; }
	$(NATS) consumer ls $(stream)

nats-sub:
	@test -n "$(subject)" || { echo "usage: make nats-sub subject=<subject>"; exit 1; }
	$(NATS) subscribe $(subject)

nats-pub:
	@test -n "$(subject)" -a -n "$(msg)" || { echo "usage: make nats-pub subject=<subject> msg='<json>'"; exit 1; }
	$(NATS) publish $(subject) '$(msg)'

# --- Migrations (per service) -----------------------------------
# Usage:
#   make migrate-add  service=auth name=create_users
#   make migrate-up    service=auth
#   make migrate-down  service=auth
#   make migrate-info  service=auth
#   make migrate-up-all   (runs pending migrations for every service)

migrate-add:
	@test -n "$(service)" -a -n "$(name)" || { echo "usage: make migrate-add service=<name> name=<migration_name>"; exit 1; }
ifeq ($(IN_DOCKER),yes)
	cd services/$(service) && sqlx migrate add -r $(name)
else
	docker exec -w /app/services/$(service) velora-workspace-1 sqlx migrate add -r $(name)
endif

migrate-up:
	@test -n "$(service)" || { echo "usage: make migrate-up service=<name>"; exit 1; }
ifeq ($(IN_DOCKER),yes)
	cd services/$(service) && sqlx migrate run --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)"
else
	docker exec -w /app/services/$(service) velora-workspace-1 sqlx migrate run --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(EXEC_DB_HOST):$(EXEC_DB_PORT)/$(DB_NAME)"
endif

migrate-down:
	@test -n "$(service)" || { echo "usage: make migrate-down service=<name>"; exit 1; }
ifeq ($(IN_DOCKER),yes)
	cd services/$(service) && sqlx migrate revert --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)"
else
	docker exec -w /app/services/$(service) velora-workspace-1 sqlx migrate revert --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(EXEC_DB_HOST):$(EXEC_DB_PORT)/$(DB_NAME)"
endif

migrate-info:
	@test -n "$(service)" || { echo "usage: make migrate-info service=<name>"; exit 1; }
ifeq ($(IN_DOCKER),yes)
	cd services/$(service) && sqlx migrate info --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)"
else
	docker exec -w /app/services/$(service) velora-workspace-1 sqlx migrate info --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(EXEC_DB_HOST):$(EXEC_DB_PORT)/$(DB_NAME)"
endif

# --- Tests -----------------------------------
# Usage:
#   make test service=auth      (tests d'un service spécifique)
#   make test-all               (tous les services)

test:
	@test -n "$(service)" || { echo "usage: make test service=<name>"; exit 1; }
ifeq ($(IN_DOCKER),yes)
	DATABASE_URL="postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)" \
	REDIS_URL="redis://redis:6379" \
	PASETO_SECRET_KEY="$(PASETO_SECRET_KEY)" \
	cargo test -p $(service)
else
	docker exec \
		-e DATABASE_URL="postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(EXEC_DB_HOST):$(EXEC_DB_PORT)/$(DB_NAME)" \
		-e REDIS_URL="redis://redis:6379" \
		-e PASETO_SECRET_KEY="$(PASETO_SECRET_KEY)" \
		velora-workspace-1 \
		cargo test -p $(service)
endif

test-all:
ifeq ($(IN_DOCKER),yes)
	DATABASE_URL="postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/velora_auth" \
	REDIS_URL="redis://redis:6379" \
	PASETO_SECRET_KEY="$(PASETO_SECRET_KEY)" \
	cargo test --workspace
else
	docker exec \
		-e DATABASE_URL="postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(EXEC_DB_HOST):$(EXEC_DB_PORT)/velora_auth" \
		-e REDIS_URL="redis://redis:6379" \
		-e PASETO_SECRET_KEY="$(PASETO_SECRET_KEY)" \
		velora-workspace-1 \
		cargo test --workspace
endif

# --- Migrations (per service) -----------------------------------
migrate-up-all:
ifeq ($(IN_DOCKER),yes)
	@for s in $(SERVICES); do \
		if [ -d services/$$s/migrations ]; then \
			db=velora_$$s; [ "$$s" = "user" ] && db=velora_users; \
			echo ">>> migrating $$s ($$db)"; \
			(cd services/$$s && sqlx migrate run --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DB_HOST):$(DB_PORT)/$$db") || exit 1; \
		fi; \
	done
else
	@for s in $(SERVICES); do \
		if [ -d services/$$s/migrations ]; then \
			db=velora_$$s; [ "$$s" = "user" ] && db=velora_users; \
			echo ">>> migrating $$s ($$db)"; \
			docker exec -w /app/services/$$s velora-workspace-1 sqlx migrate run --database-url "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(EXEC_DB_HOST):$(EXEC_DB_PORT)/$$db" || exit 1; \
		fi; \
	done
endif
