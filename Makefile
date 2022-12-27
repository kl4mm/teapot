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
test:
	RUST_LOG=debug DATABASE_URL=postgres://postgres:postgres@localhost:5432 TOKEN_SECRET=secret cargo test -- --nocapture

runusers:
	RUST_LOG=debug DATABASE_URL=postgres://postgres:postgres@localhost:5432 TOKEN_SECRET=secret cargo run -p users
runshop:
	RUST_LOG=debug DATABASE_URL=postgres://postgres:postgres@localhost:5432 TOKEN_SECRET=secret cargo run -p shop
