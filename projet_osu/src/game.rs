/// Le résultat d'un clic selon la précision du joueur.
use bevy::prelude::*;
#[derive(Debug, Clone, PartialEq)]
pub enum HitResult {
    Hit300, // ≤200ms
    Hit100, // 201–400ms
    Hit50,  // 401–450ms
    Miss,
}

impl HitResult {
    /// Points accordés selon le résultat.
    pub fn points(&self) -> u32 {
        match self {
            HitResult::Hit300 => 300,
            HitResult::Hit100 => 100,
            HitResult::Hit50  => 50,
            HitResult::Miss   => 0,
        }
    }
}

/// État complet d'une partie en cours.
#[derive(Resource)]
pub struct GameState {
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub next_object_index: usize,
    pub elapsed_ms: u64,
    /// Nombre total d'objets dans la beatmap (cercles + sliders).
    pub total_objects: u32,
    /// Nombre de misses enregistrés.
    pub miss_count: u32,
    /// Timestamp du dernier objet de la beatmap (en ms).
    pub map_duration_ms: u64,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            score: 0,
            combo: 0,
            max_combo: 0,
            next_object_index: 0,
            elapsed_ms: 0,
            total_objects: 0,
            miss_count: 0,
            map_duration_ms: 0,
        }
    }

    /// Enregistre un hit et met à jour score + combo.
    pub fn register_hit(&mut self, result: HitResult) {
        match result {
            HitResult::Miss => {
                self.miss_count += 1;
                self.combo = 0;
            }
            _ => {
                self.combo += 1;
                if self.combo > self.max_combo {
                    self.max_combo = self.combo;
                }
                self.score += result.points() * self.combo;
            }
        }
    }

    /// Évalue la précision d'un clic selon l'écart en ms avec le timing attendu.
    pub fn evaluate_timing(delta_ms: u64) -> HitResult {
        match delta_ms {
            0..=200   => HitResult::Hit300,
            201..=400 => HitResult::Hit100,
            401..=450 => HitResult::Hit50,
            _         => HitResult::Miss,
        }
    }

    // 300 * (1 + 2 + ... + n) = 300 * n*(n+1)/2
    pub fn max_possible_score(&self) -> u64 {
        let n = self.total_objects as u64;
        300 * n * (n + 1) / 2
    }

    /// Note finale de D à S.
    pub fn grade(&self) -> &'static str {
        let max = self.max_possible_score();
        if max == 0 { return "D"; }
        if self.score as u64 == max { return "S"; }
        let ratio = self.score as f64 / max as f64;
        if ratio >= 0.80 { "A" }
        else if ratio >= 0.60 { "B" }
        else if ratio >= 0.40 { "C" }
        else { "D" }
    }
}