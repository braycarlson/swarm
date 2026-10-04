#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GitStatus {
    Added,
    Conflicted,
    Deleted,
    Modified,
    Renamed,
    Staged,
    #[default]
    Unmodified,
    Untracked,
}

impl GitStatus {
    pub fn has_diff(self) -> bool {
        matches!(
            self,
            Self::Added | Self::Modified | Self::Renamed | Self::Staged,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_content_changes_carry_a_diff() {
        assert!(GitStatus::Modified.has_diff());
        assert!(GitStatus::Staged.has_diff());
        assert!(!GitStatus::Untracked.has_diff());
        assert!(!GitStatus::Deleted.has_diff());
        assert!(!GitStatus::Unmodified.has_diff());
    }
}
