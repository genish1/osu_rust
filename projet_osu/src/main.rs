use bevy::prelude::*;
use bevy::window::WindowResolution;
use audio::MusicTimer;
use beatmap::Beatmap;

mod beatmap;
mod hitobject;
mod renderer;
mod audio;
mod input;
mod game;

use game::GameState;

#[derive(Resource)]
pub struct CurrentBeatmap(pub Beatmap);

const LEVEL1: &str = include_str!("/home/genishi/rust/osu_rust/projet_osu/assets/maps/level1.osumap");
const LEVEL2: &str = include_str!("/home/genishi/rust/osu_rust/projet_osu/assets/maps/level2.osumap");

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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "osu-simple".into(),
                resolution: WindowResolution::new(1920, 1080),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(CurrentBeatmap(level1))
        .insert_resource(MusicTimer(0.0))
        .insert_resource(GameState::new())
        // Startup
        .add_systems(Startup, renderer::setup)       // caméra + cache curseur + charge textures
        .add_systems(Startup, renderer::spawn_circles.after(renderer::setup))
        .add_systems(Startup, audio::start_music)
        // Update
        .add_systems(Update, renderer::update_circles)
        .add_systems(Update, renderer::update_cursor) // ← nouveau
        .add_systems(Update, input::handle_click)
        .add_systems(Update, input::handle_slider_tick)
        .add_systems(Update, audio::update_timer)
        .run();
}