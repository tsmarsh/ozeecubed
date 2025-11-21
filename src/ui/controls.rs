use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub enum ControlMessage {
    IncreaseTimeScale,
    DecreaseTimeScale,
    IncreaseVoltageScale,
    DecreaseVoltageScale,
    ToggleTrigger,
    ToggleTriggerEdge,
    IncreaseTriggerLevel,
    DecreaseTriggerLevel,
    TogglePersistence,
    IncreasePersistence,
    DecreasePersistence,
}

#[derive(Debug, Clone)]
pub struct Measurements {
    pub frequency: Option<f32>,
    pub peak_to_peak: Option<f32>,
    pub rms: Option<f32>,
    pub duty_cycle: Option<f32>,
}

pub fn build_controls<'a>(
    time_per_div: f32,
    volts_per_div: f32,
    trigger_enabled: bool,
    trigger_level: f32,
    measurements: &Measurements,
    persistence_enabled: bool,
    persistence_frames: usize,
) -> Element<'a, ControlMessage> {
    let time_controls = column![
        text("Time/Div").size(14),
        row![
            button("-").on_press(ControlMessage::DecreaseTimeScale),
            text(format!("{:.2} ms", time_per_div * 1000.0)).width(Length::Fixed(80.0)),
            button("+").on_press(ControlMessage::IncreaseTimeScale),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    ]
    .spacing(5);

    let voltage_controls = column![
        text("Volts/Div").size(14),
        row![
            button("-").on_press(ControlMessage::DecreaseVoltageScale),
            text(format!("{volts_per_div:.2} V")).width(Length::Fixed(80.0)),
            button("+").on_press(ControlMessage::IncreaseVoltageScale),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
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
    ]
    .spacing(5);

    container(
        row![
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
