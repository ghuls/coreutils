//  * This file is part of the uutils coreutils package.
//  *
//  * For the full copyright and license information, please view the LICENSE
//  * file that was distributed with this source code.
// spell-checker:ignore (vars) zline zlines
//! Iterate over lines, including the line ending character(s).
//!
//! This module provides the [`lines`] and [`zlines`] functions,
//! similar to the [`BufRead::lines`] method. While the
//! [`BufRead::lines`] method yields [`String`] instances that do not
//! include the line ending characters (`"\n"` or `"\r\n"`), our
//! functions yield [`String`] instances that include the line ending
//! characters. This is useful if the input data does not end with a
//! newline character and you want to preserve the exact form of the
//! input data.
use std::io::BufRead;

/// The zero byte, representing the null character.
const ZERO: u8 = 0;

/// Returns an iterator over the lines, including line ending characters.
///
/// This function is just like [`BufRead::lines`], but it includes the
/// line ending characters in each yielded [`String`] if the input
/// data has them.
///
/// # Examples
///
/// If the input data does not end with a newline character (`'\n'`),
/// then the last [`String`] yielded by this iterator also does not
/// end with a newline:
///
/// ```rust,ignore
/// use std::io::BufRead;
/// use std::io::Cursor;
///
/// let cursor = Cursor::new(b"x\ny\nz");
/// let mut it = cursor.lines();
///
/// assert_eq!(it.next(), Some(String::from("x\n")));
/// assert_eq!(it.next(), Some(String::from("y\n")));
/// assert_eq!(it.next(), Some(String::from("z")));
/// assert_eq!(it.next(), None);
/// ```
pub(crate) fn lines<B>(reader: B) -> Lines<B>
where
    B: BufRead,
{
    Lines { buf: reader }
}

/// An iterator over the lines of an instance of `BufRead`.
///
/// This struct is generally created by calling [`lines`] on a `BufRead`.
/// Please see the documentation of [`lines`] for more details.
pub(crate) struct Lines<B> {
    buf: B,
}

impl<B: BufRead> Iterator for Lines<B> {
    type Item = std::io::Result<String>;

    fn next(&mut self) -> Option<std::io::Result<String>> {
        let mut buf = String::new();
        match self.buf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => Some(Ok(buf)),
            Err(e) => Some(Err(e)),
        }
    }
}

/// Returns an iterator over the lines of the given reader.
///
/// The iterator returned from this function will yield instances of
/// [`std::io::Result`]<[`Vec`]<[`u8`]>>, representing the bytes of the line
/// *including* the null character (with the possible exception of the
/// last line, which may not have one).
///
/// # Examples
///
/// ```rust,ignore
/// use std::io::Cursor;
///
/// let cursor = Cursor::new(b"x\0y\0z\0");
/// let mut iter = zlines(cursor).map(|l| l.unwrap());
/// assert_eq!(iter.next(), Some(b"x\0".to_vec()));
/// assert_eq!(iter.next(), Some(b"y\0".to_vec()));
/// assert_eq!(iter.next(), Some(b"z\0".to_vec()));
/// assert_eq!(iter.next(), None);
/// ```
pub fn zlines<B>(buf: B) -> ZLines<B> {
    ZLines { buf }
}

/// An iterator over the zero-terminated lines of an instance of `BufRead`.
pub struct ZLines<B> {
    buf: B,
}

impl<B: BufRead> Iterator for ZLines<B> {
    type Item = std::io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<std::io::Result<Vec<u8>>> {
        let mut buf = Vec::new();
        match self.buf.read_until(ZERO, &mut buf) {
            Ok(0) => None,
            Ok(_) => Some(Ok(buf)),
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::lines::lines;
    use crate::lines::zlines;
    use std::io::Cursor;

    #[test]
    fn test_null_terminated() {
        let cursor = Cursor::new(b"x\0y\0z\0");
        let mut iter = zlines(cursor).map(|l| l.unwrap());
        assert_eq!(iter.next(), Some(b"x\0".to_vec()));
        assert_eq!(iter.next(), Some(b"y\0".to_vec()));
        assert_eq!(iter.next(), Some(b"z\0".to_vec()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_not_null_terminated() {
        let cursor = Cursor::new(b"x\0y\0z");
        let mut iter = zlines(cursor).map(|l| l.unwrap());
        assert_eq!(iter.next(), Some(b"x\0".to_vec()));
        assert_eq!(iter.next(), Some(b"y\0".to_vec()));
        assert_eq!(iter.next(), Some(b"z".to_vec()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_lines() {
        let cursor = Cursor::new(b"x\ny\nz");
        let mut it = lines(cursor).map(|l| l.unwrap());

        assert_eq!(it.next(), Some(String::from("x\n")));
        assert_eq!(it.next(), Some(String::from("y\n")));
        assert_eq!(it.next(), Some(String::from("z")));
        assert_eq!(it.next(), None);
    }
}
