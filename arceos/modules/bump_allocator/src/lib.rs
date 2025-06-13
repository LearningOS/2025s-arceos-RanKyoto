#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator,AllocResult,AllocError};
use core::alloc::Layout;
use core::ptr::NonNull;

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

/// 用作早期启动（boot）时的内存分配
/// 常量泛型 PAGE_SIZE 的作用是在编译阶段就知道页的大小
pub struct EarlyAllocator<const PAGE_SIZE:usize>{
    start:usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize, //记录分配了几次字节
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    //常量函数，在编译阶段就求值（完成初始化）
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0
        }
    }
}
/// trait BaseAllocator需要实现init()和add_memory()
impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize){
        self.start = start; //初始化后不变
        self.end = start + size; //初始化后不变
        self.b_pos = start;//随着字节分配变化
        self.p_pos = self.end;//随着页分配变化
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult{
        unimplemented!()
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    /// Allocate memory with the given size (in bytes) and alignment.
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>>{
        let align = layout.align();//一定是 2的幂
        self.b_pos = (self.b_pos + align - 1) & !(align - 1); // 将b_pos向上对齐到align
        let res = unsafe { NonNull::new_unchecked(self.b_pos as *mut u8) };
        self.b_pos += layout.size();
        self.count += 1;
        Ok(res)
    }

    /// Deallocate memory at the given position, size, and alignment.
    fn dealloc(&mut self, _pos: NonNull<u8>, _layout: Layout){
        self.count -= 1;
        if self.count == 0 {//按照要求，只有降到 0 才清空已分配的字节
            self.b_pos = self.start;
        }
    }

    /// Returns total memory size in bytes.
    fn total_bytes(&self) -> usize{
        self.end - self.start
    }

    /// Returns allocated memory size in bytes.
    fn used_bytes(&self) -> usize{
        self.b_pos - self.start
    }
    
    /// Returns available memory size in bytes.
    fn available_bytes(&self) -> usize{
        self.p_pos - self.b_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    /// The size of a memory page.
    const PAGE_SIZE: usize = PAGE_SIZE;

    /// Allocate contiguous memory pages with given count and alignment.
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize>{
        let size = num_pages * Self::PAGE_SIZE;
        let align = 1 << align_pow2;
        self.p_pos = (self.p_pos - size) & !(align - 1);//向下对齐

        // 分配失败检查（确保不会和 b_pos 冲突）
        if self.p_pos < self.b_pos {
            return Err(AllocError::MemoryOverlap); 
        }
        Ok(self.p_pos)
    }

    /// Deallocate contiguous memory pages with given position and count.
    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize){
        unimplemented!()
    }

    /// Returns the total number of memory pages.
    fn total_pages(&self) -> usize{
        (self.end - self.start) / Self::PAGE_SIZE
    }

    /// Returns the number of allocated memory pages.
    fn used_pages(&self) -> usize{
        (self.end - self.p_pos) / Self::PAGE_SIZE
    }

    /// Returns the number of available memory pages.
    fn available_pages(&self) -> usize{
        (self.p_pos - self.b_pos) / Self::PAGE_SIZE
    }

}
