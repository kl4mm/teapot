CREATE TABLE IF NOT EXISTS users (
    id         BIGSERIAL,
    first_name VARCHAR(100),
    last_name  VARCHAR(100),
    email      VARCHAR(100) UNIQUE,
    password   BYTEA,
    PRIMARY KEY (id)
);

INSERT INTO users (first_name, last_name, email, password) VALUES 
('bob', 'smith', 'bob@smith.com', E'\\x673832414244616e4d6a6e646d636758504f50695a536b45506e334371444944544637396b7a466e366555');
