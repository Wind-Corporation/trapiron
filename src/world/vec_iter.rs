//! Various iterators based on glam integer vectors.

use crate::world::UVec3;

/// Iterates all positions in the cuboid delimited by two points. See [`VecIterators::iter_box`].
pub struct Vec3BoxIter {
    /// Start position (inclusive) with the smaller of each coordinate.
    begin: UVec3,

    /// End position (exclusive) with the greater of each coordinate.
    ///
    /// # Invariants
    /// - `self.begin.cmple(self.end).all()` at all times,
    /// - `self.begin == self.end` if `self.begin.cmpeq(self.end).any()`.
    end: UVec3,

    /// The next item that would be returned by this iterator if `self.end.z` was infinite.
    ///
    /// While the iterator is not yet empty, `self.next.cmplt(self.end).all()`.
    /// When the iterator is empty, `self.next.z == self.end.z` (outside the region of iteration),
    /// which is what defines an empty iterator. It follows that if `self.begin == self.end`,
    /// `self.next` must be `self.begin`.
    ///
    /// # Invariant
    /// `self.next.cmpge(self.begin).all()`
    next: UVec3,
}

impl Vec3BoxIter {
    /// Initialize `Vec3BoxIter` for a given region, possibly with non-intersecting octants.
    fn new(begin: UVec3, end: UVec3) -> Self {
        Self {
            begin,
            end: if begin.cmplt(end).all() { end } else { begin },
            next: begin,
        }
    }
}

impl Vec3BoxIter {
    /// Check whether `self` is exhaused, i.e. whether [`Self::next`] would return `None`.
    pub fn is_empty(&self) -> bool {
        self.next.z >= self.end.z
    }

    /// Obtain the next position and advance the iterator assuming it is not yet empty.
    ///
    /// # Safety
    /// `self` must not be [empty](`Self::is_empty()`).
    pub unsafe fn next_unchecked(&mut self) -> UVec3 {
        let result = self.next;

        self.next.x += 1;
        if self.next.x >= self.end.x {
            self.next.x = self.begin.x;
            self.next.y += 1;
        }
        if self.next.y >= self.end.y {
            self.next.y = self.begin.y;
            self.next.z += 1;
        }

        result
    }
}

impl Iterator for Vec3BoxIter {
    type Item = UVec3;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            // SAFETY: Enforced with runtime check above.
            unsafe { Some(self.next_unchecked()) }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let shape = self.end - self.begin; // Zero in degenerate case
        let rel = self.end - self.next; // Zero in degenerate case

        if let Some(remaining) = (|| {
            rel.x
                .checked_add(rel.y.checked_mul(shape.x)?)?
                .checked_add(rel.z.checked_mul(shape.x)?.checked_mul(shape.y)?)?
                .try_into()
                .ok()
        })() {
            (remaining, Some(remaining))
        } else {
            (usize::MAX, None)
        }
    }
}

// ExactSizeIterator is not implemented because volume may be greater than usize::MAX.

/// Utility trait that adds various iterators.
pub trait VecIterators {
    /// Iterate an axis-aligned box defined by `self` (inclusive) and _end_ (exclusive).
    ///
    /// Iterator visits all integer points _p_ with `self.x <= p.x < end.x`,
    /// `self.y <= p.y < end.y`, `self.z <= p.z < end.z`. If any coordinate of _end_ is equal to or
    /// less than that of `self`, resulting region is empty and iterator always returns `None`.
    ///
    /// Visited points are ordered by _z_, then by _y_, then by _x_.
    fn iter_box(&self, end: &Self) -> Vec3BoxIter;
}

impl VecIterators for UVec3 {
    fn iter_box(&self, end: &Self) -> Vec3BoxIter {
        Vec3BoxIter::new(*self, *end)
    }
}
