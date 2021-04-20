use rand::{thread_rng, Rng, RngCore};
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};


#[derive(Serialize, Deserialize)]
pub struct Challenge {
    pub bytes: String,
    pub nonce: i32
}

impl Challenge {

    pub fn new() -> Challenge {
        Challenge {
            nonce: thread_rng().next_u32() as i32,
            bytes: thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect()
        }
    }

    pub fn check(&self, solution: i32) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&self.bytes);
        hasher.update(i32::to_be_bytes(solution));
        let result = hasher.finalize();
        if cfg!(debug_assertions) {
            result.ends_with(&i8::to_be_bytes(self.nonce as i8))
        }
        else {
            result.ends_with(&i32::to_be_bytes(self.nonce))
        }
    }

}
