#![feature(allocator_api)]
#![feature(asm_sym)]
#![feature(asm_const)]
#![feature(default_alloc_error_handler)]
#![feature(naked_functions)]
#![feature(panic_info_message)]
#![feature(ptr_to_from_bits)]
#![no_std]
#![crate_name = "kernel"]

/// Macros, need to be loaded before everything else due to how rust parses
#[macro_use]
mod macros;

extern crate alloc;
extern crate bit_field;
#[macro_use]
extern crate lazy_static;
extern crate pic8259;
extern crate spin;

#[cfg(target_arch = "x86_64")]
#[path = "arch/amd64/mod.rs"]
pub mod arch;
pub mod unwind;

mod logging;
mod memory;
mod multiboot;

use self::arch::memory::PAGE_SIZE;
use self::memory::addr::VirtualAddress;

const KERNEL_BASE: usize = 0xFFFFFFFF80000000;

// Kernel entrypoint (called by arch/<foo>/start.S)
#[no_mangle]
pub extern "C" fn kmain(multiboot_ptr: usize) {
    extern "C" {
        static kernel_end: u8;
    }

    log!("Hello world! :)");
    let kend_vaddr: usize = unsafe { &kernel_end as *const _ as usize };
    let kend_phys_addr = kend_vaddr - KERNEL_BASE;
    // page align heap start
    let bootstrap_frame_alloc_start = kend_phys_addr + PAGE_SIZE - (kend_phys_addr % PAGE_SIZE);
    log!("kendvaddr: {:x}", kend_vaddr);
    memory::init(multiboot_ptr, bootstrap_frame_alloc_start, kend_vaddr);

    use alloc::vec::Vec;
    // Test the linked_list_allocator by allocating a larger size than the biggest slab.
    let mut nums: Vec<usize> = Vec::with_capacity(1024);
    for i in 0..1024 {
        nums.push(i);
    }

    arch::interrupt::init();
}
