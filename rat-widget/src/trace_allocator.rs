use log::debug;
use std::alloc::{GlobalAlloc, Layout, System};
use std::backtrace::Backtrace;
use std::sync::atomic::{AtomicBool, Ordering};

static DO_TRACE: AtomicBool = AtomicBool::new(false);

pub fn enable_trace_alloc() {
    DO_TRACE.store(true, Ordering::SeqCst);
}

pub fn disable_trace_alloc() {
    DO_TRACE.store(false, Ordering::SeqCst);
}

#[global_allocator]
static A: TraceAllocator = TraceAllocator;

struct TraceAllocator;

unsafe impl GlobalAlloc for TraceAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if DO_TRACE.load(Ordering::SeqCst) {
            disable_trace_alloc();
            debug!("{:#?}", Backtrace::capture());
            enable_trace_alloc();
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if DO_TRACE.load(Ordering::SeqCst) {
            disable_trace_alloc();
            debug!("{:#?}", Backtrace::capture());
            enable_trace_alloc();
        }
        unsafe { System.dealloc(ptr, layout) };
    }
}
