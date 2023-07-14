use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{hash_map, HashMap};

use crate::{
    game::{GameRule, Judge},
    Action, ActionsFwdIntoIter, Board, BoardBuilder, Color, SurroundedStatus,
};

// ****************************************************************************
//  BoardValue
// ****************************************************************************
/// Three kinds of [`BoardValue`]
///
/// This type is returned by [`BoardValue::kind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardValueKind {
    Win,
    Lose,
    Unknown,
    Finished,
}

impl Default for BoardValueKind {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for BoardValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PartialOrd for BoardValueKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        fn _kind_to_u8(kind: &BoardValueKind) -> u8 {
            use BoardValueKind::*;
            match kind {
                Win => 2,
                Unknown => 1,
                Lose => 0,
                Finished => unreachable!(),
            }
        }

        use BoardValueKind::*;
        if matches!(self, Finished) || matches!(other, Finished) {
            if self.eq(other) {
                return Some(Ordering::Equal);
            } else {
                return None;
            }
        }

        _kind_to_u8(self).partial_cmp(&_kind_to_u8(other))
    }
}

/// Value of board
///
/// The order of the values is as follows:
/// ```text
/// BoardValue::MIN = Lose(2) < Lose(4) < Lose(6) < ...
///     < Unknown
///     < ... < Win(5) < Win(3) < Win(1) = BoardValue::MAX
/// ```
/// - n of `Lose(n)` means that the player will lose in n turns at most.
/// - n of `Win(n)` means that the player will win in n turns at least.
/// - `Unknown` means that the value of the board was failed to be determined.
///     It can change to `Lose(n)` or `Win(n)` if you search more deeply.
///
/// It takes another value `Finished`, which is attributed to boards of finished games.
/// This value is, however, not compared to other values,
/// which is the reason why `BoardValue` is `PartialOrd` but not `Ord`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BoardValue {
    value: Option<usize>,
}

impl std::fmt::Display for BoardValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BoardValueKind::*;
        let kind = self.kind();
        let s = match kind {
            Unknown | Finished => format!("{}", kind),
            _ => {
                let num = self.value.unwrap();
                format!("{}({})", kind, num)
            }
        };
        write!(f, "{}", s)
    }
}

impl BoardValue {
    /// Win(1)
    pub const MAX: BoardValue = BoardValue { value: Some(1) };
    /// Lose(2)
    pub const MIN: BoardValue = BoardValue { value: Some(2) };

    pub fn kind(&self) -> BoardValueKind {
        use BoardValueKind::*;
        match self.value {
            None => Unknown,
            Some(0) => Finished,
            Some(n) => match n % 2 {
                0 => Lose,
                1 => Win,
                _ => unreachable!(),
            },
        }
    }

    pub fn win(num: usize) -> Option<Self> {
        if num % 2 == 1 {
            Some(BoardValue { value: Some(num) })
        } else {
            None
        }
    }

    pub fn lose(num: usize) -> Option<Self> {
        if num != 0 && num % 2 == 0 {
            Some(BoardValue { value: Some(num) })
        } else {
            None
        }
    }

    pub fn unknown() -> Self {
        BoardValue { value: None }
    }

    pub fn finished() -> Self {
        BoardValue { value: Some(0) }
    }

    pub fn try_unwrap(&self) -> Option<usize> {
        match self.value {
            Some(num) if num >= 1 => self.value,
            _ => None,
        }
    }

    pub fn unwrap(&self) -> usize {
        self.try_unwrap().unwrap()
    }

    pub fn is_win(&self) -> bool {
        matches!(self.kind(), BoardValueKind::Win)
    }

    pub fn is_lose(&self) -> bool {
        matches!(self.kind(), BoardValueKind::Lose)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self.kind(), BoardValueKind::Unknown)
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.kind(), BoardValueKind::Finished)
    }

    /// Returns "next" value of a series below:
    /// ```text
    /// Unknown -> Unknown
    /// Finished -> Win(1) -> Lose(2) -> Win(3) -> Lose(4) -> ...
    /// ```
    pub fn increment(&self) -> Self {
        Self {
            value: self.value.map(|num| num + 1),
        }
    }

    /// Returns "next" values of a series below:
    /// ```text
    /// Unknown -> Unknown
    /// ... -> Lose(4) -> Win(3) -> Lose(2) -> Win(1) -> Finish
    /// ```
    /// It returns `None` if self is `Finish`, otherwise `Some(next_value)`.
    pub fn try_decrement(&self) -> Option<Self> {
        let value = match self.value {
            Some(0) => return None,
            x => x.map(|num| num - 1),
        };
        Some(Self { value })
    }
}

impl From<Option<usize>> for BoardValue {
    fn from(value: Option<usize>) -> Self {
        Self { value }
    }
}

impl From<BoardValue> for Option<usize> {
    fn from(board_value: BoardValue) -> Self {
        board_value.value
    }
}

impl PartialOrd for BoardValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left_kind = self.kind();
        let right_kind = other.kind();

        use BoardValueKind::*;
        if (left_kind != right_kind) || matches!(left_kind, Unknown | Finished) {
            return left_kind.partial_cmp(&right_kind);
        }

        // In the following, left_kind == right_kind not in {Unknown, Finished}

        let left_num = self.value.as_ref().unwrap();
        let right_num = other.value.as_ref().unwrap();
        match left_kind {
            Lose => left_num.partial_cmp(right_num),
            Win => right_num.partial_cmp(left_num),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests_board_value {
    use crate::analysis::{BoardValue, BoardValueKind};

    #[test]
    fn test_create_win() {
        for n in 0..100 {
            let Some(val) = BoardValue::win(n) else {
                continue;
            };
            assert_eq!(val, BoardValue::from(Some(n)));
            assert!(val.is_win());
            assert_eq!(val.kind(), BoardValueKind::Win);
            let num = val.unwrap();
            assert_eq!(n, num);
            assert_eq!(val, BoardValue::win(num).unwrap());
        }
    }

    #[test]
    fn test_create_lose() {
        for n in 0..100 {
            let Some(val) = BoardValue::lose(n) else {
                continue;
            };
            assert_eq!(val, BoardValue::from(Some(n)));
            assert!(val.is_lose());
            assert_eq!(val.kind(), BoardValueKind::Lose);
            let num = val.unwrap();
            assert_eq!(n, num);
            assert_eq!(val, BoardValue::lose(num).unwrap());
        }
    }

    #[test]
    fn test_create_unknown() {
        let val = BoardValue::unknown();
        assert_eq!(val, BoardValue::from(None));
        assert!(val.is_unknown());
        assert_eq!(val.kind(), BoardValueKind::Unknown);
        assert!(val.try_unwrap().is_none());
    }

    #[test]
    fn test_create_finished() {
        let val = BoardValue::finished();
        assert_eq!(val, BoardValue::from(Some(0)));
        assert!(val.is_finished());
        assert_eq!(val.kind(), BoardValueKind::Finished);
        assert!(val.try_unwrap().is_none());
    }

    #[test]
    fn test_increment() {
        assert_eq!(BoardValue::unknown(), BoardValue::unknown().increment());

        let mut val = BoardValue::finished();
        for num in 1..100 {
            val = val.increment();
            assert_eq!(val, BoardValue::from(Some(num)));
            assert_eq!(val.unwrap(), num);
        }
    }

    #[test]
    fn test_try_decrement() {
        assert_eq!(
            BoardValue::unknown(),
            BoardValue::unknown().try_decrement().unwrap()
        );
        let mut val = BoardValue::lose(100).unwrap();
        for num in (0..100).rev() {
            val = val.try_decrement().unwrap();
            assert_eq!(val, BoardValue::from(Some(num)));
            if num == 0 {
                assert!(val.try_unwrap().is_none());
            } else {
                assert_eq!(val.unwrap(), num);
            }
        }
        assert!(val.try_decrement().is_none());
    }

    #[test]
    fn test_compare() {
        let unknown = BoardValue::unknown();
        let finished = BoardValue::finished();
        assert!(!(finished < finished));
        assert!(!(finished > finished));
        assert_eq!(finished, finished);
        assert!(!(finished < unknown));
        assert!(!(unknown < finished));
        assert!(!(finished > unknown));
        assert!(!(unknown > finished));

        let mut is_first_loop = true;
        for num_win in (1..100).step_by(2) {
            let win = BoardValue::win(num_win).unwrap();
            assert!(win <= BoardValue::MAX);
            if win != BoardValue::MAX {
                assert!(win < BoardValue::MAX);
            }
            assert!(unknown < win);
            assert!(!(finished < win));
            assert!(!(finished > win));
            assert!(!(win < finished));
            assert!(!(win > finished));

            for num_lose in (2..100).step_by(2) {
                let lose = BoardValue::lose(num_lose).unwrap();
                assert!(lose >= BoardValue::MIN);
                if lose != BoardValue::MIN {
                    assert!(lose > BoardValue::MIN);
                }
                assert!(lose < win);
                assert!(lose < unknown);

                if is_first_loop {
                    assert!(!(finished < lose));
                    assert!(!(finished > lose));
                    assert!(!(lose < finished));
                    assert!(!(lose > finished));
                }
            }
            is_first_loop = false;
        }
    }
}

// ****************************************************************************
//  NextBoardIter
// ****************************************************************************
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NextBoardStatus {
    Win,
    Lose,
    Unknown,
}

struct NextBoardIter {
    current_board: Board,
    current_player: Color,
    wins_if_both: bool,
    actions_iter: ActionsFwdIntoIter,
}

impl NextBoardIter {
    fn new(current_board: Board, current_player: Color, rule: GameRule) -> Self {
        use Judge::*;
        let wins_if_both = match rule.suicide_atk_judge() {
            LastWins => true,
            NextWins => false,
            Draw => panic!("validation error"),
        };
        let actions_iter = current_board
            .legal_actions(current_player, true, true, *rule.is_remove_accepted())
            .into_iter();
        Self {
            current_board,
            current_player,
            wins_if_both,
            actions_iter,
        }
    }
}

impl Iterator for NextBoardIter {
    type Item = (Action, Board, NextBoardStatus);
    fn next(&mut self) -> Option<Self::Item> {
        let action = self.actions_iter.next()?;
        let next_board = self.current_board.perform_unchecked_copied(action);

        use SurroundedStatus::*;
        let sur_status = next_board.surrounded_status();
        let is_both = matches!(sur_status, Both);

        use NextBoardStatus::*;
        let next_board_status = if (self.wins_if_both && is_both)
            || matches!(sur_status, OneSide(p) if p != self.current_player)
        {
            Win
        } else if (!self.wins_if_both && is_both)
            || matches!(sur_status, OneSide(p) if p == self.current_player)
        {
            Lose
        } else {
            Unknown
        };
        Some((action, next_board, next_board_status))
    }
}

// ****************************************************************************
//  BoardValueTree
// ****************************************************************************
#[derive(Clone)]
pub struct Actions<'a>(hash_map::Keys<'a, Action, BoardValueTree>);

impl<'a> Actions<'a> {
    fn new(iter: hash_map::Keys<'a, Action, BoardValueTree>) -> Self {
        Self(iter)
    }
}

impl<'a> Iterator for Actions<'a> {
    type Item = &'a Action;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> std::fmt::Debug for Actions<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A tree structure that contains [`Board`] and [`BoardValue`] on its nodes,
/// and [`Action`]s on its edges.
///
/// It is returned by [`create_checkmate_tree`].
#[derive(Debug, Clone)]
pub struct BoardValueTree {
    board_raw: u64,
    player: Color,
    value: BoardValue,
    children: HashMap<Action, BoardValueTree>,
}

impl BoardValueTree {
    pub fn new(board: Board, player: Color) -> Self {
        Self {
            board_raw: board.to_u64(),
            player,
            value: Default::default(),
            children: Default::default(),
        }
    }

    pub fn board(&self) -> Board {
        BoardBuilder::from_u64(self.board_raw).build_unchecked()
    }

    pub fn player(&self) -> &Color {
        &self.player
    }

    pub fn value(&self) -> &BoardValue {
        &self.value
    }

    pub fn child(&self, action: &Action) -> Option<&BoardValueTree> {
        self.children.get(action)
    }

    pub fn actions(&self) -> Actions<'_> {
        Actions::new(self.children.keys())
    }

    pub fn is_good_for_puzzle(&self, step: usize) -> bool {
        if step == 0 {
            true
        } else {
            self.children.len() == 1
                && self
                    .children
                    .values()
                    .all(|c| c.is_good_for_puzzle(step - 1))
        }
    }
}

// ****************************************************************************
//  Helper Items
// ****************************************************************************
/// Error variants on validation of arguments
#[derive(Debug, thiserror::Error)]
pub enum ArgsValidationError {
    #[error("board of finished game")]
    FinishedBoardError { board: Board },

    #[error("{} is not supported", .boad_value)]
    UnsupportedValueError { boad_value: BoardValue },

    #[error("Judge::Draw not supported")]
    DrawJudgeError,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateCheckmateTreeWithValueError {
    #[error(transparent)]
    ArgsValidationError(#[from] ArgsValidationError),

    #[error("value of board differs from value in argument")]
    ValueMismatchError(Ordering),
}

fn validate_args(
    board: Board,
    value: BoardValue,
    rule: GameRule,
) -> Result<(), ArgsValidationError> {
    use ArgsValidationError::*;
    if board.surrounded_status() != SurroundedStatus::None {
        return Err(FinishedBoardError { board });
    }
    if value.is_finished() || value.is_unknown() {
        return Err(UnsupportedValueError { boad_value: value });
    }
    if !matches!(rule.suicide_atk_judge(), Judge::NextWins | Judge::LastWins) {
        return Err(DrawJudgeError);
    }
    Ok(())
}

/// Represents closed interval between two [`BoardValue`]s
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval {
    left: BoardValue,
    right: BoardValue,
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.left, self.right)
    }
}

impl Interval {
    pub fn new(left: BoardValue, right: BoardValue) -> Self {
        Self { left, right }
    }

    pub fn left(&self) -> &BoardValue {
        &self.left
    }

    pub fn right(&self) -> &BoardValue {
        &self.right
    }

    pub fn contains(&self, item: &BoardValue) -> bool {
        self.left <= *item && *item <= self.right
    }

    pub fn single(&self) -> Option<BoardValue> {
        if self.left == self.right {
            Some(self.left)
        } else {
            None
        }
    }
}

/// Prints contents of [`BoardValueTree`] (experimental api)
///
/// Similar functions should be implemented as [`std::fmt::Display`]
/// for [`BoardValueTree`].
pub fn print_tree(tree: &BoardValueTree) {
    print_tree_core(tree, "");
}

fn print_tree_core(tree: &BoardValueTree, top: &str) {
    let next_top = format!("\t{}", top);
    println!("{}{}", top, tree.value);
    for (_, t) in tree.children.iter() {
        print_tree_core(t, &next_top);
    }
}

// ****************************************************************************
//  Functions for Analysis
// ****************************************************************************
/// Creates an [`BoardValueTree`] that describes routes to ends of the game.
///
/// Once the value of a board in a search process is determined,
/// succeeding edges whose values are not the "next" value are pruned.
/// The following diagram describes what "next" value means:
/// ```text
/// ... -> Win(5) -> Lose(4) -> Win(3) -> Lose(2) -> Win(1)
/// ```
/// Especially, the node with the value `Win(1)` becomes a leaf node,
/// that is, all the succeeding edges are pruned.
///
/// If you know the value of `board` in advance,
/// [`create_checkmate_tree_with_value`], a much faster version,
/// would be more appropriate.
///
/// # Errors
/// Returns `Err` when the argument is invalid. Specifically,
/// the following cases are invalid:
/// - `board` is already finished (at least one boss is surrounded)
/// - `value` is `Unknown`, `Win(even number)` or `Lose(odd or zero)`
/// - `rule.suicide_atk_judge` is `Judge::Draw`
pub fn create_checkmate_tree(
    board: Board,
    player: Color,
    max_depth: usize,
    rule: GameRule,
) -> Result<BoardValueTree, ArgsValidationError> {
    validate_args(board, BoardValue::MAX, rule)?;
    Ok(create_checkmate_tree_unchecked(
        board, player, max_depth, rule,
    ))
}

fn create_checkmate_tree_unchecked(
    board: Board,
    player: Color,
    max_depth: usize,
    rule: GameRule,
) -> BoardValueTree {
    if max_depth == 0 {
        return BoardValueTree::new(board, player);
    }

    let mut tree = BoardValueTree::new(board, player);
    tree.value = BoardValue::MIN;

    for (action, next_board, status) in NextBoardIter::new(board, player, rule) {
        use NextBoardStatus::*;
        match status {
            Win => {
                tree.value = BoardValue::win(1).unwrap();
                tree.children.clear();
                return tree;
            }
            Lose => continue,
            Unknown => {
                let child =
                    create_checkmate_tree_unchecked(next_board, !player, max_depth - 1, rule);
                let child_value_increment = child.value.increment();
                if child_value_increment < tree.value {
                    continue;
                }
                if child_value_increment > tree.value {
                    tree.value = child_value_increment;
                    tree.children.clear();
                }
                tree.children.insert(action, child);
            }
        }
    }
    tree
}

/// Creates an [`BoardValueTree`] that describes routes to ends of the game.
///
/// This function behaves almost similar to [`create_checkmate_tree`],
/// except that it receives `value` ([`BoardValue`]) instead of search depth,
/// and requires `value` coinsides with the value of `board`.
/// This function runs much faster than [`create_checkmate_tree_with_value`]
/// in exchange for restriction of possible arguments.
///
/// # Errors
/// Returns `Err` when the argument is invalid. Specifically,
/// the following cases are invalid:
/// - `board` is already finished (at least one boss is surrounded)
/// - `value` is `Unknown`, `Win(even number)` or `Lose(odd or zero)`
/// - `rule.suicide_atk_judge` is `Judge::Draw`
/// Furthermore, it returns `Err` when the value of `board` differs from `value`.
/// In such a case, returned value contains a variant of [`std::cmp::Ordering`],
/// which means that the value of board is greater (`Greater`) or less (`Less`)
/// than the specified `value`.
pub fn create_checkmate_tree_with_value(
    board: Board,
    value: BoardValue,
    player: Color,
    rule: GameRule,
) -> Result<BoardValueTree, CreateCheckmateTreeWithValueError> {
    type Error = CreateCheckmateTreeWithValueError;
    validate_args(board, value, rule).map_err(Error::ArgsValidationError)?;
    let (tree, cmp) = create_checkmate_tree_with_value_unchecked(board, value, player, rule);
    if cmp == Ordering::Equal {
        Ok(tree)
    } else {
        Err(Error::ValueMismatchError(cmp))
    }
}

fn create_checkmate_tree_with_value_unchecked(
    board: Board,
    value: BoardValue,
    player: Color,
    rule: GameRule,
) -> (BoardValueTree, Ordering) {
    use Ordering::*;
    let mut tree = BoardValueTree::new(board, player);
    tree.value = value;
    let mut cmp = Less;
    // tree.value = value;
    for (action, next_board, status) in NextBoardIter::new(board, player, rule) {
        use NextBoardStatus::*;
        match status {
            Win => {
                if value == BoardValue::MAX {
                    cmp = Equal;
                } else {
                    cmp = Greater;
                    tree.value = BoardValue::unknown();
                }
                tree.children.clear();
                return (tree, cmp);
            }
            Lose => continue,
            Unknown => {
                if value == BoardValue::MAX {
                    continue;
                }
                let next_value = value.try_decrement().unwrap();
                let (child, next_cmp) = create_checkmate_tree_with_value_unchecked(
                    next_board, next_value, !player, rule,
                );
                if next_cmp == Less {
                    tree.value = BoardValue::unknown();
                    tree.children.clear();
                    return (tree, Greater);
                }
                if next_cmp == Equal {
                    tree.children.insert(action, child);
                }
                cmp = cmp.max(next_cmp.reverse());
            }
        }
    }
    if cmp != Equal {
        tree.value = BoardValue::unknown();
        tree.children.clear();
    }
    (tree, cmp)
}

/// Compares the value of specified [`Board`] to a given [`BoardValue`].
///
/// It returns:
/// - `Ok(Greater)` if the value of `board` is greater than `value`
/// - `Ok(Equal)` if the value of `board` equals to `value`
/// - `Ok(Less)` if the value of `board` is less than `value
///
/// # Errors
/// Returns `Err` only when the argument is invalid. Specifically,
/// the following cases are invalid:
/// - `board` is already finished (at least one boss is surrounded)
/// - `value` is `Unknown`, `Win(even number)` or `Lose(odd or zero)`
/// - `rule.suicide_atk_judge` is `Judge::Draw`
pub fn compare_board_value(
    board: Board,
    value: BoardValue,
    player: Color,
    rule: GameRule,
) -> Result<Ordering, ArgsValidationError> {
    validate_args(board, value, rule)?;
    Ok(compare_board_value_unchecked(board, value, player, rule))
}

fn compare_board_value_unchecked(
    board: Board,
    value: BoardValue,
    player: Color,
    rule: GameRule,
) -> Ordering {
    use Ordering::*;
    let mut cmp = Less;
    for (_, next_board, status) in NextBoardIter::new(board, player, rule) {
        use NextBoardStatus::*;
        match status {
            Win => {
                if value == BoardValue::MAX {
                    return Equal;
                } else {
                    return Greater;
                }
            }
            Lose => continue,
            Unknown => {
                if value == BoardValue::MAX {
                    continue;
                }
                let next_val = value.try_decrement().unwrap();
                let next_cmp = compare_board_value_unchecked(next_board, next_val, !player, rule);
                if next_cmp == Less {
                    return Greater;
                }
                cmp = cmp.max(next_cmp.reverse());
            }
        }
    }
    cmp
}

/// Calculates [`BoardValue`] of specified [`Board`].
///
/// It searches the value of `board` by pursuing `search_depth` turns forward.
/// The result is returned in [`Interval`], which is a closed interval
/// between two [`BoardValue`]s.
///
/// # Errors
/// Returns `Err` only when the argument is invalid. Specifically,
/// the following cases are invalid:
/// - `board` is already finished (at least one boss is surrounded)
/// - `rule.suicide_atk_judge` is `Judge::Draw`
pub fn evaluate_board(
    board: Board,
    player: Color,
    search_depth: usize,
    rule: GameRule,
) -> Result<Interval, ArgsValidationError> {
    validate_args(board, BoardValue::MAX, rule)?;
    Ok(evaluate_board_unchecked(board, player, search_depth, rule))
}

fn evaluate_board_unchecked(
    board: Board,
    player: Color,
    search_depth: usize,
    rule: GameRule,
) -> Interval {
    for depth in 1..=search_depth {
        let value = BoardValue::from(Some(depth));
        if matches!(
            compare_board_value_unchecked(board, value, player, rule),
            Ordering::Equal
        ) {
            return Interval::new(value, value);
        }
    }
    let (left_num, right_num) = if search_depth % 2 == 0 {
        (search_depth + 2, search_depth + 1)
    } else {
        (search_depth + 1, search_depth + 2)
    };
    let left = BoardValue::from(Some(left_num));
    let right = BoardValue::from(Some(right_num));
    Interval::new(left, right)
}

/// Collects the best [`Action`]s by [`BoardValue`]
///
/// # Errors
/// Returns `Err` only when the argument is invalid. Specifically,
/// the following cases are invalid:
/// - `board` is already finished (at least one boss is surrounded)
/// - `rule.suicide_atk_judge` is `Judge::Draw`
pub fn find_best_actions(
    board: Board,
    player: Color,
    search_depth: usize,
    rule: GameRule,
) -> Result<Vec<Action>, ArgsValidationError> {
    validate_args(board, BoardValue::MAX, rule)?;
    Ok(find_best_actions_unchecked(
        board,
        player,
        search_depth,
        rule,
    ))
}

fn find_best_actions_unchecked(
    board: Board,
    player: Color,
    search_depth: usize,
    rule: GameRule,
) -> Vec<Action> {
    if search_depth == 0 {
        return board
            .legal_actions(player, true, true, *rule.is_remove_accepted())
            .into_iter()
            .collect();
    }

    let value_interval = evaluate_board_unchecked(board, player, search_depth, rule);
    let value = value_interval.single().unwrap_or(BoardValue::unknown());

    let mut actions = Vec::new();
    for (action, next_board, status) in NextBoardIter::new(board, player, rule) {
        use NextBoardStatus::*;
        match status {
            Win => {
                if value != BoardValue::MAX {
                    unreachable!()
                }
                actions.push(action);
                continue;
            }
            Lose => continue,
            Unknown => (),
        }

        if value == BoardValue::MAX {
            continue;
        }

        let next_value = value.try_decrement().unwrap();
        if next_value.is_unknown() {
            if !matches!(
                compare_board_value_unchecked(
                    next_board,
                    value_interval.left().try_decrement().unwrap(),
                    !player,
                    rule
                ),
                Ordering::Greater
            ) {
                actions.push(action);
            }
        } else {
            // next_value is Win or Lose
            if matches!(
                compare_board_value_unchecked(next_board, next_value, !player, rule),
                Ordering::Equal
            ) {
                actions.push(action);
            }
        }
    }
    actions
}

// ************************************************************
//  Tests
// ************************************************************
#[cfg(test)]
mod tests {
    use crate::{analysis::BoardValue, *};

    #[test]
    fn test_evaluate_board() {
        use analysis::BoardValue;
        use std::str::FromStr;
        let rule = game::GameRule::new(true).with_suicide_atk_judge(game::Judge::NextWins);
        let board_value = [
            (" B; a;TH y;b mM", 5),
            (" By;H  a;A m;  Yb", 3),
            ("bB; H;Y h;  T", 3),
            ("T; B b;  yY; t", 3),
            ("hB A;maYT; Htb;M y", 5),
            ("Ba;  H;  hA;MY b", 5),
            (" B;a  b; MtY;T  H", 7),
            ("Y; B;y hA; M b", 5),
            ("bB;T YA", 5),
        ];
        let max_depth = 10; // it must be greater than all n of Win(n)
        for (s, num) in board_value {
            let val = BoardValue::win(num).unwrap();
            let board = BoardBuilder::from_str(s).unwrap().build().unwrap();
            let evaluated = analysis::evaluate_board(board, Color::Red, max_depth, rule).unwrap();
            assert_eq!(evaluated.single().unwrap(), val);
        }
    }

    #[test]
    fn test_compare_value() {
        use analysis::BoardValue;
        use std::str::FromStr;
        let rule = game::GameRule::new(true).with_suicide_atk_judge(game::Judge::NextWins);
        let board_value = [
            (" B; a;TH y;b mM", 5),
            (" By;H  a;A m;  Yb", 3),
            ("bB; H;Y h;  T", 3),
            ("T; B b;  yY; t", 3),
            ("hB A;maYT; Htb;M y", 5),
            ("Ba;  H;  hA;MY b", 5),
            (" B;a  b; MtY;T  H", 7),
            ("Y; B;y hA; M b", 5),
            ("bB;T YA", 5),
        ];
        for (s, num) in board_value {
            let val = BoardValue::win(num).unwrap();
            let board = BoardBuilder::from_str(s).unwrap().build().unwrap();
            let cmp = analysis::compare_board_value(board, val, Color::Red, rule);
            assert!(matches!(cmp, Ok(std::cmp::Ordering::Equal)));
        }
    }

    #[test]
    fn test_checkmate_tree() {
        use std::str::FromStr;
        let rule = game::GameRule::new(true).with_suicide_atk_judge(game::Judge::NextWins);
        let board_value = [
            (" B; a;TH y;b mM", 5),
            (" By;H  a;A m;  Yb", 3),
            ("bB; H;Y h;  T", 3),
            ("T; B b;  yY; t", 3),
            ("hB A;maYT; Htb;M y", 5),
            ("bB;T YA", 5),
        ];
        for (s, num) in board_value {
            let board = BoardBuilder::from_str(s).unwrap().build().unwrap();
            let tree = analysis::create_checkmate_tree(board, Color::Red, num, rule).unwrap();
            assert!(tree.is_good_for_puzzle(num - 2));
        }
    }

    #[test]
    fn test_checkmate_tree_with_value() {
        use std::str::FromStr;
        let rule = game::GameRule::new(true).with_suicide_atk_judge(game::Judge::NextWins);
        let board_value = [
            (" B; a;TH y;b mM", 5),
            (" By;H  a;A m;  Yb", 3),
            ("bB; H;Y h;  T", 3),
            ("T; B b;  yY; t", 3),
            ("hB A;maYT; Htb;M y", 5),
            ("Ba;  H;  hA;MY b", 5),
            (" B;a  b; MtY;T  H", 7),
            ("Y; B;y hA; M b", 5),
            ("bB;T YA", 5),
        ];
        for (s, num) in board_value {
            let val = BoardValue::win(num).unwrap();
            let board = BoardBuilder::from_str(s).unwrap().build().unwrap();
            let tree =
                analysis::create_checkmate_tree_with_value(board, val, Color::Red, rule).unwrap();
            assert!(tree.is_good_for_puzzle(num - 2));
        }
    }
}
