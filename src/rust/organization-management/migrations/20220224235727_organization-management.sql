-- Add migration script here
CREATE TABLE IF NOT EXISTS organizations
(
    organization_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name    TEXT NOT NULL
);


CREATE TABLE IF NOT EXISTS users
(
    user_id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id       uuid    NOT NULL,
    username              TEXT    NOT NULL,
    email                 TEXT    NOT NULL,
    password              TEXT    NOT NULL,
    is_admin              BOOLEAN NOT NULL,
    should_reset_password BOOLEAN NOT NULL,
    CONSTRAINT user_fk
        FOREIGN KEY (organization_id)
            REFERENCES organizations (organization_id)
            ON DELETE CASCADE
);

