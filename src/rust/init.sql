CREATE TABLE organization (
    org_id SERIAL PRIMARY KEY,
    org_display_name TEXT NOT NULL,
    admin_username TEXT NOT NULL,
    admin_email TEXT NOT NULL,
    admin_password TEXT NOT NULL,
    should_reset_password BOOL NOT NULL
);

-- CREATE TABLE IF NOT EXISTS users (
--     user_id SERIAL PRIMARY KEY,
--     org_id SERIAL NOT NULL,
--     name TEXT NOT NULL,
--     email TEXT NOT NULL,
--     password TEXT NOT NULL,
--     reset_password BOOLEAN
-- );


-- org_id                  uuid       PRIMARY KEY,
--     org_display_name        char       NOT NULL,
--     admin_username          char       NOT NULL,
--     admin_email,            char       NOT NULL
--     admin_password          char       NOT NULL,
--     should_reset_password   bool       NOT NULL