use std::cell::UnsafeCell;

/// An equivalent of [std::cell::OnceCell](https://doc.rust-lang.org/stable/std/cell/struct.OnceCell.html)
/// with an additional transmute helper.
/// To guarantee the helper's safety, the current implementation detail was copied from OnceCell
/// rather than reusing it.
#[repr(transparent)]
pub struct OptionOnceCell<T> {
    // Invariant: written to at most once.
    inner: UnsafeCell<Option<T>>,
}

impl<T> OptionOnceCell<T> {
    pub fn get(&self) -> Option<&T> {
        unsafe { &*self.inner.get() }.as_ref()
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        unsafe {
            let value = Some(value);
            if (*self.inner.get()).is_none() {
                *self.inner.get() = value;
                Ok(())
            } else {
                Err(value.unwrap())
            }
        }
    }

    // Layout compatibility is attested by its equivalence to
    // `Cell::from_mut` followed by `as_slice_of_cells`.
    pub fn from_slice(slice: &mut [Option<T>]) -> &[Self] {
        unsafe { std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut Self, slice.len()) }
    }
}
