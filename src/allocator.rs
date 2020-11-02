use alloc_cortex_m::CortexMHeap;

#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
