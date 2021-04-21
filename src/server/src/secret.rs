use rand::rngs::OsRng;
use std::sync::Mutex;

use rsa::{PublicKey, RSAPrivateKey, RSAPublicKey, PaddingScheme};
use serde::Serialize;
use serde::de::DeserializeOwned;

use shared::error::*;

lazy_static! {
    static ref KEY: Mutex<(RSAPrivateKey, RSAPublicKey)> = {
        let mut rng = OsRng;
        let bits = 2048;
        let private_key = RSAPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let public_key = RSAPublicKey::from(&private_key);
        Mutex::new((private_key, public_key))
    };
}

pub fn encrypt<T>(data: T) -> Result<String>  where T: Serialize {
    shared::utils::encrypt(data, &KEY.lock().unwrap().1)
}

pub fn decrypt<T>(data: &str) -> Result<T> where T: DeserializeOwned {
    shared::utils::decrypt(data, &KEY.lock().unwrap().0)
}
