/*
event_source_id uuid PRIMARY KEY,
tenant_id uuid NOT NULL,
display_name varchar(128) NOT NULL,
description varchar(1024) NOT NULL,
created_time timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
last_updated_time timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
active boolean NOT NULL DEFAULT true
*/

use std::time::SystemTime;

use uuid::Uuid;

#[allow(dead_code)]
pub struct EventSourceRow {
    event_source_id: Uuid,
    tenant_id: Uuid,
    display_name: String,
    description: String,
    created_time: SystemTime,
    last_updated_time: SystemTime,
    active: bool,
}