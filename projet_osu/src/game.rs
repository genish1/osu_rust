/// Le résultat d'un clic selon la précision du joueur.
use bevy::prelude::*;
#[derive(Debug, Clone, PartialEq)]
pub enum HitResult {
    Hit300, // Parfait   — moins de 50ms d'écart
    Hit100, // Correct   — moins de 100ms d'écart
    Hit50,  // Raté de peu — moins de 150ms d'écart
    Miss,   // Raté
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
    /// Index du prochain objet à traiter dans la beatmap.
    pub next_object_index: usize,
    /// Temps écoulé depuis le début de la musique, en ms.
    pub elapsed_ms: u64,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            score: 0,
            combo: 0,
            max_combo: 0,
            next_object_index: 0,
            elapsed_ms: 0,
        }
    }

    /// Enregistre un hit et met à jour score + combo.
    pub fn register_hit(&mut self, result: HitResult) {
        match result {
            HitResult::Miss => {
                // Le combo se casse sur un miss
                self.combo = 0;
            }
            _ => {
                self.combo += 1;
                if self.combo > self.max_combo {
                    self.max_combo = self.combo;
                }
                // Les points sont multipliés par le combo actuel
                self.score += result.points() * self.combo;
            }
        }
    }

    /// Évalue la précision d'un clic selon l'écart en ms avec le timing attendu.
    pub fn evaluate_timing(delta_ms: u64) -> HitResult {
        match delta_ms {
            0..=200   => HitResult::Hit300,
            201..=400  => HitResult::Hit100,
            401..=450 => HitResult::Hit50,
            _         => HitResult::Miss,
        }
    }
}