use std::sync::Arc;

use dashmap::{
    mapref::entry::Entry,
    DashMap,
};
use rust_proto_new::graplinc::grapl::api::uid_allocator::v1beta1::{
    messages::{
        AllocateIdsRequest,
        AllocateIdsResponse,
        Allocation,
    },
    server::UidAllocatorApi,
};
use sqlx::PgPool;

use crate::{
    counters_db::CountersDb,
    service::UidAllocatorServiceError,
};

/// PreAllocation is intended to be a struct large enough to represent many allocations worth
/// of Uid ranges. When clients request a new allocation, first it is taken from the PreAllocation,
/// so that we only reach out to the database when the PreAllocation is exhausted.
#[derive(Clone, Copy)]
pub struct PreAllocation {
    pub start: u64,
    pub current: u64,
    pub end: u64,
}

impl PreAllocation {
    pub fn new(start: u64, end: u64) -> PreAllocation {
        assert!(start <= end);
        PreAllocation {
            start,
            current: start,
            end,
        }
    }

    /// `next` returns the next uid Allocation in the PreAllocation, or None if the
    /// PreAllocation is exhausted.
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

/// The UidAllocator holds the business logic for managing a tenant's Uid allocation, including
/// the preallocation of uids for performance purposes, limiting how many Uids are allocated at
/// once, and any other logic around the management of Uids.
///
/// The UidAllocator is intended to be cheap to Clone and should be shared across threads
/// where possible.
#[derive(Clone)]
pub struct UidAllocator {
    /// The in-memory state of pre-allocated uids for each tenant
    pub allocated_ranges: Arc<DashMap<uuid::Uuid, PreAllocation>>,
    /// The CountersDb is our source of truth for the last allocated uid for each tenant
    pub db: CountersDb,
    /// Default allocation size indicates how many uids to allocate for a tenant if the
    /// client requests an allocation of size `0`.
    /// Consider values of 10, 100, or 1_000
    /// Should not be a value greater than `maximum_allocation_size` and must not be `0`.
    pub default_alloc_size: u32,
    /// How many uids to preallocate when our last preallocation is exhausted
    /// While this can be as large as a u32, it is not recommended to set this to a value
    /// too high. Consider values such as 100, 1000, or 10_000 instead.
    pub preallocation_size: u32,
    /// The maximum size of an allocation that we'll hand out to a given client for a
    /// request. Similar to the `preallocation_size` field, this is a value that can be
    /// a full 32bit integer, but is not recommended to be too large. It should also
    /// always me smaller than the preallocation_size.
    /// Consider values such as 10, 100, or 1_000.
    pub maximum_allocation_size: u32,
}

impl UidAllocator {
    /// Creates a new instance of the UidAllocator.
    /// Panics if the preallocation_size is larger than the maximum_allocation_size, or if
    /// the either of `preallocation_size` or `maximum_allocation_size` are 0.
    pub fn new(
        pool: PgPool,
        preallocation_size: u32,
        maximum_allocation_size: u32,
        default_allocation_size: u32,
    ) -> UidAllocator {
        assert!(default_allocation_size > 0);
        assert!(preallocation_size > 0);
        assert!(maximum_allocation_size > 0);
        assert!(preallocation_size >= maximum_allocation_size);
        UidAllocator {
            allocated_ranges: Arc::new(DashMap::with_capacity(2)),
            db: CountersDb { pool },
            default_alloc_size: 0,
            preallocation_size,
            maximum_allocation_size,
        }
    }

    /// Allocates the next range of Uids for a given tenant.
    /// While `size` is the value that a tenant may have asked for, the actual size of the
    /// allocation may differ. For example, `allocate` never returns more than `maximum_allocation_size`,
    /// and may return a smaller allocation than requested if `size` is larger than the remaining range
    /// in the current preallocation.
    ///
    /// Clients should never expect consistent behavior with regards to the size of the allocation,
    /// other than that the allocation is never empty.
    ///
    /// If the current preallocated range is already exhausted `allocate` will preallocate a new range,
    /// storing that range in the counter database, and will return a new allocation from that preallocation.
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

#[tonic::async_trait]
impl UidAllocatorApi for UidAllocator {
    type Error = UidAllocatorServiceError;

    async fn allocate_ids(
        &self,
        request: AllocateIdsRequest,
    ) -> Result<AllocateIdsResponse, Self::Error> {
        let AllocateIdsRequest { tenant_id, count } = request;
        let allocation = self.allocate(tenant_id, count).await?;
        Ok(AllocateIdsResponse { allocation })
    }
}
