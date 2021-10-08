#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username postgres --dbname postgres <<-EOSQL
CREATE TABLE organization
(
    org_id                  uuid       PRIMARY KEY,
    org_display_name        char       NOT NULL,
    admin_username          char       NOT NULL,
    admin_email             char       NOT NULL,
    admin_password          char       NOT NULL,
    should_reset_password   bool       NOT NULL
)
EOSQL