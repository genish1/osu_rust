use bevy::prelude::*;
use crate::CurrentBeatmap;

/// Stocke le temps écoulé depuis le début de la musique en secondes.
#[derive(Resource)]
pub struct MusicTimer(pub f32);

/// Système de démarrage : lance la musique de la beatmap courante.
pub fn start_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    beatmap: Res<CurrentBeatmap>,
) {
    let path = format!("audio/{}", beatmap.0.audio);
    println!("Chargement audio : {}", path);
    commands.spawn(AudioPlayer::<AudioSource>(
        asset_server.load(path),
    ));
}

/// Chaque frame : incrémente le timer de la musique.
pub fn update_timer(
    mut timer: ResMut<MusicTimer>,
    time: Res<Time>,
) {
    timer.0 += time.delta_secs();
}

/// Met en pause le sink audio actif.
pub fn pause_music(sinks: Query<&AudioSink>) {
    for sink in sinks.iter() { sink.pause(); }
}

/// Reprend la lecture du sink audio.
pub fn resume_music(sinks: Query<&AudioSink>) {
    for sink in sinks.iter() { sink.play(); }
}