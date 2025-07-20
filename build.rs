use anyhow::Context;
use bootloader::DiskImageBuilder;
use std::{
    env,
    fs::{self},
    path::{Path, PathBuf},
};
use vsfs::{CreateFileOptions, ROOT_UID};

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

fn create_ram_disk(out_dir: &Path) -> anyhow::Result<PathBuf> {
    let ram_disk_path = out_dir.join("myos-ram-disk.img");
    let mut ram_disk_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ram_disk_path)
        .context("Failed to create ram disk file")?;

    let inode_count = 100;
    let data_block_count = 100;
    let format_options = vsfs::FormatVolumeOptions::new(inode_count, data_block_count);
    vsfs::format_volume(&mut ram_disk_file, format_options).expect("failed to format volume");

    let fs = vsfs::Vsfs::new(&mut ram_disk_file, vsfs::FsOptions::new()).unwrap();
    let mut root_dir = fs.root_dir();
    let mut file = root_dir
        .create_file(CreateFileOptions {
            file_name: "hello.txt",
            uid: ROOT_UID,
            gid: ROOT_UID,
            mode: 0o755,
        })
        .unwrap();
    file.write_all(b"Hello World!").unwrap();
    Ok(ram_disk_path)
}
