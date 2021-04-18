use sha2::{Sha256, Sha512, Digest};

fn main() {
    let find = b"\xAB\xAB\xAB\xAB";
    let s64 = b"sha256 ends with AA:AA:AA:AA if ";

    println!("{}", u32::from_be_bytes(find.to_owned()));

    return;

    let mut i = 0;
    loop {

        // create a Sha256 object
        let mut hasher = Sha256::new();

        // write input message
        hasher.update(s64);

        // ...
        hasher.update(i32::to_be_bytes(i));

        // ...
        let result = hasher.finalize();

        // ...
        if result.ends_with(find) {
            println!("found for {} retries! sha256({:?} + {:?}) endswith BB:BB:BB:BB", i, s64, i32::to_be_bytes(i));
            break;
        }

        if i % 10_000_000 == 0 {
            println!("{}", i);
        }

        i += 1;
    }
}
