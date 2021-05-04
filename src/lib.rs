use std::mem::MaybeUninit;

/// A circular buffer with a constant size.
pub struct CircBuf<T, const SIZE: usize> {
    /// Start of the valid data in buffer.
    start: usize,
    /// Number of valid elements after `start`.
    len: usize,
    /// Storage for potential elements.
    data: [MaybeUninit<T>; SIZE],
}

impl<T, const SIZE: usize> Default for CircBuf<T, SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const SIZE: usize> CircBuf<T, SIZE> {
    /// Create a new, empty circular buffer of the specified size.
    pub fn new() -> Self {
        Self {
            start: 0,
            len: 0,
            // SAFETY:
            // `MaybeUninit`s do not require initialization, and as such an array of
            // `MaybeUninit`s is safe to `MaybeUninit::assume_init`.
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    /// Add a new element to the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use circbuf::CircBuf;
    /// let mut buf: CircBuf<_, 8> = CircBuf::new();
    /// buf.push(1);
    /// buf.push(2);
    /// ```
    pub fn push(&mut self, elem: T) {
        let write_idx = (self.start + self.len) % SIZE;
        self.data[write_idx] = MaybeUninit::new(elem);

        if self.is_full() {
            self.start = (self.start + 1) % SIZE;
        } else {
            self.len += 1;
        }
    }

    /// Remove the oldest element from the buffer and return it if it exists.
    /// Otherwise return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use circbuf::CircBuf;
    /// let mut buf: CircBuf<_, 8> = CircBuf::new();
    /// buf.push(10);
    /// assert_eq!(buf.pop(), Some(10));
    /// assert_eq!(buf.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            // SAFETY:
            // * Length is greater than zero so the buffer contains an initialized element *somewhere*.
            // * Initialized values are always written in front of the `read_idx`.
            // * `read_idx` always move forwards after an element is removed.
            // This means `read_idx` must point to a properly initialized value and the following
            // operation is safe.
            //
            // `ptr::read` does not drop the underlying value, but this is intended:
            // Ownership of the value is transfered to the caller, and the remnents of the value
            // in the array will be overwritten by other writes later.
            let elem = unsafe { self.data[self.start].as_ptr().read() };

            self.start = (self.start + 1) % SIZE;
            self.len -= 1;

            Some(elem)
        }
    }

    /// Get the number of values currently stored in the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use circbuf::CircBuf;
    /// let mut buf: CircBuf<_, 8> = CircBuf::new();
    /// assert_eq!(buf.len(), 0);
    /// buf.push(1);
    /// assert_eq!(buf.len(), 1);
    /// buf.pop();
    /// assert_eq!(buf.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        // if self.read_idx <= self.write_idx {
        //     self.write_idx - self.read_idx
        // } else {
        //     SIZE - self.read_idx + self.write_idx
        // }
        self.len
    }

    /// Returns `true` if the buffer contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use circbuf::CircBuf;
    /// let buf: CircBuf<i32, 8> = CircBuf::new();
    /// assert!(buf.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if the buffer is full and would overwrite a value on the next push.
    ///
    /// # Examples
    ///
    /// ```
    /// use circbuf::CircBuf;
    /// let mut buf: CircBuf<_, 4> = CircBuf::new();
    /// for i in 0..4 {
    ///     buf.push(i);
    /// }
    /// assert!(buf.is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        self.len == SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pop() {
        let mut buf: CircBuf<i32, 8> = CircBuf::new();
        assert!(buf.is_empty());
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_wrapping_push() {
        let mut buf: CircBuf<_, 5> = CircBuf::new();
        for i in 0..5 {
            buf.push(i);
        }
        for (val, expected) in buf.data.iter().zip([0, 1, 2, 3, 4].iter()) {
            assert_eq!(unsafe { val.assume_init() }, *expected);
        }
        buf.push(5);
        for (val, expected) in buf.data.iter().zip([5, 1, 2, 3, 4].iter()) {
            assert_eq!(unsafe { val.assume_init() }, *expected);
        }
    }

    #[test]
    fn test_wrapping_pop() {
        let mut buf: CircBuf<_, 5> = CircBuf::new();
        for i in 0..6 {
            buf.push(i);
        }
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.len(), 4);
        assert_eq!(unsafe { buf.data[0].assume_init() }, 5);
        for (val, expected) in buf.data[2..].iter().zip([2, 3, 4].iter()) {
            assert_eq!(unsafe { val.assume_init() }, *expected);
        }
    }
}
