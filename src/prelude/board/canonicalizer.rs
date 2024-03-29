use crate::prelude::macros;

const CONGRUENT_MAPS: [[[usize; 16]; 8]; 16] = {
    // (hsize - 1) + 4 * (vsize - 1)
    let rotate_maps = [
        [0, 4, 8, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // vsize == 1
        [1, 5, 9, 13, 0, 4, 8, 12, 0, 0, 0, 0, 0, 0, 0, 0], // vsize == 2
        [2, 6, 10, 14, 1, 5, 9, 13, 0, 4, 8, 12, 0, 0, 0, 0], // vsize == 3
        [3, 7, 11, 15, 2, 6, 10, 14, 1, 5, 9, 13, 0, 4, 8, 12], // vsize == 4
    ];

    let reflect_maps = [
        [0, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // vsize == 1
        [4, 5, 6, 7, 0, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0], // vsize == 2
        [8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3, 0, 0, 0, 0], // vsize == 3
        [12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3], // vsize == 4
    ];

    let mut cons = [[[0_usize; 16]; 8]; 16];
    macros::for_loop!(let mut vsize = 1; vsize < 5; vsize += 1 => {
        macros::for_loop!(let mut hsize = 1; hsize < 5; hsize += 1 => {
            let idx = (hsize - 1) + 4 * (vsize - 1);
            let mut count = 0;
            let mut base = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

            macros::for_loop!(let mut i = 0; i < 2; i += 1 => {
                macros::for_loop!(let mut j = 0; j < 2; j += 1 => {
                    macros::for_loop!(let mut k = 0; k < 2; k += 1 => {
                        let hvsize = if k % 2 == 0 { vsize } else { hsize };
                        base = compose(rotate_maps[hvsize - 1], base);
                        cons[idx][count] = base;
                        count += 1;
                    }); // k
                }); // j
                base = compose(reflect_maps[vsize - 1], base);
            }); // i
        }); // hsize
    }); // vsize

    cons
};

const fn compose(a: [usize; 16], b: [usize; 16]) -> [usize; 16] {
    let mut a_after_b = [0_usize; 16];
    macros::for_loop!(let mut i = 0; i < 16; i += 1 => {
        a_after_b[i] = a[b[i]];
    });
    a_after_b
}

/// A mapping of positions used to canonicalize functions of a [`Board`](`crate::prelude::Board`).
///
/// To understand what `PositionMapper` does,
/// consider the following board for example.
/// ```text
/// +----+----+----+----+
/// |  b |  a |    |    |
/// +----+----+----+----+
/// |  B |    |    |    |
/// +----+----+----+----+
/// |    |  H |    |    |
/// +----+----+----+----+
/// |    |    |    |    |
/// +----+----+----+----+
/// ```
/// Boards generated by reflections, rotations and translations have
/// effectively the same value (or "equivalent") in the game.
/// For example, boards below are all equivalent to the board above.
/// ```text
/// +----+----+----+----+
/// |  a |  b |    |    |
/// +----+----+----+----+
/// |    |  B |    |    |
/// +----+----+----+----+
/// |  H |    |    |    |
/// +----+----+----+----+
/// |    |    |    |    |
/// +----+----+----+----+
///
/// +----+----+----+----+
/// |    |  B |  b |    |
/// +----+----+----+----+
/// |  H |    |  a |    |
/// +----+----+----+----+
/// |    |    |    |    |
/// +----+----+----+----+
/// |    |    |    |    |
/// +----+----+----+----+
///
/// +----+----+----+----+
/// |    |    |    |    |
/// +----+----+----+----+
/// |    |  b |  a |    |
/// +----+----+----+----+
/// |    |  B |    |    |
/// +----+----+----+----+
/// |    |    |  H |    |
/// +----+----+----+----+
/// ```
/// To reduce the degree of freedom of the game by identifying equivalent boards
/// is important for efficient analysis.
/// `PositionMapper` helps you to do it.
///
/// Now assign numbers from 0 to 15 to 4x4 squares as below:
/// ```text
/// +----+----+----+----+
/// |  0 |  1 |  2 |  3 |
/// +----+----+----+----+
/// |  4 |  5 |  6 |  7 |
/// +----+----+----+----+
/// |  8 |  9 | 10 | 11 |
/// +----+----+----+----+
/// | 12 | 13 | 14 | 15 |
/// +----+----+----+----+
/// ```
/// Correspondence between pieces and numbers is
/// ```text
/// b -> 0, a -> 1, B -> 4 and H -> 9.
/// ```
/// Then a mapping from
/// ```text
/// +----+----+----+----+
/// |  b |  a |    |    |
/// +----+----+----+----+
/// |  B |    |    |    |
/// +----+----+----+----+
/// |    |  H |    |    |
/// +----+----+----+----+
/// |    |    |    |    |
/// +----+----+----+----+
/// (b -> 0, a -> 1, B -> 4, H -> 9)
/// ```
/// to
/// ```text
/// +----+----+----+----+
/// |    |  B |  b |    |
/// +----+----+----+----+
/// |  H |    |  a |    |
/// +----+----+----+----+
/// |    |    |    |    |
/// +----+----+----+----+
/// |    |    |    |    |
/// +----+----+----+----+
/// (b -> 2, a -> 6, B -> 1, H -> 4)
/// ```
/// is expressed by a function f satisfying
/// ```text
/// f(0)=2, f(1)=6, f(4)=1 and f(9)=4.
/// ```
/// In general, all transformations by reflections, rotations and translations
/// can be expressed by a mapping
/// ```text
/// f: {0, 1, 2, ..., 15} -> {0, 1, 2, ..., 15}.
/// ```
/// The number of all combinations of reflections and rotations is 8
/// (2 reflections times 4 rotations) including identity function.
/// It gives 8 kinds of position mappings, or in mathematical notation,
/// ```text
/// f_i: {0, 1, ..., 15} -> {0, 1, ..., 15} (i=0, 1, ..., 7),
/// ```
/// which is what `PositionMapper` provides.
///
/// In the above example, the minimum rectangle, the rectangle that contains all pieces,
/// has a shape of 3x2.
/// The mapper that matches the situation is constructed by the following:
/// ```rust
/// use tokyodoves::analysis::PositionMapper;
/// let mapper = PositionMapper::try_create(3, 2).unwrap();
/// ```
/// The [`map`](`PositionMapper::map`) method maps the position number.
/// The first argument `index` indicates the kind of mapping (from 0 to 7),
/// and the second argument `pos` is the position number (from 0 to 15).
/// Formaly,
/// ```text
/// y = mapper.map(i, x)
/// ```
/// is equivalent to
/// ```text
/// y = f_i(x).
/// ```
/// Thus the mapping in the above example realizes for some `index`.
///
/// Note that `PositionMapper` does NOT treat translational transformations.
/// It supposes that the top-left corner of the minimum rectangle
/// coincides with that of 4x4 matrix, and does the same for mapped boards.
/// For full reduction of the degree of freedom,
/// implement some functions to translate the boards so that the top-left corner
/// of the minimum rectangle moves to that of 4x4 matrix,
/// besides `PositionMapper`,
#[cfg(feature = "analysis")]
#[derive(Debug, Clone, Copy)]
pub struct PositionMapper {
    maps: &'static [[usize; 16]; 8],
}

#[cfg(not(feature = "analysis"))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PositionMapper {
    maps: &'static [[usize; 16]; 8],
}

impl PositionMapper {
    pub fn try_create(vsize: usize, hsize: usize) -> Option<Self> {
        if vsize == 0 || vsize >= 5 || hsize == 0 || hsize >= 5 {
            None
        } else {
            // safety is guaranteed because (hsize - 1) + 4 * (vsize - 1) ranges from 0 to 15
            // under the validation above
            let maps = unsafe { CONGRUENT_MAPS.get_unchecked((hsize - 1) + 4 * (vsize - 1)) };
            Some(Self { maps })
        }
    }

    pub fn map(&self, index: usize, pos: usize) -> usize {
        if index >= 8 || pos >= 16 {
            0
        } else {
            // safety is guaranteed thanks to the validation above
            unsafe { *self.maps.get_unchecked(index).get_unchecked(pos) }
        }
    }
}
