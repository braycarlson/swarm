use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Theme {
    CatppuccinFrappe,
    CatppuccinLatte,
    CatppuccinMacchiato,
    CatppuccinMocha,
    Dracula,
    Gruvbox,
    Nord,
    OneDark,
    #[default]
    RosePine,
    RosePineMoon,
    TokyoNight,
    TokyoNightDay,
    TokyoNightStorm,
}

impl Theme {
    pub fn all() -> &'static [Self] {
        &[
            Self::CatppuccinLatte,
            Self::CatppuccinFrappe,
            Self::CatppuccinMacchiato,
            Self::CatppuccinMocha,
            Self::Dracula,
            Self::Gruvbox,
            Self::Nord,
            Self::OneDark,
            Self::RosePine,
            Self::RosePineMoon,
            Self::TokyoNight,
            Self::TokyoNightStorm,
            Self::TokyoNightDay,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::CatppuccinFrappe => "Catppuccin Frapp\u{e9}",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::Dracula => "Dracula",
            Self::Gruvbox => "Gruvbox",
            Self::Nord => "Nord",
            Self::OneDark => "One Dark",
            Self::RosePine => "Rose Pine",
            Self::RosePineMoon => "Rose Pine Moon",
            Self::TokyoNight => "Tokyo Night",
            Self::TokyoNightDay => "Tokyo Night Day",
            Self::TokyoNightStorm => "Tokyo Night Storm",
        }
    }
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashSet;

    use super::*;

    #[test]
    fn every_theme_is_listed_once_with_a_name() {
        let mut seen = FxHashSet::default();

        for theme in Theme::all() {
            let inserted = seen.insert(*theme);

            assert_ne!(theme.name(), "");
            assert!(inserted);
        }

        assert_eq!(seen.len(), 13);
    }
}
