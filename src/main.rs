use std::{env, process, process::Command, thread};

fn main() {
    let uefi_path = env!("UEFI_PATH");

    let mut qemu_cmd = Command::new("qemu-system-x86_64");

    qemu_cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    #[rustfmt::skip]
    qemu_cmd.args([
        "-drive", &format!("format=raw,file={uefi_path}"),
        "-serial", "stdio",
        // "-d", "cpu_reset",
    ]);

    let debug = env::args().nth(1).unwrap_or_default() == "--debug";
    let kernel_path = env!("KERNEL_PATH");
    if debug {
        qemu_cmd.args(["-s", "-S"]);
        // println!("{qemu_cmd:?}");
        // println!("{}", kernel_path);
        let qemu = thread::spawn(move || qemu_cmd.status().unwrap());

        let mut gdb_cmd = Command::new("rust-gdb");
        gdb_cmd.args([kernel_path, "-ex", "target remote :1234"]);
        println!("{gdb_cmd:?}");
        qemu.join().expect("qemu failed");
    } else {
        println!("{uefi_path}");
        let exit_status = qemu_cmd.status().unwrap();
        process::exit(exit_status.code().unwrap_or(-1));
    }
}
