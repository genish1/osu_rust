/// Un objet à frapper — cercle ou slider.
#[derive(Debug, Clone)]
pub enum HitObject {
    Circle(Circle),
    Slider(Slider),
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    /// Moment exact où le joueur doit cliquer, en millisecondes.
    pub time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Slider {
    pub x: f32,
    pub y: f32,
    pub time_ms: u64,
    pub x_end: f32,
    pub y_end: f32,
    pub end_time_ms: u64,
}

impl HitObject {
    /// Retourne le timestamp de l'objet quel que soit son type.
    pub fn time_ms(&self) -> u64 {
        match self {
            HitObject::Circle(c) => c.time_ms,
            HitObject::Slider(s) => s.time_ms,
        }
    }

    /// Retourne la position (x, y) de l'objet.
    pub fn position(&self) -> (f32, f32) {
        match self {
            HitObject::Circle(c) => (c.x, c.y),
            HitObject::Slider(s) => (s.x, s.y),
        }
    }
}