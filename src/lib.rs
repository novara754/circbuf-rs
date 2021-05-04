use std::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

/// A circular buffer with a constant size.
///
/// # Example
///
/// ```
/// use circbuf::CircBuf;
///
/// // Create a new circular buffer that can hold 16 elements
/// let mut buf = CircBuf::<i32, 16>::new();
///
/// // Fill the buffer completely
/// for i in 0..16 {
///     buf.push(i);
/// }
/// assert!(buf.is_full());
///
/// // Iterate over values
/// for n in buf.iter() {
///     println!("{}", n);
/// }
///
/// // Adding values when the buffer is full overwrites the oldest value
/// for i in 16..19 {
///     buf.push(i);
/// }
///
/// // Iterate over values again
/// for n in buf.iter() {
///     println!("{}", n);
/// }
///
/// // Index specific values
/// println!("buf[0] = {}", buf[0]);
/// // println!("buf[20] = {}", buf[20]); // panic when index invalid
///
/// // Delete values while the buffer is not empty
/// while !buf.is_empty() {
///     // Popped values are returned in Option
///     println!("{}", buf.pop().unwrap());
/// }
///
/// // Check number of elements in a buffer
/// assert_eq!(buf.len(), 0);
/// ```
///
/// # Indexing
///
/// Circular buffers can be indexed just like `Vec`s.
/// Here the index `0` refers to the oldest elements currently in the buffer:
/// ```
/// use circbuf::CircBuf;
/// let mut buf: CircBuf<_, 8> = CircBuf::new();
/// for i in 3..9 {
///     buf.push(i);
/// }
/// assert_eq!(buf[0], 3);
/// assert_eq!(buf[2], 5);
/// ```
/// Buf be careful: Just like with `Vec`s, if you try to use an invalid index it will cause a panic:
/// ```should_panic
/// use circbuf::CircBuf;
/// let buf: CircBuf<i32, 8> = CircBuf::new();
/// println!("buf[0]={}", buf[0]); // will panic!
/// ```
#[derive(Debug)]
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

    /// Returns an iterator over the elements in the buffer.
    pub fn iter(&self) -> Iter<T, SIZE> {
        Iter { buf: self, idx: 0 }
    }
}

impl<T, const SIZE: usize> Index<usize> for CircBuf<T, SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len() {
            panic!("the len is {} but the index is {}", self.len(), index);
        } else {
            let index = (self.start + index) % SIZE;
            // SAFETY:
            // - Index is less than length of valid area.
            // - Index starts at `start` which marks the start of the valid area.
            // Thus the element can be safely assumed to be initialized.
            unsafe { &*self.data[index].as_ptr() }
        }
    }
}

impl<T, const SIZE: usize> IndexMut<usize> for CircBuf<T, SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len() {
            panic!("the len is {} but the index is {}", self.len(), index);
        } else {
            let index = (self.start + index) % SIZE;
            // SAFETY:
            // - Index is less than length of valid area.
            // - Index starts at `start` which marks the start of the valid area.
            // Thus the element can be safely assumed to be initialized.
            unsafe { &mut *self.data[index].as_mut_ptr() }
        }
    }
}

/// Iterator over elements of a circular buffer.
/// Created from a `CircBuf` using [`iter`].
///
/// [`iter`]: CircBuf::iter
///
/// # Examples
///
/// ```
/// use circbuf::CircBuf;
/// let mut buf: CircBuf<_, 8> = CircBuf::new();
/// for i in 0..6 {
///     buf.push(i);
/// }
/// buf.pop();
///
/// assert_eq!(
///     buf.iter().copied().collect::<Vec<i32>>(),
///     vec![1, 2, 3, 4, 5]
/// );
/// ```
pub struct Iter<'a, T, const SIZE: usize> {
    /// Reference to the circular buffer to iterate over.
    buf: &'a CircBuf<T, SIZE>,
    /// Index of the next value to return from iterator.
    idx: usize,
}

impl<'a, T, const SIZE: usize> Iterator for Iter<'a, T, SIZE> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.buf.len() {
            let elem = &self.buf[self.idx];
            self.idx += 1;
            Some(elem)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.buf.len(), Some(self.buf.len()))
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

    #[test]
    fn test_iter() {
        let mut buf: CircBuf<_, 8> = CircBuf::new();
        for i in 0..6 {
            buf.push(i);
        }
        buf.pop();

        assert_eq!(
            buf.iter().copied().collect::<Vec<i32>>(),
            vec![1, 2, 3, 4, 5]
        );
    }
}
