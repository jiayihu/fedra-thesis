use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

pub fn setup_heap() {
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024 * (128 - 16); // Reserve okay 16KB for the stack

    unsafe {
        ALLOCATOR.init(start, size);
    }
}
