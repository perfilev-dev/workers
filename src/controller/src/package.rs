use clap::Clap;
use std::fs::{read, File};
use rsa::{RSAPublicKey, PaddingScheme, PublicKey};
use rand::rngs::OsRng;
use std::io::Write;

#[derive(Clap)]
pub struct Package {
    #[clap(short, long)]
    campaign: String,
    #[clap(short, long)]
    key: String,
    #[clap(short, long)]
    output: String,
    loader: String,
    executable: String,
}

fn get_overlay(command: &Package) -> Vec<u8> {
    let pub_key_string = read(&command.key).unwrap();
    let pub_key = RSAPublicKey::from_pkcs1(&pub_key_string).unwrap();

    let mut data = Vec::<u8>::new();

    // encrypt campaign
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let campaign_bytes = pub_key.encrypt(&mut OsRng, padding, &command.campaign.as_bytes()).unwrap();

    println!("length: {}", campaign_bytes.len());

    // prefixed campaign
    data.append(&mut i64::to_be_bytes(campaign_bytes.len() as i64).to_vec());
    data.append(&mut campaign_bytes.to_vec());

    // then executable
    data.append(&mut read(&command.executable).unwrap().to_vec());

    // and return data
    data
}

fn package(command: Package) {
    let mut loader_bytes = read(&command.loader).unwrap();
    loader_bytes.append(&mut get_overlay(&command));

    let mut output_file = File::create(&command.output).unwrap();
    output_file.write_all(&loader_bytes).unwrap();

    println!("successfully saved to {}", command.output);
}

pub fn process(command: Package) {
    package(command);
}