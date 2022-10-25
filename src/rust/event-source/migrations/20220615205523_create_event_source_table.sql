CREATE TABLE IF NOT EXISTS event_sources (
    event_source_id uuid PRIMARY KEY,
    tenant_id uuid NOT NULL,
    display_name varchar(128) NOT NULL,
    description varchar(1024) NOT NULL,
    created_time timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated_time timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    active boolean NOT NULL DEFAULT true
);