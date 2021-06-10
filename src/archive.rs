use std::fs::File;
use std::io::Read;

use zip::read::ZipArchive;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Firmware {
    bin_file: String,
    dat_file: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ManifestField {
    bootloader: Option<Firmware>,
    softdevice_bootloader: Option<Firmware>,
    application: Option<Firmware>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Manifest {
    manifest: ManifestField,
}

pub struct FirmwareData {
    pub bin: Vec<u8>,
    pub dat: Vec<u8>,
}

pub struct FirmwareArchive {
    pub bootloader: Option<FirmwareData>,
    pub softdevice_bootloader: Option<FirmwareData>,
    pub application: Option<FirmwareData>,
}

impl FirmwareArchive {
    pub fn new(path: &str) -> FirmwareArchive {
        let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();
        let mut manifest_data = String::new();

        {
            let mut manifest = archive.by_name("manifest.json").unwrap();
            manifest.read_to_string(&mut manifest_data).unwrap();
        }

        let Manifest {
            manifest:
                ManifestField {
                    bootloader,
                    softdevice_bootloader,
                    application,
                },
        } = serde_json::from_str(&manifest_data).unwrap();

        let mut extract_data = |Firmware { bin_file, dat_file }| {
            let mut bin = vec![];
            {
                let mut bin_file = archive.by_name(&bin_file).unwrap();
                bin_file.read_to_end(&mut bin).unwrap();
            }

            let mut dat = vec![];
            {
                let mut dat_file = archive.by_name(&dat_file).unwrap();
                dat_file.read_to_end(&mut dat).unwrap();
            }

            FirmwareData { bin, dat }
        };

        FirmwareArchive {
            bootloader: bootloader.map(&mut extract_data),
            softdevice_bootloader: softdevice_bootloader.map(&mut extract_data),
            application: application.map(&mut extract_data),
        }
    }
}
