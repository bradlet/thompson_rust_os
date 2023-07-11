# Build and Setup

-   By default Rust will build an executable that is able to run in your current system.
    -   This environment is called the "host" system.
-   When building an OS, you need to specify a "bare metal" target which has no OS dependencies at all.
    -   Also can't link the C runtime.
    -   An example bare metal target is `thumbv7em-none-eabihf`.
    -   For this project, will use a custom target describing a `x86_64` bare metal env.
