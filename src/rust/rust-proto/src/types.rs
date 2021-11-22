use crate::graplinc::common::v1beta1 as graplinc_common;

impl graplinc_common::Uuid {
    pub fn new() -> Self {
        Self::from(uuid::Uuid::new_v4())
    }
}

impl From<graplinc_common::Uuid> for uuid::Uuid {
    fn from(p: graplinc_common::Uuid) -> Self {
        let lsb: [u8; 8] = p.lsb.to_le_bytes();
        let msb: [u8; 8] = p.msb.to_le_bytes();
        let u: u128 = u128::from_le_bytes([
            lsb[0], lsb[1], lsb[2], lsb[3], lsb[4], lsb[5], lsb[6], lsb[7], msb[0], msb[1], msb[2],
            msb[3], msb[4], msb[5], msb[6], msb[7],
        ]);

        uuid::Builder::from_u128(u).build()
    }
}

impl From<uuid::Uuid> for graplinc_common::Uuid {
    fn from(u: uuid::Uuid) -> Self {
        let u = u.as_u128();
        let u_le: [u8; 16] = u.to_le_bytes();
        // Ugly, but `Index` isn't const, so Rust can't figure
        // out that half of a [u8; 16] is two [u8; 8]'s
        let lsb: u64 = u64::from_le_bytes([
            u_le[0], u_le[1], u_le[2], u_le[3], u_le[4], u_le[5], u_le[6], u_le[7],
        ]);
        let msb: u64 = u64::from_le_bytes([
            u_le[8], u_le[9], u_le[10], u_le[11], u_le[12], u_le[13], u_le[14], u_le[15],
        ]);
        Self { lsb, msb }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equality_from_uuid() {
        for _ in 0..1000 {
            // start with a uuid::Uuid
            let u0 = uuid::Uuid::new_v4();
            // Convert it into a protobuf Uuid
            let pu: graplinc_common::Uuid = u0.clone().into();
            // Then back to a uuid::Uuid
            let u1: uuid::Uuid = pu.into();
            // Hopefully it hasn't changed
            assert_eq!(u0, u1);
        }
    }

    #[test]
    fn test_equality_from_proto_uuid() {
        for _ in 0..1000 {
            // start with a protobuf Uuid
            let pu0 = graplinc_common::Uuid::new();
            // Convert it into a uuid::Uuid
            let u: uuid::Uuid = pu0.clone().into();
            // Then back to a protobuf Uuid
            let pu1: graplinc_common::Uuid = u.into();
            // Hopefully it hasn't changed
            assert_eq!(pu0, pu1);
        }
    }
}
