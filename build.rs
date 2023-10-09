use bootloader::DiskImageBuilder;
use std::{env, path::PathBuf};

fn main() {
    let kernel = PathBuf::from(env::var("CARGO_BIN_FILE_KERNEL").unwrap());
    let disk_builder = DiskImageBuilder::new(kernel);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join("blog_os-uefi.img");
    let bios_path = out_dir.join("blog_os-bios.img");

    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
