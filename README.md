# thompson_rust_os

Working through https://os.phil-opp.com/ for CS 506 at Portland State University - Summer 2023.

## Build

This project needs to be built for a bare-metal target environment. As part of the blog series,
we define a target triple in `x86_64_os.json`.

> cargo build --target x86_64_os.json

Note: This is also specified as the default for this repository, in `.cargo/config.toml`

This crate also requires the nightly channel, because it needs to rebuild the `core` library
crate for the custom target triple.

> rustup override set nightly

We use the bootloader crate, which implements a basic BIOS bootloader for us, alongside
`bootimage` which is provided by the blog creator to link our kernel to the bootloader
after compilation.

> cargo install bootimage

After that, run `cargo bootimage` to create a bootable disk image for this project.