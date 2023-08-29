# 1 - Freestanding Rust Binary

-   Need to tell the Rust compiler to not include the standard library because it is included by default.
-   By default the Rust compiler will encode function names with random characters and numbers for the sake
    of generating a unique function ID (Called "mangling").
    -   Will do this unless we tell it not to with `#[no_mangle]``
-   We will use `extern "C"` to specify the C calling convention instead of the Rust
    calling convention.
    -   C calling convention is stack-centric: subroutine params, registers, local vars
        all placed in memory on the stack.
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
        -   Need to use the Rust nightly build to access an unstable feature to re-build `core` for our target system.
        -   To use nightly, `rustup override set nightly`
-   Starting off on the OS, easiest way to print to screen is using the VGA text buffer.
    -   Special memory area mapped to VGA hardware so that it's contents are displayed on the screen.
    -   Memory contains 25 lines, each 80 character cells wide, where each cell contains an ASCII character, and a byte for color.
    -   Buffer located at `0xb8000`

# 3 - VGA Text Mode

-                                             To write characters to the screen in VGA Text Mode, just need to write data to the VGA buffer which is a 2d array.
    -   Memory layout: bit 0-7 = ASCII character; 8-11 foreground color; 12-14 = background color; 15 = blink.
    -   Apparently the ascii character isn't actually normal ASCII but a slightly altered character set called "code page 937"
    -                                             This uses [Memory Mapped I/O](https://en.wikipedia.org/wiki/Memory-mapped_I/O_and_port-mapped_I/O) as a way to allow
        the CPU to communicate to peripheral devices.
        -   I/O devices can monitor the memory location of the CPU's address bus and respond when the CPU accesses an address
            assigned to that bus.
        -   These reads and writes don't interact with RAM; directly access the text buffer on the VGA hardware.

# 4 - Testing

Didn't feel the need to take more notes than what's included in inline comments...

# 5 - CPU Exceptions

-   We set up an "interrupt descriptor table" to map certain CPU exceptions to specific exception handlers.
-   [See the OS Dev Wiki for all ~20 CPU exceptions on x86](https://wiki.osdev.org/Exceptions). Some examples...
    -   Page Fault: Illegal memory access; e.g. reading from unmapped memory, writing to read-only memory.
    -   Invalid opcode: unsupported instruction encountered by the CPU
    -   General Protection Fault: Many causes; e.g. executing a privileged instruction in user space, writing reserved fields
        in configuration registers.
    -                                         Double Fault: When handling an exception, if there is an exception thrown during the exception handler's execution, CPU
        raises a double fault exception.
    -   Triple Fault: If an exception occurs while the CPU tries to call the double fault handler, it issues a triple fault,
        which typically results in a system restart.

---

## Interrupt Descriptor Tables

Hardware interacts with this directly, so we have to create it following a specific preset format. Following 16 byte structure:

-   u16 --> Function Pointer: Pointer to handler function (lower bits)
-   u16 --> GDT Selector selector for code segement in [Global Descriptor Table](https://en.wikipedia.org/wiki/Global_Descriptor_Table)
-   u16 --> Options
-   u16 --> Function pointer (middle bits)
-   u32 --> Function pointer (remaining)
-   u32 --> Reserved

---

When an exception occurs the CPU:

1. Push some registers onto the stack (e.g. instruction ptr)
2. Read a corresponding entry from the IDT
3. Check if the entry is present, if not, raise double fault.
4. "Disable hardware interrupts if the entry is an interrupt gate" - ?
5. Load the Global Descriptor Table selector into the code segment.
6. Jump to the handler function pointed to by the IDT.

-   `HandlerFunc` in Rust is a type alias for an

```
extern "x86_interrupt" fn
```

-   `extern keyword` defines a function with a foreign calling convention.
    -   Often used for C code w/ `extern "C" fn`.
        -   **C calling convention** (spec'd in [System V ABI](https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf)):
            -   The first six integer arguments are passed in registers rdi, rsi, rdx, rcx, r8, r9
            -   Additional arguments are passed on the stack
            -   Results are returned in rax and rdx
-   Commonly this convention splits registers into preserved and scratch.
    -   Preserved registers are saved to the stack at the beginning of the
        function, and then restored when the function finishes execution.
    -   Scratch registers (caller-saved) can be overwritten, so to save them the invoker of the
        function needs to push these values to the stack and restore them manually.
-   Interrupt calling convention is similar to regular fn convention, but
    regular functions is invoked voluntarily a compiler-inserted `call` instruction.
    -   Interrupt exception can occur on any instruction. Compilers can't know if an instruction will cause
        a stack overflow or page fault.
    -   Since exceptions happen at any time, we can't backup registers before they occur.
    -   So we can't use a calling convention that relies on caller-saved registers -- we need one that
        saves all registers.
    -   "x86-interrupt" calling convention backs up all registers overwritten by the function (Doesn't
        mean it is saved on the stack).
    -   Read through [this part on the interrupt stack frame again](https://os.phil-opp.com/cpu-exceptions/#the-interrupt-stack-frame)
-   x86-interrupt convention handles a lot of the minutia of exception handling
    -   Doesn't retrieve arguments from the stack, always retrieves them from the stack at some offset.
    -   Handles a special return since exception handler stack frames differs from normal functions: `iretq` instead of `ret`
    -   Handles the optional error code that some exceptions include.
    -   Handles stack reallignment in-case of byte boundary disparities (16-byte alignment expected in some instructions).
    -   Check out [these posts](https://os.phil-opp.com/edition-1/extra/naked-exceptions/) to learn more about
        making exception handling work without `x86-interrupt` carrying so much of the load for us.

# 6 - Double Faults

-   Occur when the CPU fails to invoke an exception handler.
    -   Because one is not registered, or otherwise fails to execute.
    -   This is a simplification, techincally there are specific sequences of exceptions
        that result in a double fault. For example, unhandled exceptions occur when the CPU
        finds no entry in the IDT for a given exception; the actual exception thrown is a
        "General Protection Exception" -- if that is unhandled as well another is thrown,
        which results in a double fault. Check out this blog post for a reference on the
        double faulting sequences.
    -   Another example: There exists a "guard page" which lies at the bottom of the stack.
        It's used to detect stack overflows. It is never mapped to physical memory, so
        accessing it always results in a page fault. When the CPU encounters this, it tries
        to lookup the handler in the IDT and then push the interrupt stack frame onto the
        stack. But since the stack pointer starts out pointing at the guard page, this
        results in another page fault. 2x page faults is another cause for double fault.
-   Important to handle these to avoid triple faults, resulting in an automatic system reset.
-   In x86_64, double fault handlers are diverging; no return value permitted.
-   Example shows us writing to an invalid address -- since the address isn't mapped to a
    physical address in the CPU's page tables, a page fault occurs. If unhandled this results
    in a double fault.

```rust
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    blog_os::init();

    // trigger a page fault
    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    };

    // as before
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}
```

-   Some double faults (see examples above) can't be recovered with our standard
    double fault handler alone, because the stack pointer constantly points at the
    guard page at the bottom of the stack.
-   In these cases, x86_64 supports switching to another stack that we know is
    valid, while we handle the exception.
-   This is a hardware-level operation, so it can occur before the CPU pushes the
    exception stack frame.
    -   Implemented as an Interrupt Stack Table (IST); a table of 7 pointers to
        known-to-be-good stacks.
    -   Each entry in our IDT can point to whatever stack it wants using its
        `stack_pointers` field.
-	The Interrupt Stack Table is held in the [Task State Segment](https://en.wikipedia.org/wiki/Task_state_segment)
	which used to do things (e.g. hardware context switching) but now just holds
	onto some addresses essentially.
