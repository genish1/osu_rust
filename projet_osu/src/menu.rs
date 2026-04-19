use bevy::prelude::*;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::audio::AudioSink;
use crate::beatmap::Beatmap;
use crate::hitobject::HitObject;
use crate::CurrentBeatmap;
use crate::game::GameState;
use crate::audio::MusicTimer;

// ── États du jeu ──────────────────────────────────────────────────────────────

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    MapSelect,
    Countdown,
    Playing,
    Paused,
    ResultScreen,
}

// ── Ressources ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct SelectedMap(pub usize);

#[derive(Resource)]
pub struct CountdownTimer(pub f32);

/// Ce que fait le joueur en quittant l'état Paused.
#[derive(Resource, Default, PartialEq, Clone)]
pub enum PauseExit {
    #[default]
    Resume,
    Replay,
    Quit,
}

// ── Markers UI ────────────────────────────────────────────────────────────────

#[derive(Component)] pub(crate) struct MainMenuUI;
#[derive(Component)] pub(crate) struct MapSelectUI;
#[derive(Component)] pub(crate) struct CountdownUI;
#[derive(Component)] pub(crate) struct CountdownDisplay;
#[derive(Component)] pub(crate) struct MapButton(usize);
#[derive(Component)] pub(crate) struct PauseMenuUI;
#[derive(Component)] pub(crate) struct ResultUI;

#[derive(Component)]
pub(crate) enum PauseButton { Resume, Replay, Quit }

// ── Maps embarquées ───────────────────────────────────────────────────────────

const LEVEL1: &str = include_str!("/home/genishi/rust/osu_rust/projet_osu/assets/maps/level1.osumap");
const LEVEL2: &str = include_str!("/home/genishi/rust/osu_rust/projet_osu/assets/maps/level2.osumap");
const MAP_NAMES: &[&str] = &["Level 1", "Level 2"];

// ── Couleurs boutons ──────────────────────────────────────────────────────────

const BTN_NORMAL:  Color = Color::srgb(0.15, 0.50, 0.90);
const BTN_HOVERED: Color = Color::srgb(0.25, 0.62, 1.00);
const BTN_PRESSED: Color = Color::srgb(0.10, 0.35, 0.70);

// ── Helper ────────────────────────────────────────────────────────────────────

fn spawn_btn(parent: &mut ChildSpawnerCommands<'_>, label: &str) {
    parent.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(70.0), Val::Px(22.0)),
            border_radius: BorderRadius::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(BTN_NORMAL),
    )).with_children(|btn| {
        btn.spawn((
            Text::new(label),
            TextFont { font_size: 42.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

// ── Menu principal ────────────────────────────────────────────────────────────

pub fn setup_main_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(55.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.04, 0.04, 0.13)),
        MainMenuUI,
    )).with_children(|p| {
        p.spawn((
            Text::new("osu!simple"),
            TextFont { font_size: 90.0, ..default() },
            TextColor(Color::srgb(1.0, 0.38, 0.68)),
        ));
        spawn_btn(p, "Jouer");
    });
}

pub fn update_main_menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in q.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                *color = BTN_PRESSED.into();
                next_state.set(AppState::MapSelect);
            }
            Interaction::Hovered => *color = BTN_HOVERED.into(),
            Interaction::None    => *color = BTN_NORMAL.into(),
        }
    }
}

pub fn cleanup_main_menu(mut commands: Commands, q: Query<Entity, With<MainMenuUI>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}

// ── Sélection de map ──────────────────────────────────────────────────────────

pub fn setup_map_select(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(32.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.04, 0.04, 0.13)),
        MapSelectUI,
    )).with_children(|p| {
        p.spawn((
            Text::new("Choisir une map"),
            TextFont { font_size: 64.0, ..default() },
            TextColor(Color::WHITE),
        ));

        for (i, name) in MAP_NAMES.iter().enumerate() {
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(80.0), Val::Px(20.0)),
                    border_radius: BorderRadius::all(Val::Px(14.0)),
                    ..default()
                },
                BackgroundColor(BTN_NORMAL),
                MapButton(i),
            )).with_children(|btn| {
                btn.spawn((
                    Text::new(*name),
                    TextFont { font_size: 36.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        }
    });
}

pub fn update_map_select(
    mut next_state: ResMut<NextState<AppState>>,
    mut selected: ResMut<SelectedMap>,
    mut q: Query<(&Interaction, &mut BackgroundColor, &MapButton), Changed<Interaction>>,
) {
    for (interaction, mut color, btn) in q.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                *color = BTN_PRESSED.into();
                selected.0 = btn.0;
                next_state.set(AppState::Countdown);
            }
            Interaction::Hovered => *color = BTN_HOVERED.into(),
            Interaction::None    => *color = BTN_NORMAL.into(),
        }
    }
}

pub fn cleanup_map_select(mut commands: Commands, q: Query<Entity, With<MapSelectUI>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}

// ── Chargement de la beatmap ──────────────────────────────────────────────────

pub fn load_selected_beatmap(
    mut commands: Commands,
    selected: Res<SelectedMap>,
    mut game_state: ResMut<GameState>,
    mut music_timer: ResMut<MusicTimer>,
) {
    let content = match selected.0 {
        1 => LEVEL2,
        _ => LEVEL1,
    };
    let beatmap = Beatmap::parse(content).expect("Erreur parsing beatmap");

    let total_objects = beatmap.hit_objects.len() as u32;
    let map_duration_ms = beatmap.hit_objects.iter().map(|o| match o {
        HitObject::Circle(c)  => c.time_ms,
        HitObject::Slider(s)  => s.end_time_ms,
    }).max().unwrap_or(0);

    commands.insert_resource(CurrentBeatmap(beatmap));
    commands.insert_resource(CountdownTimer(3.5));

    *game_state = GameState::new();
    game_state.total_objects    = total_objects;
    game_state.map_duration_ms  = map_duration_ms;
    music_timer.0 = 0.0;
}

// ── Décompte ──────────────────────────────────────────────────────────────────

pub fn setup_countdown(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.04, 0.04, 0.13)),
        CountdownUI,
    )).with_children(|p| {
        p.spawn((
            Text::new("3"),
            TextFont { font_size: 220.0, ..default() },
            TextColor(Color::WHITE),
            CountdownDisplay,
        ));
    });
}

pub fn update_countdown(
    mut next_state: ResMut<NextState<AppState>>,
    mut timer: ResMut<CountdownTimer>,
    time: Res<Time>,
    mut text_q: Query<&mut Text, With<CountdownDisplay>>,
) {
    timer.0 -= time.delta_secs();

    if let Ok(mut text) = text_q.single_mut() {
        text.0 = if timer.0 > 2.5 {
            "3".into()
        } else if timer.0 > 1.5 {
            "2".into()
        } else if timer.0 > 0.5 {
            "1".into()
        } else if timer.0 > 0.0 {
            "GO!".into()
        } else {
            next_state.set(AppState::Playing);
            return;
        };
    }
}

pub fn cleanup_countdown(mut commands: Commands, q: Query<Entity, With<CountdownUI>>) {
    for e in q.iter() { commands.entity(e).despawn(); }
}

// ── Pause ─────────────────────────────────────────────────────────────────────

/// Appuyez sur Escape en jeu pour mettre en pause.
pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause_exit: ResMut<PauseExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        *pause_exit = PauseExit::Resume;
        next_state.set(AppState::Paused);
    }
}

pub fn setup_pause_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(28.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        GlobalZIndex(100),
        PauseMenuUI,
    )).with_children(|p| {
        p.spawn((
            Text::new("Pause"),
            TextFont { font_size: 96.0, ..default() },
            TextColor(Color::WHITE),
        ));

        spawn_pause_btn(p, "Reprendre", PauseButton::Resume);
        spawn_pause_btn(p, "Rejouer",   PauseButton::Replay);
        spawn_pause_btn(p, "Quitter",   PauseButton::Quit);
    });
}

fn spawn_pause_btn(parent: &mut ChildSpawnerCommands<'_>, label: &str, btn: PauseButton) {
    parent.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(80.0), Val::Px(22.0)),
            border_radius: BorderRadius::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(BTN_NORMAL),
        btn,
    )).with_children(|b| {
        b.spawn((
            Text::new(label),
            TextFont { font_size: 38.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

pub fn update_pause_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause_exit: ResMut<PauseExit>,
    mut q: Query<(&Interaction, &mut BackgroundColor, &PauseButton), Changed<Interaction>>,
) {
    // Escape reprend directement
    if keyboard.just_pressed(KeyCode::Escape) {
        *pause_exit = PauseExit::Resume;
        next_state.set(AppState::Playing);
        return;
    }

    for (interaction, mut color, btn) in q.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                *color = BTN_PRESSED.into();
                match btn {
                    PauseButton::Resume => {
                        *pause_exit = PauseExit::Resume;
                        next_state.set(AppState::Playing);
                    }
                    PauseButton::Replay => {
                        *pause_exit = PauseExit::Replay;
                        next_state.set(AppState::Countdown);
                    }
                    PauseButton::Quit => {
                        *pause_exit = PauseExit::Quit;
                        next_state.set(AppState::MapSelect);
                    }
                }
            }
            Interaction::Hovered => *color = BTN_HOVERED.into(),
            Interaction::None    => *color = BTN_NORMAL.into(),
        }
    }
}

// ── Écran de résultats ────────────────────────────────────────────────────────

fn grade_color(grade: &str) -> Color {
    match grade {
        "S" => Color::srgb(1.0, 0.84, 0.0),  // or
        "A" => Color::srgb(0.2, 0.85, 0.2),  // vert
        "B" => Color::srgb(0.3, 0.6,  1.0),  // bleu
        "C" => Color::srgb(1.0, 0.6,  0.2),  // orange
        _   => Color::srgb(0.9, 0.2,  0.2),  // rouge (D)
    }
}

/// OnEnter(ResultScreen) : nettoie les entités de jeu + affiche l'UI.
pub fn setup_result_screen(
    mut commands: Commands,
    game_state:  Res<GameState>,
    audio_sinks: Query<Entity, With<AudioSink>>,
    circles:     Query<Entity, With<crate::renderer::HitCircle>>,
    rings:       Query<Entity, With<crate::renderer::ApproachRing>>,
    starts:      Query<Entity, With<crate::renderer::SliderStartCircle>>,
    ends:        Query<Entity, With<crate::renderer::SliderEndCircle>>,
    bodies:      Query<Entity, With<crate::renderer::SliderBody>>,
    results:     Query<Entity, With<crate::renderer::HitResultSprite>>,
    scores:      Query<Entity, With<crate::renderer::ScoreText>>,
) {
    // Nettoyage des entités de jeu et de l'audio
    for e in audio_sinks.iter() { commands.entity(e).despawn(); }
    for e in circles.iter() { commands.entity(e).despawn(); }
    for e in rings.iter()   { commands.entity(e).despawn(); }
    for e in starts.iter()  { commands.entity(e).despawn(); }
    for e in ends.iter()    { commands.entity(e).despawn(); }
    for e in bodies.iter()  { commands.entity(e).despawn(); }
    for e in results.iter() { commands.entity(e).despawn(); }
    for e in scores.iter()  { commands.entity(e).despawn(); }

    let grade = game_state.grade();
    let score_text = format!("{}", game_state.score);

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(40.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.04, 0.04, 0.13)),
        ResultUI,
    )).with_children(|p| {
        // Titre
        p.spawn((
            Text::new("Resultats"),
            TextFont { font_size: 64.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.9)),
        ));

        // Note (très grande, colorée)
        p.spawn((
            Text::new(grade),
            TextFont { font_size: 200.0, ..default() },
            TextColor(grade_color(grade)),
        ));

        // Score
        p.spawn((
            Text::new(score_text),
            TextFont { font_size: 80.0, ..default() },
            TextColor(Color::WHITE),
        ));

        // Bouton retour
        p.spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(70.0), Val::Px(22.0)),
                border_radius: BorderRadius::all(Val::Px(14.0)),
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(BTN_NORMAL),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("Retour"),
                TextFont { font_size: 38.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

pub fn update_result_screen(
    mut next_state: ResMut<NextState<AppState>>,
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in q.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                *color = BTN_PRESSED.into();
                next_state.set(AppState::MapSelect);
            }
            Interaction::Hovered => *color = BTN_HOVERED.into(),
            Interaction::None    => *color = BTN_NORMAL.into(),
        }
    }
}

pub fn cleanup_result_screen(
    mut commands: Commands,
    q: Query<Entity, With<ResultUI>>,
) {
    for e in q.iter() { commands.entity(e).despawn(); }
}

pub fn cleanup_pause_menu(
    mut commands: Commands,
    pause_ui:    Query<Entity, With<PauseMenuUI>>,
    pause_exit:  Res<PauseExit>,
    audio_sinks: Query<(Entity, &AudioSink)>,
    circles:     Query<Entity, With<crate::renderer::HitCircle>>,
    rings:       Query<Entity, With<crate::renderer::ApproachRing>>,
    starts:      Query<Entity, With<crate::renderer::SliderStartCircle>>,
    ends:        Query<Entity, With<crate::renderer::SliderEndCircle>>,
    bodies:      Query<Entity, With<crate::renderer::SliderBody>>,
    results:     Query<Entity, With<crate::renderer::HitResultSprite>>,
    scores:      Query<Entity, With<crate::renderer::ScoreText>>,
) {
    for e in pause_ui.iter() { commands.entity(e).despawn(); }

    if *pause_exit == PauseExit::Resume {
        for (_, sink) in audio_sinks.iter() { sink.play(); }
    } else {
        // Rejouer ou Quitter : on nettoie tout
        for (e, _) in audio_sinks.iter() { commands.entity(e).despawn(); }
        for e in circles.iter() { commands.entity(e).despawn(); }
        for e in rings.iter()   { commands.entity(e).despawn(); }
        for e in starts.iter()  { commands.entity(e).despawn(); }
        for e in ends.iter()    { commands.entity(e).despawn(); }
        for e in bodies.iter()  { commands.entity(e).despawn(); }
        for e in results.iter() { commands.entity(e).despawn(); }
        for e in scores.iter()  { commands.entity(e).despawn(); }
    }
}
