//! Port of `borg-util.c`: miscellaneous utilities.
//!
//! Copyright (c) 1997 Ben Harrison, James E. Wilson, Robert A. Koeneke
//! Copyright (c) 2007-9 Andi Sidwell, Chris Carr, Ed Graham, Erik Osheim
//!
//! This work is free software; you can redistribute it and/or modify it
//! under the terms of either:
//!
//! a) the GNU General Public License as published by the Free Software
//!    Foundation, version 2, or
//!
//! b) the "Angband License":
//!    This software may be copied and distributed for educational, research,
//!    and not for profit purposes provided that this copyright and statement
//!    are included in all such copies.  Other copyrights may also apply.

use crate::cctype::c_isspace;
use std::cmp::Ordering;
use std::ffi::CStr;

/// What `borg_sort()` sorts: in C, two parallel arrays plus a compare hook
/// and a swap hook (`borg_sort_comp` / `borg_sort_swap`).
pub trait BorgSortTarget {
    /// "a should come before or with b" (the C compare hook).
    fn comp(&self, a: usize, b: usize) -> bool;
    /// Swap entries a and b (the C swap hook).
    fn swap(&mut self, a: usize, b: usize);
}

/// The Borg's own quicksort (`borg_sort_aux`). Kept as is, because ties
/// are ordered by the algorithm.
fn borg_sort_aux<T: BorgSortTarget + ?Sized>(t: &mut T, p: i32, q: i32) {
    if p >= q {
        return;
    }
    let z = p;
    let mut a = p;
    let mut b = q;
    loop {
        while !t.comp(b as usize, z as usize) {
            b -= 1;
        }
        while !t.comp(z as usize, a as usize) {
            a += 1;
        }
        if a >= b {
            break;
        }
        t.swap(a as usize, b as usize);
        a += 1;
        b -= 1;
    }
    borg_sort_aux(t, p, b);
    borg_sort_aux(t, b + 1, q);
}

/// Sort the first `n` entries of `t` (`borg_sort`).
pub fn borg_sort<T: BorgSortTarget + ?Sized>(t: &mut T, n: i32) {
    borg_sort_aux(t, 0, n - 1);
}

/// The standard hooks (`borg_sort_comp_hook` / `borg_sort_swap_hook`): sort
/// strings (by `strcmp`), break ties by the parallel `what` value.
pub struct TextWhat<'a, 'b> {
    pub text: &'a mut [&'b CStr],
    pub what: &'a mut [i16],
}

impl BorgSortTarget for TextWhat<'_, '_> {
    fn comp(&self, a: usize, b: usize) -> bool {
        // strcmp compares as unsigned char, the same as byte slices
        match self.text[a].to_bytes().cmp(self.text[b].to_bytes()) {
            Ordering::Less => true,
            Ordering::Greater => false,
            Ordering::Equal => self.what[a] <= self.what[b],
        }
    }

    fn swap(&mut self, a: usize, b: usize) {
        self.text.swap(a, b);
        self.what.swap(a, b);
    }
}

/// Trim leading and trailing whitespace from the NUL-terminated string in
/// `buf` (`borg_trim`).
///
/// Like the C version, this writes a NUL after the last non-space and
/// leaves the rest of the buffer alone. It returns the offset of the first
/// non-space character, which in C is the returned pointer.
pub fn borg_trim(buf: &mut [u8]) -> usize {
    let mut start = 0;
    while c_isspace(buf[start]) {
        start += 1;
    }
    if buf[start] == 0 {
        return start;
    }
    let len = buf[start..].iter().position(|&c| c == 0).unwrap_or(buf.len() - start);
    let mut end = start + len - 1;
    while end > start && c_isspace(buf[end]) {
        end -= 1;
    }
    buf[end + 1] = 0;
    start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_keeps_buffer_tail() {
        let mut b = *b"  ab c \x0b \0xyz\0";
        let off = borg_trim(&mut b);
        assert_eq!(off, 2);
        assert_eq!(&b[2..7], b"ab c\0");
        assert_eq!(&b[7..], b"\x0b \0xyz\0");
        let mut e = *b"   \0q";
        assert_eq!(borg_trim(&mut e), 3);
    }

    #[test]
    fn sort_orders_by_text_then_what() {
        let s: Vec<std::ffi::CString> = ["b", "a", "b", "a"]
            .iter()
            .map(|x| std::ffi::CString::new(*x).unwrap())
            .collect();
        let mut text: Vec<&CStr> = s.iter().map(|c| c.as_c_str()).collect();
        let mut what = vec![4i16, 3, 1, 2];
        let n = text.len() as i32;
        borg_sort(&mut TextWhat { text: &mut text, what: &mut what }, n);
        let got: Vec<_> = text.iter().map(|c| c.to_str().unwrap()).collect();
        assert_eq!(got, ["a", "a", "b", "b"]);
        assert_eq!(what, [2, 3, 1, 4]);
    }
}
