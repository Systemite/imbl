// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

use imbl_sized_chunks::InlineArray;

/// Inline storage for N + 1 child pointers without generic const expressions.
/// The backing type reserves N elements, one extra element, and a length word.
/// Padding can increase InlineArray's capacity, so enforce our own limit.
#[derive(Debug)]
pub(crate) struct ChildBuffer<T, const N: usize>(InlineArray<T, ([T; N], T, usize)>);

impl<T, const N: usize> ChildBuffer<T, N> {
    fn new() -> Self {
        Self(InlineArray::new())
    }

    pub(super) fn insert(&mut self, index: usize, value: T) {
        assert!(self.len() < N + 1);
        self.0.insert(index, value);
    }

    pub(super) fn remove(&mut self, index: usize) -> T {
        self.0.remove(index).expect("child index out of bounds")
    }

    pub(super) fn pop_back(&mut self) -> T {
        self.0.pop().expect("empty children")
    }

    pub(super) fn split_off(&mut self, at: usize) -> Self {
        Self(self.0.split_off(at))
    }

    pub(super) fn insert_from(&mut self, index: usize, values: impl IntoIterator<Item = T>) {
        let old_len = self.len();
        assert!(index <= old_len);
        self.extend(values);
        let count = self.len() - old_len;
        self.0[index..].rotate_right(count);
    }

    pub(super) fn drain_from_front(&mut self, other: &mut Self, count: usize) {
        assert!(count <= other.len());
        assert!(self.len() + count <= N + 1);
        let old_len = self.len();
        // Move the prefix to the back so each removal is O(1), then restore
        // its order in the destination. No pointer cloning is needed.
        other.0.rotate_left(count);
        for _ in 0..count {
            self.0.push(other.pop_back());
        }
        self.0[old_len..].reverse();
    }

    pub(super) fn drain_from_back(&mut self, other: &mut Self, count: usize) {
        assert!(count <= other.len());
        assert!(self.len() + count <= N + 1);
        let old_len = self.len();
        for _ in 0..count {
            self.0.push(other.pop_back());
        }
        self.0[old_len..].reverse();
        self.0.rotate_right(count);
    }
}

impl<T, const N: usize> Deref for ChildBuffer<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for ChildBuffer<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T, const N: usize> Extend<T> for ChildBuffer<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, values: I) {
        for value in values {
            assert!(self.len() < N + 1);
            self.0.push(value);
        }
    }
}

impl<T, const N: usize> FromIterator<T> for ChildBuffer<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        let mut out = Self::new();
        out.extend(values);
        out
    }
}

impl<T: Clone, const N: usize> Clone for ChildBuffer<T, N> {
    fn clone(&self) -> Self {
        // InlineArray's clone is faster than pushing one element at a time, but
        // leaks the elements cloned so far if a clone panics. The children are
        // shared pointers, whose clone aborts instead of panicking.
        Self(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfers_preserve_order() {
        const N: usize = 6;
        for src_len in 0..=N + 1 {
            for dst_len in 0..=N + 1 {
                for count in 0..=src_len.min(N + 1 - dst_len) {
                    let src: Vec<_> = (0..src_len).collect();
                    let dst: Vec<_> = (100..100 + dst_len).collect();
                    let mut a: ChildBuffer<_, N> = src.iter().copied().collect();
                    let mut b: ChildBuffer<_, N> = dst.iter().copied().collect();
                    b.drain_from_front(&mut a, count);
                    assert_eq!(&*a, &src[count..]);
                    assert!(b.iter().eq(dst.iter().chain(&src[..count])));

                    let mut a: ChildBuffer<_, N> = src.iter().copied().collect();
                    let mut b: ChildBuffer<_, N> = dst.iter().copied().collect();
                    b.drain_from_back(&mut a, count);
                    assert_eq!(&*a, &src[..src_len - count]);
                    assert!(b.iter().eq(src[src_len - count..].iter().chain(&dst)));
                }
            }
        }
    }

    #[test]
    fn split_and_insert_at_every_boundary() {
        let original: ChildBuffer<_, 6> = (0..7).map(|n| n.to_string()).collect();
        for at in 0..=7 {
            let mut left = original.clone();
            let right = left.split_off(at);
            assert_eq!(&*left, &original[..at]);
            assert_eq!(&*right, &original[at..]);
            left.insert_from(at, right.iter().cloned());
            assert_eq!(&*left, &*original);
        }
        for at in 0..7 {
            let mut buffer = original.clone();
            let removed = buffer.remove(at);
            buffer.insert(at, removed);
            assert_eq!(&*buffer, &*original);
        }
        let mut buffer: ChildBuffer<_, 6> = [0, 3, 4].iter().copied().collect();
        buffer.insert_from(1, [1, 2]);
        assert_eq!(&*buffer, &[0, 1, 2, 3, 4]);
    }

    #[test]
    #[should_panic]
    fn padding_does_not_increase_logical_capacity() {
        let _: ChildBuffer<u8, 6> = (0..8).collect();
    }
}
