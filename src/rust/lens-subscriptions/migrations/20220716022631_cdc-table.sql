CREATE SCHEMA IF NOT EXISTS lens_subscriptions;
CREATE SCHEMA IF NOT EXISTS partman;
CREATE EXTENSION IF NOT EXISTS pg_partman WITH SCHEMA partman;
CREATE EXTENSION IF NOT EXISTS pg_cron;

CREATE TABLE IF NOT EXISTS lens_subscriptions.lens_cdc (
    tenant_id uuid NOT NULL,
    lens_uid bigint NOT NULL,
    update_offset bigserial NOT NULL,
    lens_update bytea NOT NULL,
    PRIMARY KEY(tenant_id, lens_uid, update_offset)
);


CREATE OR REPLACE FUNCTION notifyupdate() RETURNS trigger AS $$
DECLARE
BEGIN
  PERFORM pg_notify('lens_cdc', row_to_json(NEW)::text);
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER updatenotify AFTER INSERT ON lens_subscriptions.lens_cdc FOR EACH ROW EXECUTE PROCEDURE notifyupdate();

