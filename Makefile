.PHONY: up down reset-password

run:
	docker-compose up --build

down:
	docker-compose down

reset-password:
	docker-compose run --rm api cargo run --release -- --reset-password
