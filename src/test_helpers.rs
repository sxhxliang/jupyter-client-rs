use crypto_mac::{Mac, Output};
use digest::generic_array::typenum::U64;
use digest::generic_array::GenericArray;
use hmac::Mac as Hmac;

#[derive(Debug, Clone)]
pub(crate) struct FakeAuth;

static KEY: &[u8] = b"foobar0000000000000000000000000000000000000000000000000000000000";
impl Hmac for FakeAuth {
    fn new(key: &digest::Key<Self>) -> Self
    where
        Self: digest::KeyInit {
            
        todo!()
    }

    fn new_from_slice(key: &[u8]) -> Result<Self, digest::InvalidLength>
    where
        Self: digest::KeyInit {
        todo!()
    }

    fn update(&mut self, data: &[u8]) {
        todo!()
    }

    fn chain_update(self, data: impl AsRef<[u8]>) -> Self {
        todo!()
    }

    fn finalize(self) -> digest::CtOutput<Self> {
        todo!()
    }

    fn finalize_reset(&mut self) -> digest::CtOutput<Self>
    where
        Self: digest::FixedOutputReset {
        todo!()
    }

    fn reset(&mut self)
    where
        Self: digest::Reset {
        todo!()
    }

    fn verify(self, tag: &digest::Output<Self>) -> Result<(), digest::MacError> {
        todo!()
    }

    fn verify_reset(&mut self, tag: &digest::Output<Self>) -> Result<(), digest::MacError>
    where
        Self: digest::FixedOutputReset {
        todo!()
    }

    fn verify_slice(self, tag: &[u8]) -> Result<(), digest::MacError> {
        todo!()
    }

    fn verify_slice_reset(&mut self, tag: &[u8]) -> Result<(), digest::MacError>
    where
        Self: digest::FixedOutputReset {
        todo!()
    }

    fn verify_truncated_left(self, tag: &[u8]) -> Result<(), digest::MacError> {
        todo!()
    }

    fn verify_truncated_right(self, tag: &[u8]) -> Result<(), digest::MacError> {
        todo!()
    }
}

impl Mac for FakeAuth {
    type OutputSize = U64;

    fn update(&mut self, data: &[u8]) {
        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }

    fn finalize(self) -> crypto_mac::Output<Self> {
        Output::new(GenericArray::from_slice(KEY).to_owned())
    }
}

impl FakeAuth {
    pub(crate) fn create() -> FakeAuth {
        FakeAuth{}
    }
}

pub(crate) fn expected_signature() -> String {
    let auth = FakeAuth::create();
    let res = auth.finalize();
    let code = res.into_bytes();
    let encoded = hex::encode(code);
    encoded
}

#[macro_export]
macro_rules! compare_bytestrings {
    ($a:expr, $b:expr) => {
        let a = String::from_utf8_lossy($a).into_owned();
        let b = String::from_utf8_lossy($b).into_owned();
        assert_eq!($a, $b, "result {:?} != expected {:?}", a, b);
    };
}
