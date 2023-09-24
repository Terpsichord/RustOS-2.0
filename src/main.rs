use std::{process, process::Command};

fn main() {
    let uefi_path = env!("UEFI_PATH");

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-bios")
        .arg(ovmf_prebuilt::ovmf_pure_efi())
        .arg("-drive")
        .arg(format!("format=raw,file={uefi_path}"));

    println!("{uefi_path}");
    let exit_status = cmd.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}