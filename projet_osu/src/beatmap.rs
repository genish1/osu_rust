use crate::hitobject::{Circle, HitObject, Slider};

/// Erreurs possibles lors du parsing de notre format custom.
#[derive(Debug)]
pub enum BeatmapError {
    ParseError(String),
    MissingField(String),
}

/// Une beatmap complète, prête à être jouée.
#[derive(Debug)]
pub struct Beatmap {
    pub title: String,
    pub audio: String,
    pub bpm: f32,
    pub hit_objects: Vec<HitObject>,
}

impl Beatmap {
    pub fn parse(content: &str) -> Result<Self, BeatmapError> {
        let mut title = None;
        let mut audio = None;
        let mut bpm   = None;
        let mut hit_objects = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Ignorer lignes vides et commentaires
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Métadonnées : "clé = valeur"
            if line.contains('=') {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                let key = parts[0].trim();
                let val = parts[1].trim();

                match key {
                    "title" => title = Some(val.to_string()),
                    "audio" => audio = Some(val.to_string()),
                    "bpm"   => {
                        bpm = Some(val.parse::<f32>().map_err(|_| {
                            BeatmapError::ParseError(
                                format!("bpm invalide : '{val}'")
                            )
                        })?)
                    }
                    _ => {} // clé inconnue, on ignore
                }
                continue;
            }

            // Objets : "type | x | y | time_ms | [end_time_ms]"
            if line.contains('|') {
                match parse_hit_object(line) {
                    Some(obj) => hit_objects.push(obj),
                    None => return Err(BeatmapError::ParseError(
                        format!("Ligne invalide : '{line}'")
                    )),
                }
            }
        }

        // Vérification des champs obligatoires
        let title = title.ok_or_else(|| BeatmapError::MissingField("title".into()))?;
        let audio = audio.ok_or_else(|| BeatmapError::MissingField("audio".into()))?;
        let bpm   = bpm.ok_or_else(||   BeatmapError::MissingField("bpm".into()))?;

        // Tri chronologique
        hit_objects.sort_by_key(|o| o.time_ms());

        Ok(Beatmap { title, audio, bpm, hit_objects })
    }
}

/// Parse une seule ligne objet, retourne None si invalide.
fn parse_hit_object(line: &str) -> Option<HitObject> {
    let parts: Vec<&str> = line.split('|').map(str::trim).collect();

    let kind    = parts.first()?;
    let x       = parts.get(1)?.parse::<f32>().ok()?;
    let y       = parts.get(2)?.parse::<f32>().ok()?;
    let time_ms = parts.get(3)?.parse::<u64>().ok()?;

    match *kind {
        "circle" => Some(HitObject::Circle(Circle { x, y, time_ms })),
        "slider" => {
            let end_time_ms = parts.get(4)?.parse::<u64>().ok()?;
            Some(HitObject::Slider(Slider { x, y, time_ms, end_time_ms }))
        }
        _ => None,
    }
}