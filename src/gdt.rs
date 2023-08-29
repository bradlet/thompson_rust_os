//! gdt.rs
//! Home for our Global Descriptor Table implementation, including the
//! Task State Segment. GDT is a relic, now used just for switching
//! between user & kernel space, and for loading the TSS.
//! For more information on segmentation, see chapter 16 of the OSTEP book.

use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

struct GlobalDescriptorWrapper {
	table: GlobalDescriptorTable,
	selectors: Selectors
}

lazy_static! {
	// Build out our interrupt stack table with a double fault stack impl.
	// Note: no guard page in this stack, so can't do anything stack
	// intensive in our double fault handler.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
			// Need to use this b/c we haven't implemeneted memory management.
			// If this was not mut, bootloader would map to a read-only page.
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

			// This needs to be unsafe b/c of the static mut access.
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: GlobalDescriptorWrapper = {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

		GlobalDescriptorWrapper {
			table: gdt,
			selectors: Selectors { code_selector, tss_selector }
		}
    };
}

/// Provides a clean interface to initialize a GDT hiding unsafe
/// operations and other lib access from clients.
pub fn init() {
	use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    GDT.table.load();
	unsafe {
		// Update the code segment register, so it points
		// at our new double fault stack.
        CS::set_reg(GDT.selectors.code_selector);
		// Load our Task Segment Selector so that the IDT can use
		// it to point to the double fault stack.
        load_tss(GDT.selectors.tss_selector);
    }
}