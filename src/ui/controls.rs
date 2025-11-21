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
}

pub fn build_controls<'a>(
    time_per_div: f32,
    volts_per_div: f32,
    trigger_enabled: bool,
    trigger_level: f32,
) -> Element<'a, ControlMessage> {
    let time_controls = column![
        text("Time/Div").size(14),
        row![
            button("-").on_press(ControlMessage::DecreaseTimeScale),
            text(format!("{:.2} ms", time_per_div * 1000.0))
                .width(Length::Fixed(80.0)),
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
            text(format!("{:.2} V", volts_per_div)).width(Length::Fixed(80.0)),
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
            text(format!("{:.2} V", trigger_level)).width(Length::Fixed(80.0)),
            button("+").on_press(ControlMessage::IncreaseTriggerLevel),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    ]
    .spacing(5);

    container(
        row![time_controls, voltage_controls, trigger_controls]
            .spacing(20)
            .padding(10)
            .align_y(Alignment::Start),
    )
    .into()
}
