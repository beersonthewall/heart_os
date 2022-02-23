use super::PAGE_SIZE;
use super::frame_alloc::FrameAllocator;
use super::frame::Frame;
use super::addr::{VirtualAddress};
use super::page::Page;
use super::page_table::{PageTableEntry, Table, PTE_WRITE, PTE_PRESENT};

// Recurisve page table constants.
const P4_TABLE_BASE: VirtualAddress = VirtualAddress(0xffff_ff7f_bfdf_e000);
const P3_TABLE_BASE: VirtualAddress = VirtualAddress(0xffff_ff7f_bfc0_0000);
const P2_TABLE_BASE: VirtualAddress = VirtualAddress(0xffff_ff7f_8000_0000);
const P1_TABLE_BASE: VirtualAddress = VirtualAddress(0xffff_ff00_0000_0000);

pub struct PageMapper<'a> {
    root: &'a mut Table,
}

impl<'a> PageMapper<'a> {
    pub fn init_kernel_table() -> Self {
        Self {
            root: Table::from_virtual_address(P4_TABLE_BASE),
        }
    }

    fn print_table(table: &mut Table) {
        let mut nonzero = false;
        for i in 0..512 {
            let entry = &table[i];
            let val = entry.entry();
            if entry.is_used() {
                nonzero = true;
                log!("entry {i}: {val:x}");
            }
        }
        if !nonzero {
            log!("table empty");
        }
    }

    fn next_table(entry: &mut PageTableEntry, next: Page, alloc: &mut FrameAllocator) -> &'a mut Table {
        if !entry.is_used() {
            if let Some(frame) = alloc.allocate_frame() {
                entry.set_frame(frame, PTE_WRITE | PTE_PRESENT);
            } else {
                panic!("Failed to allocate fram for next_table.");
            }
        }

        let table = unsafe { &mut *(next.virtual_address().0 as *mut Table) };
        PageMapper::print_table(table);
        return table;
    }

    pub fn map(&mut self, page: Page, frame: Frame, alloc: &mut FrameAllocator) {
        log!("writing pml4 entry");
        let entry = &mut self.root[page.pml4_offset()];
        let pdpt_page = Page {
            page_number: (P3_TABLE_BASE | (page.pml4_offset() << 12)) / PAGE_SIZE,
        };
        let pdpt = PageMapper::next_table(entry, pdpt_page, alloc);

        log!("pml4 state...");
        PageMapper::print_table(self.root);
        log!("pdpt state..");
        PageMapper::print_table(pdpt);

        log!("writing pdpt entry");
        let entry = &mut pdpt[page.pdpt_offset()];
        let pd_page = Page {
            page_number: (P2_TABLE_BASE | (page.pml4_offset() << 21) | (page.pdpt_offset() << 12)) / PAGE_SIZE,
        };
        let pd = PageMapper::next_table(entry, pd_page, alloc);
        
        log!("writing pd entry");
        let entry = &mut pd[page.pd_offset()];
        let pt_page = Page { page_number: (P1_TABLE_BASE | (page.pml4_offset() << 30) | (page.pdpt_offset() << 20) | (page.pt_offset() << 12)) / PAGE_SIZE };
        let pt = PageMapper::next_table(entry, pt_page, alloc);

        log!("writing pt entry");
        let entry = &mut pt[page.pt_offset()];
        entry.set_frame(frame, PTE_WRITE | PTE_PRESENT);
    }
}
