//! Convenient tools for playing games ("game" feature required)
//!
//! Main contents:
//! - [`Game`]<br>
//!     A struct that provides methods to play the game.
//! - [`Agent`]<br>
//!     A trait to define an agent playing the game.
//!     The following agents are included:
//!     - [`RandomAgent`]<br>
//!         An agent who chooses the next action at random.
//!     - [`AnalystAgent`]<br>
//!         An agent who chooses the next action based on some analyses.
//!     - [`ConsoleAgent`]<br>
//!         An agent who asks the next action to the console.
//!         It provides CLI with humans.
//!
//!     You can define another agent in you code
//!     by implementing the [`Agent`] trait if you want.
//! - [`Arena`]<br>
//!     A struct of an arena, where two [`Agent`]s play against.
//!
//! All entities in this module are available when "game" feature is indicated,
//! except [`AnalystAgent`] which is available when "analysis" feature is indicated.

#[cfg(feature = "analysis")]
use crate::analysis::{evaluate_board, find_best_actions};
use crate::error;
use crate::prelude::{Action, ActionContainer, ActionsFwd, Board, Color, SurroundedStatus};

// ************************************************************
//  Building Blocks
// ************************************************************
/// A struct holding detailed rules for playing games.
///
/// # Examples
/// ```rust
/// use std::str::FromStr;
/// use tokyodoves::{Color, Board, BoardBuilder};
/// use tokyodoves::game::{GameRule, Judge};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Set whether `Remove` is allowed or not
/// let rule = GameRule::new(true);
/// // Auto setting (allow `Remove`)
/// let rule = GameRule::default();
/// // Allow `Remove` and `Color::Green` moves first
/// let rule = GameRule::new(true).with_first_player(Color::Green);
/// // Set board you like as the initial board (requires error handling)
/// let initial_board = BoardBuilder::from_str("B;bh")?.build()?;
/// let rule = GameRule::default().with_initial_board(initial_board)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct GameRule {
    /// Allow [`Action::Remove`] as a legal action or not
    is_remove_accepted: bool,
    /// The player who moves first
    first_player: Color,
    /// Winner judgement when both bosses are surrounded simultaneously
    suicide_atk_judge: Judge,
    /// Initial board
    initial_board: Board,
}

impl GameRule {
    /// Constructs [`GameRule`] object
    ///
    /// Default configurations:
    /// - `first_player`<br>
    ///     ... `Color::Red`
    /// - `suicide_atk_judge` (who wins if both bosses are simultaneously surrounded)<br>
    ///     ... `Judge::NextWins` (the next player who just moved the dove wins)
    /// - `initial_board`<br>
    ///     ... `Board::new()`
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::GameRule;
    /// let rule = GameRule::new(true);
    /// ```
    pub fn new(is_remove_accepted: bool) -> Self {
        let first_player = Color::Red;
        let suicide_atk_judge = Judge::NextWins;
        let initial_board = Board::new();
        Self {
            is_remove_accepted,
            first_player,
            suicide_atk_judge,
            initial_board,
        }
    }

    /// Returns `true` if [`Action::Remove`] is accepted.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::GameRule;
    ///
    /// let rule = GameRule::new(true);
    /// assert!(rule.is_remove_accepted());
    /// ```
    pub fn is_remove_accepted(&self) -> &bool {
        &self.is_remove_accepted
    }

    /// Returns the player who moves at the beginning of the game.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Color;
    /// use tokyodoves::game::GameRule;
    ///
    /// let rule = GameRule::new(true);
    /// assert!(matches!(rule.first_player(), Color::Red));
    /// ```
    pub fn first_player(&self) -> &Color {
        &self.first_player
    }

    /// Returns judgement when the both bosses are simultaneously surrounded.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::{GameRule, Judge};
    ///
    /// let rule = GameRule::new(true);
    /// assert!(matches!(rule.suicide_atk_judge(), Judge::NextWins));
    /// ```
    pub fn suicide_atk_judge(&self) -> &Judge {
        &self.suicide_atk_judge
    }

    /// Returns the board at the beginning of the game.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Board;
    /// use tokyodoves::game::GameRule;
    ///
    /// let rule = GameRule::new(true);
    /// assert_eq!(*rule.initial_board(), Board::new());
    pub fn initial_board(&self) -> &Board {
        &self.initial_board
    }

    /// Update whether accept `Remove` in the game or not.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::GameRule;
    /// let rule = GameRule::new(false).with_is_remove_accepted(true);
    /// ```
    pub fn with_is_remove_accepted(self, is_remove_accepted: bool) -> Self {
        Self {
            is_remove_accepted,
            ..self
        }
    }

    /// Update the player moving firstly in the beginning of the game.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::Color;
    /// use tokyodoves::game::GameRule;
    ///
    /// let rule = GameRule::new(true).with_first_player(Color::Green);
    /// ```
    pub fn with_first_player(self, first_player: Color) -> Self {
        Self {
            first_player,
            ..self
        }
    }

    /// Update judgement rule when both bosses are surrounded simultaneously.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::{GameRule, Judge};
    /// let rule = GameRule::new(true).with_suicide_atk_judge(Judge::Draw);
    /// ```
    pub fn with_suicide_atk_judge(self, judge: Judge) -> Self {
        Self {
            suicide_atk_judge: judge,
            ..self
        }
    }

    /// Update initial_board of [`GameRule`].
    ///
    /// # Errors
    /// It returns:
    /// - `Err(error::GameRuleCreateErrorKind::InitialBoardError.into())`
    ///     if `initial_board` is that of finished game
    ///
    /// # Examples
    /// ```rust
    /// use std::str::FromStr;
    /// use tokyodoves::BoardBuilder;
    /// use tokyodoves::game::GameRule;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let board = BoardBuilder::from_str("BbA")?.build()?;
    /// let rule = GameRule::new(true).with_initial_board(board)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_initial_board(self, initial_board: Board) -> Result<Self, error::Error> {
        if !matches!(initial_board.surrounded_status(), SurroundedStatus::None) {
            return Err(error::GameRuleCreateErrorKind::InitialBoardError.into());
        }
        let rule = Self {
            initial_board,
            ..self
        };
        Ok(rule)
    }
}

impl Default for GameRule {
    /// Returns the default value.
    ///
    /// The default values of fields are as below:
    /// - `allow_remove`: `true`
    /// - others: follow setting in the [`new`](`Self::new`) method on `GameRule`.
    fn default() -> Self {
        Self::new(true)
    }
}

/// Judgement of winner on some event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Judge {
    /// The player just before the event is treated as the winner
    LastWins,
    /// The player just after the event is treated as the winner
    NextWins,
    /// It is a draw game
    Draw,
}

/// Status of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStatus {
    /// The game is ongoing
    Ongoing,
    /// The game has already finished; one player defeated the other
    Win(Color),
    /// The game has already finished; it was a draw game
    Draw,
}

// ************************************************************
//  Game Struct
// ************************************************************
/// A struct that provides methods to play games.
///
/// # Examples
/// The following is a simple example in which one game is played:
/// ```rust
/// use tokyodoves::ActionContainer;
/// use tokyodoves::game::Game;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a [`Game`] object allowing `Remove` action
///     let mut game = Game::new(true);
///     // Displays the status of the game
///     println!("{game}");
///
///     let mut turn = 0;
///     loop {
///         turn += 1;
///         // Collects all possible actions
///         let actions = game.legal_actions();
///         // Selects one of them by methods of ActionContainer trait
///         let action = actions[turn % actions.len()];
///         println!("  --> {}", action.try_into_ssn(game.board())?);
///
///         game.perform(action)?; // Performs the selected action
///         println!("{game}");
///
///         if !game.is_ongoing() { // If the game is not ongoing...
///             break; // breaks the loop!
///         }
///     }
///
///     match game.winner() { // Who won the game?
///         Some(player) => println!("*** {player} win! ***"), // A player won the game
///         None => println!("*** Draw! ***"), // It was a draw game
///     }
///
///     Ok(())
/// }
/// ```
/// To customize the rule more, you can create [`Game`] from [`GameRule`] object:
/// ```rust
/// use tokyodoves::Color;
/// use tokyodoves::game::{Game, GameRule};
///
/// # fn main() {
/// let mut rule = GameRule::default()
///     .with_is_remove_accepted(false)
///     .with_first_player(Color::Green);
/// let game = Game::new_with_rule(rule);
/// println!("{game}");
/// # }
/// ```
/// For more information about the default value of [`GameRule`],
/// see the [documentation](`GameRule::default`) about the implementation of
/// [`Default`] trait for `GameRule`.
#[derive(Debug, Clone, Copy)]
pub struct Game {
    board: Board,
    player: Color,
    status: GameStatus,
    rule: GameRule,
}

impl Game {
    /// Constructs [`Game`]
    pub fn new(is_remove_accepted: bool) -> Game {
        let rule = GameRule {
            is_remove_accepted,
            ..Default::default()
        };
        Self::new_with_rule(rule)
    }

    /// Constructs [`Game`] with a specified `rule`
    pub fn new_with_rule(rule: GameRule) -> Game {
        let board = rule.initial_board;
        let player = rule.first_player;
        let status = GameStatus::Ongoing;
        Game {
            board,
            player,
            status,
            rule,
        }
    }

    /// Reset [`Game`] to the initial state
    pub fn reset(&mut self) {
        *self = Self::new_with_rule(self.rule)
    }

    /// Get a reference to game rule
    pub fn rule(&self) -> &GameRule {
        &self.rule
    }

    /// Get a reference to board
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Get a reference to the next player
    pub fn next_player(&self) -> &Color {
        &self.player
    }

    /// Get a reference to status
    pub fn status(&self) -> &GameStatus {
        &self.status
    }

    /// Returns `true` if the game is ongoing
    pub fn is_ongoing(&self) -> bool {
        matches!(self.status, GameStatus::Ongoing)
    }

    /// Returns winner.
    ///
    /// It returns `Some(Color)` if winner has been determined.
    /// It returns `None` if the game is ongoing or draw.
    pub fn winner(&self) -> Option<Color> {
        use GameStatus::*;
        match self.status {
            Ongoing | Draw => None,
            Win(player) => Some(player),
        }
    }

    /// Checks if the specified [`Action`] is legal
    pub fn check_action(&self, action: Action) -> Result<(), error::Error> {
        use error::PlayingErrorKind::*;
        if self.player != *action.player() {
            return Err(PlayerMismatch.into());
        }

        if !self.rule.is_remove_accepted && matches!(action, Action::Remove(_, _)) {
            return Err(ProhibitedRemove(action).into());
        }

        self.board().check_action(action)?;
        Ok(())
    }

    /// Returns an [`ActionContainer`](`super::board::container::ActionContainer`) of legal [`Action`]s.
    pub fn legal_actions(&self) -> ActionsFwd {
        self.board
            .legal_actions(self.player, true, true, self.rule.is_remove_accepted)
    }

    /// Performs specified [`Action`].
    ///
    /// If the given action is performed successfully,
    /// a game end judgement is made.
    /// If it is determined that the game should continue,
    /// the turn moves to the next player.
    ///
    /// # Errors
    /// It returns:
    /// - `Err(error::PlayingErrorKind::GameFinished(..).into())` if the game has already been finished.
    /// - `Err(error::PlayingErrorKind::PlayerMismatch(..).into())` if the player of `action`
    ///     is different from the next player.
    /// - `Err(error::PlayingErrorKind::ProhibitedRemove(..).into())` if the action is `Action::Remove`
    ///     although `game.rule()` prohibits to remove a piece.
    /// - `Err(error::Error::BoardError(..).into())` if performing `action` is illegal for board.
    ///
    /// In any cases, [`Game`] object is left unchanged.
    pub fn perform(&mut self, action: Action) -> Result<(), error::Error> {
        if !self.is_ongoing() {
            return Err(error::PlayingErrorKind::GameFinished(self.status).into());
        }
        self.check_action(action)?;
        self.board.perform_unchecked(action);

        use GameStatus::*;
        use SurroundedStatus::*;
        match self.board.surrounded_status() {
            Both => match self.rule.suicide_atk_judge {
                Judge::LastWins => self.status = Win(self.player),
                Judge::NextWins => self.status = Win(!self.player),
                Judge::Draw => self.status = Draw,
            },
            OneSide(player) => self.status = Win(!player),
            None => self.player = !self.player,
        }

        Ok(())
    }

    pub fn display(&self) -> GameDisplay {
        GameDisplay::new(self)
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Formats of display used by [`GameDisplay`]
#[derive(Debug, Clone)]
pub enum GameDisplayFormat {
    /// Standard Display Style.
    ///
    /// It displays [`Board`] in framed style, and next player.
    /// If the game is finished, next player is displayed as "None".
    /// # Examples
    /// ```text
    /// +---+---+---+---+
    /// | b |   |   |   |
    /// +---+---+---+---+
    /// | B |   |   |   |
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// |   |   |   |   |
    /// +---+---+---+---+
    /// Next Player: Red
    /// ```
    Standard,
}

impl Default for GameDisplayFormat {
    fn default() -> Self {
        Self::Standard
    }
}

impl GameDisplayFormat {
    fn typeset(&self, game: &Game) -> String {
        use GameDisplayFormat::*;
        match self {
            Standard => Self::typeset_standard(game),
        }
    }

    fn typeset_standard(game: &Game) -> String {
        let next_player_string = if game.is_ongoing() {
            format!("{:?}", game.next_player())
        } else {
            String::from("None")
        };
        format!("{}\nNext Player: {next_player_string}", game.board())
    }
}

/// A struct to configure display styles of [`Game`].
///
/// `Display` trait is implemented for this struct
/// so that the display format depends on the internal value of [`GameDisplayFormat`],
/// which can be changed by the [`with_format`](`GameDisplay::with_format`) method.
/// See the documentation of [`GameDisplayFormat`]
/// for information about available display styles.
///
/// # Examples
/// ```rust
/// use tokyodoves::game::Game;
///
/// let game = Game::new(true);
/// println!("{}", game.display());
#[derive(Debug, Clone)]
pub struct GameDisplay<'a> {
    game: &'a Game,
    format: GameDisplayFormat,
}

impl<'a> GameDisplay<'a> {
    fn new(game: &'a Game) -> Self {
        Self {
            game,
            format: Default::default(),
        }
    }

    pub fn with_format(self, format: GameDisplayFormat) -> Self {
        Self { format, ..self }
    }
}

impl<'a> std::fmt::Display for GameDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format.typeset(self.game))
    }
}

// ************************************************************
//  Agent Trait
// ************************************************************
/// An agant who plays the game.
///
/// [`Arena`] receives two agetns and let them play the game.
pub trait Agent {
    /// Performs some action on board.
    ///
    /// Some agent may analyze the status of the game using
    /// the [`board`](`Game::board`) method on [`Game`] or other information.
    fn play(&mut self, game: &mut Game);
}

/// An [`Agent`] who chooses a next action at random.
#[derive(Default)]
pub struct RandomAgent {
    n: usize,
}

impl std::fmt::Debug for RandomAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RandomAgent")
    }
}

impl std::fmt::Display for RandomAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl RandomAgent {
    /// Creates `RandomAgent` object.
    ///
    /// # Example
    /// ```rust
    /// use tokyodoves::game::RandomAgent;
    /// let agent = RandomAgent::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    fn update_parameter(&mut self) {
        self.n = (33 * self.n + 31) % 65536
    }
}

impl Agent for RandomAgent {
    /// Performs a legal action that was chosen at random.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::{Agent, Game, RandomAgent};
    ///
    /// let mut agent = RandomAgent::new();
    /// let mut game = Game::new(true);
    /// println!("{game}");
    /// agent.play(&mut game);
    /// println!("{game}");
    /// ```
    fn play(&mut self, game: &mut Game) {
        self.update_parameter();
        let actions = game.legal_actions();
        let action = actions[self.n % actions.len()];
        game.perform(action).expect("illegal situation");
    }
}

/// An [`Agent`] who choses a next action based on analysis of the status.
/// This struct is available when "analysis" feature is activated.
#[cfg(feature = "analysis")]
pub struct AnalystAgent {
    depth: usize,
    n: usize,
    declare_about_to_end: bool,
}

#[cfg(feature = "analysis")]
impl std::fmt::Debug for AnalystAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalystAgent")
            .field("depth", &self.depth)
            .field("declare_about_to_end", &self.declare_about_to_end)
            .finish()
    }
}

#[cfg(feature = "analysis")]
impl std::fmt::Display for AnalystAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnalystAgent")
    }
}

#[cfg(feature = "analysis")]
impl AnalystAgent {
    /// Creates an [`AnalystAgent] object.
    ///
    /// This agent searches the game until next `depth` turns
    /// by tools in the [`analysis`](`crate::analysis`) module.
    /// If `declare_about_to_end` is `true`,
    /// a message to notify that the game is about to end
    /// will displayed when this agent recognizes it.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::AnalystAgent;
    /// let agent = AnalystAgent::new(3, true);
    /// ```
    pub fn new(depth: usize, declare_about_to_end: bool) -> Self {
        Self {
            depth,
            n: 0,
            declare_about_to_end,
        }
    }

    fn update_parameter(&mut self) {
        self.n = (33 * self.n + 31) % 65536
    }
}

#[cfg(feature = "analysis")]
impl Agent for AnalystAgent {
    /// Choses and performs an action based on analysis of the game.
    ///
    /// It calls the [`find_best_actions`] function
    /// in the [`analysis`](`crate::analysis`) module
    /// to find candidates.
    /// It choses and performs one of them at random.
    /// If the value of the board is exactly identified,
    /// it prints a message to notice that the game is about to end
    /// if it was constructed with `declare_about_to_end=true`.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::{Agent, Game, AnalystAgent};
    ///
    /// let mut agent = AnalystAgent::new(3, true);
    /// let mut game = Game::new(true);
    /// println!("{game}");
    /// agent.play(&mut game);
    /// println!("{game}");
    /// ```
    fn play(&mut self, game: &mut Game) {
        self.update_parameter();
        let board = *game.board();
        let player = *game.next_player();
        let rule = *game.rule();
        let candidates = find_best_actions(board, player, self.depth, rule).unwrap();
        let action = candidates[self.n % candidates.len()];

        if self.declare_about_to_end {
            if let Some(val) = evaluate_board(board, player, self.depth, rule)
                .unwrap()
                .single()
            {
                println!("!!! This game is about to end: value={val}");
            }
        }

        game.perform(action).unwrap();
    }
}

/// An [`Agent`] who asks what action should be performed to the console.
#[derive(Default)]
pub struct ConsoleAgent;

impl ConsoleAgent {
    /// Creates an [`AnalystAgent] object.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::ConsoleAgent;
    /// let agent = ConsoleAgent::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }
}

impl std::fmt::Debug for ConsoleAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConsoleAgent")
    }
}

impl std::fmt::Display for ConsoleAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl Agent for ConsoleAgent {
    /// Asks what action should be performed to the console, and performs it.
    ///
    /// It receives actions in SSN (standard short notation).
    /// See the documentation of [`Action`] for the definition of SSN.
    ///
    /// # Examples
    /// ``` ignore
    /// use tokyodoves::game::{Agent, ConsoleAgent, Game};
    ///
    /// let mut agent = ConsoleAgent::new();
    /// let mut game = Game::new(true);
    /// println!("{game}");
    /// agent.play(&mut game); // asks to the console
    /// println!("{game}");
    /// ```
    fn play(&mut self, game: &mut Game) {
        let legal_actions = game.legal_actions();

        let mut buffer = String::new();
        let action: Action;
        loop {
            buffer.clear();
            println!("Input an action in SSN:");
            std::io::stdin()
                .read_line(&mut buffer)
                .expect("read line error");
            let ssn = buffer.trim();
            let Ok(action_tmp) = Action::try_from_ssn(ssn, game.board()) else {
                println!("Invalid Input. Try Again.");
                continue;
            };
            println!("---> Infered Action: {action_tmp:?}");
            if !legal_actions.contains(action_tmp) {
                println!("Illegal Action. Try Again.");
                continue;
            }
            action = action_tmp;
            break;
        } // ask action
        game.perform(action).expect("failed to perform");
    }
}

/// A struct to let two [`Agent`]s to play the game.
#[derive(Debug, Clone)]
pub struct Arena<AR, AG>
where
    AR: Agent,
    AG: Agent,
{
    agent_red: AR,
    agent_green: AG,
    game: Game,
}

impl<AR, AG> std::fmt::Display for Arena<AR, AG>
where
    AR: Agent + std::fmt::Display,
    AG: Agent + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!(
            "Red Agent  : {}\nGreen Agent: {}\n{}",
            self.agent_red, self.agent_green, self.game
        );
        write!(f, "{s}")
    }
}

impl<AR, AG> Arena<AR, AG>
where
    AR: Agent,
    AG: Agent,
{
    /// Creates an arena.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::{Arena, ConsoleAgent, RandomAgent, Game};
    ///
    /// let red = ConsoleAgent::new();
    /// let green = RandomAgent::new();
    /// let game = Game::new(true);
    /// let arena = Arena::new(red, green, game);
    /// ```
    pub fn new(agent_red: AR, agent_green: AG, game: Game) -> Self {
        Self {
            agent_red,
            agent_green,
            game,
        }
    }

    /// Let agents play the game until the game ends.
    ///
    /// In each turn,
    /// [`Game`] object given on construction is passed to the agent of the turn,
    /// and the agent receives it as an argument of the [`play`](`Agent::play`) method
    /// to perform an action.
    ///
    /// If `verbose` is `true`, the status will be displayed on every turn.
    /// If `verbose` is `false`, only the state at the end of the game will be displayed.
    ///
    /// # Examples
    /// ```rust
    /// use tokyodoves::game::{Arena, RandomAgent, Game};
    /// // use tokyodoves::game::AnalystAgent; // available when feature = "analysis"
    /// let red = RandomAgent::new();
    /// let green = RandomAgent::new();
    /// // let green = AnalystAgent::new(3, true); // available when feature = "analysis"
    /// let game = Game::new(true);
    /// let mut arena = Arena::new(red, green, game);
    /// arena.auto_play(true);
    /// ```
    pub fn auto_play(&mut self, verbose: bool) {
        let mut num_turns = 0_u128;
        loop {
            num_turns += 1;
            if verbose {
                println!("{}", self.game);
            }

            match self.game.next_player() {
                Color::Red => self.agent_red.play(&mut self.game),
                Color::Green => self.agent_green.play(&mut self.game),
            }

            if !self.game.is_ongoing() {
                break;
            }
        }

        println!("~~~~~~ Game Finished! ~~~~~~");
        println!("Total {num_turns} turns");
        println!("{}", self.game);
        match self.game.winner() {
            Some(player) => println!("---> {player} win!"),
            None => println!("---> Draw!"),
        }
    }
}
