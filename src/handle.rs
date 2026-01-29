//! Opaque handle wrapper for Memvid instances.

use memvid_core::Memvid;

/// Opaque handle to a Memvid instance.
///
/// This struct owns the underlying Memvid and is exposed to C as an opaque pointer.
/// The handle must be freed with `memvid_close()`.
///
/// # Thread Safety
///
/// `MemvidHandle` is NOT thread-safe. All operations on a handle must occur
/// from the same thread that created it, or external synchronization must be used.
pub struct MemvidHandle {
    inner: Memvid,
}

impl MemvidHandle {
    /// Create a new handle wrapping a Memvid instance.
    pub fn new(memvid: Memvid) -> Box<Self> {
        Box::new(Self { inner: memvid })
    }

    /// Get a reference to the inner Memvid.
    pub fn as_ref(&self) -> &Memvid {
        &self.inner
    }

    /// Get a mutable reference to the inner Memvid.
    pub fn as_mut(&mut self) -> &mut Memvid {
        &mut self.inner
    }

    /// Convert a raw pointer to a mutable reference.
    ///
    /// # Safety
    ///
    /// The pointer must be valid and non-null.
    pub unsafe fn from_ptr_mut<'a>(ptr: *mut MemvidHandle) -> Option<&'a mut Self> {
        unsafe { ptr.as_mut() }
    }
}
