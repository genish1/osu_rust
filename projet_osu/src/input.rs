use bevy::prelude::*;
use crate::audio::MusicTimer;
use crate::game::{GameState, HitResult};
use crate::renderer::{ApproachRing, HitCircle, HitResultSprite, GameTextures, SliderStartCircle, SliderEndCircle, SliderBody};

/// État d'un slider : le joueur a-t-il cliqué au bon moment sur le début ?
#[derive(Component)]
pub struct SliderHitState {
    pub time_ms: u64,
    pub end_time_ms: u64,
    /// true si le joueur a cliqué dans la fenêtre de tolérance
    pub started: bool,
    /// true si ce slider a déjà été évalué (évite le double score)
    pub scored: bool,
}

/// Retourne le handle d'image correspondant au résultat.
fn result_image(textures: &GameTextures, result: &HitResult) -> Handle<Image> {
    match result {
        HitResult::Hit300 => textures.hit_results[3].clone(),
        HitResult::Hit100 => textures.hit_results[2].clone(),
        HitResult::Hit50  => textures.hit_results[1].clone(),
        HitResult::Miss   => textures.hit_results[0].clone(),
    }
}

/// Spawne l'image de résultat à la position du cercle touché.
fn spawn_hit_result(commands: &mut Commands, textures: &GameTextures, result: &HitResult, pos: Vec2) {
    commands.spawn((
        Sprite {
            image: result_image(textures, result),
            custom_size: Some(Vec2::splat(70.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 10.0),
        HitResultSprite { timer: 0.8 },
    ));
}

/// Système 1 — détecte le clic sur un HitCircle ou le début d'un slider.
pub fn handle_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut game_state: ResMut<GameState>,
    timer: Res<MusicTimer>,
    textures: Res<GameTextures>,
    circles: Query<(Entity, &HitCircle, &Transform)>,
    rings: Query<(Entity, &ApproachRing)>,
    slider_starts: Query<(Entity, &SliderStartCircle, &Transform)>,
) {
    let clicked = mouse.just_pressed(MouseButton::Left)
        || keyboard.just_pressed(KeyCode::KeyX)
        || keyboard.just_pressed(KeyCode::KeyC);

    if !clicked {
        return;
    }

    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let cursor = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    let width = window.width();
    let height = window.height();
    let cursor_world = Vec2::new(
        cursor.x - width / 2.0,
        height / 2.0 - cursor.y,
    );

    let elapsed_ms = (timer.0 * 1000.0) as u64;

    // --- HitCircles normaux ---
    for (entity, circle, transform) in circles.iter() {
        // Ignorer les cercles pas encore apparus
        if elapsed_ms < circle.time_ms.saturating_sub(1500) { continue; }
        let pos = transform.translation.truncate();
        if (cursor_world - pos).length() < 50.0 {
            let delta_ms = elapsed_ms.abs_diff(circle.time_ms);
            let result = GameState::evaluate_timing(delta_ms);
            println!("Circle hit ! delta={}ms → {:?}", delta_ms, result);
            spawn_hit_result(&mut commands, &textures, &result, pos);
            game_state.register_hit(result);
            println!("Score : {} | Combo : {}", game_state.score, game_state.combo);

            commands.entity(entity).despawn();
            for (ring_entity, ring) in rings.iter() {
                if ring.time_ms == circle.time_ms {
                    commands.entity(ring_entity).despawn();
                }
            }
            return;
        }
    }

    // --- Cercle de début de slider ---
    for (entity, start, transform) in slider_starts.iter() {
        // Ignorer les sliders pas encore apparus
        if elapsed_ms < start.time_ms.saturating_sub(1500) { continue; }
        let pos = transform.translation.truncate();
        if (cursor_world - pos).length() < 50.0 {
            let delta_ms = elapsed_ms.abs_diff(start.time_ms);
            if delta_ms <= 450 {
                println!("Slider démarré ! delta={}ms", delta_ms);
                commands.entity(entity).insert(SliderHitState {
                    time_ms: start.time_ms,
                    end_time_ms: start.end_time_ms,
                    started: true,
                    scored: false,
                });
            } else {
                println!("Slider manqué (hors fenêtre) → Miss");
                spawn_hit_result(&mut commands, &textures, &HitResult::Miss, pos);
                game_state.register_hit(HitResult::Miss);
            }
            return;
        }
    }
}

/// Système 2 — évalue les sliders à la fin de leur durée.
/// Critère unique : le bouton est-il encore maintenu à end_time_ms ?
pub fn handle_slider_tick(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    timer: Res<MusicTimer>,
    mut game_state: ResMut<GameState>,
    textures: Res<GameTextures>,
    mut slider_states: Query<(Entity, &mut SliderHitState, &SliderStartCircle, &Transform)>,
    slider_ends: Query<(Entity, &SliderEndCircle)>,
    slider_bodies: Query<(Entity, &SliderBody)>,
    rings: Query<(Entity, &ApproachRing)>,
) {
    let elapsed_ms = (timer.0 * 1000.0) as u64;

    let held = mouse.pressed(MouseButton::Left)
        || keyboard.pressed(KeyCode::KeyX)
        || keyboard.pressed(KeyCode::KeyC);

    for (entity, mut state, start, transform) in slider_states.iter_mut() {
        if state.scored {
            continue;
        }

        // On évalue uniquement quand on atteint la fin du slider
        if elapsed_ms < state.end_time_ms {
            continue;
        }

        let pos = transform.translation.truncate();
        if held {
            println!("Slider réussi → Hit300");
            spawn_hit_result(&mut commands, &textures, &HitResult::Hit300, pos);
            game_state.register_hit(HitResult::Hit300);
        } else {
            println!("Slider raté (non maintenu) → Miss");
            spawn_hit_result(&mut commands, &textures, &HitResult::Miss, pos);
            game_state.register_hit(HitResult::Miss);
        }
        state.scored = true;
        println!("Score : {} | Combo : {}", game_state.score, game_state.combo);

        // Nettoyage de toutes les entités liées à ce slider
        commands.entity(entity).despawn();

        for (end_entity, end) in slider_ends.iter() {
            if end.time_ms == start.time_ms {
                commands.entity(end_entity).despawn();
            }
        }
        for (body_entity, body) in slider_bodies.iter() {
            if body.time_ms == start.time_ms {
                commands.entity(body_entity).despawn();
            }
        }
        for (ring_entity, ring) in rings.iter() {
            if ring.time_ms == start.time_ms {
                commands.entity(ring_entity).despawn();
            }
        }
    }
}