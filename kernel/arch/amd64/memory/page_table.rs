use crate::memory::addr::{PhysicalAddress, VirtualAddress};
use crate::memory::frame::Frame;
use core::ops::{Index, IndexMut};

pub const PTE_PRESENT: u64 = 1;
pub const PTE_WRITE: u64 = 1 << 1;

#[derive(Debug)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
    pub fn is_used(&self) -> bool {
        self.0 != 0
    }

    pub fn set_frame(&mut self, frame: Frame, options: u64) {
        self.0 = frame.physical_address() | options;
    }

    pub fn entry(&self) -> u64 {
        self.0
    }

    pub fn frame(&self) -> Frame {
        Frame::from_physical_address(PhysicalAddress::new(self.0 as usize))
    }
}

const TABLE_SIZE: usize = 512;

#[repr(align(4096), C)]
pub struct Table {
    entries: [PageTableEntry; TABLE_SIZE],
}

impl Table {
    pub fn from_virtual_address<'a>(address: VirtualAddress) -> &'a mut Table {
        return unsafe { &mut *(address.0 as *mut Table) };
    }
}

impl Index<usize> for Table {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for Table {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}
