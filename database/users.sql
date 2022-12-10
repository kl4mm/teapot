SELECT 'CREATE DATABASE users'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'users')\gexec

\c users

CREATE TABLE IF NOT EXISTS users (
    id         BIGSERIAL,
    first_name VARCHAR(100),
    last_name  VARCHAR(100),
    email      VARCHAR(100) UNIQUE,
    password   BYTEA,
    PRIMARY KEY (id)
);

INSERT INTO users (first_name, last_name, email, password) VALUES 
('bob', 'smith', 'bob@smith.com', 'password');
