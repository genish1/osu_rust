use bevy::prelude::*;
use bevy::window::WindowResolution;
mod beatmap;
mod hitobject;

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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Sans caméra 2D, Bevy n'affiche rien du tout
    commands.spawn(Camera2d);
}   