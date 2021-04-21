use clap::Clap;
use rsa::{PublicKey, RSAPrivateKey, RSAPublicKey, PaddingScheme, PublicKeyPemEncoding, PrivateKeyPemEncoding, PublicKeyEncoding, PrivateKeyEncoding};
use std::fs::{File, read_to_string, read};
use std::io::{Write, Read};
use rand::rngs::OsRng;
use shared::utils;
use shared::utils::verify_file_sign;

#[derive(Clap)]
pub struct Pki {
    #[clap(subcommand)]
    subcmd: PkiSubCommand,
}

#[derive(Clap)]
enum PkiSubCommand {
    GenRsa(GenRsa),
    Sign(Sign),
    Verify(Verify)
}

#[derive(Clap)]
struct GenRsa {
    path: String,
}

#[derive(Clap)]
struct Sign {
    pub_key_path: String,
    path: String,
}

#[derive(Clap)]
struct Verify {
    private_key_path: String,
    path: String,
    sign: String
}

/// Generates RSA keys in PKCS format
fn gen_rsa(path: &str) {
    let mut private_key_path = path.to_string();
    if private_key_path.ends_with("/") {
        println!("specify path to private key!");
        return;
    }

    let pub_key_path = format!("{}.pub", private_key_path);

    let mut rng = OsRng;
    let bits = 2048;
    let private_key = RSAPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RSAPublicKey::from(&private_key);

    // store to files
    let mut private_key_file = File::create(private_key_path).unwrap();
    private_key_file.write_all(&private_key.to_pkcs8().unwrap()).unwrap();

    // also store key.pem
    private_key_file = File::create(format!("{}.pem", path)).unwrap();
    private_key_file.write_all(&private_key.to_pem_pkcs8().unwrap().as_bytes()).unwrap();

    let mut public_key_file = File::create(pub_key_path).unwrap();
    public_key_file.write_all(&public_key.to_pkcs1().unwrap()).unwrap();
}


/// Sign binary using public key
fn sign(pub_key_path: &str, path: &str) {
    let pub_key_string = read(pub_key_path).unwrap();
    let pub_key = RSAPublicKey::from_pkcs1(&pub_key_string).unwrap();

    // actually used for debug
    println!("sha256: {}", hex::encode(utils::sha256(&read(path).unwrap())));
    println!("sign: {}", utils::get_file_sign(path, &pub_key).unwrap());
}


fn verify(private_key_path: &str, path: &str, sign: &str) {
    let private_key_string = read(private_key_path).unwrap();
    let private_key = RSAPrivateKey::from_pkcs8(&private_key_string).unwrap();

    // ...
    println!("sha256: {}", hex::encode(utils::sha256(&read(path).unwrap())));
    println!("is sign ok? {}", utils::verify_file_sign(path, sign, &private_key).unwrap());
}


pub fn process(command: Pki) {
    match command.subcmd {
        PkiSubCommand::GenRsa(args) => gen_rsa(&args.path),
        PkiSubCommand::Sign(args) => sign(&args.pub_key_path, &args.path),
        PkiSubCommand::Verify(args) => verify(&args.private_key_path, &args.path, &args.sign)
    }
}
