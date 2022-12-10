up:
	docker-compose up
down:
	docker-compose down
build:
	docker-compose build --no-cache
clean:
	docker image prune
sql:
	cd database && sh concat.sh && cd ..
