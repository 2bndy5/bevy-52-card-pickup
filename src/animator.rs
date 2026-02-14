use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    animation::{AnimationEvent, AnimationTargetId, animated_field},
    prelude::*,
};

use crate::{
    cards::{BOARD_HALF_SIZE, CARD_HALF_SIZE, CARD_THICKNESS, Card},
    state::{CardsCollected, GameState},
};

#[derive(Debug, AnimationEvent, Clone, Copy)]
pub struct CollectingCard {
    pub card: Card,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct AnimatorNodeId(pub AnimationNodeIndex);

/// Holds information about the animation we programmatically create.
pub struct AnimationInfo {
    /// The name of the animation target (in this case, the text).
    pub target_name: Name,
    /// The ID of the animation target, derived from the name.
    pub target_id: AnimationTargetId,
    /// The animation graph asset.
    pub graph: Handle<AnimationGraph>,
    /// The index of the node within that graph.
    pub node_index: AnimationNodeIndex,
}

impl AnimationInfo {
    pub const ANIMATION_DURATION: f32 = 1.0;

    /// Programmatically creates the UI animation.
    pub fn create(
        transform: &Transform,
        card: &Card,
        animation_graphs: &mut Assets<AnimationGraph>,
        animation_clips: &mut Assets<AnimationClip>,
    ) -> AnimationInfo {
        // Create an ID that identifies the text node we're going to animate.
        let animation_target_name = Name::new(format!("Card-{card}"));
        let animation_target_id = AnimationTargetId::from_name(&animation_target_name);

        // Allocate an animation clip.
        let mut animation_clip = AnimationClip::default();
        animation_clip.add_event(Self::ANIMATION_DURATION, CollectingCard { card: *card });

        let animation_domain = interval(0.0, Self::ANIMATION_DURATION).unwrap();

        let start = transform.translation;
        let end = transform.translation.with_y(52.0);
        // The easing curve is parametrized over [0, 1], so we reparametrize it
        let translation_curve = EasingCurve::new(start, end, EaseFunction::SmoothStepOut)
            .reparametrize_linear(animation_domain)
            .expect("this curve has bounded domain, so this should never fail");

        let rotation_curve = EasingCurve::new(
            transform.rotation,
            Quat::from_axis_angle(Vec3::X, PI + FRAC_PI_2),
            EaseFunction::SmoothStepOut,
        )
        .reparametrize_linear(interval(0.5, Self::ANIMATION_DURATION).unwrap())
        .expect("this curve has bounded domain, so this should never fail");

        animation_clip.add_curve_to_target(
            animation_target_id,
            AnimatableCurve::new(animated_field!(Transform::translation), translation_curve),
        );
        animation_clip.add_curve_to_target(
            animation_target_id,
            AnimatableCurve::new(animated_field!(Transform::rotation), rotation_curve),
        );

        // Save our animation clip as an asset.
        let animation_clip_handle = animation_clips.add(animation_clip);

        // Create an animation graph with that clip.
        let (animation_graph, animation_node_index) =
            AnimationGraph::from_clip(animation_clip_handle);
        let animation_graph_handle = animation_graphs.add(animation_graph);

        AnimationInfo {
            target_name: animation_target_name,
            target_id: animation_target_id,
            graph: animation_graph_handle,
            node_index: animation_node_index,
        }
    }
}

/// System that runs when a card's flip animation is finished.
///
/// This function will replace the card's animation with a new one that stacks the picked card on the pile of collected cards.
/// It also increments the [`CardsCollected`] resource, which is used to determine the position (Y axis) of the pile.
/// Once all cards are collected, this function will trigger the [`GameState::Win`] state.
pub fn collect_card(
    event: On<CollectingCard>,
    query: Query<(
        &Card,
        &mut Transform,
        &mut AnimationPlayer,
        &AnimationTargetId,
        &AnimatorNodeId,
        &mut AnimationGraphHandle,
    )>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut cards_collected: ResMut<CardsCollected>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (
        card,
        transform,
        mut animation_player,
        animation_target_id,
        animation_node_id,
        mut animation_graph_handle,
    ) in query
    {
        if card.rank == event.card.rank
            && card.suit == event.card.suit
            && !card.playable
            && card.face_up
        {
            cards_collected.0 += 1;
            info!("Collecting Card {}", card);

            let mut animation_clip = AnimationClip::default();
            let collection_curve = EasingCurve::new(
                transform.translation,
                Transform::from_xyz(
                    BOARD_HALF_SIZE.x + CARD_HALF_SIZE.x,
                    cards_collected.0 as f32 * CARD_THICKNESS,
                    BOARD_HALF_SIZE.y - CARD_HALF_SIZE.y,
                )
                .translation,
                EaseFunction::SmootherStepOut,
            )
            .reparametrize_linear(interval(0.0, AnimationInfo::ANIMATION_DURATION).unwrap())
            .expect("this curve has bounded domain, so this should never fail");
            animation_clip.add_curve_to_target(
                *animation_target_id,
                AnimatableCurve::new(animated_field!(Transform::translation), collection_curve),
            );
            if cards_collected.0 >= 52 {
                info!("All cards collected!");
                animation_clip.add_event_fn(
                    AnimationInfo::ANIMATION_DURATION + 0.1,
                    |commands, _entity, _time, _weight| {
                        commands.set_state(GameState::Win);
                    },
                );
                cards_collected.0 = 0;
            }
            let animation_clip_handle = animation_clips.add(animation_clip);
            let (animation_graph, new_node_index) =
                AnimationGraph::from_clip(animation_clip_handle);
            let new_graph_handle = animation_graphs.add(animation_graph);
            let old_handle = animation_graph_handle.0.clone();
            animation_graph_handle.0 = new_graph_handle;
            animation_player.stop(animation_node_id.0);
            animation_graphs.remove(old_handle.id());
            animation_player.play(new_node_index);
        }
    }
}

/// System that runs when a card is pressed.
pub fn pressed_card(
    entity_event: On<Pointer<Press>>,
    mut query: Query<(&mut Card, &AnimatorNodeId, &mut AnimationPlayer)>,
) {
    let entity = entity_event.event_target();
    if let Ok((mut card, animation_node_index, mut animation_player)) = query.get_mut(entity)
        && card.playable
        && !card.face_up
    {
        card.playable = false;
        card.face_up = true;
        info!("Picking up Card {}", card.as_ref());
        animation_player.play(animation_node_index.0);
        card.set_changed();
    }
}
