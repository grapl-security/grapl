-- Add migration script here
CREATE TABLE IF NOT EXISTS organization (
                                            org_id SERIAL PRIMARY KEY,
                                            name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
                                    user_id SERIAL PRIMARY KEY,
                                    org_id SERIAL NOT NULL,
                                    name TEXT NOT NULL,
                                    email TEXT NOT NULL,
                                    password TEXT NOT NULL,
                                    reset_password BOOLEAN
);