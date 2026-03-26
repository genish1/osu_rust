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
) {
    for obj in &beatmap.0.hit_objects {
        // On ne gère que les cercles pour l'instant
        if let HitObject::Circle(circle) = obj {
            // Convertir les coordonnées osu (origine haut-gauche)
            // vers Bevy (origine centre écran)
            let x = circle.x - 400.0;
            let y = -(circle.y - 300.0);

            // Le cercle principal
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(30.0))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
                Transform::from_xyz(x, y, 0.0),
                HitCircle { time_ms: circle.time_ms },
            ));

            // L'approach ring (commence grand et rétrécit)
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(30.0))),
                MeshMaterial2d(materials.add(Color::srgba(0.0, 0.8, 1.0, 0.6))),
                Transform::from_xyz(x, y, 1.0),
                ApproachRing { time_ms: circle.time_ms },
            ));
        }
    }
}

/// Chaque frame : anime les approach rings et supprime les cercles expirés.
pub fn update_circles(
    mut commands: Commands,
    time: Res<Time>,
    mut rings: Query<(Entity, &ApproachRing, &mut Transform)>,
    circles: Query<(Entity, &HitCircle)>,
) {
    let elapsed_ms = (time.elapsed_secs() * 1000.0) as u64;

    // Approach ring : apparaît 800ms avant, rétrécit jusqu'au timing
    for (entity, ring, mut transform) in rings.iter_mut() {
        let appear_at = ring.time_ms.saturating_sub(800);

        if elapsed_ms < appear_at {
            // Pas encore visible
            transform.scale = Vec3::splat(0.0);
            continue;
        }

        // Calcul du ratio : 1.0 (grand) → 0.0 (taille normale)
        let progress = (elapsed_ms - appear_at) as f32 / 800.0;
        // Le ring commence à 3x la taille et rétrécit vers 1x
        // Nouveau — le ring s'arrête exactement au bord du cercle
        let scale = 1.0 + (1.0 - progress) * 2.0;
        // Dès que le ring atteint la taille du cercle, on le supprime
        if scale <= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.scale = Vec3::splat(scale);
    }

    // Cercles principaux : disparaissent après leur timing
    for (entity, circle) in circles.iter() {
        if elapsed_ms > circle.time_ms + 150 {
            commands.entity(entity).despawn();
        }
    }
}