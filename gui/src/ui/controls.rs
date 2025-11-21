use iced::widget::{button, column, container, row, slider, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
    SideBySide,
    Stacked,
}

#[derive(Debug, Clone)]
pub enum ControlMessage {
    IncreaseTimeScale,
    DecreaseTimeScale,
    SetTimeScale(f32),
    IncreaseVoltageScale,
    DecreaseVoltageScale,
    SetVoltageScale(f32),
    ToggleTrigger,
    ToggleTriggerEdge,
    IncreaseTriggerLevel,
    DecreaseTriggerLevel,
    SetTriggerLevel(f32),
    TogglePersistence,
    IncreasePersistence,
    DecreasePersistence,
    SetPersistenceFrames(u8),
    SetLayoutMode(LayoutMode),
}

#[derive(Debug, Clone)]
pub struct Measurements {
    pub frequency: Option<f32>,
    pub peak_to_peak: Option<f32>,
    pub rms: Option<f32>,
    pub duty_cycle: Option<f32>,
}

pub struct ControlState {
    pub time_per_div: f32,
    pub volts_per_div: f32,
    pub trigger_enabled: bool,
    pub trigger_level: f32,
    pub persistence_enabled: bool,
    pub persistence_frames: usize,
}

pub fn build_controls<'a>(
    state: &ControlState,
    measurements: &Measurements,
) -> Element<'a, ControlMessage> {
    let time_per_div = state.time_per_div;
    let volts_per_div = state.volts_per_div;
    let trigger_enabled = state.trigger_enabled;
    let trigger_level = state.trigger_level;
    let persistence_enabled = state.persistence_enabled;
    let persistence_frames = state.persistence_frames;
    // Convert time_per_div to logarithmic scale for slider (10µs to 1s)
    // log10(0.00001) = -5, log10(1.0) = 0
    let time_log = time_per_div.log10();

    let time_controls = column![
        text("Time/Div").size(14),
        row![
            button("-").on_press(ControlMessage::DecreaseTimeScale),
            text(format!("{:.2} ms", time_per_div * 1000.0)).width(Length::Fixed(80.0)),
            button("+").on_press(ControlMessage::IncreaseTimeScale),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        slider(-5.0..=0.0, time_log, |val| {
            ControlMessage::SetTimeScale(10_f32.powf(val))
        })
        .step(0.01)
        .width(Length::Fixed(150.0)),
    ]
    .spacing(5);

    // Convert volts_per_div to logarithmic scale for slider (0.01V to 10V)
    // log10(0.01) = -2, log10(10.0) = 1
    let volts_log = volts_per_div.log10();

    let voltage_controls = column![
        text("Volts/Div").size(14),
        row![
            button("-").on_press(ControlMessage::DecreaseVoltageScale),
            text(format!("{volts_per_div:.2} V")).width(Length::Fixed(80.0)),
            button("+").on_press(ControlMessage::IncreaseVoltageScale),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        slider(-2.0..=1.0, volts_log, |val| {
            ControlMessage::SetVoltageScale(10_f32.powf(val))
        })
        .step(0.01)
        .width(Length::Fixed(150.0)),
    ]
    .spacing(5);

    let trigger_controls = column![
        text("Trigger").size(14),
        row![
            button(if trigger_enabled { "ON" } else { "OFF" })
                .on_press(ControlMessage::ToggleTrigger),
            button("Edge").on_press(ControlMessage::ToggleTriggerEdge),
        ]
        .spacing(5),
        row![
            button("-").on_press(ControlMessage::DecreaseTriggerLevel),
            text(format!("{trigger_level:.2} V")).width(Length::Fixed(80.0)),
            button("+").on_press(ControlMessage::IncreaseTriggerLevel),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        slider(-10.0..=10.0, trigger_level, ControlMessage::SetTriggerLevel)
            .step(0.1)
            .width(Length::Fixed(150.0)),
    ]
    .spacing(5);

    let measurements_display = column![
        text("Measurements").size(14),
        text(if let Some(freq) = measurements.frequency {
            if freq >= 1000.0 {
                format!("Freq: {:.2} kHz", freq / 1000.0)
            } else {
                format!("Freq: {freq:.1} Hz")
            }
        } else {
            "Freq: --".to_string()
        })
        .size(11),
        text(if let Some(pk_pk) = measurements.peak_to_peak {
            format!("Vpp: {pk_pk:.3} V")
        } else {
            "Vpp: --".to_string()
        })
        .size(11),
        text(if let Some(rms_val) = measurements.rms {
            format!("Vrms: {rms_val:.3} V")
        } else {
            "Vrms: --".to_string()
        })
        .size(11),
        text(if let Some(duty) = measurements.duty_cycle {
            format!("Duty: {duty:.1}%")
        } else {
            "Duty: --".to_string()
        })
        .size(11),
    ]
    .spacing(3);

    let persistence_controls = column![
        text("Persistence").size(14),
        row![button(if persistence_enabled { "ON" } else { "OFF" })
            .on_press(ControlMessage::TogglePersistence),]
        .spacing(5),
        row![
            button("-").on_press(ControlMessage::DecreasePersistence),
            text(format!("{persistence_frames}")).width(Length::Fixed(80.0)),
            button("+").on_press(ControlMessage::IncreasePersistence),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        slider(1.0..=30.0, persistence_frames as f32, |val| {
            ControlMessage::SetPersistenceFrames(val as u8)
        })
        .step(1.0)
        .width(Length::Fixed(150.0)),
    ]
    .spacing(5);

    let layout_selector = column![
        text("Layout").size(14),
        row![
            button("◧").on_press(ControlMessage::SetLayoutMode(LayoutMode::SideBySide)),
            button("⬒").on_press(ControlMessage::SetLayoutMode(LayoutMode::Stacked)),
        ]
        .spacing(5),
    ]
    .spacing(5);

    container(
        row![
            layout_selector,
            time_controls,
            voltage_controls,
            trigger_controls,
            persistence_controls,
            measurements_display
        ]
        .spacing(20)
        .padding(10)
        .align_y(Alignment::Start),
    )
    .into()
}
