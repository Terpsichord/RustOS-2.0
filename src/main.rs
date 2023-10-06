use std::{process, process::Command};

fn main() {
    let uefi_path = env!("UEFI_PATH");

    let mut cmd = Command::new("qemu-system-x86_64");

    #[rustfmt::skip]
    cmd.args([
        "-bios", ovmf_prebuilt::ovmf_pure_efi(),
        "-drive", format!("format=raw,file={uefi_path}"),
        "-serial", "stdio",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
    ]);

    println!("{uefi_path}");
    let exit_status = cmd.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}
