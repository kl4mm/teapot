up:
	docker-compose up
down:
	docker-compose down
build:
	docker-compose build --no-cache
clean:
	docker image prune
sql:
	cd database && cat users.sql shop.sql > init.sql && cd ..
