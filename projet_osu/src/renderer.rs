use bevy::prelude::*;
use crate::hitobject::HitObject;

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

// ─── Ressource : handles des textures chargées une seule fois ──────────────────

#[derive(Resource)]
pub struct GameTextures {
    pub hitcircle: Handle<Image>,
    pub approachcircle: Handle<Image>,
    pub cursor: Handle<Image>,
    pub numbers: [Handle<Image>; 10],
}

// ─── Setup ─────────────────────────────────────────────────────────────────────

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut windows: Query<&mut Window>,
) {
    // Caméra 2D
    commands.spawn(Camera2d);

    // Cacher le curseur système
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.visible = false;
    }

    // Charger toutes les textures
    let textures = GameTextures {
        hitcircle:      asset_server.load("hitcircle.png"),
        approachcircle: asset_server.load("approachcircle.png"),
        cursor:         asset_server.load("cursor.png"),
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
    };

    // Spawn du curseur custom (position initiale hors écran)
    commands.spawn((
        Sprite {
            image: textures.cursor.clone(),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 100.0), // z élevé = toujours devant
        CustomCursor,
    ));

    commands.insert_resource(textures);
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

                // Cercle principal (hitcircle.png)
                let circle_entity = commands.spawn((
                    Sprite {
                        image: textures.hitcircle.clone(),
                        custom_size: Some(Vec2::splat(70.0)), // diamètre = 70px
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, 0.0),
                    HitCircle { time_ms: circle.time_ms },
                )).id();

                // Chiffre par-dessus le cercle (enfant → suit automatiquement)
                let number_entity = commands.spawn((
                    Sprite {
                        image: textures.numbers[digit].clone(),
                        custom_size: Some(Vec2::splat(30.0)),
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
                        custom_size: Some(Vec2::splat(70.0)),
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
                let radius = 35.0_f32;
                let half_length = (length / 2.0).max(radius);
                let digit = counter % 10;
                counter += 1;

                // Corps du slider — on garde un mesh gris (pas de sprite dédié)
                // On l'insère via SpriteBundle étiré : on utilise hitcircle teinté
                // Pour la capsule on garde un Mesh2d car pas d'image dédiée
                commands.spawn((
                    Sprite {
                        image: textures.hitcircle.clone(),
                        color: Color::srgba(0.4, 0.4, 0.4, 0.7),
                        // On étire le sprite pour simuler la capsule
                        custom_size: Some(Vec2::new(70.0, half_length * 2.0 + 70.0)),
                        ..default()
                    },
                    Transform {
                        translation: Vec3::new(mid.x, mid.y, 0.0),
                        rotation: Quat::from_rotation_z(angle),
                        ..default()
                    },
                    SliderBody { time_ms: slider.time_ms },
                ));

                // Cercle de fin
                commands.spawn((
                    Sprite {
                        image: textures.hitcircle.clone(),
                        color: Color::srgb(1.0, 0.5, 0.0), // teinte orange
                        custom_size: Some(Vec2::splat(70.0)),
                        ..default()
                    },
                    Transform::from_xyz(p2.x, p2.y, 1.0),
                    SliderEndCircle { time_ms: slider.time_ms },
                ));

                // Cercle de début
                let start_entity = commands.spawn((
                    Sprite {
                        image: textures.hitcircle.clone(),
                        custom_size: Some(Vec2::splat(70.0)),
                        ..default()
                    },
                    Transform::from_xyz(p1.x, p1.y, 2.0),
                    SliderStartCircle {
                        time_ms: slider.time_ms,
                        end_time_ms: slider.end_time_ms,
                    },
                )).id();

                // Chiffre sur le cercle de début
                let number_entity = commands.spawn((
                    Sprite {
                        image: textures.numbers[digit].clone(),
                        custom_size: Some(Vec2::splat(30.0)),
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
                        custom_size: Some(Vec2::splat(70.0)),
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

/// Anime les approach rings et supprime les objets expirés.
pub fn update_circles(
    mut commands: Commands,
    timer: Res<crate::audio::MusicTimer>,
    mut rings: Query<(Entity, &ApproachRing, &mut Transform)>,
    circles: Query<(Entity, &HitCircle)>,
    slider_starts: Query<(Entity, &SliderStartCircle)>,
    slider_ends: Query<(Entity, &SliderEndCircle)>,
    slider_bodies: Query<(Entity, &SliderBody)>,
) {
    let elapsed_ms = (timer.0 * 1000.0) as u64;

    // Approach rings : rétrécissement + suppression
    for (entity, ring, mut transform) in rings.iter_mut() {
        let appear_at = ring.time_ms.saturating_sub(800);

        if elapsed_ms < appear_at {
            transform.scale = Vec3::splat(0.0);
            continue;
        }

        let progress = (elapsed_ms - appear_at) as f32 / 800.0;
        // Démarre à 3× la taille, finit à 1×
        let scale = 1.0 + (1.0 - progress) * 2.0;

        if scale <= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.scale = Vec3::splat(scale);
    }

    // HitCircles expirés (despawn_recursive pour supprimer les enfants = chiffres)
    for (entity, circle) in circles.iter() {
        if elapsed_ms > circle.time_ms + 1000 {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Sliders expirés
    for (entity, start) in slider_starts.iter() {
        if elapsed_ms > start.end_time_ms + 1000 {
            commands.entity(entity).despawn_recursive();
        }
    }
    for (entity, end) in slider_ends.iter() {
        if elapsed_ms > end.time_ms + 5000 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, body) in slider_bodies.iter() {
        if elapsed_ms > body.time_ms + 5000 {
            commands.entity(entity).despawn();
        }
    }
}

/// Déplace le curseur custom à la position de la souris chaque frame.
pub fn update_cursor(
    windows: Query<&Window>,
    mut cursor_query: Query<&mut Transform, With<CustomCursor>>,
) {
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let cursor_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    // Conversion position écran (0,0 = coin haut-gauche) → monde Bevy (0,0 = centre)
    let width = window.width();
    let height = window.height();
    let world_x = cursor_pos.x - width / 2.0;
    let world_y = height / 2.0 - cursor_pos.y;

    if let Ok(mut transform) = cursor_query.single_mut() {
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }
}