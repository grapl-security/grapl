CREATE SCHEMA IF NOT EXISTS graph_schema_manager;

CREATE OR REPLACE FUNCTION graph_schema_manager.megabytes(bytes integer) RETURNS integer AS
$$
BEGIN
    RETURN bytes * 1000 * 1000;

END;
$$ LANGUAGE plpgsql
    IMMUTABLE;

CREATE TABLE graph_schema_manager.node_identity_algorithm
(
    identity_algorithm text     NOT NULL CHECK (identity_algorithm IN ('session', 'static')),
    tenant_id          uuid     NOT NULL,
    node_type          text     NOT NULL CHECK (node_type <> '' AND length(node_type) < 32),
    schema_version     smallint NOT NULL,
    PRIMARY KEY (identity_algorithm, tenant_id, node_type, schema_version)
);


CREATE TABLE graph_schema_manager.session_identity_arguments
(
    tenant_id                      uuid     NOT NULL,
    identity_algorithm             text     NOT NULL CHECK (identity_algorithm = 'session'),
    node_type                      text     NOT NULL CHECK (node_type <> '' AND length(node_type) < 32),
    schema_version                 smallint NOT NULL,
    pseudo_key_properties          text[]   NOT NULL CHECK (array_length(pseudo_key_properties, 1) > 0),
    negation_key_properties        text[]   NOT NULL,
    creation_timestamp_property    text     NOT NULL CHECK (creation_timestamp_property <> ''),
    last_seen_timestamp_property   text     NOT NULL CHECK (last_seen_timestamp_property <> ''),
    termination_timestamp_property text     NOT NULL CHECK (termination_timestamp_property <> ''),
    FOREIGN KEY (identity_algorithm, tenant_id, node_type, schema_version)
        REFERENCES graph_schema_manager.node_identity_algorithm (identity_algorithm, tenant_id, node_type, schema_version) ON DELETE CASCADE
);


CREATE TABLE graph_schema_manager.node_schemas
(
    tenant_id            uuid      NOT NULL,
    identity_algorithm   text      NOT NULL CHECK (identity_algorithm IN ('session', 'static')),
    node_type            text      NOT NULL CHECK (node_type <> '' AND length(node_type) < 32),
    schema_version       smallint  NOT NULL,
    deployment_timestamp timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
    raw_schema           bytea     NOT NULL CHECK (length(raw_schema) < graph_schema_manager.megabytes(1)),
    schema_type          text      NOT NULL CHECK (schema_type <> ''),
    PRIMARY KEY (tenant_id, node_type, schema_version),
    FOREIGN KEY (tenant_id, node_type, schema_version, identity_algorithm)
        REFERENCES graph_schema_manager.node_identity_algorithm (tenant_id, node_type, schema_version, identity_algorithm) ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS graph_schema_manager.static_identity_arguments
(
    tenant_id             uuid     NOT NULL,
    identity_algorithm    text     NOT NULL CHECK (identity_algorithm = 'static'),
    node_type             text     NOT NULL CHECK (node_type <> '' AND length(node_type) < 32),
    schema_version        smallint NOT NULL,
    static_key_properties text[]   NOT NULL CHECK (array_length(static_key_properties, 1) > 0),
    FOREIGN KEY (tenant_id, node_type, schema_version, identity_algorithm)
        REFERENCES graph_schema_manager.node_identity_algorithm (tenant_id, node_type, schema_version, identity_algorithm) ON DELETE CASCADE
);

CREATE TYPE graph_schema_manager.property_type AS ENUM (
    'ImmutableString',
    'ImmutableI64',
    'MaxI64',
    'MinI64',
    'ImmutableU64',
    'MaxU64',
    'MinU64'
);

CREATE TABLE IF NOT EXISTS graph_schema_manager.property_schemas
(
    tenant_id      uuid                         NOT NULL,
    node_type      text                         NOT NULL CHECK (node_type <> '' AND length(node_type) < 32),
    schema_version smallint                     NOT NULL,
    property_name  text                         NOT NULL CHECK (property_name <> '' AND length(property_name) < 32),
    property_type  graph_schema_manager.property_type NOT NULL,
    -- Indicates this property should be dropped after identification
    identity_only  bool                         NOT NULL,
    PRIMARY KEY (tenant_id, node_type, schema_version, property_name),
    FOREIGN KEY (tenant_id, node_type, schema_version)
        REFERENCES graph_schema_manager.node_schemas (tenant_id, node_type, schema_version)
);

CREATE TYPE graph_schema_manager.edge_cardinality AS ENUM ('ToMany', 'ToOne');

CREATE TABLE IF NOT EXISTS graph_schema_manager.edge_schemas
(
    tenant_id                uuid                            NOT NULL,
    node_type                text                            NOT NULL CHECK (node_type <> '' AND length(node_type) < 32),
    schema_version           smallint                        NOT NULL,
    forward_edge_name        text                            NOT NULL CHECK (forward_edge_name <> '' AND length(forward_edge_name) < 32),
    reverse_edge_name        text                            NOT NULL CHECK (reverse_edge_name <> '' AND length(reverse_edge_name) < 32),
    forward_edge_cardinality graph_schema_manager.edge_cardinality NOT NULL,
    reverse_edge_cardinality graph_schema_manager.edge_cardinality NOT NULL,
    PRIMARY KEY (tenant_id, node_type, schema_version, forward_edge_name, reverse_edge_name),
    FOREIGN KEY (tenant_id, node_type, schema_version)
        REFERENCES graph_schema_manager.node_schemas (tenant_id, node_type, schema_version)
);-- Add migration script here
