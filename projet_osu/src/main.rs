use bevy::prelude::*;
use bevy::window::{WindowMode, PresentMode, MonitorSelection};

mod beatmap;
mod hitobject;
mod renderer;
mod audio;
mod input;
mod game;
mod menu;

use audio::MusicTimer;
use beatmap::Beatmap;
use game::GameState;
use menu::{AppState, PauseExit, SelectedMap};

/// Ressource qui contient la beatmap en cours de jeu.
#[derive(Resource)]
pub struct CurrentBeatmap(pub Beatmap);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "osu-simple".into(),
                // Plein écran borderless : élimine la couche compositeur
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                // Sans VSync : réduit la latence d'entrée
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        // ── État initial + ressources permanentes ────────────────────────────
        .init_state::<AppState>()
        .insert_resource(SelectedMap::default())
        .insert_resource(PauseExit::default())
        .insert_resource(MusicTimer(0.0))
        .insert_resource(GameState::new())
        // ── Startup : caméra + textures + curseur custom (une seule fois) ──
        .add_systems(Startup, renderer::setup)
        // ── Curseur custom actif dans tous les états ─────────────────────────
        .add_systems(Update, renderer::update_cursor)
        // ── Menu principal ────────────────────────────────────────────────────
        .add_systems(OnEnter(AppState::MainMenu),  menu::setup_main_menu)
        .add_systems(Update, menu::update_main_menu.run_if(in_state(AppState::MainMenu)))
        .add_systems(OnExit(AppState::MainMenu),   menu::cleanup_main_menu)
        // ── Sélection de map ──────────────────────────────────────────────────
        .add_systems(OnEnter(AppState::MapSelect), menu::setup_map_select)
        .add_systems(Update, menu::update_map_select.run_if(in_state(AppState::MapSelect)))
        .add_systems(OnExit(AppState::MapSelect),  menu::cleanup_map_select)
        // ── Décompte ──────────────────────────────────────────────────────────
        .add_systems(OnEnter(AppState::Countdown), (
            menu::load_selected_beatmap,
            menu::setup_countdown,
        ))
        .add_systems(Update, menu::update_countdown.run_if(in_state(AppState::Countdown)))
        .add_systems(OnExit(AppState::Countdown),  menu::cleanup_countdown)
        // ── Jeu ───────────────────────────────────────────────────────────────
        .add_systems(OnEnter(AppState::Playing), (
            renderer::setup_game,
            renderer::spawn_circles.after(renderer::setup_game),
            audio::start_music,
        ))
        .add_systems(Update, (
            renderer::update_circles,
            renderer::update_hit_results,
            renderer::update_score,
            renderer::check_map_end,
            input::handle_click,
            input::handle_slider_tick,
            audio::update_timer,
            menu::handle_pause_input,
        ).run_if(in_state(AppState::Playing)))
        // ── Pause ─────────────────────────────────────────────────────────────
        .add_systems(OnEnter(AppState::Paused), (
            menu::setup_pause_menu,
            audio::pause_music,
        ))
        .add_systems(Update, menu::update_pause_menu.run_if(in_state(AppState::Paused)))
        .add_systems(OnExit(AppState::Paused), menu::cleanup_pause_menu)
        // ── Écran de résultats ────────────────────────────────────────────────
        .add_systems(OnEnter(AppState::ResultScreen), menu::setup_result_screen)
        .add_systems(Update, menu::update_result_screen.run_if(in_state(AppState::ResultScreen)))
        .add_systems(OnExit(AppState::ResultScreen),  menu::cleanup_result_screen)
        .run();
}
