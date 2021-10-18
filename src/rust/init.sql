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


-- INSERT INTO organization(org_id, org_display_name, admin_username, admin_email, admin_password, should_reset_password)
-- VALUES
-- (1, '12315'),
-- (2, 'TESTORG'),
-- (3, 'USER'),
-- (4, 'TEST@TEST.COM'),
-- (5, 'PASSWORD'),
-- (6, 1);
--
--
-- INSERT INTO users (user_id, org_id, name, email, password)
-- VALUES
-- (1, '34234'),
-- (2, '12315'),
-- (3, 'newuser'),
-- (4,'email@meial.com'),
-- (5,'password');


