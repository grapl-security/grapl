ALTER TABLE plugin_deployment
      ADD COLUMN deployed boolean NOT NULL DEFAULT true,
      DROP COLUMN deploy_time,
      ADD COLUMN timestamp timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP;
