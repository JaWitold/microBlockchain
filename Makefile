.DEFAULT_GOAL := help

.PHONY: all build up kill down restart help \
	rust-restart

COMPOSE_FILE=docker-compose.gen.yaml
GEN_SCRIPT=./scripts/generate.py

RUST_PREFIX=rusty_miner
REDIS_PREFIX=redis
RABBITMQ_PREFIX=rabbitmq
NEXTJS_PREFIX=nextjs

define get_prefix_files
	$(shell seq -f "$(1)_%g" 1 $(n))
endef

REDIS_CONTAINERS=$(call get_prefix_files,$(REDIS_PREFIX))
RABBITMQ_CONTAINERS=$(call get_prefix_files,$(RABBITMQ_PREFIX))
RUST_CONTAINERS=$(call get_prefix_files,$(RUST_PREFIX))
NEXTJS_CONTAINERS=$(call get_prefix_files,$(NEXTJS_PREFIX))

ALL_CONTAINERS=$(REDIS_CONTAINERS) $(RABBITMQ_CONTAINERS) \
	$(RUST_CONTAINERS) $(NEXTJS_CONTAINERS)

all: down build up

build:
	python3 $(GEN_SCRIPT) $(n) $(RUST_PREFIX) $(REDIS_PREFIX) \
		$(RABBITMQ_PREFIX) $(NEXTJS_PREFIX)
	docker-compose -f $(COMPOSE_FILE) build $(ALL_CONTAINERS)

# Run containers in specific order to avoid infinite restarts of rust containers
up: $(COMPOSE_FILE)
	docker-compose -f $(COMPOSE_FILE) up -d $(REDIS_CONTAINERS)
	docker-compose -f $(COMPOSE_FILE) up -d $(RABBITMQ_CONTAINERS)
	docker-compose -f $(COMPOSE_FILE) up -d $(NEXTJS_CONTAINERS)
	docker-compose -f $(COMPOSE_FILE) up -d $(RUST_CONTAINERS)

kill: $(COMPOSE_FILE)
	docker-compose -f $(COMPOSE_FILE) kill -s SIGINT

down: $(COMPOSE_FILE)
	docker-compose -f $(COMPOSE_FILE) down --remove-orphans

restart: $(COMPOSE_FILE)
	docker-compose -f $(COMPOSE_FILE) restart

rust-restart: $(COMPOSE_FILE)
	docker-compose -f $(COMPOSE_FILE) kill -s SIGINT \
		$(shell seq -f "rusty_miner_%g" 1 $(n))
	docker-compose -f $(COMPOSE_FILE) up -d --force-recreate \
		--no-deps --build -d $(shell seq -f "rusty_miner_%g" 1 $(n))

$(COMPOSE_FILE): $(GEN_SCRIPT)
	python3 $(GEN_SCRIPT) $(n) $(RUST_PREFIX) $(REDIS_PREFIX) \
		$(RABBITMQ_PREFIX) $(NEXTJS_PREFIX)

help:
	@echo "Usage: make [target] [n=<n>]"
	@echo ""
	@echo "General purpose targets:"
	@echo "  all n=<n>		- down, build, up"
	@echo "  build n=<n>		- build docker images"
	@echo "  up n=<n>      		- start docker containers"
	@echo "  kill     		- stop docker containers"
	@echo "  down     		- stop and remove orphan docker containers"
	@echo "  restart  		- restart docker containers"
	@echo "  help     		- show this help"
	@echo ""
	@echo "Other targets:"
	@echo "  rust-restart n=<n> 	- rebuild and restart n rust miner containers." \
		"From 1 to n."
	@echo ""
