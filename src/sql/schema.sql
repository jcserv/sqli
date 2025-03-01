DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS orders CASCADE;

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    amount DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users (name, email, status) VALUES
    ('John Doe', 'john@example.com', 'active'),
    ('Jane Smith', 'jane@example.com', 'active'),
    ('Bob Johnson', 'bob@example.com', 'inactive'),
    ('Alice Brown', 'alice@example.com', 'active'),
    ('Charlie Wilson', 'charlie@example.com', 'active');

INSERT INTO orders (user_id, amount, created_at) VALUES
    (1, 99.99, CURRENT_TIMESTAMP - INTERVAL '2 days'),
    (1, 150.50, CURRENT_TIMESTAMP - INTERVAL '10 days'),
    (2, 25.99, CURRENT_TIMESTAMP - INTERVAL '5 days'),
    (3, 74.25, CURRENT_TIMESTAMP - INTERVAL '1 day'),
    (5, 199.99, CURRENT_TIMESTAMP - INTERVAL '7 days'),
    (5, 49.99, CURRENT_TIMESTAMP - INTERVAL '3 days'),
    (5, 39.95, CURRENT_TIMESTAMP - INTERVAL '1 day'),
    (1, 29.95, CURRENT_TIMESTAMP);