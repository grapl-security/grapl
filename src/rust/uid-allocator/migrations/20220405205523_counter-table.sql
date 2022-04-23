-- Add migration script here

CREATE TABLE IF NOT EXISTS counters (
  tenant_id uuid    PRIMARY KEY,
  counter   bigint  NOT NULL DEFAULT 1
);

