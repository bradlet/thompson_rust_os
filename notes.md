# 1 - Freestanding Rust Binary

-   Need to tell the Rust compiler to not include the standard library because it is included by default.
-   Main is called as the last step of execution; starts in the C runtime library "crt0", then the Rust runtime, then main is called.
    -   We want to overwrite this entrypoint because we won't be using either of those runtimes.
-   By default Rust will build an executable that is able to run in your current system.
    -   This environment is called the "host" system.
-   When building an OS, you need to specify a "bare metal" target which has no OS dependencies at all.
    -   Also can't link the C runtime.
    -   An example bare metal target is `thumbv7em-none-eabihf`.
    -   For this project, will use a custom target describing a `x86_64` bare metal env.

# 2 - Minimal Rust Kernel

-   When powering on, a computer will execute firmware code in ROM (read only memory). Runs POST (power on self tests),
    detects available RAM, and pre-initializes CPU and other hardware, then boots the operating system kernel.
-   x86 has two firmware standards:
    -   Basic input/output system (BIOS) - Simple, old, well-supported (any x86 machine since the 80's).
    -   Universal Extensible Firmware Interface (UEFI) - Modern, much more features, harder to setup.
-   BIOS booting puts CPU into 16-bit compatability mode "real mode" before booting the OS for backwards compatability to the 80's
-   BIOS startup sequence:
    -   Read BIOS firmware from special flash memory on the motherboard.
    -   Run hardware's POST (self-test) and init routines.
    -   Looks for a bootable disk.
    -   If found, control goes to the disk's bootloader; a 512-byte segment of executable code at the beginning of the disk.
        -   Most bootloaders are larger than 512B, so they are broken into later stages which are loaded by the first stage.
    -   Bootloader needs to find the kernel image location on disk, switch CPU from 16b real mode -> 32b protected mode -> 64b long mode.
        -   Once in long mode, 64 bit registers and complete main memory are available.
    -   Last, bootloader needs to query information (e.g. map of memory) from BIOS and pass it to the OS kernel.
    -   Bootloaders need to be written in assembly (not part of this project
        -- using [bootimage](https://github.com/rust-osdev/bootimage) which prepends a bootloader on our kernel for us)
-   The Free Software Foundation created an open bootloader standard called Multiboot so that every OS wouldn't implement it's
    own bootloader, only compatible with that OS.
    -   Not used in this project b/c of subsequently listed drawbacks...
    -   Any compliant bootloader can load any compliant OS.
    -   [GNU GRUB](https://en.wikipedia.org/wiki/GNU_GRUB) is the most popular on Linux systems.
        -   Only need to prepend a multiboot header at the start of the kernel file to boot an OS with GRUB; issues:
            -   Only supports protected mode (32 bit); you need to configure the switch to long mode.
            -   Lots of architecture specific stuff, like an altered default page size, and architecture-dependent boot info
                gets passed to the kernel.
            -   Both poorly documented.
            -   GRUB needs to be on the system, so Windows and Mac dev is tough.
-   We can describe our own `target triple` using a json file which specifically defines the target system architecture.
    -   See `x86_64_os.json`
    -   Define many things like the Endian-ness for the target system, how to handle stack unwinding (panic-strategy abort means
        we don't do stack unwinding), etc.
    -   Compiler features disabled or enabled in the `features` field using '+' or '-' prefixes.
    -   `mmx` and `sse` determine if the system support Single Instruction Multiple Data instructions
        -   Using SIMD registers causing performance problems b/c the OS needs to restore all registers to their OG state when
            continuing an interrupted program.
        -   SIMD state is 512-1600 bytes, all needs to be realoded to main memory for each syscall or hardware interrupt.
    -   Need to enable `soft-float` b/c x86_64 systems have a dependency on SIMD for floating-point operations -- this tells
        the system to emulate floating point operations through software functions.
    -   The Rust `core` library is distributed w/ the compiler as a precompiled library, so it only supports valid host triples.
        So `core` needs to be recompiled for our custom target triple.
