use clap::Clap;
use std::fs::read;
use rsa::RSAPublicKey;
use shared::utils;
use shared::api::{Api, UploadParameters};

#[derive(Clap)]
pub struct Host {
    #[clap(short, long)]
    host: String,
    #[clap(short('P'), long)]
    port: Option<usize>,
    #[clap(subcommand)]
    subcmd: HostSubCommand,
}

#[derive(Clap)]
enum HostSubCommand {
    Upload(Upload),
}

#[derive(Clap)]
struct Upload {
    #[clap(short, long)]
    key: String,
    path: String
}

impl Host {

    // upload binary
    pub fn update(&self, pub_key_path: &str, path: &str) {
        let bytes = read(path).unwrap();

        let pub_key_string = read(pub_key_path).unwrap();
        let pub_key = RSAPublicKey::from_pkcs1(&pub_key_string).unwrap();

        let api = Api::new(&self.host, self.port.unwrap_or(8000), false);
        api.upload_binary(UploadParameters {
            base64: base64::encode(&bytes),
            sign: utils::get_sign(&bytes, &pub_key).unwrap()
        }).unwrap();

        println!("successfully uploaded binary!");
    }

}


pub fn process(host: Host) {
    match &host.subcmd {
        HostSubCommand::Upload(args) => host.update(&args.key, &args.path)
    }
}
