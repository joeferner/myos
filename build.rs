use anyhow::Context;
use bootloader::DiskImageBuilder;
use fatfs::{FormatVolumeOptions, format_volume};
use std::{
    env,
    fs::{self},
    io::Write,
    path::PathBuf,
};

fn main() {
    // set by cargo for the kernel artifact dependency
    let kernel_path = env::var("CARGO_BIN_FILE_KERNEL").unwrap();
    let mut disk_builder = DiskImageBuilder::new(PathBuf::from(kernel_path));

    // specify output paths
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join("myos-uefi.img");

    // create ram disk
    let ram_disk_path = create_ram_disk(&out_dir).unwrap();

    // create the disk images
    disk_builder.set_ramdisk(ram_disk_path);
    disk_builder.create_uefi_image(&uefi_path).unwrap();

    // pass the disk image paths via environment variables
    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
}

fn create_ram_disk(out_dir: &PathBuf) -> anyhow::Result<PathBuf> {
    // TODO switch to ext4
    const MB: u64 = 1024 * 1024;
    let size = 10 * 1024 * 1024; // TODO calculate fat size needed
    let size_padded_and_rounded = ((size + 1024 * 64 - 1) / MB + 1) * MB;

    let ram_disk_path = out_dir.join("myos-ram-disk.img");
    let ram_disk_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ram_disk_path)
        .context("Failed to create ram disk FAT file")?;
    ram_disk_file
        .set_len(size_padded_and_rounded)
        .context("failed to set ram disk file length")?;

    let format_options = FormatVolumeOptions::new().volume_label(*b"MYOS-BOOT  ");
    format_volume(&ram_disk_file, format_options).unwrap();

    let fs = fatfs::FileSystem::new(ram_disk_file, fatfs::FsOptions::new()).unwrap();
    let root_dir = fs.root_dir();
    let mut file = root_dir.create_file("hello.txt").unwrap();
    file.write_all(b"Hello World!").unwrap();
    Ok(ram_disk_path)
}
