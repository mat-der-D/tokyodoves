//! Position holders

use super::bitutil::calc_adjacents;
use super::container::DoveSet;
use crate::prelude::pieces::{color_to_index, dove_to_index, Color, Dove};

/// [`Dove`] -> position
#[derive(Debug, Clone, Copy, Default)]
pub struct DovePositions {
    positions: [u64; 6], // B, A, Y, M, T, H
}

impl DovePositions {
    pub fn new(positions: [u64; 6]) -> Self {
        DovePositions { positions }
    }

    fn set_position(&mut self, dove: Dove, bit: u64) {
        let i = dove_to_index(dove);
        self.positions[i] = bit;
    }

    fn position_of(&self, dove: Dove) -> &u64 {
        let i = dove_to_index(dove);
        &self.positions[i]
    }

    fn union(&self) -> u64 {
        self.positions.into_iter().fold(0, |union, x| union | x)
    }

    fn union_except(&self, dove: Dove) -> u64 {
        let index = dove_to_index(dove);
        (0..6)
            .filter(|i| *i != index)
            .fold(0, |union, i| union | self.positions[i])
    }

    fn doves_in_hand(&self) -> DoveSet {
        let hash = self
            .positions
            .into_iter()
            .enumerate()
            .fold(0, |cum, (i, x)| cum | (((x == 0) as u8) << i));
        DoveSet { hash }
    }

    fn doves_on_field(&self) -> DoveSet {
        let hash = self
            .positions
            .into_iter()
            .enumerate()
            .fold(0, |cum, (i, x)| cum | (((x != 0) as u8) << i));
        DoveSet { hash }
    }
}

/// [`Color`] -> [`DovePositions`],
/// inducing ([`Color`], [`Dove`]) -> position
#[derive(Debug, Clone, Copy, Default)]
pub struct ColorDovePositions {
    positions: [DovePositions; 2],
}

impl ColorDovePositions {
    pub fn new(positions: [DovePositions; 2]) -> Self {
        Self { positions }
    }

    pub fn dove_positions(&self, color: Color) -> &DovePositions {
        let i = color_to_index(color);
        &self.positions[i]
    }

    pub fn dove_positions_mut(&mut self, color: Color) -> &mut DovePositions {
        let i = color_to_index(color);
        &mut self.positions[i]
    }

    pub fn set_position(&mut self, color: Color, dove: Dove, bit: u64) {
        self.dove_positions_mut(color).set_position(dove, bit);
    }

    pub fn position_of(&self, color: Color, dove: Dove) -> &u64 {
        self.dove_positions(color).position_of(dove)
    }

    pub fn union(&self) -> u64 {
        self.positions.into_iter().fold(0, |cum, p| cum | p.union())
    }

    pub fn union_in_color(&self, color: Color) -> u64 {
        self.dove_positions(color).union()
    }

    pub fn union_except(&self, color: Color, dove: Dove) -> u64 {
        self.dove_positions(color).union_except(dove) | self.union_in_color(!color)
    }

    pub fn doves_in_hand(&self, color: Color) -> DoveSet {
        self.dove_positions(color).doves_in_hand()
    }

    pub fn doves_on_field(&self, color: Color) -> DoveSet {
        self.dove_positions(color).doves_on_field()
    }

    pub fn swap_color(&mut self) {
        self.positions.swap(0, 1);
    }

    pub fn isolated(&self) -> bool {
        let all = self.union();
        all != all & calc_adjacents(all)
    }
}