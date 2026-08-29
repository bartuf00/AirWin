//! Main application view
//!
//! Layout with the device list on the left, the action panel on the right
//! and a status bar plus a bounded activity log at the bottom.

use iced::{
    widget::{
        button, column, container, row, scrollable, text, text_input, Space,
        horizontal_rule, vertical_rule,
    },
    Alignment, Element, Length,
};

use crate::ui::{
    components,
    messages::{Message, NotificationMessage, NotificationType},
    styles,
    Theme,
};

/// Main view state (borrows everything from the app)
pub struct MainView<'a> {
    discovered_devices: &'a [crate::network::DiscoveredDevice],
    selected_device: Option<&'a crate::network::DiscoveredDevice>,
    is_scanning: bool,
    airplay_status: &'a crate::protocols::airplay::AirPlayStatus,
    airdrop_status: &'a crate::protocols::airdrop::AirDropStatus,
    file_transfer_progress: Option<f32>,
    notifications: &'a [NotificationMessage],
    show_link_dialog: bool,
    link_url: &'a str,
}

/// Maximum height reserved for the activity log at the bottom of the window.
const ACTIVITY_LOG_HEIGHT: f32 = 132.0;

pub fn render<'a>(
    discovered_devices: &'a [crate::network::DiscoveredDevice],
    selected_device: Option<&'a crate::network::DiscoveredDevice>,
    is_scanning: bool,
    airplay_status: &'a crate::protocols::airplay::AirPlayStatus,
    airdrop_status: &'a crate::protocols::airdrop::AirDropStatus,
    file_transfer_progress: Option<f32>,
    notifications: &'a [NotificationMessage],
    show_link_dialog: bool,
    link_url: &'a str,
    theme: &Theme,
) -> Element<'a, Message> {
    MainView::new(
        discovered_devices,
        selected_device,
        is_scanning,
        airplay_status,
        airdrop_status,
        file_transfer_progress,
        notifications,
        show_link_dialog,
        link_url,
    )
    .view(theme)
}

impl<'a> MainView<'a> {
    pub fn new(
        discovered_devices: &'a [crate::network::DiscoveredDevice],
        selected_device: Option<&'a crate::network::DiscoveredDevice>,
        is_scanning: bool,
        airplay_status: &'a crate::protocols::airplay::AirPlayStatus,
        airdrop_status: &'a crate::protocols::airdrop::AirDropStatus,
        file_transfer_progress: Option<f32>,
        notifications: &'a [NotificationMessage],
        show_link_dialog: bool,
        link_url: &'a str,
    ) -> Self {
        Self {
            discovered_devices,
            selected_device,
            is_scanning,
            airplay_status,
            airdrop_status,
            file_transfer_progress,
            notifications,
            show_link_dialog,
            link_url,
        }
    }

    pub fn view(&self, theme: &Theme) -> Element<'a, Message> {
        let main_content = row![
            self.device_panel(theme),
            vertical_rule(1),
            self.action_panel(theme),
        ]
        .spacing(styles::spacing::MEDIUM)
        .height(Length::Fill);

        let mut content = column![
            self.header(theme),
            horizontal_rule(1),
            main_content,
        ]
        .spacing(styles::spacing::SMALL);

        if !self.notifications.is_empty() {
            content = content.push(horizontal_rule(1));
            content = content.push(self.activity_log(theme));
        }

        content = content.push(horizontal_rule(1));
        content = content.push(self.status_bar(theme));

        let base: Element<Message> = container(content)
            .padding(styles::spacing::MEDIUM.0)
            .into();

        if self.show_link_dialog {
            container(
                column![
                    base,
                    self.link_dialog(theme),
                ]
            )
            .padding(styles::spacing::MEDIUM.0)
            .into()
        } else {
            base
        }
    }

    fn header(&self, theme: &Theme) -> Element<'a, Message> {
        row![
            text("AirWin")
                .size(24)
                .style(styles::colors::TEXT_PRIMARY),

            Space::with_width(Length::Fill),

            row![
                button(
                    text(if self.is_scanning { "Stop" } else { "Scan" })
                        .size(14)
                )
                .on_press(if self.is_scanning {
                    Message::StopScanning
                } else {
                    Message::StartScanning
                })
                .style(if self.is_scanning {
                    iced::theme::Button::Secondary
                } else {
                    iced::theme::Button::Primary
                }),

                button(
                    text(match theme {
                        Theme::Light => "Dark",
                        Theme::Dark => "Light",
                    })
                    .size(14)
                )
                .on_press(Message::ThemeChanged(match theme { Theme::Light => Theme::Dark, Theme::Dark => Theme::Light })),
            ]
            .spacing(styles::spacing::SMALL)
        ]
        .align_items(Alignment::Center)
        .padding(styles::spacing::MEDIUM.0)
        .into()
    }

    fn device_panel(&self, _theme: &Theme) -> Element<'a, Message> {
        let header = row![
            text("Discovered Devices")
                .size(18)
                .style(styles::colors::TEXT_SECONDARY),

            Space::with_width(Length::Fill),

            text(format!("({})", self.discovered_devices.len()))
                .size(14)
                .style(styles::colors::TEXT_MUTED),
        ]
        .align_items(Alignment::Center);

        let device_list: Element<'a, Message> = if self.discovered_devices.is_empty() {
            if self.is_scanning {
                container(
                    column![
                        text("...")
                            .size(48)
                            .style(styles::colors::TEXT_MUTED),
                        text("Scanning...")
                            .size(16)
                            .style(styles::colors::TEXT_MUTED),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(styles::spacing::MEDIUM)
                )
                .center_x()
                .center_y()
                .height(Length::Fill)
                .into()
            } else {
                container(
                    column![
                        text("No devices found")
                            .size(16)
                            .style(styles::colors::TEXT_MUTED),
                        text("Press Scan to search again")
                            .size(14)
                            .style(styles::colors::TEXT_MUTED),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(styles::spacing::SMALL)
                )
                .center_x()
                .center_y()
                .height(Length::Fill)
                .into()
            }
        } else {
            let devices: Element<'a, Message> = self.discovered_devices
                .iter()
                .cloned()
                .fold(
                    column![].spacing(styles::spacing::SMALL),
                    |col, device| {
                        let is_selected = self.selected_device
                            .as_ref()
                            .map(|selected| selected.name == device.name)
                            .unwrap_or(false);

                        let desc = format!("{} • {}:{}",
                            match device.service_type {
                                crate::network::ServiceType::AirDrop => "AirDrop",
                                crate::network::ServiceType::AirPlay => "AirPlay",
                                _ => "Other",
                            },
                            device.address,
                            device.port
                        );
                        col.push(
                            components::selection_card(
                                &device.name,
                                &desc,
                                is_selected,
                                Message::DeviceSelected(device.clone()),
                            )
                        )
                    }
                )
                .into();

            scrollable(devices)
                .height(Length::Fill)
                .into()
        };

        container(
            column![
                header,
                Space::with_height(styles::spacing::MEDIUM),
                device_list,
            ]
        )
        .padding(styles::spacing::MEDIUM.0)
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .into()
    }

    fn action_panel(&self, theme: &Theme) -> Element<'a, Message> {
        let header = text("Actions")
            .size(18)
            .style(styles::colors::TEXT_SECONDARY);

        let content = if let Some(device) = self.selected_device {
            column![
                self.selected_device_info(device, theme),

                Space::with_height(styles::spacing::LARGE),

                self.airdrop_actions(theme),

                Space::with_height(styles::spacing::MEDIUM),

                if matches!(device.service_type, crate::network::ServiceType::AirPlay) {
                    self.airplay_actions(theme)
                } else {
                    Space::with_height(0).into()
                },

                Space::with_height(Length::Fill),

                if let Some(progress) = self.file_transfer_progress {
                    self.transfer_progress(progress, theme)
                } else {
                    Space::with_height(styles::spacing::SMALL).into()
                },
            ]
        } else {
            column![
                container(
                    column![
                        text("Select a device")
                            .size(16)
                            .style(styles::colors::TEXT_MUTED),
                        text("to get started")
                            .size(14)
                            .style(styles::colors::TEXT_MUTED),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(styles::spacing::SMALL)
                )
                .center_x()
                .center_y()
                .height(Length::Fill)
            ]
        };

        container(
            column![
                header,
                Space::with_height(styles::spacing::MEDIUM),
                content,
            ]
        )
        .padding(styles::spacing::MEDIUM.0)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .into()
    }

    fn selected_device_info(
        &self,
        device: &crate::network::DiscoveredDevice,
        _theme: &Theme,
    ) -> Element<'a, Message> {
        column![
            text(&device.name)
                .size(16)
                .style(styles::colors::TEXT_PRIMARY),

            text(format!("{} • {}:{}",
                match device.service_type {
                    crate::network::ServiceType::AirDrop => "AirDrop",
                    crate::network::ServiceType::AirPlay => "AirPlay",
                    _ => "Other",
                },
                device.address,
                device.port
            ))
                .size(12)
                .style(styles::colors::TEXT_MUTED),
        ]
        .spacing(styles::spacing::SMALL)
        .into()
    }

    fn airdrop_actions(&self, _theme: &Theme) -> Element<'a, Message> {
        let status_text = match self.airdrop_status {
            crate::protocols::airdrop::AirDropStatus::Idle => "Ready",
            crate::protocols::airdrop::AirDropStatus::Connecting => "Connecting...",
            crate::protocols::airdrop::AirDropStatus::Connected => "Connected",
            crate::protocols::airdrop::AirDropStatus::Transferring(_) => "Transferring...",
            crate::protocols::airdrop::AirDropStatus::Failed(_) => "Error",
        };

        column![
            text("AirDrop")
                .size(14)
                .style(styles::colors::TEXT_SECONDARY),

            text(status_text)
                .size(12)
                .style(styles::colors::TEXT_MUTED),

            Space::with_height(styles::spacing::SMALL),

            button(
                text("Send File")
                    .size(14)
            )
            .on_press_maybe(
                if matches!(self.airdrop_status, crate::protocols::airdrop::AirDropStatus::Idle | crate::protocols::airdrop::AirDropStatus::Connected) {
                    self.selected_device.map(|d| Message::SendFile(d.clone()))
                } else {
                    None
                }
            )
            .width(Length::Fill),

            button(
                text("Send Link")
                    .size(14)
            )
            .on_press_maybe(
                if matches!(self.airdrop_status, crate::protocols::airdrop::AirDropStatus::Idle | crate::protocols::airdrop::AirDropStatus::Connected) {
                    Some(Message::ShowLinkDialog)
                } else {
                    None
                }
            )
            .width(Length::Fill),
        ]
        .spacing(styles::spacing::SMALL)
        .into()
    }

    fn airplay_actions(&self, _theme: &Theme) -> Element<'a, Message> {
        let (status_text, button_text, button_action) = match self.airplay_status {
            crate::protocols::airplay::AirPlayStatus::Idle => {
                ("Disconnected", "Connect", self.selected_device.map(|d| Message::StartScreenMirroring(d.clone())))
            },
            crate::protocols::airplay::AirPlayStatus::Connecting => {
                ("Connecting...", "Connecting...", None)
            },
            crate::protocols::airplay::AirPlayStatus::Connected => {
                ("Connected", "Disconnect", Some(Message::StopScreenMirroring))
            },
            crate::protocols::airplay::AirPlayStatus::Failed(_) => {
                ("Error", "Retry", self.selected_device.map(|d| Message::StartScreenMirroring(d.clone())))
            },
        };

        column![
            text("AirPlay")
                .size(14)
                .style(styles::colors::TEXT_SECONDARY),

            text(status_text)
                .size(12)
                .style(styles::colors::TEXT_MUTED),

            Space::with_height(styles::spacing::SMALL),

            button(
                text(button_text)
                    .size(14)
            )
            .on_press_maybe(button_action)
            .width(Length::Fill),
        ]
        .spacing(styles::spacing::SMALL)
        .into()
    }

    fn transfer_progress(&self, progress: f32, _theme: &Theme) -> Element<'a, Message> {
        column![
            text("Transferring")
                .size(14)
                .style(styles::colors::TEXT_SECONDARY),

            iced::Element::<Message>::from(components::primary_progress_bar(progress)),

            text(format!("{:.1}%", progress))
                .size(12)
                .style(styles::colors::TEXT_MUTED),
        ]
        .spacing(styles::spacing::SMALL)
        .into()
    }

    fn status_bar(&self, _theme: &Theme) -> Element<'a, Message> {
        let left = if self.is_scanning {
            "Scanning...".to_string()
        } else {
            format!("Devices: {}", self.discovered_devices.len())
        };
        let right = self.selected_device
            .map(|d| d.name.clone())
            .unwrap_or_else(|| "No device selected".to_string());
        container(
            row![
                text(left).style(styles::colors::TEXT_SECONDARY),
                Space::with_width(Length::Fill),
                text(right).style(styles::colors::TEXT_MUTED),
            ]
            .align_items(Alignment::Center)
        )
        .padding([0, styles::spacing::MEDIUM.0 as u16])
        .into()
    }

    /// Bounded, scrollable activity log pinned above the status bar.
    /// Never grows past `ACTIVITY_LOG_HEIGHT`, so the device list keeps its space.
    /// The newest entry is shown as a highlighted banner on the first line.
    fn activity_log(&self, _theme: &Theme) -> Element<'a, Message> {
        let entries = self.notifications.iter().rev().fold(
            column![].spacing(styles::spacing::TINY),
            |col, n| {
                let color = match n.notification_type {
                    NotificationType::Success => styles::colors::SUCCESS,
                    NotificationType::Error => styles::colors::ERROR,
                    NotificationType::Warning => styles::colors::WARNING,
                    NotificationType::Info => styles::colors::INFO,
                };
                col.push(
                    row![
                        text("●").size(10).style(color),
                        text(format!("{} — {}", n.title, n.content))
                            .size(12)
                            .style(styles::colors::TEXT_SECONDARY),
                    ]
                    .spacing(styles::spacing::SMALL)
                    .align_items(Alignment::Center)
                )
            },
        );

        container(
            column![
                text("Activity")
                    .size(12)
                    .style(styles::colors::TEXT_MUTED),
                self.latest_banner(),
                scrollable(entries).height(Length::Fill),
            ]
            .spacing(styles::spacing::TINY)
        )
        .padding(styles::spacing::SMALL.0)
        .width(Length::Fill)
        .height(Length::Fixed(ACTIVITY_LOG_HEIGHT))
        .style(styles::container_secondary)
        .into()
    }

    fn latest_banner(&self) -> Element<'a, Message> {
        let Some(latest) = self.notifications.last() else {
            return Space::with_height(0).into();
        };

        let color = match latest.notification_type {
            NotificationType::Success => styles::colors::SUCCESS,
            NotificationType::Error => styles::colors::ERROR,
            NotificationType::Warning => styles::colors::WARNING,
            NotificationType::Info => styles::colors::INFO,
        };

        container(
            row![
                text("●").size(11).style(color),
                text(format!("{} — {}", latest.title, latest.content))
                    .size(13)
                    .style(styles::colors::TEXT_PRIMARY),
            ]
            .spacing(styles::spacing::SMALL)
            .align_items(Alignment::Center)
        )
        .padding([styles::spacing::TINY.0 as u16, styles::spacing::MEDIUM.0 as u16])
        .width(Length::Fill)
        .style(move |_theme: &iced::Theme| container::Appearance {
            background: Some(iced::Background::Color(styles::colors::SURFACE_VARIANT)),
            border: iced::Border {
                color,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn link_dialog(&self, _theme: &Theme) -> Element<'a, Message> {
        let dialog_content = column![
            text("Send Link")
                .size(18)
                .style(styles::colors::TEXT_SECONDARY),

            Space::with_height(styles::spacing::MEDIUM),

            text_input("Enter URL...", self.link_url)
                .on_input(Message::LinkInputChanged)
                .width(Length::Fill),

            Space::with_height(styles::spacing::MEDIUM),

            row![
                button(
                    text("Cancel")
                        .size(14)
                )
                .on_press(Message::HideLinkDialog),

                Space::with_width(styles::spacing::MEDIUM),

                button(
                    text("Send")
                        .size(14)
                )
                .on_press_maybe(
                    if !self.link_url.trim().is_empty() {
                        self.selected_device.map(|d| Message::SendLink(d.clone(), self.link_url.to_string()))
                    } else {
                        None
                    }
                ),
            ]
            .align_items(Alignment::Center),
        ]
        .spacing(styles::spacing::MEDIUM)
        .max_width(400);

        container(
            container(dialog_content)
                .padding(styles::spacing::LARGE.0)
        )
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
