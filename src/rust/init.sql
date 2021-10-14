CREATE TABLE organization
(
    org_id                  uuid       PRIMARY KEY UNIQUE NOT NULL,
    org_display_name        char       NOT NULL,
    admin_username          char       NOT NULL,
    admin_email             char       NOT NULL,
    admin_password          char       NOT NULL,
    should_reset_password   bool       NOT NULL
);


CREATE TABLE users
(
    user_id                 uuid       PRIMARY KEY UNIQUE NOT NULL,
    org_id                  uuid       NOT NULL,
    name                    char       NOT NULL,
    email                   char       NOT NULL,
    password                char       NOT NULL
);


-- INSERT INTO organization(org_id, org_display_name, admin_email, admin_password, admin_username, should_reset_password) VALUES
-- (1, 'A'),
-- (2, 'B'),
-- (3, 'C');