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
SELECT 'CREATE DATABASE shop'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'shop')\gexec

\c shop

CREATE TABLE IF NOT EXISTS inventory(
    id          BIGSERIAL,
    name        VARCHAR(100),
    price       INT,
    quantity    INT,
    image_url   VARCHAR(255),
    description TEXT,
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT  inventory_quantity CHECK (quantity >= 0),
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS address(
    id         BIGSERIAL,
    user_id    BIGINT,
    address_1  VARCHAR(100),
    address_2  VARCHAR(100),
    postcode   VARCHAR(100),
    city       VARCHAR(100),
    PRIMARY KEY (id)
);
CREATE INDEX address_user_id ON address (user_id);

CREATE TABLE IF NOT EXISTS orders(
    id           UUID,
    user_id      BIGINT,
    inventory_id BIGINT REFERENCES "inventory" (id),
    quantity     INT,
    address_id   BIGSERIAL REFERENCES "address" (id),
    PRIMARY KEY (id, inventory_id)
);
CREATE INDEX orders_user_id ON orders (user_id);

INSERT INTO address (user_id, address_1, address_2, postcode, city) VALUES 
(1, '1 bob st', '', 'm1abc', 'manchester');

INSERT INTO inventory (name, price, quantity, image_url, description) VALUES
('Clipper Earl Grey - 80 Teabags', 299, 100, 'https://digitalcontent.api.tesco.com/v2/media/ghs/06da6f5a-c9cc-4c1e-aa3b-4491ab29e3d8/a8e9adb6-5e21-42cb-9876-24ca40a1d269_150922527.jpeg?h=540&w=540', '80 Unbleached, plastic-free bags of organic earl grey tea'),
('Twining English Breakfast - 160 Teabags', 600, 100, 'https://assets.sainsburys-groceries.co.uk/gol/7975122/1/640x640.jpg', 'Golden and well rounded. Its a tea with a lot of body and a light finish');

BEGIN;
    DO $$
    DECLARE order_id UUID = gen_random_uuid();
    BEGIN
        INSERT INTO orders VALUES (order_id, 1, 1, 1, 1);
        INSERT INTO orders VALUES (order_id, 1, 2, 1, 1);
        UPDATE inventory SET quantity = quantity - 1 WHERE id IN (1, 2);
    END
    $$;
COMMIT;
