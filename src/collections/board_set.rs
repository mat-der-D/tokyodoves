//! A module containing a light [`Board`](`crate::Board`) container
//! [`BoardSet`] and associated items

use crate::collections::io::{Fragment, FragmentIter};
use crate::prelude::{Board, BoardBuilder};
use std::{
    collections::{HashMap, HashSet},
    io::{BufWriter, Read, Write},
};

fn u64_to_board(hash: u64) -> Board {
    BoardBuilder::from(hash).build_unchecked()
}

// ********************************************************************
//  Capacity
// ********************************************************************
/// A capacity, i.e., what size of memory is allocated by [`BoardSet`]
/// or [`RawBoardSet`].
///
/// Note that, unlike [`HashSet`],
/// the capacity does not behaves like `usize`.
/// It has an internal data `HashMap<u32, usize>`,
/// where keys represents top half of `u64` expression of [`Board`]s
/// and values represents how many elements the set can hold.
/// An addition of two capacities is defined by
/// the addition of values of internal hash maps
/// sharing the same keys.
///
///
/// # Examples
/// ```rust
/// use tokyodoves::Board;
/// use tokyodoves::collections::BoardSet;
///
/// let mut set = BoardSet::new();
/// set.insert(Board::new());
/// let capacity = set.capacity();
/// ```
#[derive(Debug, Clone, Default)]
pub struct Capacity(HashMap<u32, usize>);

impl PartialEq for Capacity {
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().all(|(t, n)| {
            other
                .0
                .get(t)
                .map(|nn| *n == *nn)
                .unwrap_or_else(|| *n == 0)
        })
    }
}

impl Eq for Capacity {}

impl Capacity {
    /// Returns an empty capacity.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::collections::{BoardSet, Capacity};
    ///
    /// let empty = BoardSet::new().capacity();
    /// assert_eq!(empty, Capacity::new());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of [`Board`]s (or `u64`s)
    /// the [`BoardSet`] (or [`RawBoardSet`]) with the capacity
    /// can hold.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set = BoardSet::new();
    /// set.insert(Board::new());
    /// assert!(set.capacity().len() >= 1);
    /// ```
    pub fn len(&self) -> usize {
        self.0.values().sum()
    }

    /// Returns `true` if no capacity.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set = BoardSet::new();
    /// assert!(set.capacity().is_empty());
    /// set.insert(Board::new());
    /// assert!(!set.capacity().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.values().all(|&n| n == 0)
    }
}

impl std::ops::Add for Capacity {
    type Output = Capacity;
    /// Creates a new capacity by adding the capacities of `self` and `rhs`.
    ///
    /// # Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use tokyodoves::{BoardBuilder, Board};
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut set_left = BoardSet::new();
    /// set_left.insert(Board::new());
    /// let mut set_right = BoardSet::new();
    /// set_left.insert(BoardBuilder::from_str("BbH")?.build()?);
    ///
    /// let capacity = set_left.capacity() + set_right.capacity();
    /// let mut set_added = BoardSet::with_capacity(capacity);
    /// set_added.extend(set_left); // without memory allocation
    /// set_added.extend(set_right); // without memory allocation
    /// # Ok(())
    /// # }
    /// ```
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign for Capacity {
    /// Adds the capacity of `rhs` to that of `self`.
    ///
    /// # Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use tokyodoves::{BoardBuilder, Board};
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut set_left = BoardSet::new();
    /// set_left.insert(Board::new());
    /// let mut set_right = BoardSet::new();
    /// set_left.insert(BoardBuilder::from_str("BbH")?.build()?);
    ///
    /// let mut capacity = set_left.capacity();
    /// capacity += set_right.capacity();
    /// let mut set_added = BoardSet::with_capacity(capacity);
    /// set_added.extend(set_left); // without memory allocation
    /// set_added.extend(set_right); // without memory allocation
    /// # Ok(())
    /// # }
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        for (top, num_bottoms) in rhs.0 {
            *self.0.entry(top).or_default() += num_bottoms;
        }
    }
}

// ********************************************************************
//  BoardSet
// ********************************************************************
/// A light set of [`Board`]s.
///
/// Its methods are similar to those of [`HashSet`](`std::collections::HashSet`).
/// See the documentation for quick understanding.
///
/// It has a [`RawBoardSet`] internally, which is a set of `u64` expressions of
/// [`Board`]s created by the [`to_u64`](`Board::to_u64`) method on [`Board`].
/// [`RawBoardSet`] also has similar methods to `BoardSet`
/// except that the elements are `u64`, not [`Board`].
///
/// In general, the memory size of the set is smaller than `HashSet<Board>`,
/// when they have the same number of elements.
///
/// `BoardSet` supports i/o utility methods [`load`](`BoardSet::load`),
/// [`load_filter`](`BoardSet::load_filter`) and [`save`](`BoardSet::save`).
/// A binary file will be saved when the [`save`](`BoardSet::save`) method is called,
/// which can be reloaded by the [`load`](`BoardSet::load`) method.
/// The [`load_filter`](`BoardSet::load_filter`) method provides a way
/// to load a part satisfying a criterion.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct BoardSet {
    raw: RawBoardSet,
}

impl std::fmt::Debug for BoardSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec: Vec<Board> = self.iter().collect();
        f.debug_set().entries(&vec).finish()
    }
}

impl<const N: usize> From<[Board; N]> for BoardSet {
    fn from(value: [Board; N]) -> Self {
        Self::from_iter(value)
    }
}

impl FromIterator<Board> for BoardSet {
    fn from_iter<T: IntoIterator<Item = Board>>(iter: T) -> Self {
        Self {
            raw: iter.into_iter().map(|b| b.to_u64()).collect(),
        }
    }
}

impl Extend<Board> for BoardSet {
    fn extend<T: IntoIterator<Item = Board>>(&mut self, iter: T) {
        self.raw.extend(iter.into_iter().map(|b| b.to_u64()))
    }
}

impl<'a> Extend<&'a Board> for BoardSet {
    fn extend<T: IntoIterator<Item = &'a Board>>(&mut self, iter: T) {
        self.raw.extend(iter.into_iter().map(|b| b.to_u64()))
    }
}

impl IntoIterator for BoardSet {
    type Item = Board;
    type IntoIter = IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(RawIntoIter::new(self.raw))
    }
}

impl<'a> IntoIterator for &'a BoardSet {
    type Item = Board;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl From<RawBoardSet> for BoardSet {
    fn from(raw: RawBoardSet) -> Self {
        Self::from_raw(raw)
    }
}

impl BoardSet {
    /// Creates an empty `BoardSet`.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::collections::BoardSet;
    /// let set = BoardSet::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an `BoardSet` by loading a file.
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let set = BoardSet::new_from_file(path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_from_file(path: impl AsRef<std::path::Path>) -> std::io::Result<BoardSet> {
        let raw_set = RawBoardSet::new_from_file(path)?;
        Ok(raw_set.into())
    }

    /// Returns a reference to the internal [`RawBoardSet`].
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::{BoardSet, board_set::RawBoardSet};
    ///
    /// let board = Board::new();
    /// let mut set = BoardSet::new();
    /// set.insert(board);
    /// let mut raw_set = RawBoardSet::new();
    /// raw_set.insert(board.to_u64());
    /// assert_eq!(*set.raw(), raw_set);
    /// ```
    pub fn raw(&self) -> &RawBoardSet {
        &self.raw
    }

    /// Returns a mutable reference to the internal [`RawBoardSet`].
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let board = Board::new();
    /// let mut set1 = BoardSet::new();
    /// set1.insert(board);
    /// let mut set2 = BoardSet::new();
    /// set2.raw_mut().insert(board.to_u64());
    /// assert_eq!(set1, set2);
    /// ```
    pub fn raw_mut(&mut self) -> &mut RawBoardSet {
        &mut self.raw
    }

    /// Returns the internal [`RawBoardSet`].
    ///
    /// The ownership of the set is moved.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::{BoardSet, board_set::RawBoardSet};
    ///
    /// let board = Board::new();
    /// let mut set1 = BoardSet::new();
    /// set1.insert(board);
    /// let raw_set1 = set1.into_raw();
    /// let mut raw_set2 = RawBoardSet::new();
    /// raw_set2.insert(board.to_u64());
    /// assert_eq!(raw_set1, raw_set2);
    /// ```
    pub fn into_raw(self) -> RawBoardSet {
        self.raw
    }

    /// Creates the set that has a [`RawBoardSet`] internally.
    ///
    /// The ownership of `raw` is moved.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::{BoardSet, board_set::RawBoardSet};
    ///
    /// let board = Board::new();
    /// let mut raw_set1 = RawBoardSet::new();
    /// let set1 = BoardSet::from_raw(raw_set1);
    /// let mut set2 = BoardSet::new();
    /// set2.insert(board);
    /// assert_eq!(set1, set2);
    /// ```
    pub fn from_raw(raw: RawBoardSet) -> Self {
        Self { raw }
    }

    /// Returns [`Capacity`] required to load all elements specified by `reader`.
    pub fn required_capacity<R>(reader: R) -> std::io::Result<Capacity>
    where
        R: Read,
    {
        RawBoardSet::required_capacity(reader)
    }

    /// Returns [`Capacity`] required to load all elements (`e`) specified by `reader`,
    /// under the condition of `f` (where `f(&e)` returns `true`).
    pub fn required_capacity_filter<R, F>(reader: R, f: F) -> std::io::Result<Capacity>
    where
        R: Read,
        F: FnMut(&u64) -> bool,
    {
        RawBoardSet::required_capacity_filter(reader, f)
    }

    /// Creates an empty `BoardSet` with at least the specified capacity.
    ///
    /// The set will be able to hold sufficient elements required by `capacity`
    /// without reallocating.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    /// let mut set0 = BoardSet::new();
    /// set0.insert(Board::new());
    /// let capacity = set0.capacity();
    /// let set1 = BoardSet::with_capacity(capacity);
    /// ```
    pub fn with_capacity(capacity: Capacity) -> Self {
        Self {
            raw: RawBoardSet::with_capacity(capacity),
        }
    }

    /// Returns the [`Capacity`] of the set.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::collections::BoardSet;
    /// let set = BoardSet::new();
    /// let capacity = set.capacity();
    /// ```
    pub fn capacity(&self) -> Capacity {
        self.raw.capacity()
    }

    /// An iterator visiting all elements in arbitrary order.
    /// The iterator element type is [`Board`].
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    /// let mut set = BoardSet::new();
    /// set.insert(Board::new());
    /// for x in set.iter() {
    ///     println!("{x}");
    /// }
    /// ```
    pub fn iter(&self) -> Iter {
        Iter(RawIter::new(&self.raw))
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    /// let mut set = BoardSet::new();
    /// assert_eq!(set.len(), 0);
    /// set.insert(Board::new());
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    /// let mut set = BoardSet::new();
    /// assert!(set.is_empty());
    /// set.insert(Board::new());
    /// assert!(!set.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Clears the set, returning all elements as an iterator.
    /// Keeps the allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed,
    /// it drops the remaining elements.
    /// The returned iterator keeps a mutable borrow on the set to optimize
    /// its implementation.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::{BoardSet, Capacity};
    ///
    /// let mut set = BoardSet::new();
    /// set.insert(Board::new());
    /// set.drain();
    /// assert!(set.is_empty());
    /// assert_ne!(set.capacity(), Capacity::new());
    /// ```
    pub fn drain(&mut self) -> Drain {
        Drain(RawDrain::new(&mut self.raw))
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
    /// The elements are visited in unsorted (and unspecified) order.
    ///
    /// # Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use tokyodoves::{Board, BoardBuilder};
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut set = BoardSet::new();
    /// set.insert(Board::new());
    /// let set1 = set.clone();
    /// set.insert(BoardBuilder::from_str("BbA")?.build()?);
    /// set.retain(|b| b.count_doves_on_field() == 2);
    /// assert_eq!(set1, set);
    /// # Ok(())
    /// # }
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Board) -> bool,
    {
        self.raw.retain(|&h| f(&u64_to_board(h)))
    }

    /// Removes all loaded values from the set.
    ///
    /// It returns `Ok(true)` if some elements in the set are removed.
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::BoardSet;
    /// use tokyodoves::Board;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let mut set = BoardSet::new();
    /// set.insert(Board::new());
    /// set.remove_by_loading(File::open(path)?)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_loaded_values<R>(&mut self, reader: R) -> std::io::Result<bool>
    where
        R: Read,
    {
        self.raw.remove_loaded_values(reader)
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set = BoardSet::new();
    /// set.insert(Board::new());
    /// set.clear();
    /// assert!(set.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.raw.clear()
    }

    /// Reserves capacity for at least `additional` more elements
    /// to be inserted in the `BoardSet` without reallocating.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set0 = BoardSet::new();
    /// set0.insert(Board::new());
    /// let capacity = set0.capacity();
    /// let mut set1 = BoardSet::new();
    /// set1.reserve(capacity);
    /// ```
    pub fn reserve(&mut self, additional: Capacity) {
        self.raw.reserve(additional)
    }

    /// Shrinks the capacity of the set as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit()
    }

    /// Visits the boards representing the difference, i.e.,
    /// the boards that are in `self` but not in `other`.
    pub fn difference<'a>(&'a self, other: &'a BoardSet) -> Difference<'a> {
        Difference(RawDifference::new(&self.raw, &other.raw))
    }

    /// Visits the boards representing the symmetric difference, i.e.,
    /// the boards that are in `self` or in `other` but not in both.
    pub fn symmetric_difference<'a>(&'a self, other: &'a BoardSet) -> SymmetricDifference<'a> {
        SymmetricDifference(RawSymmetricDifference::new(&self.raw, &other.raw))
    }

    /// Visits the boards representing the intersection, i.e.,
    /// the boards that are both in `self` and `other`.
    pub fn intersection<'a>(&'a self, other: &'a BoardSet) -> Intersection<'a> {
        Intersection(RawIntersection::new(&self.raw, &other.raw))
    }

    /// Visits the boards representing the union, i.e.,
    /// all the boards in `self` or `other`, without duplicates.
    pub fn union<'a>(&'a self, other: &'a BoardSet) -> Union<'a> {
        Union(RawUnion::new(&self.raw, &other.raw))
    }

    /// Returns `true` if the set contains a board.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set = BoardSet::new();
    /// let board = Board::new();
    /// set.insert(board);
    /// assert!(set.contains(&board));
    /// ```
    pub fn contains(&self, board: &Board) -> bool {
        self.raw.contains(&board.to_u64())
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use tokyodoves::{Board, BoardBuilder};
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let board0 = Board::new();
    /// let board1 = BoardBuilder::from_str("BbH")?.build()?;
    ///
    /// let mut set0 = BoardSet::new();
    /// set0.insert(board0);
    /// let mut set1 = BoardSet::new();
    /// set1.insert(board1);
    /// assert!(set0.is_disjoint(&set1));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_disjoint(&self, other: &BoardSet) -> bool {
        self.raw.is_disjoint(&other.raw)
    }

    /// Returns `true` if the set is a subset of another, i.e.,
    /// `other` contains at least all the boards in `self`.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set0 = BoardSet::new();
    /// set0.insert(Board::new());
    /// let set1 = BoardSet::new();
    /// assert!(set1.is_subset(&set0));
    /// ```
    pub fn is_subset(&self, other: &BoardSet) -> bool {
        self.raw.is_subset(&other.raw)
    }

    /// Returns `true` if the set is a superset of another, i.e.,
    /// `self` contains at least all the boards in `other`.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set0 = BoardSet::new();
    /// set0.insert(Board::new());
    /// let set1 = BoardSet::new();
    /// assert!(set0.is_superset(&set1));
    /// ```
    pub fn is_superset(&self, other: &BoardSet) -> bool {
        self.raw.is_superset(&other.raw)
    }

    /// Adds a board to the set.
    ///
    /// Returns whether the board was newly inserted. That is:
    /// - If the set did not previously contain this board, `true` is returned.
    /// - If the set already contained this board, `false` is returned.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set = BoardSet::new();
    /// assert_eq!(set.len(), 0);
    /// set.insert(Board::new());
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn insert(&mut self, board: Board) -> bool {
        self.raw.insert(board.to_u64())
    }

    /// Removes a board from the set.
    /// Returns whether the board was present in the set.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set = BoardSet::new();
    /// let board = Board::new();
    /// set.insert(board);
    /// assert_eq!(set.len(), 1);
    /// set.remove(&board);
    /// assert_eq!(set.len(), 0);
    /// ```
    pub fn remove(&mut self, board: &Board) -> bool {
        self.raw.remove(&board.to_u64())
    }

    /// Removes and returns the board in the set, if any,
    /// that is equal to the given one.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set = BoardSet::new();
    /// let board = Board::new();
    /// set.insert(board);
    /// assert_eq!(set.len(), 1);
    /// set.take(&board);
    /// assert_eq!(set.len(), 0);
    pub fn take(&mut self, board: &Board) -> Option<Board> {
        self.raw.take(&board.to_u64()).map(u64_to_board)
    }

    /// Captures the ownership of the given set and absorb all elements in it.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set1 = BoardSet::new();
    /// let mut set2 = BoardSet::new();
    /// set2.insert(Board::new());
    /// set1.absorb(set2);
    /// assert_eq!(set1.len(), 1);
    /// ```
    /// If the size of the set to be absorbed is large,
    /// allocating memory in advance may accelerate the process:
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set1 = BoardSet::new();
    /// let mut set2 = BoardSet::new();
    /// set2.insert(Board::new());
    ///
    /// // Suppose set2 is very large
    /// set1.reserve(set2.capacity()); // allocate sufficient memory
    /// set1.absorb(set2);
    /// assert_eq!(set1.len(), 1);
    /// ```
    pub fn absorb(&mut self, set: BoardSet) {
        self.raw.absorb(set.raw);
    }

    /// Absorb all elements drained from the given set.
    /// The absorbed set will be empty, while it will keep the allocated memory.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set1 = BoardSet::new();
    /// let mut set2 = BoardSet::new();
    /// set2.insert(Board::new());
    /// set1.absorb_drained(&mut set2);
    /// assert_eq!(set1.len(), 1);
    /// assert!(set2.is_empty());
    /// ```
    /// If the size of the set to be absorbed is large,
    /// allocating memory in advance may accelerate the process:
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set1 = BoardSet::new();
    /// let mut set2 = BoardSet::new();
    /// set2.insert(Board::new());
    ///
    /// // Suppose set2 is very large
    /// set1.reserve(set2.capacity()); // allocate sufficient memory
    /// set1.absorb_drained(&mut set2);
    /// assert_eq!(set1.len(), 1);
    /// assert!(set2.is_empty());
    /// ```
    pub fn absorb_drained(&mut self, set: &mut BoardSet) {
        self.raw.absorb_drained(&mut set.raw);
    }

    /// Inserts all elements given by `reader` into `self`.
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let mut set = BoardSet::new();
    /// set.load(File::open(path)?);
    /// # Ok(())
    /// # }
    /// ```
    /// The following is more efficient especially when the target file is large.
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let capacity = BoardSet::required_capacity(File::open(path)?)?;
    /// let mut set = BoardSet::with_capacity(capacity);
    /// set.load(File::open(path)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn load<R>(&mut self, reader: R) -> std::io::Result<()>
    where
        R: Read,
    {
        self.raw.load(reader)
    }

    /// Inserts all elements (`e`) given by `reader` under the condition of `f`
    /// (where `f(&e)` is `true`) into `self`.
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let filter = |board| board.count_doves_on_field() >= 3;
    /// let mut set = BoardSet::new();
    /// set.load_filter(File::open(path)?, filter);
    /// # Ok(())
    /// # }
    /// ```
    /// The following is more efficient especially when the target file is large.
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let filter = |board| board.count_doves_on_field() >= 3;
    /// let capacity = BoardSet::required_capacity_filter(File::open(path)?, filter);
    /// let mut set = BoardSet::with_capacity(capacity);
    /// set.load_filter(File::open(path)?, filter);
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_filter<R, F>(&mut self, reader: R, f: F) -> std::io::Result<()>
    where
        R: Read,
        F: FnMut(&u64) -> bool,
    {
        self.raw.load_filter(reader, f)
    }

    /// Writes all elements in the set to `writer`.
    /// The saved data can be loaded both
    /// by the [`load`](`BoardSet::load`) method on [`BoardSet`],
    /// the [`load_filter`](`BoardSet::load_filter`) method on `BoardSet`,
    /// the [`load`](`RawBoardSet::load`) method on [`RawBoardSet`]
    /// and the [`load_filter`](`RawBoardSet::load_filter`) method on `RawBoardSet`.
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::BoardSet;
    ///
    /// let mut set = BoardSet::new();
    /// set.insert(Board::new());
    /// let target_path = "/some/target/path.tdl";
    /// set.save(File::create(target_path)?);
    /// ```
    pub fn save<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: Write,
    {
        self.raw.save(writer)
    }

    /// Splits the set into two sets.
    ///
    /// The argument `left_len` indicates the length of the left component
    /// of the returned value.
    ///
    /// # Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use tokyodoves::{Board, BoardBuilder};
    /// use tokyodoves::collections::BoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut set = BoardSet::new();
    /// set.insert(Board::new());
    /// set.insert(BoardBuilder::from_str("BbH")?.build()?);
    /// set.insert(BoardBuilder::from_str("BbA")?.build()?);
    /// let (left, right) = set.split(2);
    /// assert_eq!(left.len(), 2);
    /// assert_eq!(right.len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn split(self, left_len: usize) -> (Self, Self) {
        let (left_raw, right_raw) = self.into_raw().split(left_len);
        (left_raw.into(), right_raw.into())
    }
}

impl std::ops::BitAnd<&BoardSet> for &BoardSet {
    type Output = BoardSet;
    /// Returns the intersection of `self` and `rhs` as a new `BoardSet`.
    fn bitand(self, rhs: &BoardSet) -> Self::Output {
        Self::Output {
            raw: self.raw.bitand(&rhs.raw),
        }
    }
}

impl std::ops::BitOr<&BoardSet> for &BoardSet {
    type Output = BoardSet;
    /// Returns the union of `self` and `rhs` as a new `BoardSet`.
    fn bitor(self, rhs: &BoardSet) -> Self::Output {
        Self::Output {
            raw: self.raw.bitor(&rhs.raw),
        }
    }
}

impl std::ops::BitXor<&BoardSet> for &BoardSet {
    type Output = BoardSet;
    /// Returns the symmetric difference of `self` and `rhs` as a new `BoardSet`.
    fn bitxor(self, rhs: &BoardSet) -> Self::Output {
        Self::Output {
            raw: self.raw.bitxor(&rhs.raw),
        }
    }
}

impl std::ops::Sub<&BoardSet> for &BoardSet {
    type Output = BoardSet;
    /// Returns the difference of `self` and `rhs` as a new `BoardSet`.
    fn sub(self, rhs: &BoardSet) -> Self::Output {
        Self::Output {
            raw: self.raw.sub(&rhs.raw),
        }
    }
}

/// An owing iterator over the items of a [`BoardSet`].
///
/// This struct is created by the [`into_iter`](`IntoIterator::into_iter`)
/// method on [`BoardSet`] (provided by the [`IntoIterator`] trait).
/// See the documentation for more.
///
/// # Examples
/// ```rust
/// use tokyodoves::Board;
/// use tokyodoves::collections::BoardSet;
///
/// let mut set = BoardSet::new();
/// set.insert(Board::new());
/// let mut iter = set.into_iter();
/// ```
pub struct IntoIter(RawIntoIter);

/// A draining iterator over the items of a [`BoardSet`].
///
/// This struct is created by the [`drain`](`BoardSet::drain`) method
/// on [`BoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use tokyodoves::Board;
/// use tokyodoves::collections::BoardSet;
///
/// let mut set = BoardSet::new();
/// set.insert(Board::new());
/// let mut drain = set.drain();
/// ```
pub struct Drain<'a>(RawDrain<'a>);

/// An iterator over the items of a [`BoardSet`].
///
/// This struct is created by the [`iter`](`BoardSet::iter`) method
/// on [`BoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use tokyodoves::Board;
/// use tokyodoves::collections::BoardSet;
///
/// let mut set = BoardSet::new();
/// set.insert(Board::new());
/// let mut iter = set.iter();
/// ```
#[derive(Clone)]
pub struct Iter<'a>(RawIter<'a>);

/// A lazy iterator producing elements in the difference of [`BoardSet`]s.
///
/// This struct is created by the [`difference`](`BoardSet::difference`) method
/// on [`BoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use std::str::FromStr;
/// use tokyodoves::{Board, BoardBuilder};
/// use tokyodoves::collections::BoardSet;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut set1 = BoardSet::new();
/// set1.insert(Board::new());
/// let mut set2 = BoardSet::new();
/// set2.insert(BoardBuilder::from_str("BbH")?.build()?);
///
/// let mut difference = set1.difference(&set2);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Difference<'a>(RawDifference<'a>);

/// A lazy iterator producing elements in the symmetric difference of [`BoardSet`]s.
///
/// This struct is created by the [`symmetric_difference`](`BoardSet::symmetric_difference`) method
/// on [`BoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use std::str::FromStr;
/// use tokyodoves::{Board, BoardBuilder};
/// use tokyodoves::collections::BoardSet;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut set1 = BoardSet::new();
/// set1.insert(Board::new());
/// let mut set2 = BoardSet::new();
/// set2.insert(BoardBuilder::from_str("BbH")?.build()?);
///
/// let mut symmetric_difference = set1.symmetric_difference(&set2);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct SymmetricDifference<'a>(RawSymmetricDifference<'a>);

/// A lazy iterator producing elements in the intersection of [`BoardSet`]s.
///
/// This struct is created by the [`intersection`](`BoardSet::intersection`) method
/// on [`BoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use std::str::FromStr;
/// use tokyodoves::{Board, BoardBuilder};
/// use tokyodoves::collections::BoardSet;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut set1 = BoardSet::new();
/// set1.insert(Board::new());
/// let mut set2 = BoardSet::new();
/// set2.insert(BoardBuilder::from_str("BbH")?.build()?);
///
/// let mut intersection = set1.intersection(&set2);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Intersection<'a>(RawIntersection<'a>);

/// A lazy iterator producing elements in the union of [`BoardSet`]s.
///
/// This struct is created by the [`union`](`BoardSet::union`) method
/// on [`BoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use std::str::FromStr;
/// use tokyodoves::{Board, BoardBuilder};
/// use tokyodoves::collections::BoardSet;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut set1 = BoardSet::new();
/// set1.insert(Board::new());
/// let mut set2 = BoardSet::new();
/// set2.insert(BoardBuilder::from_str("BbH")?.build()?);
///
/// let mut union_iter = set1.union(&set2);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Union<'a>(RawUnion<'a>);

macro_rules! impl_debug {
    (<$iter_name:expr, $iter:ident>) => {
        impl<'a> std::fmt::Debug for $iter<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let vec: Vec<<Self as Iterator>::Item> = self.clone().collect();
                f.debug_tuple($iter_name).field(&vec).finish()
            }
        }
    };

    ($(<$iters2_name:expr, $iters2:ident>)*) => {
        $(impl_debug!(<$iters2_name, $iters2>);)*
    };
}

macro_rules! impl_debug_sealed {
    ({$iter_name:expr, $iter:ident}) => {
        impl std::fmt::Debug for $iter {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}([..])", $iter_name)
            }
        }
    };

    (<$iter_name:expr, $iter:ident>) => {
        impl<'a> std::fmt::Debug for $iter<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}([..])", $iter_name)
            }
        }
    };

    ($({$iters1_name:expr, $iters1:ident})* $(<$iters2_name:expr, $iters2:ident>)*) => {
        $(impl_debug_sealed!({$iters1_name, $iters1});)*
        $(impl_debug_sealed!(<$iters2_name, $iters2>);)*
    };
}

macro_rules! impl_iterators {
    ({$iter:ident => $raw:ident}) => {
        impl Iterator for $iter {
            type Item = Board;
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next().map(u64_to_board)
            }
        }
    };

    (<$iter:ident => $raw:ident>) => {
        impl<'a> Iterator for $iter<'a> {
            type Item = Board;
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next().map(u64_to_board)
            }
        }
    };

    ($({$iters1:ident => $raws1:ident})* $(<$iters2:ident => $raws2:ident>)*) => {
        $(impl_iterators!({$iters1 => $raws1});)*
        $(impl_iterators!(<$iters2 => $raws2>);)*
    };
}

impl_debug!(
    < "Iter", Iter >
    < "RawIter", RawIter >
    < "Difference", Difference >
    < "RawDifference", RawDifference >
    < "SymmetricDifference", SymmetricDifference >
    < "RawSymmetricDifference", RawSymmetricDifference >
    < "Intersection", Intersection >
    < "RawIntersection", RawIntersection >
    < "Union", Union >
    < "RawUnion", RawUnion >
);

impl_debug_sealed!(
    { "IntoIter", IntoIter }
    { "RawIntoIter", RawIntoIter }
    < "Drain", Drain >
    < "RawDrain", RawDrain >
);

impl_iterators!(
    { IntoIter => RawIntoIter }
    < Drain => RawDrain >
    < Iter => RawIter >
    < Difference => RawDifference >
    < SymmetricDifference => RawSymmetricDifference >
    < Intersection => RawIntersection >
    < Union => RawUnion >
);

// ********************************************************************
//  BoardSet
// ********************************************************************
/// A set of [`u64`] built in [`BoardSet`].
///
/// Its methods are similar to those of [`HashSet`](`std::collections::HashSet`).
/// Furthermore, the methods on `RawBoardSet` are almost the same as those on [`BoardSet`].
/// See their documentations for quick understanding.
///
/// As an internal data of [`BoardSet`],
/// items contained in `RawBoardSet` are `u64` expressions of [`Board`]s
/// created by [`to_u64`](`Board::to_u64`) method on [`Board`].
/// Almost all methods on [`BoardSet`] are implemented simply by calling
/// the methods of `RawBoardSet` with the same name
/// and converting inputs or outputs between `u64` and [`Board`].
#[derive(Clone, Default)]
pub struct RawBoardSet {
    pub(crate) top2bottoms: HashMap<u32, HashSet<u32>>,
}

impl std::fmt::Debug for RawBoardSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vec: Vec<u64> = self.iter().collect();
        f.debug_set().entries(&vec).finish()
    }
}

impl PartialEq for RawBoardSet {
    fn eq(&self, other: &Self) -> bool {
        self.top2bottoms.iter().all(|(t, b)| {
            other
                .top2bottoms
                .get(t)
                .map(|bb| *b == *bb)
                .unwrap_or_else(|| b.is_empty())
        })
    }
}

impl Eq for RawBoardSet {}

impl From<BoardSet> for RawBoardSet {
    fn from(value: BoardSet) -> Self {
        value.into_raw()
    }
}

impl<const N: usize> From<[u64; N]> for RawBoardSet {
    fn from(value: [u64; N]) -> Self {
        Self::from_iter(value)
    }
}

impl FromIterator<u64> for RawBoardSet {
    fn from_iter<T: IntoIterator<Item = u64>>(iter: T) -> Self {
        let mut set = Self::new();
        for item in iter {
            set.insert(item);
        }
        set
    }
}

impl Extend<u64> for RawBoardSet {
    fn extend<T: IntoIterator<Item = u64>>(&mut self, iter: T) {
        iter.into_iter().for_each(|h| {
            self.insert(h);
        });
    }
}

impl<'a> Extend<&'a u64> for RawBoardSet {
    fn extend<T: IntoIterator<Item = &'a u64>>(&mut self, iter: T) {
        self.extend(iter.into_iter().cloned())
    }
}

impl IntoIterator for RawBoardSet {
    type Item = u64;
    type IntoIter = RawIntoIter;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<'a> IntoIterator for &'a RawBoardSet {
    type Item = u64;
    type IntoIter = RawIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl RawBoardSet {
    /// Creates an empty `RawBoardSet`.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::collections::board_set::RawBoardSet;
    /// let set = RawBoardSet::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an `RawBoardSet` by loading a file.
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let set = RawBoardSet::new_from_file(path)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_from_file(path: impl AsRef<std::path::Path>) -> std::io::Result<RawBoardSet> {
        let capacity = Self::required_capacity(std::fs::File::open(&path)?)?;
        let mut set = Self::with_capacity(capacity);
        set.load(std::fs::File::open(path)?)?;
        Ok(set)
    }

    /// Returns [`Capacity`] required to load all elements specified by `reader`.
    pub fn required_capacity<R>(reader: R) -> std::io::Result<Capacity>
    where
        R: Read,
    {
        let mut count = HashMap::new();
        let mut top = 0;
        let mut iter = FragmentIter::new(reader);
        loop {
            let Some(fragment) = iter.try_next()? else {
                break;
            };
            use Fragment::*;
            match fragment {
                Delimiter => continue,
                Top(top_) => top = top_,
                Bottom(_) => *count.entry(top).or_default() += 1,
            }
        }
        Ok(Capacity(count))
    }

    /// Returns [`Capacity`] required to load all elements (`e`) specified by `reader`,
    /// under the condition of `f` (where `f(&e)` returns `true`).
    pub fn required_capacity_filter<R, F>(reader: R, mut f: F) -> std::io::Result<Capacity>
    where
        R: Read,
        F: FnMut(&u64) -> bool,
    {
        let mut count = HashMap::new();
        let mut top = 0;
        let mut iter = FragmentIter::new(reader);

        loop {
            let Some(fragment) = iter.try_next()? else {
                break;
            };
            use Fragment::*;
            match fragment {
                Delimiter => continue,
                Top(top_) => top = top_,
                Bottom(bottom) => {
                    let hash = Self::u32_u32_to_u64(top, bottom);
                    if f(&hash) {
                        *count.entry(top).or_default() += 1;
                    }
                }
            }
        }
        Ok(Capacity(count))
    }

    /// Creates an empty `BoardSet` with at least the specified capacity.
    ///
    /// The set will be able to hold sufficient elements required by `capacity`
    /// without reallocating.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set0 = RawBoardSet::new();
    /// set0.insert(Board::new().to_u64());
    /// let capacity = set0.capacity();
    /// let set1 = RawBoardSet::with_capacity(capacity);
    /// ```
    pub fn with_capacity(capacity: Capacity) -> Self {
        let mut top2bottoms = HashMap::with_capacity(capacity.0.len());
        for (top, num_bottoms) in capacity.0 {
            top2bottoms.insert(top, HashSet::with_capacity(num_bottoms));
        }
        Self { top2bottoms }
    }

    /// Returns the [`Capacity`] of the set.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::collections::board_set::RawBoardSet;
    /// let set = RawBoardSet::new();
    /// let capacity = set.capacity();
    /// ```
    pub fn capacity(&self) -> Capacity {
        let mut count = HashMap::with_capacity(self.top2bottoms.len());
        for (k, v) in self.top2bottoms.iter() {
            count.insert(*k, v.capacity());
        }
        Capacity(count)
    }

    /// An iterator visiting all elements in arbitrary order.
    /// The iterator element type is [`Board`].
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    /// let mut set = RawBoardSet::new();
    /// set.insert(Board::new().to_u64());
    /// for x in set.iter() {
    ///     println!("{x}");
    /// }
    /// ```
    pub fn iter(&self) -> RawIter {
        RawIter::new(self)
    }

    pub(crate) fn u64_to_u32_u32(n: u64) -> (u32, u32) {
        ((n >> 32) as u32, n as u32)
    }

    pub(crate) fn u32_u32_to_u64(top: u32, bottom: u32) -> u64 {
        ((top as u64) << 32) | (bottom as u64)
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    /// let mut set = RawBoardSet::new();
    /// assert_eq!(set.len(), 0);
    /// set.insert(Board::new().to_u64());
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.top2bottoms.values().map(|s| s.len()).sum()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    /// let mut set = RawBoardSet::new();
    /// assert!(set.is_empty());
    /// set.insert(Board::new().to_u64());
    /// assert!(!set.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.top2bottoms.values().all(|s| s.is_empty())
    }

    /// Clears the set, returning all elements as an iterator.
    /// Keeps the allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed,
    /// it drops the remaining elements.
    /// The returned iterator keeps a mutable borrow on the set to optimize
    /// its implementation.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::Capacity;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set = RawBoardSet::new();
    /// set.insert(Board::new().to_u64());
    /// set.drain();
    /// assert!(set.is_empty());
    /// assert_ne!(set.capacity(), Capacity::new());
    /// ```
    pub fn drain(&mut self) -> RawDrain {
        RawDrain::new(self)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
    /// The elements are visited in unsorted (and unspecified) order.
    ///
    /// # Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use tokyodoves::{Board, BoardBuilder};
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut set = RawBoardSet::new();
    /// set.insert(Board::new().to_u64());
    /// let set1 = set.clone();
    /// set.insert(BoardBuilder::from_str("BbA")?.build()?.to_u64());
    /// set.retain(|b| (b & 0xfff << 48).count_ones() == 2);
    /// assert_eq!(set1, set);
    /// # Ok(())
    /// # }
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&u64) -> bool,
    {
        for (&top, bottoms) in self.top2bottoms.iter_mut() {
            bottoms.retain(|&b| {
                let hash = RawBoardSet::u32_u32_to_u64(top, b);
                f(&hash)
            });
        }
    }

    /// Removes all loaded values from the set.
    ///
    /// It returns `Ok(true)` if some elements in the set are removed.
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    /// use tokyodoves::Board;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let mut set = RawBoardSet::new();
    /// set.insert(Board::new().to_u64());
    /// set.remove_by_loading(File::open(path)?)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_loaded_values<R>(&mut self, reader: R) -> std::io::Result<bool>
    where
        R: Read,
    {
        let mut removed = false;
        let mut dummy = HashSet::new();
        let mut bottoms = &mut dummy;
        let mut is_capturing = false;
        let mut iter = FragmentIter::new(reader);
        loop {
            let Some(fragment) = iter.try_next()? else {
                return Ok(removed);
            };

            use Fragment::*;
            match fragment {
                Delimiter => continue,
                Top(top_) => match self.top2bottoms.get_mut(&top_) {
                    Some(bottoms_) => {
                        is_capturing = true;
                        bottoms = bottoms_;
                    }
                    None => {
                        is_capturing = false;
                        bottoms = &mut dummy;
                    }
                },
                Bottom(bottom_) => {
                    if !is_capturing {
                        continue;
                    }
                    removed |= bottoms.remove(&bottom_);
                }
            }
        }
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set = RawBoardSet::new();
    /// set.insert(Board::new().to_u64());
    /// set.clear();
    /// assert!(set.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.top2bottoms.clear()
    }

    /// Reserves capacity for at least `additional` more elements
    /// to be inserted in the `RawBoardSet` without reallocating.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set0 = RawBoardSet::new();
    /// set0.insert(Board::new().to_u64());
    /// let capacity = set0.capacity();
    /// let mut set1 = RawBoardSet::new();
    /// set1.reserve(capacity);
    /// ```
    pub fn reserve(&mut self, additional: Capacity) {
        for (top, additional_len) in additional.0 {
            match self.top2bottoms.get_mut(&top) {
                Some(bottoms) => {
                    bottoms.reserve(additional_len);
                }
                None => {
                    self.top2bottoms
                        .insert(top, HashSet::with_capacity(additional_len));
                }
            };
        }
    }

    /// Shrinks the capacity of the set as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.top2bottoms.retain(|_, v| !v.is_empty());
        self.top2bottoms.shrink_to_fit();
        self.top2bottoms
            .values_mut()
            .for_each(|b| b.shrink_to_fit());
    }

    /// Visits the values representing the difference, i.e.,
    /// the values that are in `self` but not in `other`.
    pub fn difference<'a>(&'a self, other: &'a RawBoardSet) -> RawDifference<'a> {
        RawDifference::new(self, other)
    }

    /// Visits the values representing the symmetric difference, i.e.,
    /// the values that are in `self` or in `other` but not in both.
    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a RawBoardSet,
    ) -> RawSymmetricDifference<'a> {
        RawSymmetricDifference::new(self, other)
    }

    /// Visits the values representing the intersection, i.e.,
    /// the values that are both in `self` and `other`.
    pub fn intersection<'a>(&'a self, other: &'a RawBoardSet) -> RawIntersection<'a> {
        RawIntersection::new(self, other)
    }

    /// Visits the values representing the union, i.e.,
    /// all the values in `self` or `other`, without duplicates.
    pub fn union<'a>(&'a self, other: &'a RawBoardSet) -> RawUnion<'a> {
        RawUnion::new(self, other)
    }

    /// Returns `true` if the set contains a value.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set = RawBoardSet::new();
    /// let hash = Board::new().to_u64();
    /// set.insert(hash);
    /// assert!(set.contains(&hash));
    /// ```
    pub fn contains(&self, hash: &u64) -> bool {
        let (k, v) = Self::u64_to_u32_u32(*hash);
        self.top2bottoms.get(&k).map_or(false, |x| x.contains(&v))
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use tokyodoves::{Board, BoardBuilder};
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let hash0 = Board::new().to_u64();
    /// let hash1 = BoardBuilder::from_str("BbH")?.build()?.to_u64();
    ///
    /// let mut set0 = RawBoardSet::new();
    /// set0.insert(hash0);
    /// let mut set1 = RawBoardSet::new();
    /// set1.insert(hash1);
    /// assert!(set0.is_disjoint(&set1));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_disjoint(&self, other: &RawBoardSet) -> bool {
        if self.len() <= other.len() {
            self.iter().all(|v| !other.contains(&v))
        } else {
            other.iter().all(|v| !self.contains(&v))
        }
    }

    /// Returns `true` if the set is a subset of another, i.e.,
    /// `other` contains at least all the boards in `self`.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set0 = RawBoardSet::new();
    /// set0.insert(Board::new().to_u64());
    /// let set1 = RawBoardSet::new();
    /// assert!(set1.is_subset(&set0));
    /// ```
    pub fn is_subset(&self, other: &RawBoardSet) -> bool {
        if self.len() <= other.len() {
            self.iter().all(|v| other.contains(&v))
        } else {
            false
        }
    }

    /// Returns `true` if the set is a superset of another, i.e.,
    /// `self` contains at least all the boards in `other`.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set0 = RawBoardSet::new();
    /// set0.insert(Board::new().to_u64());
    /// let set1 = RawBoardSet::new();
    /// assert!(set0.is_superset(&set1));
    /// ```
    pub fn is_superset(&self, other: &RawBoardSet) -> bool {
        other.is_subset(self)
    }

    /// Adds a value to the set.
    ///
    /// Returns whether the value was newly inserted. That is:
    /// - If the set did not previously contain this value, `true` is returned.
    /// - If the set already contained this value, `false` is returned.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set = RawBoardSet::new();
    /// assert_eq!(set.len(), 0);
    /// set.insert(Board::new().to_u64());
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn insert(&mut self, hash: u64) -> bool {
        let (k, v) = Self::u64_to_u32_u32(hash);
        self.top2bottoms.entry(k).or_default().insert(v)
    }

    /// Removes a value from the set.
    /// Returns whether the value was present in the set.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set = RawBoardSet::new();
    /// let hash = Board::new().to_u64();
    /// set.insert(hash);
    /// assert_eq!(set.len(), 1);
    /// set.remove(&hash);
    /// assert_eq!(set.len(), 0);
    /// ```
    pub fn remove(&mut self, hash: &u64) -> bool {
        let (k, v) = Self::u64_to_u32_u32(*hash);
        let Some(set) = self.top2bottoms.get_mut(&k) else {
            return false;
        };
        let removed = set.remove(&v);
        if set.is_empty() {
            self.top2bottoms.remove(&k);
        }
        removed
    }

    /// Removes and returns the value in the set, if any,
    /// that is equal to the given one.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set = RawBoardSet::new();
    /// let hash = Board::new().to_u64();
    /// set.insert(hash);
    /// assert_eq!(set.len(), 1);
    /// set.take(&hash);
    /// assert_eq!(set.len(), 0);
    /// ```
    pub fn take(&mut self, hash: &u64) -> Option<u64> {
        let (k, v) = Self::u64_to_u32_u32(*hash);
        let set = self.top2bottoms.get_mut(&k)?;
        let taken = set.take(&v).map(|bottom| Self::u32_u32_to_u64(k, bottom));
        if set.is_empty() {
            self.top2bottoms.remove(&k);
        }
        taken
    }

    /// Captures the ownership of the given set and absorb all elements in it.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set1 = RawBoardSet::new();
    /// let mut set2 = RawBoardSet::new();
    /// set2.insert(Board::new().to_u64());
    /// set1.absorb(set2);
    /// assert_eq!(set1.len(), 1);
    /// ```
    /// If the size of the set to be absorbed is large,
    /// allocating memory in advance may accelerate the process:
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set1 = RawBoardSet::new();
    /// let mut set2 = RawBoardSet::new();
    /// set2.insert(Board::new().to_u64());
    ///
    /// // Suppose set2 is very large
    /// set1.reserve(set2.capacity()); // allocate sufficient memory
    /// set1.absorb(set2);
    /// assert_eq!(set1.len(), 1);
    /// ```
    pub fn absorb(&mut self, set: RawBoardSet) {
        for (top, bottoms) in set.top2bottoms {
            if bottoms.is_empty() {
                continue;
            }
            self.top2bottoms.entry(top).or_default().extend(bottoms);
        }
    }

    /// Absorb all elements drained from the given set.
    /// The absorbed set will be empty, while it will keep the allocated memory.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set1 = RawBoardSet::new();
    /// let mut set2 = RawBoardSet::new();
    /// set2.insert(Board::new().to_u64());
    /// set1.absorb_drained(&mut set2);
    /// assert_eq!(set1.len(), 1);
    /// assert!(set2.is_empty());
    /// ```
    /// If the size of the set to be absorbed is large,
    /// allocating memory in advance may accelerate the process:
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set1 = RawBoardSet::new();
    /// let mut set2 = RawBoardSet::new();
    /// set2.insert(Board::new().to_u64());
    ///
    /// // Suppose set2 is very large
    /// set1.reserve(set2.capacity()); // allocate sufficient memory
    /// set1.absorb_drained(&mut set2);
    /// assert_eq!(set1.len(), 1);
    /// assert!(set2.is_empty());
    /// ```
    pub fn absorb_drained(&mut self, set: &mut RawBoardSet) {
        for (top, bottoms) in set.top2bottoms.iter_mut() {
            if bottoms.is_empty() {
                continue;
            }
            self.top2bottoms
                .entry(*top)
                .or_default()
                .extend(bottoms.drain());
        }
    }

    /// Inserts all elements given by `reader` into `self`.
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let mut set = RawBoardSet::new();
    /// set.load(File::open(path)?);
    /// # Ok(())
    /// # }
    /// ```
    /// The following is more efficient especially when the target file is large.
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let capacity = RawBoardSet::required_capacity(File::open(path)?)?;
    /// let mut set = RawBoardSet::with_capacity(capacity);
    /// set.load(File::open(path)?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn load<R>(&mut self, reader: R) -> std::io::Result<()>
    where
        R: Read,
    {
        let mut iter = FragmentIter::new(reader);
        let mut dummy = HashSet::new();
        let mut set = &mut dummy;
        let mut top = 0;
        loop {
            let Some(next) = iter.try_next()? else {
                return Ok(());
            };

            use Fragment::*;
            match next {
                Delimiter => {
                    if set.is_empty() {
                        set = &mut dummy;
                        self.top2bottoms.remove(&top);
                    }
                }
                Top(top_) => {
                    set = self.top2bottoms.entry(top_).or_default();
                    top = top_;
                }
                Bottom(bottom_) => {
                    set.insert(bottom_);
                }
            }
        }
    }

    /// Inserts all elements (`e`) given by `reader` under the condition of `f`
    /// (where `f(&e)` is `true`) into `self`.
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let filter = |hash| (hash & (0xfff << 48)).count_ones() >= 3;
    /// let mut set = RawBoardSet::new();
    /// set.load_filter(File::open(path)?, filter);
    /// # Ok(())
    /// # }
    /// ```
    /// The following is more efficient especially when the target file is large.
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let path = "/some/path/of/binary/file.tdl";
    /// let filter = |hash| (hash & (0xfff << 48)).count_ones() >= 3;
    /// let capacity = RawBoardSet::required_capacity_filter(File::open(path)?, filter);
    /// let mut set = RawBoardSet::with_capacity(capacity);
    /// set.load_filter(File::open(path)?, filter);
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_filter<R, F>(&mut self, reader: R, mut f: F) -> std::io::Result<()>
    where
        R: Read,
        F: FnMut(&u64) -> bool,
    {
        let mut iter = FragmentIter::new(reader);
        let mut dummy = HashSet::new();
        let mut set = &mut dummy;
        let mut top = 0;
        loop {
            let Some(next) = iter.try_next()? else {
                return Ok(());
            };

            use Fragment::*;
            match next {
                Delimiter => {
                    if set.is_empty() {
                        set = &mut dummy;
                        self.top2bottoms.remove(&top);
                    }
                }
                Top(top_) => {
                    set = self.top2bottoms.entry(top_).or_default();
                    top = top_;
                }
                Bottom(bottom) => {
                    let hash = Self::u32_u32_to_u64(top, bottom);
                    if f(&hash) {
                        set.insert(bottom);
                    }
                }
            }
        }
    }

    /// Writes all elements in the set to `writer`.
    /// The saved data can be loaded both by [`BoardSet::load`] and [`RawBoardSet::load`].
    ///
    /// # Examples
    /// ``` ignore
    /// use std::fs::File;
    /// use tokyodoves::Board;
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// let mut set = RawBoardSet::new();
    /// set.insert(Board::new().to_u64());
    /// let target_path = "/some/target/path.tdl";
    /// set.save(File::create(target_path)?);
    /// ```
    pub fn save<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: Write,
    {
        let mut writer = BufWriter::new(writer);
        for (top, bottoms) in self.top2bottoms.iter() {
            writer.write_all(&top.to_be_bytes())?;
            for bottom in bottoms.iter() {
                writer.write_all(&bottom.to_be_bytes())?;
            }
            writer.write_all(&u32::MAX.to_be_bytes())?;
        }
        writer.flush()?;
        Ok(())
    }

    /// Splits the set into two sets.
    ///
    /// The argument `left_len` indicates the length of the left component
    /// of the returned value.
    ///
    /// # Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use tokyodoves::{Board, BoardBuilder};
    /// use tokyodoves::collections::board_set::RawBoardSet;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut set = RawBoardSet::new();
    /// set.insert(Board::new().to_u64());
    /// set.insert(BoardBuilder::from_str("BbH")?.build()?.to_u64());
    /// set.insert(BoardBuilder::from_str("BbA")?.build()?.to_u64());
    /// let (left, right) = set.split(2);
    /// assert_eq!(left.len(), 2);
    /// assert_eq!(right.len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn split(self, len_left: usize) -> (Self, Self) {
        let mut left = RawBoardSet::new();
        let mut right = RawBoardSet::new();

        let mut add_to_left = true;
        let mut len_left_tmp = 0;
        for (top, bottoms) in self.top2bottoms {
            if !add_to_left {
                right.top2bottoms.insert(top, bottoms);
                continue;
            }

            let residual = len_left - len_left_tmp;
            if bottoms.len() <= residual {
                len_left_tmp += bottoms.len();
                left.top2bottoms.insert(top, bottoms);
            } else {
                let mut bottoms_iter = bottoms.into_iter();
                let left_bottoms = (&mut bottoms_iter).take(residual).collect();
                let right_bottoms = bottoms_iter.collect();
                left.top2bottoms.insert(top, left_bottoms);
                right.top2bottoms.insert(top, right_bottoms);
            }

            if len_left_tmp == len_left {
                add_to_left = false;
            }
        }
        (left, right)
    }
}

impl std::ops::BitAnd<&RawBoardSet> for &RawBoardSet {
    type Output = RawBoardSet;
    /// Returns the intersection of `self` and `rhs` as a new `RawBoardSet`.
    fn bitand(self, rhs: &RawBoardSet) -> Self::Output {
        self.intersection(rhs).collect()
    }
}

impl std::ops::BitOr<&RawBoardSet> for &RawBoardSet {
    type Output = RawBoardSet;
    /// Returns the union of `self` and `rhs` as a new `RawBoardSet`.
    fn bitor(self, rhs: &RawBoardSet) -> Self::Output {
        self.union(rhs).collect()
    }
}

impl std::ops::BitXor<&RawBoardSet> for &RawBoardSet {
    type Output = RawBoardSet;
    /// Returns the symmetric difference of `self` and `rhs` as a new `RawBoardSet`.
    fn bitxor(self, rhs: &RawBoardSet) -> Self::Output {
        self.symmetric_difference(rhs).collect()
    }
}

impl std::ops::Sub<&RawBoardSet> for &RawBoardSet {
    type Output = RawBoardSet;
    /// Returns the difference of `self` and `rhs` as a new `RawBoardSet`.
    fn sub(self, rhs: &RawBoardSet) -> Self::Output {
        self.difference(rhs).collect()
    }
}

type MapIter<'a> = std::collections::hash_map::Iter<'a, u32, HashSet<u32>>;
type SetIter<'a> = std::collections::hash_set::Iter<'a, u32>;

/// An iterator over the items of a [`RawBoardSet`].
///
/// This struct is created by the [`iter`](`RawBoardSet::iter`) method
/// on [`RawBoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use tokyodoves::Board;
/// use tokyodoves::collections::board_set::RawBoardSet;
///
/// let mut set = RawBoardSet::new();
/// set.insert(Board::new().to_u64());
/// let mut iter = set.iter();
/// ```
#[derive(Clone)]
pub struct RawIter<'a> {
    map_iter: MapIter<'a>, // iterator of top2bottoms
    state: Option<(
        u32,         // key of top2bottoms
        SetIter<'a>, // iterator of value of top2bottoms
    )>,
}

impl<'a> RawIter<'a> {
    fn new(set: &'a RawBoardSet) -> Self {
        Self {
            map_iter: set.top2bottoms.iter(),
            state: None,
        }
    }
}

impl<'a> Iterator for RawIter<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((top, set_iter)) = self.state.as_mut() else {
                let (top, set) = self.map_iter.next()?;
                self.state = Some((*top, set.iter()));
                continue;
            };

            let Some(bottom) = set_iter.next() else {
                let (next_top, next_set) = self.map_iter.next()?;
                *top = *next_top;
                *set_iter = next_set.iter();
                continue;
            };
            return Some(RawBoardSet::u32_u32_to_u64(*top, *bottom));
        }
    }
}

type MapIntoIter = std::collections::hash_map::IntoIter<u32, HashSet<u32>>;
type SetIntoIter = std::collections::hash_set::IntoIter<u32>;

/// An owing iterator over the items of a [`RawBoardSet`].
///
/// This struct is created by the [`into_iter`](`IntoIterator::into_iter`)
/// method on [`RawBoardSet`] (provided by the [`IntoIterator`] trait).
/// See the documentation for more.
///
/// # Examples
/// ```rust
/// use tokyodoves::Board;
/// use tokyodoves::collections::board_set::RawBoardSet;
///
/// let mut set = RawBoardSet::new();
/// set.insert(Board::new().to_u64());
/// let mut iter = set.into_iter();
/// ```
pub struct RawIntoIter {
    map_iter: MapIntoIter, // iterator of set.top2bottoms
    state: Option<(
        u32,         // key of set.top2bottoms
        SetIntoIter, // iterator of value of set.top2bottoms
    )>,
}

impl RawIntoIter {
    fn new(set: RawBoardSet) -> Self {
        Self {
            map_iter: set.top2bottoms.into_iter(),
            state: None,
        }
    }
}

impl Iterator for RawIntoIter {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((top, set_iter)) = self.state.as_mut() else {
                let (top, set) = self.map_iter.next()?;
                self.state = Some((top, set.into_iter()));
                continue;
            };

            let Some(bottom) = set_iter.next() else {
                let (next_top, next_set) = self.map_iter.next()?;
                *top = next_top;
                *set_iter = next_set.into_iter();
                continue;
            };
            return Some(RawBoardSet::u32_u32_to_u64(*top, bottom));
        }
    }
}

/// A draining iterator over the items of a [`RawBoardSet`].
///
/// This struct is created by the [`drain`](`RawBoardSet::drain`) method
/// on [`RawBoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use tokyodoves::Board;
/// use tokyodoves::collections::board_set::RawBoardSet;
///
/// let mut set = RawBoardSet::new();
/// set.insert(Board::new().to_u64());
/// let mut drain = set.drain();
/// ```
pub struct RawDrain<'a>(_RawDrain<'a>);

impl<'a> RawDrain<'a> {
    fn new(set: &'a mut RawBoardSet) -> Self {
        Self(_RawDrain::new(set))
    }
}

impl<'a> Iterator for RawDrain<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

type MapDrain<'a> = std::collections::hash_map::IterMut<'a, u32, HashSet<u32>>;
type SetDrain<'a> = std::collections::hash_set::Drain<'a, u32>;

struct _RawDrain<'a> {
    map_iter: MapDrain<'a>, // iterator of top2bottoms
    state: Option<(
        u32,          // key of top2bottoms
        SetDrain<'a>, // iterator of value of top2bottoms
    )>,
}

impl<'a> _RawDrain<'a> {
    fn new(set: &'a mut RawBoardSet) -> Self {
        Self {
            map_iter: set.top2bottoms.iter_mut(),
            state: None,
        }
    }
}

impl<'a> Iterator for _RawDrain<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((top, set_iter)) = self.state.as_mut() else {
                let (top, set) = self.map_iter.next()?;
                self.state = Some((*top, set.drain()));
                continue;
            };

            let Some(bottom) = set_iter.next() else {
                let (next_top, next_set) = self.map_iter.next()?;
                *top = *next_top;
                *set_iter = next_set.drain();
                continue;
            };
            return Some(RawBoardSet::u32_u32_to_u64(*top, bottom));
        }
    }
}

impl<'a> Drop for _RawDrain<'a> {
    fn drop(&mut self) {
        self.map_iter.by_ref().for_each(|(_, v)| {
            v.drain();
        });
    }
}

/// A lazy iterator producing elements in the difference of [`RawBoardSet`]s.
///
/// This struct is created by the [`difference`](`RawBoardSet::difference`) method
/// on [`RawBoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use std::str::FromStr;
/// use tokyodoves::{Board, BoardBuilder};
/// use tokyodoves::collections::board_set::RawBoardSet;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut set1 = RawBoardSet::new();
/// set1.insert(Board::new().to_u64());
/// let mut set2 = RawBoardSet::new();
/// set2.insert(BoardBuilder::from_str("BbH")?.build()?.to_u64());
///
/// let mut difference = set1.difference(&set2);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct RawDifference<'a> {
    left: RawIter<'a>,
    right: &'a RawBoardSet,
}

impl<'a> RawDifference<'a> {
    fn new(left: &'a RawBoardSet, right: &'a RawBoardSet) -> Self {
        Self {
            left: left.iter(),
            right,
        }
    }
}

impl<'a> Iterator for RawDifference<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.left.next()?;
            if !self.right.contains(&item) {
                return Some(item);
            }
        }
    }
}

/// A lazy iterator producing elements in the symmetric difference of [`RawBoardSet`]s.
///
/// This struct is created by the [`symmetric_difference`](`RawBoardSet::symmetric_difference`) method
/// on [`RawBoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use std::str::FromStr;
/// use tokyodoves::{Board, BoardBuilder};
/// use tokyodoves::collections::board_set::RawBoardSet;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut set1 = RawBoardSet::new();
/// set1.insert(Board::new().to_u64());
/// let mut set2 = RawBoardSet::new();
/// set2.insert(BoardBuilder::from_str("BbH")?.build()?.to_u64());
///
/// let mut symmetric_difference = set1.symmetric_difference(&set2);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct RawSymmetricDifference<'a> {
    left: &'a RawBoardSet,
    left_iter: RawIter<'a>,
    right: &'a RawBoardSet,
    right_iter: RawIter<'a>,
}

impl<'a> RawSymmetricDifference<'a> {
    fn new(left: &'a RawBoardSet, right: &'a RawBoardSet) -> Self {
        Self {
            left,
            left_iter: left.iter(),
            right,
            right_iter: right.iter(),
        }
    }
}

impl<'a> Iterator for RawSymmetricDifference<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item_left) = self.left_iter.next() {
                if !self.right.contains(&item_left) {
                    return Some(item_left);
                }
            } else {
                let item_right = self.right_iter.next()?;
                if !self.left.contains(&item_right) {
                    return Some(item_right);
                }
            }
        }
    }
}

/// A lazy iterator producing elements in the intersection of [`RawBoardSet`]s.
///
/// This struct is created by the [`intersection`](`RawBoardSet::intersection`) method
/// on [`RawBoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use std::str::FromStr;
/// use tokyodoves::{Board, BoardBuilder};
/// use tokyodoves::collections::board_set::RawBoardSet;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut set1 = RawBoardSet::new();
/// set1.insert(Board::new().to_u64());
/// let mut set2 = RawBoardSet::new();
/// set2.insert(BoardBuilder::from_str("BbH")?.build()?.to_u64());
///
/// let mut intersection = set1.intersection(&set2);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct RawIntersection<'a> {
    left_iter: RawIter<'a>,
    right: &'a RawBoardSet,
}

impl<'a> RawIntersection<'a> {
    fn new(left: &'a RawBoardSet, right: &'a RawBoardSet) -> Self {
        Self {
            left_iter: left.iter(),
            right,
        }
    }
}

impl<'a> Iterator for RawIntersection<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.left_iter.next()?;
            if self.right.contains(&item) {
                return Some(item);
            }
        }
    }
}

/// A lazy iterator producing elements in the union of [`RawBoardSet`]s.
///
/// This struct is created by the [`union`](`RawBoardSet::union`) method
/// on [`RawBoardSet`]. See its documentation for more.
///
/// # Examples
/// ```rust
/// use std::str::FromStr;
/// use tokyodoves::{Board, BoardBuilder};
/// use tokyodoves::collections::board_set::RawBoardSet;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut set1 = RawBoardSet::new();
/// set1.insert(Board::new().to_u64());
/// let mut set2 = RawBoardSet::new();
/// set2.insert(BoardBuilder::from_str("BbH")?.build()?.to_u64());
///
/// let mut union_iter = set1.union(&set2);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct RawUnion<'a> {
    left_iter: RawIter<'a>,
    right: &'a RawBoardSet,
    right_iter: RawIter<'a>,
}

impl<'a> RawUnion<'a> {
    fn new(left: &'a RawBoardSet, right: &'a RawBoardSet) -> Self {
        Self {
            left_iter: left.iter(),
            right,
            right_iter: right.iter(),
        }
    }
}

impl<'a> Iterator for RawUnion<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.left_iter.next() {
                Some(item) => {
                    if !self.right.contains(&item) {
                        return Some(item);
                    }
                }
                None => return self.right_iter.next(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{collections::*, *};

    fn create_from_strs(strs: &[&str]) -> BoardSet {
        let mut set = BoardSet::new();
        for board_str in strs {
            let board = BoardBuilder::from_str(board_str).unwrap().build_unchecked();
            set.insert(board);
        }
        set
    }

    #[test]
    fn test_empty_capacity() {
        let mut set = BoardSet::new();
        assert_eq!(set.capacity(), Capacity::new());
        set.insert(Board::new());
        set.drain();
        assert_ne!(set.capacity(), Capacity::new());
    }

    #[test]
    fn test_set_calculation() {
        let set1 = create_from_strs(&["Bb", "Bbh", "Bba"]);
        let set2 = create_from_strs(&["Bb", "BbH", "BbA"]);
        let set1and2 = create_from_strs(&["Bb"]);
        let set1or2 = create_from_strs(&["Bb", "Bbh", "Bba", "BbH", "BbA"]);
        let set1xor2 = create_from_strs(&["Bbh", "Bba", "BbH", "BbA"]);
        let set1minus2 = create_from_strs(&["Bbh", "Bba"]);

        assert_eq!(&set1 & &set2, set1and2);
        assert_eq!(set1.raw() & set2.raw(), *set1and2.raw());

        assert_eq!(&set1 | &set2, set1or2);
        assert_eq!(set1.raw() | set2.raw(), *set1or2.raw());

        assert_eq!(&set1 ^ &set2, set1xor2);
        assert_eq!(set1.raw() ^ set2.raw(), *set1xor2.raw());

        assert_eq!(&set1 - &set2, set1minus2);
        assert_eq!(set1.raw() - set2.raw(), *set1minus2.raw());

        assert!(set1or2.is_superset(&set1));
        assert!(set1or2.raw().is_superset(set1.raw()));

        assert!(set1.is_subset(&set1or2));
        assert!(set1.raw().is_subset(set1or2.raw()));

        assert!(set1xor2.is_disjoint(&set1and2));
        assert!(set1xor2.raw().is_disjoint(set1and2.raw()));
    }

    #[test]
    fn test_absorb_extend() {
        let set1 = create_from_strs(&["Bb", "Bbh", "Bba"]);
        let set2 = create_from_strs(&["Bb", "BbH", "BbA"]);
        let set1or2 = create_from_strs(&["Bb", "Bbh", "Bba", "BbH", "BbA"]);

        let mut set1absorb2 = set1.clone();
        set1absorb2.absorb(set2.clone());
        assert_eq!(set1absorb2, set1or2);

        let mut set1absorb2_raw = set1.raw().clone();
        set1absorb2_raw.absorb(set2.raw().clone());
        assert_eq!(set1absorb2_raw, *set1or2.raw());

        let mut set1extend2 = set1.clone();
        set1extend2.extend(set2.iter());
        assert_eq!(set1absorb2, set1extend2);

        let mut set1extend2_raw = set1.raw().clone();
        set1extend2_raw.extend(set2.raw().iter());
        assert_eq!(set1absorb2_raw, set1extend2_raw);
    }

    #[test]
    fn test_drain() {
        let mut set = create_from_strs(&["Bb", "Bbh", "Bba"]);
        let set1 = set.clone();
        let set2 = BoardSet::from_iter(set.drain());
        assert!(set.is_empty());
        assert_eq!(set1, set2);
    }

    #[test]
    fn test_drain_drop() {
        let mut set = create_from_strs(&["Bb", "Bbh", "Bba"]);
        let capacity = set.capacity();
        {
            set.drain();
        }
        assert!(set.is_empty());
        assert_eq!(capacity, set.capacity());
    }

    #[test]
    fn test_split() {
        let set = create_from_strs(&["Bb", "Bbh", "Bba"]);

        let (left, right) = set.clone().split(0);
        assert_eq!(left.len(), 0);
        assert_eq!(right.len(), 3);
        assert_eq!(&left | &right, set);

        let (left, right) = set.clone().split(1);
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 2);
        assert_eq!(&left | &right, set);

        let (left, right) = set.clone().split(10);
        assert_eq!(left.len(), 3);
        assert_eq!(&left | &right, set);

        let set = set.into_raw();

        let (left, right) = set.clone().split(0);
        assert_eq!(left.len(), 0);
        assert_eq!(right.len(), 3);
        assert_eq!(&left | &right, set);

        let (left, right) = set.clone().split(1);
        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 2);
        assert_eq!(&left | &right, set);

        let (left, right) = set.clone().split(10);
        assert_eq!(left.len(), 3);
        assert_eq!(&left | &right, set);
    }
}
