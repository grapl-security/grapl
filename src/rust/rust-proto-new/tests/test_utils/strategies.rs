use bytes::Bytes;
use proptest::prelude::*;
use uuid::Uuid;

//
// Bytes
//

pub fn bytes(size: usize) -> impl Strategy<Value = Bytes> {
    proptest::collection::vec(any::<u8>(), size).prop_map(Bytes::from)
}

//
// Uuid
//

prop_compose! {
    pub fn uuids()(
        int in any::<u128>()
    ) -> Uuid {
        Uuid::from_u128_le(int)
    }
}
