CREATE TABLE IF NOT EXISTS users
(
    user_id               uuid     PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id       uuid     NOT NULL,
    username              TEXT     NOT NULL,
    email                 TEXT     NOT NULL,
    password              TEXT     NOT NULL,
    should_reset_password BOOLEAN  NOT NULL
);

CREATE TABLE IF NOT EXISTS organization
(
    organization_id                uuid   PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name          TEXT NOT NULL,
    CONSTRAINT user_fk
        FOREIGN KEY (organization_id)
            REFERENCES users (user_id)
            ON DELETE CASCADE
);
