use crate::{grow, Owns};
use core::{
    alloc::{AllocErr, AllocInit, AllocRef, Layout, MemoryBlock, ReallocPlacement},
    ptr::NonNull,
};

/// An allocator equivalent of an "or" operator in algebra.
///
/// An allocation request is first attempted with the `Primary` allocator. If that fails, the
/// request is forwarded to the `Fallback` allocator. All other requests are dispatched
/// appropriately to one of the two allocators.
///
/// A `FallbackAlloc` is useful for fast, special-purpose allocators backed up by general-purpose
/// allocators like [`Global`] or [`System`].
///
/// [`Global`]: https://doc.rust-lang.org/alloc/alloc/struct.Global.html
/// [`System`]: https://doc.rust-lang.org/std/alloc/struct.System.html
///
/// # Example
///
/// ```rust
/// #![feature(allocator_api)]
///
/// use alloc_compose::{FallbackAlloc, Owns, Region};
/// use std::alloc::{AllocInit, AllocRef, Layout, System};
///
/// let mut data = [0; 32];
/// let mut alloc = FallbackAlloc {
///     primary: Region::new(&mut data),
///     fallback: System,
/// };
///
/// let small_memory = alloc.alloc(Layout::new::<u32>(), AllocInit::Uninitialized)?;
/// let big_memory = alloc.alloc(Layout::new::<[u32; 64]>(), AllocInit::Uninitialized)?;
///
/// assert!(alloc.primary.owns(small_memory));
/// assert!(!alloc.primary.owns(big_memory));
///
/// unsafe {
///     // `big_memory` was allocated from `System`, we can dealloc it directly
///     System.dealloc(big_memory.ptr, Layout::new::<[u32; 64]>());
///     alloc.dealloc(small_memory.ptr, Layout::new::<u32>());
/// };
/// # Ok::<(), core::alloc::AllocErr>(())
/// ```
#[derive(Debug, Copy, Clone)]
pub struct FallbackAlloc<Primary, Fallback> {
    /// The primary allocator
    pub primary: Primary,
    /// The fallback allocator
    pub fallback: Fallback,
}

unsafe impl<Primary, Fallback> AllocRef for FallbackAlloc<Primary, Fallback>
where
    Primary: AllocRef + Owns,
    Fallback: AllocRef,
{
    fn alloc(&mut self, layout: Layout, init: AllocInit) -> Result<MemoryBlock, AllocErr> {
        match self.primary.alloc(layout, init) {
            primary @ Ok(_) => primary,
            Err(_) => self.fallback.alloc(layout, init),
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if self.primary.owns(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            self.primary.dealloc(ptr, layout)
        } else {
            self.fallback.dealloc(ptr, layout)
        }
    }

    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit,
    ) -> Result<MemoryBlock, AllocErr> {
        if self.primary.owns(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            if let Ok(memory) = self.primary.grow(ptr, layout, new_size, placement, init) {
                Ok(memory)
            } else {
                grow(
                    &mut self.primary,
                    &mut self.fallback,
                    ptr,
                    layout,
                    new_size,
                    placement,
                    init,
                )
            }
        } else {
            self.fallback.grow(ptr, layout, new_size, placement, init)
        }
    }

    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
    ) -> Result<MemoryBlock, AllocErr> {
        if self.primary.owns(MemoryBlock {
            ptr,
            size: layout.size(),
        }) {
            self.primary.shrink(ptr, layout, new_size, placement)
        } else {
            self.fallback.shrink(ptr, layout, new_size, placement)
        }
    }
}

impl<Primary, Fallback> Owns for FallbackAlloc<Primary, Fallback>
where
    Primary: Owns,
    Fallback: Owns,
{
    fn owns(&self, memory: MemoryBlock) -> bool {
        self.primary.owns(memory) || self.fallback.owns(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::FallbackAlloc;
    use crate::{helper, ChunkAlloc, Owns, Region};
    use std::alloc::{AllocInit, AllocRef, Layout, ReallocPlacement, System};

    #[test]
    fn alloc() {
        let mut data = [0; 32];
        let mut alloc = FallbackAlloc {
            primary: helper::tracker(Region::new(&mut data)),
            fallback: helper::tracker(System),
        };

        let small_memory = alloc
            .alloc(Layout::new::<u32>(), AllocInit::Uninitialized)
            .expect("Could not allocate 4 bytes");
        let big_memory = alloc
            .alloc(Layout::new::<[u8; 64]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 64 bytes");

        assert!(alloc.primary.owns(small_memory));
        assert!(!alloc.primary.owns(big_memory));
        unsafe {
            alloc.dealloc(small_memory.ptr, Layout::new::<u32>());
            alloc.dealloc(big_memory.ptr, Layout::new::<[u8; 64]>());
        };
    }

    #[test]
    fn grow() {
        let mut data = [0; 80];
        let mut alloc = FallbackAlloc {
            primary: helper::tracker(Region::new(&mut data)),
            fallback: helper::tracker(System),
        };

        let memory = alloc
            .alloc(Layout::new::<[u8; 32]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 32 bytes");
        assert!(alloc.primary.owns(memory));

        unsafe {
            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 32]>(),
                    64,
                    ReallocPlacement::InPlace,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 64 bytes");
            assert!(alloc.primary.owns(memory));

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 64]>(),
                    80,
                    ReallocPlacement::InPlace,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 80 bytes");
            assert!(alloc.primary.owns(memory));

            assert!(
                alloc
                    .grow(
                        memory.ptr,
                        Layout::new::<[u8; 80]>(),
                        96,
                        ReallocPlacement::InPlace,
                        AllocInit::Uninitialized,
                    )
                    .is_err()
            );

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 80]>(),
                    96,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 96 bytes");
            assert!(!alloc.primary.owns(memory));

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 96]>(),
                    128,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 128 bytes");
            assert!(!alloc.primary.owns(memory));

            alloc.dealloc(memory.ptr, Layout::new::<[u8; 128]>());
        };
    }

    #[test]
    fn shrink() {
        let mut data = [0; 80];
        let mut alloc = FallbackAlloc {
            primary: helper::tracker(Region::new(&mut data)),
            fallback: helper::tracker(System),
        };

        let memory = alloc
            .alloc(Layout::new::<[u8; 64]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 64 bytes");
        assert!(alloc.primary.owns(memory));

        unsafe {
            let memory = alloc
                .shrink(
                    memory.ptr,
                    Layout::new::<[u8; 64]>(),
                    32,
                    ReallocPlacement::MayMove,
                )
                .expect("Could not shrink to 32 bytes");
            assert!(alloc.primary.owns(memory));

            let memory = alloc
                .grow(
                    memory.ptr,
                    Layout::new::<[u8; 32]>(),
                    128,
                    ReallocPlacement::MayMove,
                    AllocInit::Uninitialized,
                )
                .expect("Could not grow to 128 bytes");
            assert!(!alloc.primary.owns(memory));

            let memory = alloc
                .shrink(
                    memory.ptr,
                    Layout::new::<[u8; 128]>(),
                    96,
                    ReallocPlacement::MayMove,
                )
                .expect("Could not shrink to 96 bytes");
            assert!(!alloc.primary.owns(memory));

            alloc.dealloc(memory.ptr, Layout::new::<[u8; 96]>());
        }
    }

    #[test]
    fn owns() {
        let mut data_1 = [0; 32];
        let mut data_2 = [0; 64];
        let mut alloc = FallbackAlloc {
            primary: Region::new(&mut data_1),
            fallback: Region::new(&mut data_2),
        };

        let memory = alloc
            .alloc(Layout::new::<[u8; 32]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 32 bytes");
        assert!(alloc.primary.owns(memory));
        assert!(alloc.owns(memory));

        let memory = alloc
            .alloc(Layout::new::<[u8; 64]>(), AllocInit::Uninitialized)
            .expect("Could not allocate 64 bytes");
        assert!(alloc.fallback.owns(memory));
        assert!(alloc.owns(memory));
    }
}
