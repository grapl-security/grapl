-- Add migration script here
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
    password                char       NOT NULL,
    FOREIGN KEY (org_id) REFERENCES    organization(org_id)
);

