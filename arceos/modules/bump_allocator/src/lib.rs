#![no_std]

use core::ptr::NonNull;
use core::{alloc::Layout, usize};

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,

    byte_next: usize,
    byte_count: usize,

    page_next: usize,
    page_count: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            byte_next: 0,
            byte_count: 0,
            page_next: 0,
            page_count: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;

        self.byte_count = 0;
        self.byte_next = start;
        self.page_next = self.end;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        todo!()
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let start = align_up(self.byte_next, layout.align());
        let end = start + layout.size();
        // memory is not enough
        if end > self.page_next {
            Err(AllocError::NoMemory)
        } else {
            // alloc byte
            self.byte_count += 1;
            self.byte_next = end;
            NonNull::new(start as *mut u8).ok_or(AllocError::NoMemory)
        }
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        self.byte_count -= 1;
        if self.byte_count == 0 {
            self.byte_next = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.byte_next - self.start
    }

    fn available_bytes(&self) -> usize {
        self.page_next - self.byte_next
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        let start = align_down(self.page_next - (Self::PAGE_SIZE * num_pages), align_pow2);

        // memory not enough
        if start < self.byte_next {
            Err(AllocError::NoMemory)
        } else {
            self.page_count += 1;
            self.page_next = start;
            Ok(start)
        }
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        self.page_count -= 1;
        if self.page_count == 0 {
            self.page_next = self.end;
        }
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        self.page_count
    }

    fn available_pages(&self) -> usize {
        (self.page_next - self.byte_next) / Self::PAGE_SIZE
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}
