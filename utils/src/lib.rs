use sha3::{Digest, Sha3_256};

pub type HashLock = [u8; 32];
pub type SecretKey = [u8; 32];

pub fn try_lock(secret_key: SecretKey, hashlock: HashLock) -> bool {
    let mut hasher = Sha3_256::new();
    hasher.update(secret_key);
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out == hashlock
}

pub fn gen_lock(secret_key: SecretKey) -> HashLock {
    let mut hasher = Sha3_256::new();
    hasher.update(secret_key);
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out
}

#[test]
fn lock_mechanism() {
    let key = b"ssssssssssssssssssssssssssssssss";
    let lock = gen_lock(*key);
    assert_eq!(
        [
            165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43,
            58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154
        ],
        lock
    );
    assert!(try_lock(*key, lock));
}
