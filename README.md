# KRNLSTRING

`KRNLSTRING` is a Rust crate that provides safe abstractions for working with Windows Unicode strings (`UNICODE_STRING`). This crate is designed to be used in `#![no_std]` environments, making it suitable for drivers or other low-level programming where the Rust standard library cannot be used. It leverages the `alloc` crate for dynamic memory management without requiring the full standard library.

## Features

- Safe wrapper for `UNICODE_STRING` that owns its buffer.
- Ensures the UTF-16 buffer remains valid as long as the `OwnedUnicodeString` instance exists.
- Provides conversion utilities to and from Rust strings (`&str`), as well as Windows string types (`PCWSTR`, `PWSTR`).
- Compatible with `#![no_std]` environments.
- Supports concatenation of `OwnedUnicodeString` instances and Rust strings using the `Add` trait.
- Enables comparison between `OwnedUnicodeString` instances using the `PartialEq` trait.

## Usage Example

```rust
# extern crate alloc;
# use alloc::vec::Vec;
use krnlstring::OwnedUnicodeString;

let mut my_string = OwnedUnicodeString::from("Hello, world!");

println!("{}", my_string);

```

## Performance

`KRNLSTRING` is optimized for minimal memory copying and efficient buffer management. The `OwnedUnicodeString` struct directly owns its UTF-16 buffer using a `Vec<u16>`, which reduces the need for unnecessary memory allocations and deallocations.

Unlike other implementations that might require converting the UTF-16 buffer to a Rust `String` for display, which would involve a memory copy, `KRNLSTRING` provides a zero-copy formatter. This formatter allows the `OwnedUnicodeString` to be formatted and displayed directly without converting the entire buffer to a `String`, thereby saving both memory and processing time.

## Safety

**Warning:** This project is still in the learning and development phase. As a beginner in Rust and Windows kernel driver development, I created this project to learn and leverage open source. **This code should not be used in production until it has been audited by experts**.

The goal of this project is to learn and receive feedback from the open source community. Contributions are welcome, whether to improve the code, add new features, or enhance the documentation. I am open to all suggestions and discussions to improve this project.

## Contributing

If you wish to contribute, please fork the repository and submit a pull request. All contributions, big or small, are welcome!
