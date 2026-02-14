use bevy::prelude::*;

/// The different states of the game.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    /// World is being spawned.
    #[default]
    Loading,
    /// Main menu is being displayed.
    Menu,
    /// Cards are being dealt.
    Deal,
    /// Game is in progress.
    ///
    /// Cards are de-spawned when exiting this state.
    Play,
    /// Win screen is being displayed.
    Win,
}

/// Resource that tracks how many cards have been collected by the player.
///
/// This counter is reset to `0` when entering [`GameState::Win`].
#[derive(Resource, Debug, Default)]
pub struct CardsCollected(pub u8);
