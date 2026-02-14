use std::f32::consts::PI;

use bevy::{
    animation::AnimatedBy,
    color::palettes::{css::WHITE, tailwind::GREEN_300},
    prelude::*,
};
use rand::{RngExt, rng};

mod cards;
use cards::{
    BOARD_HALF_SIZE, CARD_HALF_SIZE, CARD_THICKNESS, Card, CardBundle, CardMaterial, shuffle_deck,
};
mod animator;
use animator::{collect_card, pressed_card};
mod menu;
use menu::{button_detector, hello_menu, win_menu};
mod state;
use state::{CardsCollected, GameState};

const CAMERA_DISTANCE: f32 = 668.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .init_state::<GameState>()
        .add_systems(Startup, setup_world)
        .init_resource::<Assets<CardMaterial>>()
        .init_resource::<CardsCollected>()
        .add_systems(OnEnter(GameState::Deal), deal)
        .add_systems(OnEnter(GameState::Win), win_menu)
        .add_systems(OnEnter(GameState::Menu), hello_menu)
        .add_observer(collect_card)
        .add_systems(Update, button_detector.run_if(in_state(GameState::Menu)))
        .add_systems(Update, button_detector.run_if(in_state(GameState::Win)))
        .run();
}

/// Sets up the 3d world, including the camera, lighting, and floor.
///
/// Also sets the game state to [`GameState::Menu`] when done.
fn setup_world(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, CAMERA_DISTANCE, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::from(WHITE),
            illuminance: 500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, CAMERA_DISTANCE, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    let floor_mesh = meshes.add(Plane3d::new(Vec3::Y, BOARD_HALF_SIZE * 2.0));
    let table_material = StandardMaterial {
        base_color: Color::from(GREEN_300),
        perceptual_roughness: 1.0,
        ..default()
    };
    let floor_material = materials.add(table_material);
    commands.spawn((Mesh3d(floor_mesh), MeshMaterial3d(floor_material)));
    game_state.set(GameState::Menu);
}

/// System that runs when entering the [`GameState::Deal`] state.
///
/// This function shuffles the deck and spawns the cards in random positions on the board.
/// When finished, this also sets the game state to [`GameState::Play`].
fn deal(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
) {
    let mut deck = shuffle_deck();
    let mut rand_ng = rng();

    let hover_back = asset_server.load("images/Back Red.png");
    let hover_material = materials.add(StandardMaterial {
        base_color_texture: Some(hover_back),
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    let mut count = 0.0;
    let cap_x = BOARD_HALF_SIZE.x - CARD_HALF_SIZE.x;
    let cap_y = BOARD_HALF_SIZE.y - CARD_HALF_SIZE.y;
    while let Some(mut card) = deck.pop() {
        card.playable = true;
        let x = rand_ng.random_range(-cap_x..cap_x);
        let y = rand_ng.random_range(-cap_y..cap_y);
        let mut transform = Transform::from_xyz(x, count, y).looking_to(Dir3::Y, Dir3::Z);
        let rand_skew = rand_ng.random_range(-PI..PI);
        transform.rotate_axis(Dir3::Y, rand_skew);
        let card_bundle = CardBundle::new(
            card,
            &asset_server,
            &mut materials,
            &mut meshes,
            transform,
            &mut animation_graphs,
            &mut animation_clips,
        );
        let children = card_bundle.make_children(&asset_server, &mut materials, &mut meshes);
        let card_back_material = card_bundle.material.0.clone();
        let card_entity = commands
            .spawn((DespawnOnExit(GameState::Play), card_bundle))
            .with_children(|parent| {
                parent.spawn(children);
            })
            .observe(update_material_on::<Pointer<Over>>(hover_material.clone()))
            .observe(update_material_on::<Pointer<Out>>(card_back_material))
            .observe(pressed_card)
            .observe(update_material_on::<Pointer<Release>>(
                hover_material.clone(),
            ))
            .id();
        commands.entity(card_entity).insert(AnimatedBy(card_entity));
        count += CARD_THICKNESS;
    }
    game_state.set(GameState::Play);
}

/// Returns an observer that updates the entity's material to the one specified.
#[allow(clippy::type_complexity)]
fn update_material_on<E: EntityEvent>(
    new_material: Handle<StandardMaterial>,
) -> impl Fn(On<E>, Query<(&Card, &mut MeshMaterial3d<StandardMaterial>)>) {
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |event, mut query| {
        if let Ok((card, mut material)) = query.get_mut(event.event_target())
            && card.playable
            && !card.face_up
        {
            info!("Updating material for Card {card}");
            material.0 = new_material.clone();
        }
    }
}
