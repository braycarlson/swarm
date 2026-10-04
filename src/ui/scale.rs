use eframe::egui;

use crate::constants::{UI_SCALE_FALLBACK, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::model::options::Options;

const SCALE_STEPS: [(f32, f32); 8] = [
    (720.0, 1.0),
    (900.0, 1.1),
    (1_080.0, 1.3),
    (1_200.0, 1.4),
    (1_440.0, 1.6),
    (1_600.0, 1.8),
    (1_800.0, 2.0),
    (2_160.0, 2.2),
];
const SCALE_TALLEST: f32 = 2.5;

pub fn effective(options: &Options, ui: &egui::Ui) -> f32 {
    if let Some(scale) = options.ui_scale {
        debug_assert!(scale >= UI_SCALE_MIN);
        debug_assert!(scale <= UI_SCALE_MAX);

        return scale;
    }

    let pixels_per_point = ui.ctx().pixels_per_point();

    let Some(monitor) = ui.input(|input| input.viewport().monitor_size) else {
        return UI_SCALE_FALLBACK;
    };

    let height_pixels = (monitor.y * pixels_per_point).round();

    if !height_pixels.is_finite() {
        return UI_SCALE_FALLBACK;
    }

    if height_pixels < 1.0 {
        return UI_SCALE_FALLBACK;
    }

    scale_for_height(height_pixels)
}

pub fn scale_for_height(height_pixels: f32) -> f32 {
    let scale = SCALE_STEPS
        .iter()
        .find(|(height_max, _)| height_pixels <= *height_max)
        .map_or(SCALE_TALLEST, |(_, scale)| *scale);

    debug_assert!(scale >= UI_SCALE_MIN);
    debug_assert!(scale <= UI_SCALE_MAX);

    scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_for_height_is_monotonic() {
        let heights = [
            0.0,
            720.0,
            721.0,
            900.0,
            1_080.0,
            1_200.0,
            1_440.0,
            1_600.0,
            1_800.0,
            2_160.0,
            4_320.0,
        ];

        let mut previous = 0.0;

        for height in heights {
            let scale = scale_for_height(height);

            assert!(scale >= previous);

            previous = scale;
        }
    }

    #[test]
    fn the_thresholds_match_the_documented_steps() {
        assert!((scale_for_height(1_080.0) - 1.3).abs() < f32::EPSILON);
        assert!((scale_for_height(1_081.0) - 1.4).abs() < f32::EPSILON);
        assert!((scale_for_height(5_000.0) - SCALE_TALLEST).abs() < f32::EPSILON);
    }
}
