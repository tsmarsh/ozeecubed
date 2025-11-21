#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum TriggerMode {
    Auto,
    Normal,
    Single,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerEdge {
    Rising,
    Falling,
}

#[derive(Debug, Clone)]
pub struct TriggerSettings {
    pub enabled: bool,
    #[allow(dead_code)]
    pub mode: TriggerMode,
    pub edge: TriggerEdge,
    pub level: f32, // Voltage level for trigger
}

impl Default for TriggerSettings {
    fn default() -> Self {
        TriggerSettings {
            enabled: true,
            mode: TriggerMode::Auto,
            edge: TriggerEdge::Rising,
            level: 0.0,
        }
    }
}

impl TriggerSettings {
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(-10.0, 10.0);
    }

    pub fn toggle_edge(&mut self) {
        self.edge = match self.edge {
            TriggerEdge::Rising => TriggerEdge::Falling,
            TriggerEdge::Falling => TriggerEdge::Rising,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_trigger_settings() {
        let settings = TriggerSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.mode, TriggerMode::Auto);
        assert_eq!(settings.edge, TriggerEdge::Rising);
        assert_eq!(settings.level, 0.0);
    }

    #[test]
    fn test_toggle_enabled() {
        let mut settings = TriggerSettings::default();
        assert!(settings.enabled);

        settings.toggle_enabled();
        assert!(!settings.enabled);

        settings.toggle_enabled();
        assert!(settings.enabled);
    }

    #[test]
    fn test_set_level() {
        let mut settings = TriggerSettings::default();

        settings.set_level(5.0);
        assert_eq!(settings.level, 5.0);

        settings.set_level(-3.5);
        assert_eq!(settings.level, -3.5);
    }

    #[test]
    fn test_set_level_clamping() {
        let mut settings = TriggerSettings::default();

        // Test upper bound
        settings.set_level(15.0);
        assert_eq!(settings.level, 10.0);

        // Test lower bound
        settings.set_level(-15.0);
        assert_eq!(settings.level, -10.0);

        // Test within bounds
        settings.set_level(7.5);
        assert_eq!(settings.level, 7.5);
    }

    #[test]
    fn test_toggle_edge() {
        let mut settings = TriggerSettings::default();
        assert_eq!(settings.edge, TriggerEdge::Rising);

        settings.toggle_edge();
        assert_eq!(settings.edge, TriggerEdge::Falling);

        settings.toggle_edge();
        assert_eq!(settings.edge, TriggerEdge::Rising);
    }

    #[test]
    fn test_trigger_mode_equality() {
        assert_eq!(TriggerMode::Auto, TriggerMode::Auto);
        assert_eq!(TriggerMode::Normal, TriggerMode::Normal);
        assert_eq!(TriggerMode::Single, TriggerMode::Single);
        assert_ne!(TriggerMode::Auto, TriggerMode::Normal);
    }

    #[test]
    fn test_trigger_edge_equality() {
        assert_eq!(TriggerEdge::Rising, TriggerEdge::Rising);
        assert_eq!(TriggerEdge::Falling, TriggerEdge::Falling);
        assert_ne!(TriggerEdge::Rising, TriggerEdge::Falling);
    }
}
