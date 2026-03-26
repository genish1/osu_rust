use bevy::prelude::*;
use crate::beatmap::Beatmap;
use crate::hitobject::HitObject;

/// Composant attaché à chaque cercle affiché à l'écran.
#[derive(Component)]
pub struct HitCircle {
    pub time_ms: u64,
}

/// Composant pour l'approach ring (le cercle qui rétrécit).
#[derive(Component)]
pub struct ApproachRing {
    pub time_ms: u64,
}

/// Système de démarrage : spawn la caméra.
pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawn tous les cercles de la beatmap au démarrage.
pub fn spawn_circles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    beatmap: Res<crate::CurrentBeatmap>,
    windows: Query<&Window>,
) {
    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let width = window.width();
    let height = window.height();

    for obj in &beatmap.0.hit_objects {
        if let HitObject::Circle(circle) = obj {
            // Conversion dynamique selon la taille de la fenêtre
            let x = circle.x - width / 2.0;
            let y = height / 2.0 - circle.y;

            // Le cercle principal
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(35.0))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
                Transform::from_xyz(x, y, 0.0),
                HitCircle { time_ms: circle.time_ms },
            ));

            // L'approach ring
            // Nouveau — juste le contour
            commands.spawn((
                Mesh2d(meshes.add(Annulus::new(33.0, 35.0))), // rayon intérieur, rayon extérieur
                MeshMaterial2d(materials.add(Color::srgb(0.0, 0.8, 1.0))),
                Transform::from_xyz(x, y, 1.0),
                ApproachRing { time_ms: circle.time_ms },
            ));
        }
    }
}

/// Chaque frame : anime les approach rings et supprime les cercles expirés.
pub fn update_circles(
    mut commands: Commands,
    timer: Res<crate::audio::MusicTimer>,
    mut rings: Query<(Entity, &ApproachRing, &mut Transform)>,
    circles: Query<(Entity, &HitCircle)>,
) {
    let elapsed_ms = (timer.0 * 1000.0) as u64;

    for (entity, ring, mut transform) in rings.iter_mut() {
        let appear_at = ring.time_ms.saturating_sub(800);

        if elapsed_ms < appear_at {
            transform.scale = Vec3::splat(0.0);
            continue;
        }

        let progress = (elapsed_ms - appear_at) as f32 / 800.0;
        let scale = 1.0 + (1.0 - progress) * 2.0;

        if scale <= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.scale = Vec3::splat(scale);
    }

    for (entity, circle) in circles.iter() {
        if elapsed_ms > circle.time_ms + 1000 {
            commands.entity(entity).despawn();
        }
    }
}