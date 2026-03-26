use bevy::prelude::*;
use crate::audio::MusicTimer;
use crate::game::GameState;
use crate::renderer::{ApproachRing, HitCircle};

pub fn handle_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    mut game_state: ResMut<GameState>,
    timer: Res<MusicTimer>,
    circles: Query<(Entity, &HitCircle, &Transform)>,
    rings: Query<(Entity, &ApproachRing)>,
) {
    // Clic souris OU touches X/C du clavier
    let clicked = mouse.just_pressed(MouseButton::Left)
        || keyboard.just_pressed(KeyCode::KeyX)
        || keyboard.just_pressed(KeyCode::KeyC);

    if !clicked {
        return;
    }
    println!("Clic détecté !");
    
    let elapsed_ms = (timer.0 * 1000.0) as u64;
    println!("Elapsed : {}ms", elapsed_ms);
   // Nouveau
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let cursor = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Convertir position écran → coordonnées monde Bevy
    let width = window.width();
    let height = window.height();
    let world_x = cursor.x - width / 2.0;
    let world_y = height / 2.0 - cursor.y;
    let cursor_world = Vec2::new(world_x, world_y);

    let elapsed_ms = (timer.0 * 1000.0) as u64;

    for (entity, circle, transform) in circles.iter() {
        let circle_pos = Vec2::new(
            transform.translation.x,
            transform.translation.y,
        );

        let distance = (cursor_world - circle_pos).length();
        if distance < 30.0 {
            let delta_ms = if elapsed_ms > circle.time_ms {
                elapsed_ms - circle.time_ms
            } else {
                circle.time_ms - elapsed_ms
            };

            let result = GameState::evaluate_timing(delta_ms);
            println!("Hit ! delta={}ms → {:?}", delta_ms, result);
            game_state.register_hit(result);
            println!("Score : {} | Combo : {}", game_state.score, game_state.combo);

            // Supprimer le cercle et son approach ring
            commands.entity(entity).despawn();

            for (ring_entity, ring) in rings.iter() {
                if ring.time_ms == circle.time_ms {
                    commands.entity(ring_entity).despawn();
                }
            }

            break;
        }
    }
}