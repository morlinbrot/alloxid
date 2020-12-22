CREATE TABLE todos (
    id SERIAL PRIMARY KEY,
    text VARCHAR(140) NOT NULL,
    completed boolean NOT NULL
);
