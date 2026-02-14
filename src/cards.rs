#![allow(dead_code)]
use bevy::{
    animation::AnimationTargetId, ecs::relationship::RelatedSpawnerCommands, prelude::*,
    render::render_resource::AsBindGroup,
};
use rand::{rng, seq::SliceRandom};
use std::{
    f32::consts::PI,
    fmt::{self, Display},
};

use crate::animator::{AnimationInfo, AnimatorNodeId};

pub const CARD_W: f32 = 84.0;
pub const CARD_H: f32 = 120.0;
pub const CARD_THICKNESS: f32 = 0.1;
pub const CARD_SIZE_RATIO: f32 = CARD_W / CARD_H;
pub const CARD_HALF_SIZE: Vec2 = Vec2 {
    x: CARD_W / 2.0,
    y: CARD_H / 2.0,
};
pub const BOARD_HALF_SIZE: Vec2 = Vec2 { x: 354.0, y: 270.0 };

#[derive(Bundle)]
pub struct CardBundle<M: Material> {
    pub card: Card,
    pub material: MeshMaterial3d<M>,
    pub mesh: Mesh3d,
    pub transform: Transform,
    pub animation_target_name: Name,
    pub animation_player: AnimationPlayer,
    pub animation_graph_handle: AnimationGraphHandle,
    pub animation_target_id: AnimationTargetId,
    pub animation_node_index: AnimatorNodeId,
}

impl CardBundle<StandardMaterial> {
    const CARD_MESH: Cuboid = Cuboid {
        half_size: CARD_HALF_SIZE.extend(0.1),
    };
    pub fn new(
        card: Card,
        asset_server: &AssetServer,
        materials: &mut Assets<StandardMaterial>,
        meshes: &mut Assets<Mesh>,
        transform: Transform,
        animation_graphs: &mut Assets<AnimationGraph>,
        animation_clips: &mut Assets<AnimationClip>,
    ) -> Self {
        let back_material = materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(Card::back_resource_name())),
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        });
        let mesh = meshes.add(Self::CARD_MESH);

        let AnimationInfo {
            target_name: animation_target_name,
            target_id: animation_target_id,
            graph: animation_graph,
            node_index: animation_node_index,
        } = AnimationInfo::create(&transform, &card, animation_graphs, animation_clips);

        // Build an animation player (Component) to play animation(s) on
        // the player's Entity (`AnimatedBy` Component).
        let animation_player = AnimationPlayer::default();

        Self {
            card,
            material: MeshMaterial3d(back_material),
            mesh: Mesh3d(mesh),
            transform,
            animation_target_name,
            animation_player,
            animation_graph_handle: AnimationGraphHandle(animation_graph),
            animation_target_id,
            animation_node_index: AnimatorNodeId(animation_node_index),
        }
    }

    pub fn make_children(
        &self,
        asset_server: &Res<AssetServer>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> (Mesh3d, MeshMaterial3d<StandardMaterial>, Transform) {
        let face_material = MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(self.card.face_resource_name())),
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        }));
        let face_mesh = Mesh3d(meshes.add(Self::CARD_MESH));
        let mut face_transform = Transform::from_rotation(Quat::from_axis_angle(Vec3::X, PI));
        face_transform.translation.z += 0.1;
        (face_mesh, face_material, face_transform)
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
    pub face_up: bool,
    pub playable: bool,
}

impl Card {
    pub fn spawn(self, node: &mut RelatedSpawnerCommands<'_, ChildOf>) {
        node.spawn(self);
    }

    pub fn face_resource_name(&self) -> String {
        let suit_str = match self.suit {
            Suit::Clubs => "Clubs",
            Suit::Diamonds => "Diamonds",
            Suit::Hearts => "Hearts",
            Suit::Spades => "Spades",
        };
        format!(
            "images/{suit_str}/{}{}.png",
            self.rank.as_u8(),
            match self.suit {
                Suit::Clubs => "c",
                Suit::Diamonds => "d",
                Suit::Hearts => "h",
                Suit::Spades => "s",
            }
        )
    }

    pub fn back_resource_name() -> String {
        "images/Back Blue.png".to_string()
    }

    pub fn resource_name(&self) -> String {
        if self.face_up {
            self.face_resource_name()
        } else {
            Self::back_resource_name()
        }
    }

    pub fn can_stack(&self, other: &Card) -> bool {
        // Can stack if the other card is one rank lower and of opposite color
        self.rank.as_u8() + 1 == other.rank.as_u8() && self.suit.is_red() != other.suit.is_red()
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.rank, self.suit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    pub fn as_u8(&self) -> u8 {
        match self {
            Rank::Ace => 1,
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
        }
    }

    pub fn list() -> [Rank; 13] {
        [
            Rank::Ace,
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
        ]
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Rank::Ace => "A",
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub fn is_red(&self) -> bool {
        matches!(self, Suit::Diamonds | Suit::Hearts)
    }

    // pub fn is_black(&self) -> bool {
    //     matches!(self, Suit::Clubs | Suit::Spades)
    // }

    pub fn as_u8(&self) -> u8 {
        match self {
            Suit::Clubs => 0,
            Suit::Diamonds => 1,
            Suit::Hearts => 2,
            Suit::Spades => 3,
        }
    }

    pub fn list() -> [Suit; 4] {
        [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades]
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Suit::Clubs => "clubs",
            Suit::Diamonds => "diamonds",
            Suit::Hearts => "hearts",
            Suit::Spades => "spades",
        };
        write!(f, "{}", s)
    }
}

pub fn shuffle_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    for &suit in Suit::list().iter() {
        for &rank in Rank::list().iter() {
            deck.push(Card {
                rank,
                suit,
                face_up: false,
                playable: false,
            });
        }
    }
    let mut rand_ng = rng();
    deck.shuffle(&mut rand_ng);
    deck
}

#[derive(Debug, AsBindGroup, Clone, Asset, TypePath)]
pub struct CardMaterial {
    #[texture(0)]
    pub texture: Handle<Image>,
    #[uniform(1)]
    pub color: LinearRgba,
    #[texture(2)]
    pub face_texture: Handle<Image>,
}

impl Material for CardMaterial {}
