.PHONY: all build up down

all: down build up

build:
	python3 ./scripts/generate.py $(n)
	docker-compose -f ./docker-compose.gen.yaml build

up:
	docker-compose -f ./docker-compose.gen.yaml up -d

kill:
	docker-compose -f ./docker-compose.gen.yaml kill -s SIGINT

down:
	docker-compose -f ./docker-compose.gen.yaml down --remove-orphans

restart:
	docker-compose -f ./docker-compose.gen.yaml restart
