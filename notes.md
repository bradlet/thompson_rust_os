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
    - Basic input/output system (BIOS) - Simple, old, well-supported.
    - Universal Extensible Firmware Interface (UEFI) - Modern, much more features, harder to setup.
