use crate::{AllocAll, CallbackRef, Owns};
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::NonNull,
};

/// Calls the provided callbacks when invoking methods on `AllocRef`.
///
/// A typical use case for a `Proxy` allocator is collecting statistics. `alloc-compose` provides
/// different implementations for [`CallbackRef`][].
///
/// # Examples
///
/// ```rust
/// #![feature(allocator_api)]
///
/// use alloc_compose::{stats, CallbackRef, Proxy};
/// use std::alloc::{AllocInit, AllocRef, Global, Layout};
///
/// let counter = stats::Counter::default();
/// let mut alloc = Proxy {
///     alloc: Global,
///     callbacks: counter.by_ref(),
/// };
///
/// unsafe {
///     let memory = alloc.alloc(Layout::new::<u32>(), AllocInit::Uninitialized)?;
///     alloc.dealloc(memory.ptr, Layout::new::<u32>());
/// }
///
/// assert_eq!(counter.num_allocs(), 1);
/// assert_eq!(counter.num_deallocs(), 1);
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
///
/// If more information is needed, one can either implement `CallbackRef` itself or use a more
/// fine-grained callback:
///
/// ```rust
/// # #![feature(allocator_api)]
/// # use alloc_compose::{stats, CallbackRef, Proxy};
/// # use std::alloc::{AllocInit, AllocRef, Layout};
/// use alloc_compose::{
///     stats::{AllocInitFilter, ResultFilter},
///     Region,
/// };
///
/// let counter = stats::FilteredCounter::default();
/// let mut data = [0; 32];
/// let mut alloc = Proxy {
///     alloc: Region::new(&mut data),
///     callbacks: counter.by_ref(),
/// };
///
/// unsafe {
///     let memory = alloc.alloc(Layout::new::<u32>(), AllocInit::Uninitialized)?;
///     alloc.dealloc(memory.ptr, Layout::new::<u32>());
///
///     alloc
///         .alloc(Layout::new::<[u32; 64]>(), AllocInit::Zeroed)
///         .unwrap_err();
/// }
///
/// assert_eq!(counter.num_allocs(), 2);
/// assert_eq!(
///     counter.num_allocs_filter(AllocInitFilter::None, ResultFilter::Ok),
///     1
/// );
/// assert_eq!(
///     counter.num_allocs_filter(AllocInit::Zeroed, ResultFilter::Err),
///     1
/// );
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Proxy<A, C> {
    pub alloc: A,
    pub callbacks: C,
}

unsafe impl<A: AllocRef, C: CallbackRef> AllocRef for Proxy<A, C> {
    #[track_caller]
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        self.callbacks.before_alloc(layout, init);
        let result = self.alloc.alloc(layout, init);
        self.callbacks.after_alloc(layout, init, result);
        result
    }

    #[track_caller]
    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.callbacks.before_dealloc(ptr, layout);
        self.alloc.dealloc(ptr, layout);
        self.callbacks.after_dealloc(ptr, layout);
    }

    #[track_caller]
    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<MemoryBlock, AllocErr> {
        self.callbacks
            .before_grow(ptr, layout, new_size, placement, init);
        let result = self.alloc.grow(ptr, layout, new_size, placement, init);
        self.callbacks
            .after_grow(ptr, layout, new_size, placement, init, result);
        result
    }

    #[track_caller]
    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        self.callbacks
            .before_shrink(ptr, layout, new_size, placement);
        let result = self.alloc.shrink(ptr, layout, new_size, placement);
        self.callbacks
            .after_shrink(ptr, layout, new_size, placement, result);
        result
    }
}

impl<A: AllocAll, C: CallbackRef> AllocAll for Proxy<A, C> {
    #[track_caller]
    fn alloc_all(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        self.callbacks.before_alloc_all(layout, init);
        let result = self.alloc.alloc_all(layout, init);
        self.callbacks.after_alloc_all(layout, init, result);
        result
    }

    #[track_caller]
    fn dealloc_all(&mut self) {
        self.callbacks.before_dealloc_all();
        self.alloc.dealloc_all();
        self.callbacks.after_dealloc_all();
    }

    #[track_caller]
    #[inline]
    fn capacity(&self) -> usize {
        self.alloc.capacity()
    }

    #[track_caller]
    #[inline]
    fn capacity_left(&self) -> usize {
        self.alloc.capacity()
    }

    #[track_caller]
    #[inline]
    fn is_empty(&self) -> bool {
        self.alloc.is_empty()
    }

    #[track_caller]
    #[inline]
    fn is_full(&self) -> bool {
        self.alloc.is_full()
    }
}

impl<A: Owns, C: CallbackRef> Owns for Proxy<A, C> {
    fn owns(&self, memory: MemoryBlock) -> bool {
        self.callbacks.before_owns();
        let owns = self.alloc.owns(memory);
        self.callbacks.after_owns(owns);
        owns
    }
}