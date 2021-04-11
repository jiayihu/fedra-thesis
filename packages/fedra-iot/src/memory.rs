use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    log::error!("Heap used {}, free {}", ALLOCATOR.used(), ALLOCATOR.free());

    panic!("Allocation error: {:?}", layout);
}

pub fn setup_heap() {
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1024 * (128 - 8); // Reserve 8KB for the stack, of which 4KB is the WASM value stack

    unsafe {
        ALLOCATOR.init(start, size);
    }
}

pub fn stats() {
    log::info!("Heap used {}, free {}", ALLOCATOR.used(), ALLOCATOR.free());
}
