use bevy::prelude::*;
use bevy::window::WindowResolution;
mod beatmap;
mod hitobject;
mod renderer;

use beatmap::Beatmap;

/// Ressource qui contient la beatmap en cours de jeu.
#[derive(Resource)]
pub struct CurrentBeatmap(pub Beatmap);

const LEVEL1: &str = include_str!("/home/genishi/rust/osu_rust/projet_osu/asset/maps/level1.osumap");
const LEVEL2: &str = include_str!("/home/genishi/rust/osu_rust/projet_osu/asset/maps/level2.osumap");

fn main() {
    match beatmap::Beatmap::parse(LEVEL1) {
        Ok(map)  => println!("✓ {} — {} objets", map.title, map.hit_objects.len()),
        Err(e)   => println!("Erreur level1 : {:?}", e),
    }
    match beatmap::Beatmap::parse(LEVEL2) {
        Ok(map)  => println!("✓ {} — {} objets", map.title, map.hit_objects.len()),
        Err(e)   => println!("Erreur level2 : {:?}", e),
    }
    let level1 = Beatmap::parse(LEVEL1).expect("Erreur level1.osumap");

    App::new()
        // DefaultPlugins inclut tout le nécessaire :
        // fenêtre, rendu, clavier, souris, audio...
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "osu-simple".into(),       // titre de la fenêtre
                resolution: WindowResolution::new(1920.0, 1080.0), // largeur x hauteur
                ..default()                        // reste par défaut
            }),
            ..default()
        }))
        .insert_resource(CurrentBeatmap(level1))
        .add_systems(Startup, renderer::setup)
        .add_systems(Startup, renderer::spawn_circles)
        .add_systems(Update, renderer::update_circles)
        .run();
}

fn setup(mut commands: Commands) {
    // Sans caméra 2D, Bevy n'affiche rien du tout
    commands.spawn(Camera2d);
}   