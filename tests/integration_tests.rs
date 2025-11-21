use ozeecubed::oscilloscope::{TriggerSettings, WaveformData};
use ozeecubed::oscilloscope::trigger::{TriggerEdge, TriggerMode};

#[test]
fn test_oscilloscope_workflow() {
    // Create a waveform
    let mut waveform = WaveformData::new(48000);

    // Generate a test signal (sine wave)
    let mut samples = vec![];
    for i in 0..4800 {
        let t = i as f32 / 48000.0;
        let frequency = 440.0; // A4 note
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
        samples.push(sample);
    }

    waveform.update_samples(samples);

    // Create trigger settings
    let mut trigger_settings = TriggerSettings::default();
    trigger_settings.enabled = true;
    trigger_settings.edge = TriggerEdge::Rising;
    trigger_settings.level = 0.0;

    // Get display samples
    let display_samples = waveform.get_display_samples(&trigger_settings);

    // Verify we got samples
    assert!(!display_samples.is_empty());

    // Verify samples are normalized
    for (x, _y) in &display_samples {
        assert!(*x >= 0.0 && *x <= 1.0);
    }
}

#[test]
fn test_time_scale_changes_affect_display() {
    let mut waveform = WaveformData::new(48000);

    // Generate samples
    let samples: Vec<f32> = (0..4800).map(|i| (i as f32 / 100.0).sin()).collect();
    waveform.update_samples(samples);

    let trigger_settings = TriggerSettings::default();

    // Get initial display samples
    let initial_count = waveform.calculate_samples_per_screen();

    // Increase time scale (zoom out)
    waveform.increase_time_scale();
    let zoomed_out_count = waveform.calculate_samples_per_screen();

    // Decrease time scale (zoom in)
    waveform.decrease_time_scale();
    waveform.decrease_time_scale();
    let zoomed_in_count = waveform.calculate_samples_per_screen();

    // More time per division should mean more samples per screen
    assert!(zoomed_out_count > initial_count);
    assert!(zoomed_in_count < initial_count);
}

#[test]
fn test_voltage_scale_changes_affect_normalization() {
    let mut waveform = WaveformData::new(48000);

    // Create samples with known values
    let samples = vec![1.0; 1000];
    waveform.update_samples(samples);

    let mut trigger_settings = TriggerSettings::default();
    trigger_settings.enabled = false; // Disable triggering for predictable results

    // Get display samples with default voltage scale (0.5 V/div)
    let display_samples_1 = waveform.get_display_samples(&trigger_settings);

    // Change voltage scale
    waveform.increase_voltage_scale(); // Now 1.0 V/div

    let display_samples_2 = waveform.get_display_samples(&trigger_settings);

    // With larger voltage scale, same voltage appears smaller (lower y value)
    if let (Some((_x1, y1)), Some((_x2, y2))) = (display_samples_1.first(), display_samples_2.first()) {
        assert!(y2.abs() < y1.abs());
    }
}

#[test]
fn test_trigger_edge_detection() {
    let mut waveform = WaveformData::new(48000);

    // Create a simple ramp up then down
    let mut samples = vec![];
    for i in 0..100 {
        samples.push(-1.0 + (i as f32 / 50.0));
    }
    for i in 0..100 {
        samples.push(1.0 - (i as f32 / 50.0));
    }
    waveform.update_samples(samples);

    // Test rising edge trigger
    let mut trigger_settings = TriggerSettings::default();
    trigger_settings.enabled = true;
    trigger_settings.edge = TriggerEdge::Rising;
    trigger_settings.level = 0.0;

    let rising_samples = waveform.get_display_samples(&trigger_settings);

    // Test falling edge trigger
    trigger_settings.edge = TriggerEdge::Falling;
    let falling_samples = waveform.get_display_samples(&trigger_settings);

    // Both should produce valid results
    assert!(!rising_samples.is_empty());
    assert!(!falling_samples.is_empty());
}

#[test]
fn test_trigger_settings_modifications() {
    let mut settings = TriggerSettings::default();

    // Test default state
    assert!(settings.enabled);
    assert_eq!(settings.mode, TriggerMode::Auto);
    assert_eq!(settings.edge, TriggerEdge::Rising);
    assert_eq!(settings.level, 0.0);

    // Test toggle operations
    settings.toggle_enabled();
    assert!(!settings.enabled);

    settings.toggle_edge();
    assert_eq!(settings.edge, TriggerEdge::Falling);

    // Test level clamping
    settings.set_level(15.0);
    assert_eq!(settings.level, 10.0);

    settings.set_level(-15.0);
    assert_eq!(settings.level, -10.0);
}

#[test]
fn test_empty_waveform_handling() {
    let waveform = WaveformData::new(48000);
    let trigger_settings = TriggerSettings::default();

    // Empty waveform should return empty display samples
    let display_samples = waveform.get_display_samples(&trigger_settings);
    assert!(display_samples.is_empty());
}

#[test]
fn test_very_short_waveform() {
    let mut waveform = WaveformData::new(48000);
    let samples = vec![0.0, 1.0, 0.0];
    waveform.update_samples(samples);

    let mut trigger_settings = TriggerSettings::default();
    trigger_settings.enabled = false;

    let display_samples = waveform.get_display_samples(&trigger_settings);

    // Should handle short waveforms gracefully
    assert!(!display_samples.is_empty());
    assert!(display_samples.len() <= 3);
}

#[test]
fn test_scale_value_limits() {
    let mut waveform = WaveformData::new(48000);

    // Test minimum time scale
    for _ in 0..100 {
        waveform.decrease_time_scale();
    }
    assert_eq!(waveform.time_per_division, 0.00001);

    // Test minimum voltage scale
    for _ in 0..100 {
        waveform.decrease_voltage_scale();
    }
    assert_eq!(waveform.volts_per_division, 0.01);

    // Test that increasing still works after hitting minimum
    waveform.increase_time_scale();
    assert!(waveform.time_per_division > 0.00001);

    waveform.increase_voltage_scale();
    assert!(waveform.volts_per_division > 0.01);
}
