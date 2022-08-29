pub const MAX_I_64_TABLE_NAME: &str = "max_i64";
pub const MIN_I_64_TABLE_NAME: &str = "min_i64";
pub const IMM_I_64_TABLE_NAME: &str = "imm_i64";
pub const MAX_U_64_TABLE_NAME: &str = "max_u64";
pub const MIN_U_64_TABLE_NAME: &str = "min_u64";
pub const IMM_U_64_TABLE_NAME: &str = "imm_u64";
pub const IMM_STRING_TABLE_NAME: &str = "imm_string";

pub fn tenant_keyspace_name(tenant_id: uuid::Uuid) -> String {
    // scylla keyspace names must be alphanumeric + underscores, and max out at 48.
    // fun fact: the result of this is exactly 48
    format!("tenant_keyspace_{}", tenant_id.simple())
}
