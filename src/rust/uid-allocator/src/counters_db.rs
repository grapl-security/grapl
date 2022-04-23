use sqlx::PgPool;
use crate::allocator::PreAllocation;
use crate::service::{UidAllocatorServiceError};

#[derive(Debug)]
pub struct Count {
    pub prev: i64,
    pub new: i64,
}

#[derive(Clone)]
pub struct CountersDb {
    pub pool: PgPool,
}

impl CountersDb {

    pub async fn preallocate(&self, tenant_id: uuid::Uuid, size: u32) -> Result<PreAllocation, UidAllocatorServiceError> {
        let mut conn = self.pool.acquire().await?;
        // Increments the tenant's allocation counter by `size`, and returns the new value as well as the previous value
        let count = sqlx::query_as!(Count,
            "
            UPDATE counters
            SET counter = counter + $1
            FROM (
                     SELECT counter as prev
                     FROM counters
                     WHERE counters.tenant_id = $2
                     LIMIT 1
                 ) as c
            WHERE counters.tenant_id = $2
            RETURNING counter as new, c.prev
            ",
        size as i32,
        tenant_id
        ).fetch_optional(&mut conn).await?;

        let count = match count {
            Some(count) => count,
            None => return Err(UidAllocatorServiceError::UnknownTenant(tenant_id))
        };

        Ok(PreAllocation::new(count.prev as u64, count.new as u64))
    }

}
