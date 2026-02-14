use crate::state::GameState;
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

/// System that spawns the menu when entering the [`GameState::Win`] state.
pub fn win_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/UbuntuNerdFont-Medium.ttf");
    let font_component = TextFont {
        font: font.clone(),
        font_size: 24.0,
        ..Default::default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..Default::default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.75)),
            DespawnOnExit(GameState::Win),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Auto,
                    height: Val::Auto,
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|parent| {
                parent.spawn((
                    // embedded the Octicons rocket icon via unicode code-point
                    Text::new("Congratulations! \u{f427}"),
                    TextColor(Color::WHITE),
                    TextFont {
                        // nerd font required for code-point to render correctly
                        font: font.clone(),
                        font_size: 40.0,
                        ..Default::default()
                    },
                ));
                parent.spawn((
                    Text::new("You picked up all 52 cards!"),
                    TextColor(Color::WHITE),
                    TextFont {
                        font: font.clone(),
                        font_size: 32.0,
                        ..Default::default()
                    },
                ));
                parent.spawn((
                    Text::new("Now go and play a real game. NERD!"),
                    TextColor(Color::WHITE),
                    font_component.clone(),
                ));
            });
            spawn_button(root, font_component.clone());
        });
}

/// System that runs during [`GameState::Win`] and [`GameState::Menu`] to detect when the "Start Game" button is pressed.
///
/// When the button is pressed, this system sets the game state to [`GameState::Deal`], which starts the game.
pub fn button_detector(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for interaction in query {
        if *interaction == Interaction::Pressed {
            info!("Restarting game");
            game_state.set(GameState::Deal);
        }
    }
}

/// System that runs when entering the [`GameState::Menu`] state (on game startup only).
pub fn hello_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/UbuntuNerdFont-Medium.ttf");
    let font_component = TextFont {
        font: font.clone(),
        font_size: 24.0,
        ..Default::default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..Default::default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.75)),
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Auto,
                    height: Val::Auto,
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                BackgroundColor(Color::NONE),
                children![(
                    // embedded the MD hand wave emoji via unicode code-point
                    Text::new("\u{F1821} Welcome to 52 Card Pickup!"),
                    TextColor(Color::WHITE),
                    TextFont {
                        // nerd font required for code-point to render correctly
                        font: font.clone(),
                        font_size: 32.0,
                        ..Default::default()
                    },
                )],
            ));
            spawn_button(root, font_component.clone());
        });
}

/// Spawns the "Start Game" button in the menu, which starts the game when pressed.
fn spawn_button(commands: &mut RelatedSpawnerCommands<'_, ChildOf>, font_component: TextFont) {
    commands.spawn((
        Button,
        Node {
            width: Val::Auto,
            height: Val::Auto,
            padding: UiRect::all(Val::Px(16.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..Default::default()
        },
        BackgroundColor(Color::srgb(0.125, 0.85, 0.125)),
        children![(
            // embedded the FontAwesome Play icon (circle variant) via unicode code-point
            Text::new("\u{F01D} Start Game"),
            TextColor(Color::WHITE),
            // TextFont component (pointing to nerd font asset) must be in same bundle as
            // the Text component for the font to render the unicode code-point correctly.
            font_component,
        )],
    ));
}
