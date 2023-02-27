use fvm_shared::address::Address;
use quickcheck::Arbitrary;

/// Unfortunately an arbitrary `DelegatedAddress` can be inconsistent
/// with bytes that do not correspond to its length. This struct fixes
/// that so we can generate arbitrary addresses that don't fail equality
/// after a roundtrip.
#[derive(Clone, Debug)]
pub struct ArbAddress(pub Address);

impl Arbitrary for ArbAddress {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let addr = Address::arbitrary(g);
        let bz = addr.to_bytes();
        let addr = Address::from_bytes(&bz).expect("address roundtrip works");
        Self(addr)
    }
}
