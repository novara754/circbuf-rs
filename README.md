[![CI](https://github.com/vzwGrey/circbuf-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/vzwGrey/circbuf-rs/actions/workflows/ci.yaml)

# circbuf

Basic [circular buffer](https://en.wikipedia.org/wiki/Circular_buffer) library for Rust.

## Example

```rust
use circbuf::CircBuf;

fn main() {
    // Create a new circular buffer that can hold 16 elements
    let mut buf = CircBuf::<i32, 16>::new();
    // Fill the buffer completely
    for i in 0..16 {
      buf.push(i);
    }
    assert!(buf.is_full());

    // Iterate over values
    for n in buf.iter() {
      println!("{}", n);
    }

    // Index specific values
    println!("buf[0] = {}", buf[0]);
    // println!("buf[20] = {}", buf[20]); // panic when index invalid

    // Delete values while the buffer is not empty
    while !buf.is_empty() {
      // Popped values are returned in Option
      println!("{}", buf.pop().unwrap());
    }

    // Check number of elements in a buffer
    assert_eq!(buf.len(), 0);
}
```

## License

Licensed under the [MIT License](./LICENSE)
