//! A module containing a light [`Board`](`crate::Board`) container
//! [`BoardSet`] and associated items ("analysis" feature required)

pub mod board_set;
pub(crate) mod io;

pub use board_set::{BoardSet, Capacity};
pub use io::*;
