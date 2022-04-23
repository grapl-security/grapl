use dashmap::{
    mapref::entry::Entry,
    DashMap,
};
use rust_proto_new::graplinc::grapl::api::uid_allocator::v1beta1::messages::Allocation;
use sqlx::PgPool;

use crate::{
    counters_db::CountersDb,
    service::UidAllocatorServiceError,
};

// PreAllocation is intended to be a struct large enough to represent many allocations worth
// of Uid ranges. When clients request a new allocation, first it is taken from the PreAllocation,
// so that we only reach out to the database when the PreAllocation is exhausted.
#[derive(Clone, Copy)]
pub struct PreAllocation {
    pub start: u64,
    pub current: u64,
    pub end: u64,
}

#[derive(Clone)]
pub struct UidAllocator {
    pub allocated_ranges: DashMap<uuid::Uuid, PreAllocation>,
    pub db: CountersDb,
    pub preallocation_size: u32,
    pub maximum_allocation_size: u32,
}

impl PreAllocation {
    pub fn new(start: u64, end: u64) -> PreAllocation {
        PreAllocation {
            start,
            current: start,
            end,
        }
    }

    pub fn next(&mut self, requested_size: u32) -> Option<Allocation> {
        assert!(requested_size > 0);
        // casting just makes the requested size easier to work with, but we
        // take a u32 because the requested size should never be close to that
        let mut requested_size = requested_size as u64;

        // If we have exhausted the PreAllocation, return None.
        if self.current < self.end {
            let previous = self.current;
            // If the requested size is larger than the remaining space, we truncate.
            if requested_size > self.end - self.current {
                requested_size = self.end - self.current;
            }
            self.current += requested_size;
            Some(Allocation {
                start: previous,
                offset: (self.current - previous) as u32,
            })
        } else {
            None
        }
    }
}

impl UidAllocator {
    pub fn new(
        pool: PgPool,
        preallocation_size: u32,
        maximum_allocation_size: u32,
    ) -> UidAllocator {
        assert!(preallocation_size > 1);
        assert!(maximum_allocation_size > 1);
        assert!(preallocation_size > maximum_allocation_size);
        UidAllocator {
            allocated_ranges: DashMap::with_capacity(2),
            db: CountersDb { pool },
            preallocation_size,
            maximum_allocation_size,
        }
    }

    pub async fn allocate(
        &self,
        tenant_id: uuid::Uuid,
        size: u32,
    ) -> Result<Allocation, UidAllocatorServiceError> {
        // We aren't going to hand out 2^32 ids at once, so we truncate so the maximum allocation size
        let size = std::cmp::min(size, self.maximum_allocation_size);
        // First, we check if we have a PreAllocation for this tenant. If not, we create one.
        match self.allocated_ranges.entry(tenant_id) {
            Entry::Occupied(mut entry) => {
                let preallocation = entry.get_mut();
                let allocation = preallocation.next(size);
                // If we have an available allocation we return it immediately.
                // If the allocation is exhausted, we preallocate more.
                match allocation {
                    Some(allocation) => Ok(allocation),
                    None => {
                        let mut new_preallocation = self
                            .db
                            .preallocate(tenant_id, self.preallocation_size)
                            .await?;
                        // Can not fail after preallocation
                        let allocation = new_preallocation.next(size).unwrap();
                        *preallocation = new_preallocation;
                        Ok(allocation)
                    }
                }
            }
            Entry::Vacant(entry) => {
                let mut preallocation = self
                    .db
                    .preallocate(tenant_id, self.preallocation_size)
                    .await?;
                // Can not fail after preallocation
                let allocation = preallocation.next(size).unwrap();
                entry.insert(preallocation);
                Ok(allocation)
            }
        }
    }
}
