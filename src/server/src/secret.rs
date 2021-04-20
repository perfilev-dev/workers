use rand::rngs::OsRng;
use std::sync::Mutex;

use rsa::{PublicKey, RSAPrivateKey, RSAPublicKey, PaddingScheme};
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;

use crate::error::*;

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
    let serialized = serde_json::to_string(&data)?;
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let encrypted = KEY.lock().unwrap().1.encrypt(&mut OsRng, padding, serialized.as_bytes())?;
    Ok(base64::encode(encrypted))
}

pub fn decrypt<T>(data: &str) -> Result<T> where T: DeserializeOwned {
    let decoded = base64::decode(data)?;
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let decrypted =  KEY.lock().unwrap().0.decrypt(padding, &decoded)?;
    Ok(serde_json::from_str(&String::from_utf8(decrypted)?)?)
}
