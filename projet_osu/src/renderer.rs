use bevy::prelude::*;
use crate::hitobject::HitObject;
use crate::game::GameState;

// ─── Composants ────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct HitCircle {
    pub time_ms: u64,
}

#[derive(Component)]
pub struct ApproachRing {
    pub time_ms: u64,
}

#[derive(Component)]
pub struct SliderStartCircle {
    pub time_ms: u64,
    pub end_time_ms: u64,
}

#[derive(Component)]
pub struct SliderEndCircle {
    pub time_ms: u64,
}

#[derive(Component)]
pub struct SliderBody {
    pub time_ms: u64,
}

/// Curseur custom — suit la souris chaque frame.
#[derive(Component)]
pub struct CustomCursor;

/// Chiffre affiché sur un HitCircle (enfant de l'entité cercle).
#[derive(Component)]
pub struct NumberSprite;

/// Image de résultat (300/100/50/Miss) qui s'affiche puis disparaît.
#[derive(Component)]
pub struct HitResultSprite {
    /// Secondes restantes avant suppression.
    pub timer: f32,
}

/// Marker pour le nœud UI affichant le score.
#[derive(Component)]
pub struct ScoreText;

// ─── Ressource : handles des textures chargées une seule fois ──────────────────

#[derive(Resource)]
pub struct GameTextures {
    pub hitcircle: Handle<Image>,
    pub approachcircle: Handle<Image>,
    pub cursor: Handle<Image>,
    pub numbers: [Handle<Image>; 10],
    /// [0] = miss, [1] = 50, [2] = 100, [3] = 300
    pub hit_results: [Handle<Image>; 4],
}

// ─── Setup ─────────────────────────────────────────────────────────────────────

/// Startup : caméra 2D, masquage curseur système, curseur custom, textures.
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut cursor_options: Query<&mut bevy::window::CursorOptions>,
) {
    commands.spawn(Camera2d);

    // Mutation directe du composant (pas de commande différée) pour que
    // le système changed_cursor_options de bevy_winit détecte le changement
    // dans le même frame et appelle set_cursor_visible(false) sur le vrai OS.
    if let Ok(mut c) = cursor_options.single_mut() {
        c.visible = false;
    }

    let cursor_handle: Handle<Image> = asset_server.load("cursor.png");

    // Curseur custom en tant qu'élément UI (toujours au-dessus des nodes UI plein-écran)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(48.0),
            height: Val::Px(48.0),
            ..default()
        },
        ImageNode::new(cursor_handle.clone()),
        GlobalZIndex(200),
        CustomCursor,
    ));

    commands.insert_resource(GameTextures {
        cursor: cursor_handle,
        hitcircle:      asset_server.load("hitcircle.png"),
        approachcircle: asset_server.load("approachcircle.png"),
        numbers: [
            asset_server.load("default-0.png"),
            asset_server.load("default-1.png"),
            asset_server.load("default-2.png"),
            asset_server.load("default-3.png"),
            asset_server.load("default-4.png"),
            asset_server.load("default-5.png"),
            asset_server.load("default-6.png"),
            asset_server.load("default-7.png"),
            asset_server.load("default-8.png"),
            asset_server.load("default-9.png"),
        ],
        hit_results: [
            asset_server.load("hit0.png"),
            asset_server.load("hit50.png"),
            asset_server.load("hit100.png"),
            asset_server.load("hit300.png"),
        ],
    });
}

/// OnEnter(Playing) : spawne l'UI score.
pub fn setup_game(
    mut commands: Commands,
) {
    commands.spawn((
        Text::new("0"),
        TextFont { font_size: 48.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        ScoreText,
    ));
}

// ─── Spawn des objets ──────────────────────────────────────────────────────────

/// Convertit coordonnées .osumap → monde Bevy
fn to_world(x: f32, y: f32, width: f32, height: f32) -> Vec2 {
    Vec2::new(x - width / 2.0, height / 2.0 - y)
}

pub fn spawn_circles(
    mut commands: Commands,
    textures: Res<GameTextures>,
    beatmap: Res<crate::CurrentBeatmap>,
    windows: Query<&Window>,
) {
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let width = window.width();
    let height = window.height();

    // On numérote les cercles de 0 à 9 en boucle
    let mut counter: usize = 0;

    for obj in &beatmap.0.hit_objects {
        match obj {
            HitObject::Circle(circle) => {
                let pos = to_world(circle.x, circle.y, width, height);
                let digit = counter % 10;
                counter += 1;

                // Cercle principal (hitcircle.png) — caché jusqu'à l'apparition de l'approach ring
                let circle_entity = commands.spawn((
                    Sprite {
                        image: textures.hitcircle.clone(),
                        custom_size: Some(Vec2::splat(100.0)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, 0.0),
                    Visibility::Hidden,
                    HitCircle { time_ms: circle.time_ms },
                )).id();

                // Chiffre par-dessus le cercle (enfant → suit automatiquement)
                let number_entity = commands.spawn((
                    Sprite {
                        image: textures.numbers[digit].clone(),
                        custom_size: Some(Vec2::splat(42.0)),
                        ..default()
                    },
                    // z = 0.1 relatif au parent → devant le cercle
                    Transform::from_xyz(0.0, 0.0, 0.1),
                    NumberSprite,
                )).id();

                // On attache le chiffre comme enfant du cercle
                commands.entity(circle_entity).add_child(number_entity);

                // Approach ring (approachcircle.png) — entité séparée
                commands.spawn((
                    Sprite {
                        image: textures.approachcircle.clone(),
                        custom_size: Some(Vec2::splat(100.0)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, 1.0),
                    ApproachRing { time_ms: circle.time_ms },
                ));
            }

            HitObject::Slider(slider) => {
                let p1 = to_world(slider.x, slider.y, width, height);
                let p2 = to_world(slider.x_end, slider.y_end, width, height);

                let diff = p2 - p1;
                let length = diff.length();
                let angle = diff.y.atan2(diff.x) - std::f32::consts::FRAC_PI_2;
                let mid = (p1 + p2) / 2.0;
                let radius = 50.0_f32;
                let half_length = (length / 2.0).max(radius);
                let digit = counter % 10;
                counter += 1;

                // Corps du slider — caché jusqu'à l'apparition de l'approach ring
                commands.spawn((
                    Sprite {
                        image: textures.hitcircle.clone(),
                        color: Color::srgba(0.4, 0.4, 0.4, 0.7),
                        custom_size: Some(Vec2::new(100.0, half_length * 2.0 + 100.0)),
                        ..default()
                    },
                    Transform {
                        translation: Vec3::new(mid.x, mid.y, 0.0),
                        rotation: Quat::from_rotation_z(angle),
                        ..default()
                    },
                    Visibility::Hidden,
                    SliderBody { time_ms: slider.time_ms },
                ));

                // Cercle de fin — caché jusqu'à l'apparition de l'approach ring
                commands.spawn((
                    Sprite {
                        image: textures.hitcircle.clone(),
                        color: Color::srgb(1.0, 0.5, 0.0),
                        custom_size: Some(Vec2::splat(100.0)),
                        ..default()
                    },
                    Transform::from_xyz(p2.x, p2.y, 1.0),
                    Visibility::Hidden,
                    SliderEndCircle { time_ms: slider.time_ms },
                ));

                // Cercle de début — caché jusqu'à l'apparition de l'approach ring
                let start_entity = commands.spawn((
                    Sprite {
                        image: textures.hitcircle.clone(),
                        custom_size: Some(Vec2::splat(100.0)),
                        ..default()
                    },
                    Transform::from_xyz(p1.x, p1.y, 2.0),
                    Visibility::Hidden,
                    SliderStartCircle {
                        time_ms: slider.time_ms,
                        end_time_ms: slider.end_time_ms,
                    },
                )).id();

                // Chiffre sur le cercle de début
                let number_entity = commands.spawn((
                    Sprite {
                        image: textures.numbers[digit].clone(),
                        custom_size: Some(Vec2::splat(42.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.1),
                    NumberSprite,
                )).id();
                commands.entity(start_entity).add_child(number_entity);

                // Approach ring sur le cercle de début
                commands.spawn((
                    Sprite {
                        image: textures.approachcircle.clone(),
                        custom_size: Some(Vec2::splat(100.0)),
                        ..default()
                    },
                    Transform::from_xyz(p1.x, p1.y, 3.0),
                    ApproachRing { time_ms: slider.time_ms },
                ));
            }
        }
    }
}

// ─── Update ────────────────────────────────────────────────────────────────────

/// Anime les approach rings, gère la visibilité des objets et supprime les expirés.
pub fn update_circles(
    mut commands: Commands,
    timer: Res<crate::audio::MusicTimer>,
    mut rings:         Query<(Entity, &ApproachRing, &mut Transform)>,
    mut circles:       Query<(Entity, &HitCircle, &mut Visibility)>,
    mut slider_starts: Query<(Entity, &SliderStartCircle, &mut Visibility), Without<HitCircle>>,
    mut slider_ends:   Query<(Entity, &SliderEndCircle,   &mut Visibility), (Without<HitCircle>, Without<SliderStartCircle>)>,
    mut slider_bodies: Query<(Entity, &SliderBody,        &mut Visibility), (Without<HitCircle>, Without<SliderStartCircle>, Without<SliderEndCircle>)>,
) {
    let elapsed_ms = (timer.0 * 1000.0) as u64;

    // Approach rings : rétrécissement + suppression
    for (entity, ring, mut transform) in rings.iter_mut() {
        let appear_at = ring.time_ms.saturating_sub(1500);

        if elapsed_ms < appear_at {
            transform.scale = Vec3::splat(0.0);
            continue;
        }

        let progress = (elapsed_ms - appear_at) as f32 / 1500.0;
        let scale = 1.0 + (1.0 - progress) * 2.0;

        if scale <= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.scale = Vec3::splat(scale);
    }

    // HitCircles : apparition synchronisée + expiration
    for (entity, circle, mut vis) in circles.iter_mut() {
        if elapsed_ms >= circle.time_ms.saturating_sub(1500) && *vis == Visibility::Hidden {
            *vis = Visibility::Visible;
        }
        if elapsed_ms > circle.time_ms + 1000 {
            commands.entity(entity).despawn();
        }
    }

    // Sliders : apparition synchronisée + expiration
    for (entity, start, mut vis) in slider_starts.iter_mut() {
        if elapsed_ms >= start.time_ms.saturating_sub(1500) && *vis == Visibility::Hidden {
            *vis = Visibility::Visible;
        }
        if elapsed_ms > start.end_time_ms + 1000 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, end, mut vis) in slider_ends.iter_mut() {
        if elapsed_ms >= end.time_ms.saturating_sub(1500) && *vis == Visibility::Hidden {
            *vis = Visibility::Visible;
        }
        if elapsed_ms > end.time_ms + 5000 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, body, mut vis) in slider_bodies.iter_mut() {
        if elapsed_ms >= body.time_ms.saturating_sub(1500) && *vis == Visibility::Hidden {
            *vis = Visibility::Visible;
        }
        if elapsed_ms > body.time_ms + 5000 {
            commands.entity(entity).despawn();
        }
    }
}

/// Anime les sprites de résultat (fondu + suppression).
pub fn update_hit_results(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitResultSprite, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut hit, mut sprite) in query.iter_mut() {
        hit.timer -= dt;
        // Fondu sur toute la durée
        let alpha = (hit.timer / 0.8).clamp(0.0, 1.0);
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
        if hit.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Détecte la fin de la map et bascule vers l'écran de résultats.
pub fn check_map_end(
    mut next_state: ResMut<NextState<crate::menu::AppState>>,
    timer:      Res<crate::audio::MusicTimer>,
    game_state: Res<crate::game::GameState>,
    circles:        Query<(), With<HitCircle>>,
    slider_starts:  Query<(), With<SliderStartCircle>>,
) {
    if game_state.map_duration_ms == 0 { return; }
    let elapsed_ms = (timer.0 * 1000.0) as u64;
    // Attendre que le timer soit passé après la dernière note
    if elapsed_ms <= game_state.map_duration_ms + 500 { return; }
    if circles.is_empty() && slider_starts.is_empty() {
        next_state.set(crate::menu::AppState::ResultScreen);
    }
}

/// Met à jour l'affichage du score en haut à droite.
pub fn update_score(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if let Ok(mut text) = query.single_mut() {
        text.0 = game_state.score.to_string();
    }
}

/// Déplace le curseur custom (node UI) à la position de la souris chaque frame.
pub fn update_cursor(
    windows: Query<&Window>,
    mut cursor_query: Query<&mut Node, With<CustomCursor>>,
) {
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    if let Ok(mut node) = cursor_query.single_mut() {
        node.left = Val::Px(cursor_pos.x);
        node.top  = Val::Px(cursor_pos.y);
    }
}