//! # KRNLSTRING
//!
//! `KRNLSTRING` is a Rust crate that provides safe abstractions for working with Windows Unicode strings `UNICODE_STRING`.
//!
//! This crate is designed to be used in `#![no_std]` environments, making it suitable for drivers or other low-level
//! programming where the Rust standard library cannot be used. It leverages the `alloc` crate for dynamic memory
//! management without requiring the full standard library.
//!
//! ## Features
//!
//! - Safe wrapper for `UNICODE_STRING` that owns its buffer.
//! - Ensures the UTF-16 buffer remains valid as long as the `OwnedUnicodeString` instance exists.
//! - Provides conversion utilities to and from Rust strings (`&str`), as well as Windows string types (`PCWSTR`, `PWSTR`).
//! - Compatible with `#![no_std]` environments.
//! - Supports concatenation of `OwnedUnicodeString` instances and Rust strings using the `Add` trait.
//! - Enables comparison between `OwnedUnicodeString` instances using the `PartialEq` trait.
//!
//! ## Usage Example
//!
//! ```rust
//! # extern crate alloc;
//! # use alloc::vec::Vec;
//! use krnlstring::OwnedUnicodeString;
//!
//! let mut my_string = OwnedUnicodeString::from("Hello, world!");
//!
//! println!("{}", my_string);
//! ```
//!
//!
//! ## Performance
//!
//! `KRNLSTRING` is optimized for minimal memory copying and efficient buffer management.
//! The `OwnedUnicodeString` struct directly owns its UTF-16 buffer using a `Vec<u16>`, which reduces the need for
//! unnecessary memory allocations and deallocations.
//!
//! Unlike other implementations that might require converting the UTF-16 buffer to a Rust `String` for display,
//! which would involve a memory copy, `KRNLSTRING` provides a zero-copy formatter. This formatter allows
//! the `OwnedUnicodeString` to be formatted and displayed directly without converting the entire buffer to a `String`,
//! thereby saving both memory and processing time.
//!
//! When converting from Rust strings (`&str`) to `OwnedUnicodeString`, the crate encodes the string directly into
//! UTF-16 format without intermediate copies. Similarly, when converting to Windows string types (`PCWSTR` and `PWSTR`),
//! it ensures that the buffer is used as-is, with only necessary modifications to ensure null-termination.
//!
//! ## Safety
//!
//! This crate aims to provide memory-safe abstractions for working with Windows Unicode strings.
//! All functions and methods ensure that buffers are properly managed to avoid memory safety issues
//! such as dangling pointers or buffer overflows.

#![cfg_attr(not(test), no_std)]
extern crate alloc;

use core::slice;
use alloc::vec::Vec;
use core::char::decode_utf16;
use core::fmt;
use core::mem::size_of;
use core::ops::Add;
use windows_sys::core::{PCWSTR, PWSTR};
use windows_sys::Win32::Foundation::UNICODE_STRING;


/// A safe wrapper around Windows `UNICODE_STRING` that owns its UTF-16 buffer.
///
/// The `OwnedUnicodeString` structure provides a safe abstraction over the Windows `UNICODE_STRING` type, which is used
/// for handling Unicode strings in Windows environments. This structure owns a UTF-16 buffer and ensures its validity
/// throughout the lifetime of the `OwnedUnicodeString` instance, preventing memory safety issues such as dangling pointers
/// and buffer overflows.
///
/// The safety of `OwnedUnicodeString` is primarily derived from its ownership model. It manages the UTF-16 buffer internally
/// using a `Vec<u16>`, which allows for dynamic resizing and ensures proper memory deallocation when the `OwnedUnicodeString`
/// instance is dropped. By owning the buffer, the structure ensures that the memory is only released when it is no longer
/// in use, thereby preventing use-after-free errors.
///
/// However, it's important to note that the `UNICODE_STRING` structure used internally contains a mutable pointer (`*mut u16`)
/// to the UTF-16 buffer (`PWSTR`), which directly points to the underlying `Vec<u16>`. This mutable pointer is necessary
/// because the Microsoft bindings for Windows APIs use this type to interact with strings. As a result, if the user decides
/// to manually modify the vector (`Vec<u16>`) without properly adjusting the associated length fields (`Length` and `MaximumLength`),
/// it can lead to undefined behavior or potential memory safety issues. Therefore, while the buffer is accessible and mutable,
/// any manual modifications should be performed with caution.
///
/// The mutable pointer and direct access to the buffer are unavoidable due to the design of the Windows API bindings provided
/// by Microsoft, which require the use of `*mut u16` (`PWSTR`). This design choice respects the type requirements of these bindings
/// but also places the responsibility on the user to handle any low-level modifications with care to maintain the integrity of
/// the Unicode string.
///
/// # Fields
///
/// - `unicode_string`: A `UNICODE_STRING` structure that points to the UTF-16 buffer. This structure is updated to
///   reflect the current state of the buffer, including its length and maximum length. The `Buffer` field is a mutable pointer (`*mut u16`)
///   to the UTF-16 data.
/// - `buffer`: A `Vec<u16>` that owns and manages the UTF-16 buffer, ensuring that its lifetime is tied to the `OwnedUnicodeString` structure.
///   The buffer's memory is automatically managed, reducing the risk of memory leaks or unsafe memory access.
///
/// # Safety
///
/// `OwnedUnicodeString` ensures that the UTF-16 buffer remains valid and properly managed as long as the `OwnedUnicodeString` instance exists.
/// This design guarantees that memory is safely allocated and deallocated and that the buffer is correctly formatted for use with Windows APIs.
/// However, due to the mutable pointer in the underlying `UNICODE_STRING`, caution must be exercised if manually modifying the buffer to
/// prevent mismatches in length or buffer overflows.
pub struct OwnedUnicodeString {
    unicode_string: UNICODE_STRING,
    buffer: Vec<u16>,
}

impl OwnedUnicodeString {
    fn is_null_terminated(&self) -> bool {
        self.buffer.last() == Some(&0)
    }

    fn ensure_is_null_terminated(&mut self) {
        if !self.is_null_terminated() {
            self.buffer.push(0u16);
            self.unicode_string.MaximumLength += size_of::<u16>() as u16;
        }
    }

    fn compute_size(&mut self) {
        let maximum_length = (self.buffer.len() * size_of::<u16>()) as u16;
        let mut count = 0;

        if self.is_null_terminated() {
            for &value in self.buffer.iter().rev() {
                if value == 0 {
                    count += 1;
                } else {
                    break;
                }
            }
        }

        let length= maximum_length - (count * size_of::<u16>()) as u16;

        self.unicode_string.Length = length;
        self.unicode_string.MaximumLength = maximum_length
    }


}

impl From<Vec<u16>> for OwnedUnicodeString {
    /// Converts a `Vec<u16>` to an `OwnedUnicodeString`.
    ///
    /// This implementation takes ownership of the provided `Vec<u16>`, allowing for direct manipulation
    /// of the UTF-16 buffer. It initializes an `UNICODE_STRING` with the provided vector, calculates
    /// the length and maximum length of the buffer, and ensures that it remains valid and properly
    /// managed throughout the instance's lifetime.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the input `Vec<u16>` represents a valid UTF-16 encoded string.
    /// This function will calculate the lengths based on the vector's contents and adjust the
    /// `UNICODE_STRING` fields accordingly.
    fn from(mut value: Vec<u16>) -> Self {

        let unicode_string = UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: value.as_mut_ptr(),
        };

        let mut result = Self {
            unicode_string,
            buffer: value,
        };

        result.compute_size();

        result

    }
}

impl From<&str> for OwnedUnicodeString {
    /// Converts a Rust string slice (`&str`) to an `OwnedUnicodeString`.
    ///
    /// This implementation encodes the Rust string as UTF-16 and stores the result in a `Vec<u16>`,
    /// which is then used to initialize the `OwnedUnicodeString`. This allows for seamless integration
    /// with Rust's native string types while leveraging the safety and efficiency of UTF-16 buffers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use krnlstring::OwnedUnicodeString;
    ///
    /// let my_string = OwnedUnicodeString::from("Hello, world!");
    /// ```
    fn from(value: &str) -> Self {
        Self::from(value.encode_utf16().collect::<Vec<u16>>())
    }
}

impl AsRef<UNICODE_STRING> for OwnedUnicodeString {
    /// Provides a reference to the internal `UNICODE_STRING`.
    ///
    /// This implementation allows for safe access to the underlying `UNICODE_STRING` structure, which
    /// can be useful for interoperability with Windows APIs that expect a `UNICODE_STRING` pointer.
    /// The returned reference reflects the current state of the buffer and its lengths.
    fn as_ref(&self) -> &UNICODE_STRING {
        &self.unicode_string
    }
}

impl Into<PCWSTR> for &mut OwnedUnicodeString {
    /// Converts a mutable reference to an `OwnedUnicodeString` into a `PCWSTR`.
    ///
    /// This conversion ensures that the UTF-16 buffer is null-terminated, as required for use
    /// with many Windows API functions that expect a `PCWSTR` (a pointer to a constant, null-terminated
    /// UTF-16 string). The conversion does not make a copy of the buffer, maintaining a zero-copy approach.
    ///
    /// # Safety
    ///
    /// The buffer must remain valid for the lifetime of the `PCWSTR` returned. The caller should
    /// ensure that the `OwnedUnicodeString` is not mutated in a way that invalidates the pointer.
    fn into(self) -> PCWSTR {
        self.ensure_is_null_terminated();
        self.buffer.as_ptr()
    }
}

impl Into<PWSTR> for &mut OwnedUnicodeString{
    /// Converts a mutable reference to an `OwnedUnicodeString` into a `PWSTR`.
    ///
    /// Similar to `Into<PCWSTR>`, this conversion ensures that the UTF-16 buffer is properly null-terminated
    /// and returns a mutable pointer (`PWSTR`). This is useful for APIs that require a mutable UTF-16 string buffer.
    ///
    /// # Safety
    ///
    /// The buffer must remain valid and should not be modified in a way that would invalidate the pointer
    /// while it is being used as a `PWSTR`.
    fn into(self) -> PWSTR {
        self.ensure_is_null_terminated();
        self.buffer.as_mut_ptr()
    }
}

impl fmt::Display for OwnedUnicodeString {
    /// Formats the `OwnedUnicodeString` as a Rust string for display purposes.
    ///
    /// This implementation provides a `Display` formatter that allows the `OwnedUnicodeString` to be printed
    /// directly using Rust's `println!` and other formatting macros. It decodes the UTF-16 buffer to a Rust
    /// string slice, converting any invalid UTF-16 sequences to the Unicode replacement character (`�`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use krnlstring::OwnedUnicodeString;
    ///
    /// let my_string = OwnedUnicodeString::from("Hello, world!");
    /// println!("{}", my_string);
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let utf16_slice = unsafe {
            slice::from_raw_parts(
                self.unicode_string.Buffer,
                (self.unicode_string.Length / size_of::<u16>() as u16) as usize
            )
        };
        for utf16 in decode_utf16(utf16_slice.iter().copied()) {
            match utf16 {
                Ok(ch) => write!(f, "{}", ch)?,
                Err(_) => write!(f, "{}", "�")?,
            }
        }
        Ok(())
    }
}

impl Add for OwnedUnicodeString {
    type Output = OwnedUnicodeString;

    /// Concatenates two `OwnedUnicodeString` instances.
    ///
    /// This implementation of the `Add` trait allows for the concatenation of two `OwnedUnicodeString` instances,
    /// resulting in a new `OwnedUnicodeString` that contains the combined UTF-16 buffers of the operands.
    /// It ensures that the resulting buffer is properly null-terminated and that the lengths are updated accordingly.
    ///
    /// # Safety
    ///
    /// The internal buffer is resized to accommodate the concatenated strings, and lengths are recalculated to prevent
    /// overflows or invalid reads.
    ///
    fn add(mut self, rhs: Self) -> Self::Output {
        let rhs_slice = unsafe {
            slice::from_raw_parts(
                rhs.unicode_string.Buffer,
                (rhs.unicode_string.Length / size_of::<u16>() as u16) as usize
            )
        };
        self.buffer.extend(rhs_slice);
        self.compute_size();
        self
    }
}

impl Add<&str> for OwnedUnicodeString {
    type Output = OwnedUnicodeString;

    /// Concatenates an `OwnedUnicodeString` with a Rust string slice (`&str`).
    ///
    /// This implementation allows for concatenating a Rust `&str` directly onto an `OwnedUnicodeString`, returning a new
    /// `OwnedUnicodeString` with the combined content. The string slice is encoded as UTF-16 before concatenation.
    fn add(self, rhs: &str) -> Self::Output {
        let other = OwnedUnicodeString::from(rhs);
        self + other
    }
}


impl PartialEq for OwnedUnicodeString {

    /// Compares two `OwnedUnicodeString` instances for equality.
    ///
    /// This implementation of the `PartialEq` trait allows for the comparison of two `OwnedUnicodeString` instances
    /// based on the contents of their UTF-16 buffers. It checks if the lengths and contents of both buffers match,
    /// providing a simple and efficient way to compare Unicode strings.
    fn eq(&self, other: &Self) -> bool {
        let self_slice = &self.buffer[..(self.unicode_string.Length / size_of::<u16>() as u16) as usize];
        let other_slice = &other.buffer[..(other.unicode_string.Length / size_of::<u16>() as u16) as usize];
        self_slice == other_slice
    }
}

#[cfg(test)]
mod test_krnlstring {
    use alloc::{format, vec};
    use super::*;

    #[test]
    fn test_fmt() {
        let owned_unicode = OwnedUnicodeString::from("Hello, world !");
        let formated = format!("{}", owned_unicode);
        assert_eq!(formated,"Hello, world !");
    }

    #[test]
    fn test_eq() {
        let owned_unicode = OwnedUnicodeString::from("Hello, world !");
        let same = OwnedUnicodeString::from("Hello, world !");
        let result = owned_unicode == same;
        assert_eq!(result,true)
    }

    #[test]
    fn test_add() {
        let owned_unicode = OwnedUnicodeString::from("Hello, world !");
        let other_str: &str = " Bye";
        let other = OwnedUnicodeString::from(" !");
        let expected1 = OwnedUnicodeString::from("Hello, world ! Bye");
        let expected2 = OwnedUnicodeString::from("Hello, world ! Bye !");
        let  concat1 =  owned_unicode + other_str;
        let mut result = concat1 == expected1;
        assert_eq!(result,true);
        let  concat2 =  concat1  + other;
        result = concat2 == expected2;
        assert_eq!(result,true);
    }

    #[test]
    fn test_empty_string() {
        let owned_unicode = OwnedUnicodeString::from("");
        let expected = OwnedUnicodeString::from(Vec::new());
        let  result = owned_unicode == expected;
        assert_eq!(result, true);
    }

    #[test]
    fn test_unicode_characters() {
        let unicode_str = "こんにちは"; // "Hello" in Japanese
        let owned_unicode = OwnedUnicodeString::from(unicode_str);
        let formated = format!("{}", owned_unicode);
        assert_eq!(formated, unicode_str);
    }

    #[test]
    fn test_conversion_to_pcwstr_pwstr() {
        let mut owned_unicode = OwnedUnicodeString::from("Hello, world!");

        let pcwstr: PCWSTR = (&mut owned_unicode).into();
        let pwstr: PWSTR = (&mut owned_unicode).into();

        unsafe {
            assert_eq!(*pcwstr, *pwstr);
        }

        assert!(owned_unicode.is_null_terminated());
    }

    #[test]
    fn test_add_special_characters() {
        let owned_unicode = OwnedUnicodeString::from("Line1\n");
        let other = OwnedUnicodeString::from("Line2\tEnd");
        let expected = OwnedUnicodeString::from("Line1\nLine2\tEnd");

        let result = owned_unicode + other;
        assert_eq!(result == expected, true);
    }

    #[test]
    fn test_buffer_overflow_protection() {
        let mut owned_unicode = OwnedUnicodeString::from("Test");

        // Manually extend the buffer to simulate potential overflow
        owned_unicode.buffer.push(1);

        // Ensure the buffer still respects the max length
        owned_unicode.compute_size();
        assert!(owned_unicode.unicode_string.Length <= owned_unicode.unicode_string.MaximumLength);
    }

    #[test]
    fn test_multiple_consecutive_null_characters() {
        let mut owned_unicode = OwnedUnicodeString::from("Test");

        // Add multiple null characters
        owned_unicode.buffer.extend(vec![0, 0, 0]);

        owned_unicode.compute_size();

        // Check length is properly adjusted
        let expected_length = (4 * size_of::<u16>()) as u16;
        assert_eq!(owned_unicode.unicode_string.Length, expected_length);
    }

    #[test]
    fn test_large_input_handling() {
        let large_string = "A".repeat(10000);
        let owned_unicode = OwnedUnicodeString::from(large_string.as_str());

        // Check the length is correctly calculated
        assert_eq!(owned_unicode.unicode_string.Length, (10000 * size_of::<u16>()) as u16);
    }

    #[test]
    fn test_equality_case_sensitivity() {
        let upper_case = OwnedUnicodeString::from("HELLO");
        let lower_case = OwnedUnicodeString::from("hello");

        assert_ne!(upper_case == lower_case, true);
    }

    #[test]
    fn test_fmt_invalid_utf16_sequence() {
        let mut owned_unicode = OwnedUnicodeString::from("Hello");

        // Manually add invalid UTF-16 sequence
        owned_unicode.buffer.push(0xD800); // Half of a surrogate pair
        owned_unicode.compute_size();

        let formated = format!("{}", owned_unicode);
        assert_eq!(formated, "Hello�");
    }
}