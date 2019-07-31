use super::{PublicKey, SecretKey, BLS_SIG_BYTE_SIZE};
use hex::encode as hex_encode;
use milagro_bls::Signature as RawSignature;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_hex::HexVisitor;
use ssz::{ssz_encode, Decode, DecodeError, Encode};

/// A single BLS signature.
///
/// This struct is a wrapper upon a base type and provides helper functions (e.g., SSZ
/// serialization).
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Signature {
    signature: RawSignature,
    is_empty: bool,
}

impl Signature {
    /// Instantiate a new Signature from a message and a SecretKey.
    pub fn new(msg: &[u8], domain: u64, sk: &SecretKey) -> Self {
        Signature {
            signature: RawSignature::new(msg, domain, sk.as_raw()),
            is_empty: false,
        }
    }

    /// Instantiate a new Signature from a message and a SecretKey, where the message has already
    /// been hashed.
    pub fn new_hashed(x_real_hashed: &[u8], x_imaginary_hashed: &[u8], sk: &SecretKey) -> Self {
        Signature {
            signature: RawSignature::new_hashed(x_real_hashed, x_imaginary_hashed, sk.as_raw()),
            is_empty: false,
        }
    }

    /// Verify the Signature against a PublicKey.
    pub fn verify(&self, msg: &[u8], domain: u64, pk: &PublicKey) -> bool {
        if self.is_empty {
            return false;
        }
        self.signature.verify(msg, domain, pk.as_raw())
    }

    /// Verify the Signature against a PublicKey, where the message has already been hashed.
    pub fn verify_hashed(
        &self,
        x_real_hashed: &[u8],
        x_imaginary_hashed: &[u8],
        pk: &PublicKey,
    ) -> bool {
        self.signature
            .verify_hashed(x_real_hashed, x_imaginary_hashed, pk.as_raw())
    }

    /// Returns the underlying signature.
    pub fn as_raw(&self) -> &RawSignature {
        &self.signature
    }

    /// Returns a new empty signature.
    pub fn empty_signature() -> Self {
        // Set RawSignature = infinity
        let mut empty: Vec<u8> = vec![0; BLS_SIG_BYTE_SIZE];
        empty[0] += u8::pow(2, 6) + u8::pow(2, 7);
        Signature {
            signature: RawSignature::from_bytes(&empty).unwrap(),
            is_empty: true,
        }
    }

    // Converts a BLS Signature to bytes
    pub fn as_bytes(&self) -> Vec<u8> {
        if self.is_empty {
            return vec![0; 96];
        }
        self.signature.as_bytes()
    }

    // Convert bytes to BLS Signature
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        for byte in bytes {
            if *byte != 0 {
                let raw_signature = RawSignature::from_bytes(&bytes).map_err(|_| {
                    DecodeError::BytesInvalid(
                        format!("Invalid Signature bytes: {:?}", bytes).to_string(),
                    )
                })?;
                return Ok(Signature {
                    signature: raw_signature,
                    is_empty: false,
                });
            }
        }
        Ok(Signature::empty_signature())
    }

    // Check for empty Signature
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }

    /// Display a signature as a hex string of its bytes.
    #[cfg(test)]
    pub fn as_hex_string(&self) -> String {
        hex_encode(self.as_bytes())
    }
}

impl_ssz!(Signature, BLS_SIG_BYTE_SIZE, "Signature");

impl_tree_hash!(Signature, U96);

impl_cached_tree_hash!(Signature, U96);

impl Serialize for Signature {
    /// Serde serialization is compliant the Ethereum YAML test format.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex_encode(ssz_encode(self)))
    }
}

impl<'de> Deserialize<'de> for Signature {
    /// Serde serialization is compliant the Ethereum YAML test format.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = deserializer.deserialize_str(HexVisitor)?;
        let signature = Self::from_ssz_bytes(&bytes[..])
            .map_err(|e| serde::de::Error::custom(format!("invalid ssz ({:?})", e)))?;
        Ok(signature)
    }
}

#[cfg(test)]
mod tests {
    use super::super::Keypair;
    use super::*;
    use ssz::ssz_encode;
    use tree_hash::TreeHash;

    #[test]
    pub fn test_ssz_round_trip() {
        let keypair = Keypair::random();

        let original = Signature::new(&[42, 42], 0, &keypair.sk);

        let bytes = ssz_encode(&original);
        let decoded = Signature::from_ssz_bytes(&bytes).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    // TODO: once `CachedTreeHash` is fixed, this test should _not_ panic.
    #[should_panic]
    pub fn test_cached_tree_hash() {
        let keypair = Keypair::random();
        let original = Signature::new(&[42, 42], 0, &keypair.sk);

        let mut cache = cached_tree_hash::TreeHashCache::new(&original).unwrap();

        assert_eq!(
            cache.tree_hash_root().unwrap().to_vec(),
            original.tree_hash_root()
        );

        let modified = Signature::new(&[99, 99], 0, &keypair.sk);

        cache.update(&modified).unwrap();

        assert_eq!(
            cache.tree_hash_root().unwrap().to_vec(),
            modified.tree_hash_root()
        );
    }

    #[test]
    pub fn test_empty_signature() {
        let sig = Signature::empty_signature();

        let sig_as_bytes: Vec<u8> = sig.as_raw().as_bytes();

        assert_eq!(sig_as_bytes.len(), BLS_SIG_BYTE_SIZE);
        for (i, one_byte) in sig_as_bytes.iter().enumerate() {
            if i == 0 {
                assert_eq!(*one_byte, u8::pow(2, 6) + u8::pow(2, 7));
            } else {
                assert_eq!(*one_byte, 0);
            }
        }
    }

    fn add_flag_value(value: &mut u8, a_flag: bool, b_flag: bool, c_flag: bool) {
        if a_flag {
            *value += 32;
        }
        if b_flag {
            *value += 64;
        }
        if c_flag {
            *value += 128;
        }
    }

    fn fill_x(x: &[u8], sig: &mut [u8], least_significant_index: usize) {
        assert!(x.len() > 0);
        assert!(x.len() <= least_significant_index + 1);
        assert!(least_significant_index < sig.len());
        sig[least_significant_index + 1 - x.len()..least_significant_index + 1].copy_from_slice(x);
    }

    fn construct_signature(a_flag1: bool, b_flag1: bool, c_flag1: bool, x1: &[u8], a_flag2: bool,
                           b_flag2: bool, c_flag2: bool, x2: &[u8]) -> [u8; BLS_SIG_BYTE_SIZE] {
        let mut signature = [0; BLS_SIG_BYTE_SIZE];
        fill_x(x1, &mut signature[0..], BLS_SIG_BYTE_SIZE / 2 - 1);
        assert!(signature[0] < 32);
        add_flag_value(&mut signature[0], a_flag1, b_flag1, c_flag1);
        fill_x(x2, &mut signature[BLS_SIG_BYTE_SIZE / 2..], BLS_SIG_BYTE_SIZE / 2 - 1);
        assert!(signature[BLS_SIG_BYTE_SIZE / 2] < 32);
        add_flag_value(&mut signature[BLS_SIG_BYTE_SIZE / 2], a_flag2, b_flag2, c_flag2);
        signature
    }

    fn construct_signature_from_hex(a_flag1: bool, b_flag1: bool, c_flag1: bool, x1: &str,
                                    a_flag2: bool, b_flag2: bool, c_flag2: bool, x2: &str)
                                    -> [u8; BLS_SIG_BYTE_SIZE] {
        construct_signature(a_flag1, b_flag1, c_flag1, hex::decode(x1).unwrap().as_slice(),
                            a_flag2, b_flag2, c_flag2, hex::decode(x2).unwrap().as_slice())
    }

    const Q_HEX: &str =
        "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab";
    const Q_MINUS_ONE_HEX: &str =
        "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaaa";

    #[test]
    pub fn test_valid_signature() {
        let signature_bytes = construct_signature_from_hex(true, false, true, Q_MINUS_ONE_HEX,
                                                           false, false, false, Q_MINUS_ONE_HEX);
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_ok());
    }

    #[test]
    pub fn test_invalid_signature_x1_ge_q() {
        let signature_bytes = construct_signature_from_hex(true, false, true, Q_HEX, false,
                                                           false, false, Q_MINUS_ONE_HEX);
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_err());
    }

    #[test]
    pub fn test_invalid_signature_x2_ge_q() {
        let signature_bytes = construct_signature_from_hex(true, false, true, Q_MINUS_ONE_HEX,
                                                           false, false, false, Q_HEX);
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_err());
    }

    #[test]
    pub fn test_invalid_signature_a_flag2_eq_1() {
        let signature_bytes = construct_signature_from_hex(true, false, true, Q_MINUS_ONE_HEX,
                                                           true, false, false, Q_MINUS_ONE_HEX);
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_err());
    }

    #[test]
    pub fn test_invalid_signature_b_flag2_eq_1() {
        let signature_bytes = construct_signature_from_hex(true, false, true, Q_MINUS_ONE_HEX,
                                                           false, true, false, Q_MINUS_ONE_HEX);
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_err());
    }

    #[test]
    pub fn test_invalid_signature_c_flag2_eq_1() {
        let signature_bytes = construct_signature_from_hex(true, false, true, Q_MINUS_ONE_HEX,
                                                           false, false, true, Q_MINUS_ONE_HEX);
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_err());
    }

    #[test]
    pub fn test_verify_invalid_signature_c_flag2_eq_1() {
        let keypair = Keypair::random();

        let msg = &[42, 42];
        let domain = 0;
        let signature = Signature::new(msg, domain, &keypair.sk);
        let mut signature_bytes = signature.as_bytes();

        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_ok() && signature.unwrap().verify(msg, domain, &keypair.pk));

        //set c_flag2 to 1
        assert!(signature_bytes[BLS_SIG_BYTE_SIZE / 2] < 32);
        add_flag_value(&mut signature_bytes[BLS_SIG_BYTE_SIZE / 2], false, false, true);

        let signature = Signature::from_bytes(&signature_bytes[..]);
        //result should be err see also test test_invalid_signature_c_flag2_eq_1, but in this
        // test we also allow it to be ok if verify returns false
        assert!(signature.is_err() || !signature.unwrap().verify(msg, domain, &keypair.pk));
    }

    #[test]
    pub fn test_invalid_signature_c_flag1_eq_0() {
        let signature_bytes = construct_signature_from_hex(true, false, false, Q_MINUS_ONE_HEX,
                                                           false, false, false, Q_MINUS_ONE_HEX);
        let signature = Signature::from_bytes(&signature_bytes[..]);
        //we also allow empty since uncompressed signatures are parsed as empty in our code
        assert!(signature.is_err() || signature.unwrap().is_empty());
    }

    #[test]
    pub fn test_valid_signature_infinity() {
        let signature_bytes = construct_signature_from_hex(false, true, true, "00",
                                                           false, false, false, "00");
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_ok());
    }

    #[test]
    pub fn test_invalid_signature_b_flag1_eq_1_a_flag1_eq_1() {
        let signature_bytes = construct_signature_from_hex(true, true, true, "00",
                                                           false, false, false, "00");
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_err());
    }

    #[test]
    pub fn test_invalid_signature_b_flag1_eq_1_x1_eq_1() {
        let signature_bytes = construct_signature_from_hex(false, true, true, "01",
                                                           false, false, false, "00");
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_err());
    }

    #[test]
    pub fn test_invalid_signature_b_flag1_eq_1_x2_eq_1() {
        let signature_bytes = construct_signature_from_hex(false, true, true, "0",
                                                           false, false, false, "1");
        let signature = Signature::from_bytes(&signature_bytes[..]);
        assert!(signature.is_err());
    }
}
