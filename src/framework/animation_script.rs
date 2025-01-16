use std::{collections::HashMap, error::Error, fs, path::PathBuf};
use toml;

use super::prelude::*;

type Config = HashMap<String, Vec<(String, f32)>>;

#[derive(Debug)]
struct ParsedKeyframe {
    beats: f32,
    value: f32,
}

pub struct AnimationScript {
    pub animation: Animation,
    #[allow(dead_code)]
    path: PathBuf,
    config: Config,
    keyframe_sequences: HashMap<String, Vec<Keyframe>>,
}

impl AnimationScript {
    pub fn new(path: PathBuf, animation: Animation) -> Self {
        let config =
            Self::import_script(&path).expect("Unable to import script");

        let mut script = Self {
            animation,
            config,
            path,
            keyframe_sequences: HashMap::new(),
        };

        script.precompute_keyframes();
        script
    }

    fn import_script(path: &PathBuf) -> Result<Config, Box<dyn Error>> {
        let file_content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&file_content)?;
        debug!("{:?}", config);
        Ok(config)
    }

    fn parse_bar_beat_16th(time_str: &str) -> Result<f32, Box<dyn Error>> {
        let parts: Vec<f32> = time_str
            .split('.')
            .map(|s| s.parse::<f32>())
            .collect::<Result<Vec<f32>, _>>()?;

        if parts.len() != 3 {
            return Err("Time string must be in format bar.beat.16th".into());
        }

        let [bars, beats, sixteenths] = [parts[0], parts[1], parts[2]];
        let total_beats = (bars * 4.0) + beats + (sixteenths * 0.25);

        Ok(total_beats)
    }

    fn precompute_keyframes(&mut self) {
        for (param, time_values) in &self.config {
            let mut parsed_keyframes = Vec::new();

            for (time_str, value) in time_values {
                if let Ok(beats) = Self::parse_bar_beat_16th(time_str) {
                    parsed_keyframes.push(ParsedKeyframe {
                        beats,
                        value: *value,
                    });
                }
            }

            let mut keyframes = Vec::new();
            for i in 0..parsed_keyframes.len() {
                let current = &parsed_keyframes[i];
                let duration = if i < parsed_keyframes.len() - 1 {
                    parsed_keyframes[i + 1].beats - current.beats
                } else {
                    0.0
                };

                keyframes.push(Keyframe::new(current.value, duration));
            }

            self.keyframe_sequences.insert(param.clone(), keyframes);
        }
    }

    pub fn get(&self, param: &str) -> f32 {
        if let Some(keyframes) = self.keyframe_sequences.get(param) {
            self.animation.lerp(keyframes.clone(), 0.0)
        } else {
            panic!("No param named {}", param);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    use crate::framework::animation::tests::{init, BPM};

    #[test]
    #[serial]
    fn test_animation_script_interpolation() {
        init(0);
        let animation = Animation::new(BPM);
        let script = AnimationScript::new(
            to_absolute_path(file!(), "./animation_script.toml"),
            animation,
        );

        let radius = script.get("radius");
        assert_eq!(radius, 0.0, "Should start at initial radius");

        init(32);
        let radius = script.get("radius");
        assert!(
            (radius - 250.0).abs() < 0.01,
            "Radius should be approximately 250.0 at halfway point"
        );

        let hue = script.get("hue");
        assert!(
            (hue - 0.5).abs() < 0.01,
            "Hue should be approximately 0.5 at halfway point"
        );
    }

    #[test]
    #[serial]
    fn test_bar_beat_16th_conversion() {
        let test_cases = vec![
            ("0.0.0", 0.0),
            ("1.0.0", 4.0),  // 1 bar = 4 beats
            ("0.1.0", 1.0),  // 1 beat = 1 beat
            ("0.0.2", 0.5),  // 2 sixteenths = 0.5 beats
            ("2.2.2", 10.5), // 2 bars + 2 beats + 2 sixteenths = 8 + 2 + 0.5 = 10.5
        ];

        for (input, expected) in test_cases {
            let result = AnimationScript::parse_bar_beat_16th(input)
                .expect("Failed to parse time string");
            assert!(
                (result - expected).abs() < 0.001,
                "Converting {} should yield {} but got {}",
                input,
                expected,
                result
            );
        }
    }
}
