use crate::oscilloscope::trigger::{TriggerEdge, TriggerSettings};

#[derive(Debug, Clone)]
pub struct WaveformData {
    pub samples: Vec<f32>,
    pub time_per_division: f32,  // seconds per division
    pub volts_per_division: f32, // volts per division
    pub sample_rate: u32,
}

impl WaveformData {
    pub fn new(sample_rate: u32) -> Self {
        WaveformData {
            samples: Vec::new(),
            time_per_division: 0.001, // 1ms per division
            volts_per_division: 0.5,  // 0.5V per division
            sample_rate,
        }
    }

    pub fn update_samples(&mut self, new_samples: Vec<f32>) {
        self.samples = new_samples;
    }

    pub fn get_display_samples(&self, trigger_settings: &TriggerSettings) -> Vec<(f32, f32)> {
        if self.samples.is_empty() {
            return vec![];
        }

        let samples_per_screen = self.calculate_samples_per_screen();

        // Find trigger point
        let trigger_index = if trigger_settings.enabled {
            self.find_trigger_point(trigger_settings)
        } else {
            // Free-run mode: just use the most recent samples
            self.samples.len().saturating_sub(samples_per_screen)
        };

        // Extract the relevant window of samples
        let end_index = (trigger_index + samples_per_screen).min(self.samples.len());
        let start_index = trigger_index.min(end_index.saturating_sub(samples_per_screen));

        // Convert to normalized coordinates
        self.samples[start_index..end_index]
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let x = (i as f32) / (samples_per_screen as f32);
                let y = sample / self.volts_per_division;
                (x, y)
            })
            .collect()
    }

    pub fn calculate_samples_per_screen(&self) -> usize {
        // Assuming 10 divisions horizontally
        let divisions = 10.0;
        let total_time = self.time_per_division * divisions;
        (total_time * self.sample_rate as f32) as usize
    }

    fn find_trigger_point(&self, settings: &TriggerSettings) -> usize {
        let threshold = settings.level;

        for i in 1..self.samples.len() {
            let prev = self.samples[i - 1];
            let curr = self.samples[i];

            let triggered = match settings.edge {
                TriggerEdge::Rising => prev < threshold && curr >= threshold,
                TriggerEdge::Falling => prev > threshold && curr <= threshold,
            };

            if triggered {
                return i;
            }
        }

        // No trigger found, return start of buffer
        0
    }

    pub fn increase_time_scale(&mut self) {
        self.time_per_division *= 2.0;
    }

    pub fn decrease_time_scale(&mut self) {
        self.time_per_division = (self.time_per_division / 2.0).max(0.00001);
    }

    pub fn increase_voltage_scale(&mut self) {
        self.volts_per_division *= 2.0;
    }

    pub fn decrease_voltage_scale(&mut self) {
        self.volts_per_division = (self.volts_per_division / 2.0).max(0.01);
    }

    /// Calculate the frequency of the waveform using zero-crossing detection
    pub fn calculate_frequency(&self) -> Option<f32> {
        if self.samples.len() < 3 {
            return None;
        }

        // Find zero crossings (rising edge)
        let mut crossings = Vec::new();
        for i in 1..self.samples.len() {
            if self.samples[i - 1] < 0.0 && self.samples[i] >= 0.0 {
                crossings.push(i);
            }
        }

        // Need at least 2 crossings to calculate period
        if crossings.len() < 2 {
            return None;
        }

        // Calculate average period between crossings
        let mut total_period = 0.0;
        let mut count = 0;

        for i in 1..crossings.len() {
            let period_samples = crossings[i] - crossings[i - 1];
            total_period += period_samples as f32;
            count += 1;
        }

        if count == 0 {
            return None;
        }

        let avg_period_samples = total_period / count as f32;
        let period_seconds = avg_period_samples / self.sample_rate as f32;

        if period_seconds > 0.0 {
            Some(1.0 / period_seconds)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_waveform() {
        let waveform = WaveformData::new(48000);
        assert_eq!(waveform.sample_rate, 48000);
        assert_eq!(waveform.time_per_division, 0.001);
        assert_eq!(waveform.volts_per_division, 0.5);
        assert!(waveform.samples.is_empty());
    }

    #[test]
    fn test_update_samples() {
        let mut waveform = WaveformData::new(48000);
        let samples = vec![0.0, 0.5, 1.0, 0.5, 0.0];

        waveform.update_samples(samples.clone());
        assert_eq!(waveform.samples, samples);
    }

    #[test]
    fn test_calculate_samples_per_screen() {
        let waveform = WaveformData::new(48000);
        let samples_per_screen = waveform.calculate_samples_per_screen();

        // 10 divisions * 0.001 seconds * 48000 samples/sec = 480 samples
        assert_eq!(samples_per_screen, 480);
    }

    #[test]
    fn test_increase_time_scale() {
        let mut waveform = WaveformData::new(48000);
        let initial = waveform.time_per_division;

        waveform.increase_time_scale();
        assert_eq!(waveform.time_per_division, initial * 2.0);

        waveform.increase_time_scale();
        assert_eq!(waveform.time_per_division, initial * 4.0);
    }

    #[test]
    fn test_decrease_time_scale() {
        let mut waveform = WaveformData::new(48000);
        // Start with default 0.001, increase to 0.004
        waveform.increase_time_scale(); // 0.002
        waveform.increase_time_scale(); // 0.004

        waveform.decrease_time_scale();
        assert_eq!(waveform.time_per_division, 0.002);

        waveform.decrease_time_scale();
        assert_eq!(waveform.time_per_division, 0.001);
    }

    #[test]
    fn test_decrease_time_scale_minimum() {
        let mut waveform = WaveformData::new(48000);
        // Set to a very small value by decreasing many times
        for _ in 0..20 {
            waveform.decrease_time_scale();
        }

        // Should hit minimum of 0.00001
        assert_eq!(waveform.time_per_division, 0.00001);
    }

    #[test]
    fn test_increase_voltage_scale() {
        let mut waveform = WaveformData::new(48000);
        let initial = waveform.volts_per_division;

        waveform.increase_voltage_scale();
        assert_eq!(waveform.volts_per_division, initial * 2.0);

        waveform.increase_voltage_scale();
        assert_eq!(waveform.volts_per_division, initial * 4.0);
    }

    #[test]
    fn test_decrease_voltage_scale() {
        let mut waveform = WaveformData::new(48000);
        // Start with default 0.5, decrease to get to lower values
        waveform.decrease_voltage_scale(); // 0.25
        waveform.decrease_voltage_scale(); // 0.125

        let current = waveform.volts_per_division;
        assert!(current < 0.5);
    }

    #[test]
    fn test_decrease_voltage_scale_minimum() {
        let mut waveform = WaveformData::new(48000);
        // Decrease many times to hit the minimum
        for _ in 0..20 {
            waveform.decrease_voltage_scale();
        }

        // Should hit minimum of 0.01
        assert_eq!(waveform.volts_per_division, 0.01);
    }

    #[test]
    fn test_find_trigger_point_rising_edge() {
        let mut waveform = WaveformData::new(48000);
        let samples = vec![-1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5];
        waveform.update_samples(samples);

        let settings = TriggerSettings {
            edge: TriggerEdge::Rising,
            level: 0.25,
            ..Default::default()
        };

        let trigger_point = waveform.find_trigger_point(&settings);
        assert_eq!(trigger_point, 3); // Should trigger between 0.0 and 0.5
    }

    #[test]
    fn test_find_trigger_point_falling_edge() {
        let mut waveform = WaveformData::new(48000);
        let samples = vec![1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5];
        waveform.update_samples(samples);

        let settings = TriggerSettings {
            edge: TriggerEdge::Falling,
            level: 0.25,
            ..Default::default()
        };

        let trigger_point = waveform.find_trigger_point(&settings);
        assert_eq!(trigger_point, 2); // Should trigger between 0.5 and 0.0
    }

    #[test]
    fn test_find_trigger_point_no_trigger() {
        let mut waveform = WaveformData::new(48000);
        let samples = vec![-1.0, -0.9, -0.8, -0.7, -0.6];
        waveform.update_samples(samples);

        let settings = TriggerSettings {
            edge: TriggerEdge::Rising,
            level: 0.0,
            ..Default::default()
        };

        let trigger_point = waveform.find_trigger_point(&settings);
        assert_eq!(trigger_point, 0); // No trigger found, should return 0
    }

    #[test]
    fn test_get_display_samples_empty() {
        let waveform = WaveformData::new(48000);
        let settings = TriggerSettings::default();

        let display_samples = waveform.get_display_samples(&settings);
        assert!(display_samples.is_empty());
    }

    #[test]
    fn test_get_display_samples_normalized() {
        let mut waveform = WaveformData::new(48000);
        let samples = vec![0.0, 0.5, 1.0, 0.5, 0.0];
        waveform.update_samples(samples);

        let settings = TriggerSettings {
            enabled: false,
            ..Default::default()
        };

        let display_samples = waveform.get_display_samples(&settings);

        assert!(!display_samples.is_empty());

        // Check that x coordinates are normalized (0.0 to 1.0)
        for (x, _) in &display_samples {
            assert!(*x >= 0.0 && *x <= 1.0);
        }
    }

    #[test]
    fn test_get_display_samples_with_trigger() {
        let mut waveform = WaveformData::new(48000);

        // Create a signal with a clear trigger point
        let mut samples = vec![];
        for i in 0..1000 {
            let t = i as f32 / 100.0;
            samples.push((t * 2.0 * std::f32::consts::PI).sin());
        }
        waveform.update_samples(samples);

        let settings = TriggerSettings {
            enabled: true,
            edge: TriggerEdge::Rising,
            level: 0.0,
        };

        let display_samples = waveform.get_display_samples(&settings);
        assert!(!display_samples.is_empty());
    }

    #[test]
    fn test_calculate_frequency_440hz() {
        let mut waveform = WaveformData::new(48000);

        // Generate 440 Hz sine wave
        let mut samples = vec![];
        for i in 0..48000 {
            // 1 second of 440 Hz
            let t = i as f32 / 48000.0;
            let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
            samples.push(sample);
        }
        waveform.update_samples(samples);

        let freq = waveform.calculate_frequency();
        assert!(freq.is_some());

        let measured = freq.unwrap();
        // Allow 1% error tolerance
        assert!((measured - 440.0).abs() < 5.0, "Expected ~440Hz, got {measured}");
    }

    #[test]
    fn test_calculate_frequency_no_crossings() {
        let mut waveform = WaveformData::new(48000);
        // DC signal (no zero crossings)
        waveform.update_samples(vec![1.0; 1000]);

        let freq = waveform.calculate_frequency();
        assert!(freq.is_none());
    }

    #[test]
    fn test_calculate_frequency_empty() {
        let waveform = WaveformData::new(48000);
        let freq = waveform.calculate_frequency();
        assert!(freq.is_none());
    }
}
