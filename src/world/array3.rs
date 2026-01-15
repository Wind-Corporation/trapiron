//! An optimized dynamically allocated 3D array.

use crate::world::{
    UVec3,
    vec_iter::{Vec3BoxIter, VecIterators},
};

/// An optimized dynamically allocated 3D array of _T_.
pub struct Array3<T> {
    /// The objects stored in this array.
    ///
    /// # Invariant
    /// `self.data.len() == shape.x * shape.y * shape.z`.
    data: Vec<T>,

    /// The dimensions of this array.
    ///
    /// # Invariant
    /// `shape.x * shape.y * shape.z` does not overflow.
    shape: UVec3,
}

impl<T> Array3<T> {
    /// Compute the length of the data vector for the given shape, or detect that it would be too
    /// large.
    fn checked_len_from_shape(shape: UVec3) -> Option<usize> {
        shape
            .x
            .checked_mul(shape.y)?
            .checked_mul(shape.z)?
            .try_into()
            .ok()
    }

    /// Compute the data index of the element at _pos_ assuming _pos_ is in bounds.
    ///
    /// # Safety
    /// _pos_ must be in bounds for `self`. Out of bounds positions may lead to arithmetic overflow.
    unsafe fn unsafe_pos_to_index(&self, pos: UVec3) -> usize {
        (pos.x + pos.y * self.shape.x + pos.z * self.shape.x * self.shape.y) as usize
    }

    /// Check that _pos_ is in bounds and compute its data index.
    fn try_pos_to_index(&self, pos: UVec3) -> Option<usize> {
        if pos.cmplt(self.shape).all() {
            // SAFETY: Enforced with runtime check above.
            unsafe { Some(self.unsafe_pos_to_index(pos)) }
        } else {
            None
        }
    }

    /// Compute the data index of the element at _pos_, panicking if _pos_ is out of bounds.
    ///
    /// # Panics
    /// Panics if _pos_ is out of bounds, i.e. any of its coordinates is not less than that of the
    /// [shape](Self::shape) of this array.
    fn pos_to_index(&self, pos: UVec3) -> usize {
        self.try_pos_to_index(pos).expect(&format!(
            "Position should be between {} (inclusive) and {} (exclusive), got {}",
            UVec3::ZERO,
            self.shape,
            pos
        ))
    }

    /// Create a new array with given shape, generating its elements with _generator_.
    ///
    /// The generator is called with the positions of the elements it should provide.
    ///
    /// # Panics
    /// Panics if _shape_ is too large, or if memory allocation fails for backing vector.
    pub fn generate<F>(shape: UVec3, mut generator: F) -> Self
    where
        F: FnMut(UVec3) -> T,
    {
        let len = Self::checked_len_from_shape(shape)
            .expect("Array3 backing buffer size too large for usize");

        let mut data = Vec::with_capacity(len);
        for pos in UVec3::ZERO.iter_box(&shape) {
            data.push(generator(pos));
        }

        Self { data, shape }
    }
}

impl<T: Clone> Array3<T> {
    /// Create a new array with given shape, filled with clones of _value_.
    ///
    /// # Panics
    /// Panics if _shape_ is too large, or if memory allocation fails for backing vector.
    pub fn fill(shape: UVec3, value: &T) -> Self {
        Self::generate(shape, |_| value.clone())
    }
}

impl<T: Default> Array3<T> {
    /// Create a new array with given shape filled with default values.
    ///
    /// # Panics
    /// Panics if _shape_ is too large, or if memory allocation fails for backing vector.
    pub fn default(shape: UVec3) -> Self {
        Self::generate(shape, |_| Default::default())
    }
}

impl<T> Array3<T> {
    /// Get dimensions of `self`: size along X, Y and Z coordinates.
    ///
    /// This array contains positions (0; 0; 0) (inclusive) through `self.shape()` (exclusive).
    pub fn shape(&self) -> UVec3 {
        self.shape
    }
}

impl<T> std::ops::Index<UVec3> for Array3<T> {
    type Output = T;

    fn index(&self, pos: UVec3) -> &Self::Output {
        let index = self.pos_to_index(pos);
        unsafe { self.data.get_unchecked(index) }
    }
}

impl<T> std::ops::IndexMut<UVec3> for Array3<T> {
    fn index_mut(&mut self, pos: UVec3) -> &mut Self::Output {
        let index = self.pos_to_index(pos);
        unsafe { self.data.get_unchecked_mut(index) }
    }
}

macro_rules! position_iter {
    (name: $name:ident, elem_iter: $elem_iter:ty, ref: $ref:ty) => {
        pub struct $name<'a, T> {
            pos_iter: Vec3BoxIter,
            elem_iter: $elem_iter,
        }

        impl<'a, T> std::iter::Iterator for $name<'a, T> {
            type Item = (UVec3, $ref);
            fn next(&mut self) -> Option<Self::Item> {
                if let Some(element) = self.elem_iter.next() {
                    unsafe { Some((self.pos_iter.next_unchecked(), element)) }
                } else {
                    None
                }
            }
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.elem_iter.size_hint()
            }
        }

        impl<'a, T> std::iter::ExactSizeIterator for $name<'a, T> {
            fn len(&self) -> usize {
                self.elem_iter.len()
            }
        }
    };
}

position_iter! {name: PositionIter, elem_iter: std::slice::Iter<'a, T>, ref: &'a T}
position_iter! {name: PositionIterMut, elem_iter: std::slice::IterMut<'a, T>, ref: &'a mut T}

macro_rules! iter {
    (name: $name:ident, parent: $parent:ty, ref: $ref:ty) => {
        pub struct $name<'a, T>($parent);

        impl<'a, T> std::iter::Iterator for $name<'a, T> {
            type Item = $ref;
            fn next(&mut self) -> Option<Self::Item> {
                Some(self.0.next()?.1)
            }
        }

        impl<'a, T> std::iter::ExactSizeIterator for $name<'a, T> {
            fn len(&self) -> usize {
                self.0.len()
            }
        }
    };
}

impl<T> Array3<T> {
    /// Iterate over all positions in `self`.
    ///
    /// Positions are ordered by increasing Z, then by increasing Y, then by increasing X.
    ///
    /// See also [`Self::pos_iter`] to iterate over contents and positions at once.
    pub fn positions(&self) -> Vec3BoxIter {
        UVec3::ZERO.iter_box(&self.shape)
    }

    /// Iterate over all elements in `self`, each annotated with its position.
    ///
    /// Positions are ordered by increasing Z, then by increasing Y, then by increasing X.
    pub fn pos_iter(&self) -> PositionIter<'_, T> {
        PositionIter {
            pos_iter: self.positions(),
            elem_iter: self.data.iter(),
        }
    }

    /// Iterate over mutable references to all elements in `self`, each annotated with its position.
    ///
    /// Positions are ordered by increasing Z, then by increasing Y, then by increasing X.
    pub fn pos_iter_mut(&mut self) -> PositionIterMut<'_, T> {
        PositionIterMut {
            pos_iter: self.positions(),
            elem_iter: self.data.iter_mut(),
        }
    }
}

iter! {name: Iter, parent: PositionIter<'a, T>, ref: &'a T}
iter! {name: IterMut, parent: PositionIterMut<'a, T>, ref: &'a mut T}

impl<'a, T> std::iter::IntoIterator for &'a Array3<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.pos_iter())
    }
}

impl<'a, T> std::iter::IntoIterator for &'a mut Array3<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut(self.pos_iter_mut())
    }
}
