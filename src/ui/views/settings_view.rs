//! Application settings view
//!
//! Configures application preferences, network settings
//! and communication protocols.

use iced::{
    widget::{
        button, checkbox, column, container, pick_list, row, scrollable, text,
        text_input, Space, horizontal_rule, slider,
    },
    Alignment, Element, Length,
};

use crate::ui::{
    messages::Message,
    styles,
    Theme,
};

// Static choices for `pick_list` controls to avoid references to temporaries
const AIRDROP_VISIBILITIES: [AirDropVisibility; 3] = [
    AirDropVisibility::Everyone,
    AirDropVisibility::ContactsOnly,
    AirDropVisibility::ReceivingOff,
];
 
const AIRPLAY_QUALITIES: [AirPlayQuality; 4] = [
    AirPlayQuality::Auto,
    AirPlayQuality::Low,
    AirPlayQuality::Medium,
    AirPlayQuality::High,
]; 
 
const LOG_LEVELS: [LogLevel; 5] = [
    LogLevel::Error,
    LogLevel::Warn,
    LogLevel::Info,
    LogLevel::Debug,
    LogLevel::Trace,
];

// Empty static choices for the network interface list (placeholder)
const EMPTY_INTERFACES: [&str; 0] = [];

/// Settings view state
#[derive(Debug, Clone)]
pub struct SettingsView {
    // General settings
    auto_discovery: bool,
    discovery_interval: u32,
    show_notifications: bool,
    minimize_to_tray: bool,
    
    // AirDrop settings
    airdrop_enabled: bool,
    airdrop_visibility: AirDropVisibility,
    auto_accept_from_contacts: bool,
    
    // AirPlay settings
    airplay_enabled: bool,
    airplay_quality: AirPlayQuality,
    airplay_audio_only: bool,
    
    // Network settings
    network_interface: Option<String>,
    available_interfaces: Vec<String>,
    custom_port: Option<u16>,
    // Persistent text version of the custom port for `text_input`
    custom_port_text: String,
    
    // Advanced settings
    debug_mode: bool,
    log_level: LogLevel,
    max_concurrent_transfers: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AirDropVisibility {
    Everyone,
    ContactsOnly,
    ReceivingOff,
}

impl std::fmt::Display for AirDropVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AirDropVisibility::Everyone => write!(f, "Everyone"),
            AirDropVisibility::ContactsOnly => write!(f, "Contacts Only"),
            AirDropVisibility::ReceivingOff => write!(f, "Receiving Off"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AirPlayQuality {
    Low,
    Medium,
    High,
    Auto,
}

impl std::fmt::Display for AirPlayQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AirPlayQuality::Low => write!(f, "Low"),
            AirPlayQuality::Medium => write!(f, "Medium"),
            AirPlayQuality::High => write!(f, "High"),
            AirPlayQuality::Auto => write!(f, "Auto"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "Error"),
            LogLevel::Warn => write!(f, "Warning"),
            LogLevel::Info => write!(f, "Info"),
            LogLevel::Debug => write!(f, "Debug"),
            LogLevel::Trace => write!(f, "Trace"),
        }
    }
}

impl SettingsView {
    /// Create a new settings view
    pub fn new(
        auto_discovery: bool,
        discovery_interval: u32,
        show_notifications: bool,
        minimize_to_tray: bool,
        airdrop_enabled: bool,
        airdrop_visibility: AirDropVisibility,
        auto_accept_from_contacts: bool,
        airplay_enabled: bool,
        airplay_quality: AirPlayQuality,
        airplay_audio_only: bool,
        network_interface: Option<String>,
        available_interfaces: Vec<String>,
        custom_port: Option<u16>,
        debug_mode: bool,
        log_level: LogLevel,
        max_concurrent_transfers: u32,
    ) -> Self {
        Self {
            auto_discovery,
            discovery_interval,
            show_notifications,
            minimize_to_tray,
            airdrop_enabled,
            airdrop_visibility,
            auto_accept_from_contacts,
            airplay_enabled,
            airplay_quality,
            airplay_audio_only,
            network_interface,
            available_interfaces,
            custom_port,
            custom_port_text: custom_port.map(|p| p.to_string()).unwrap_or_default(),
            debug_mode,
            log_level,
            max_concurrent_transfers,
        }
    }

    /// AirPlay settings section
    fn airplay_settings(&self, _theme: &Theme) -> Element<Message> {
        let section_header = text("AirPlay")
            .size(18);

        let settings = column![
            // AirPlay abilitato
            checkbox(
                "Enable AirPlay",
                self.airplay_enabled
            )
            .on_toggle(|_| Message::Tick),
            
            if self.airplay_enabled {
                column![
                    // Quality
                    row![
                        text("Video quality:")
                            .size(14)
                            .width(Length::FillPortion(1)),
                        
                        pick_list(
                            &AIRPLAY_QUALITIES[..],
                            Some(self.airplay_quality.clone()),
                            |_| Message::Tick
                        )
                        .width(Length::FillPortion(2)),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(styles::spacing::MEDIUM),
                    
                    // Audio only
                    checkbox(
                        "Audio only (better performance)",
                        self.airplay_audio_only
                    )
                    .on_toggle(|_| Message::Tick),
                ]
                .spacing(styles::spacing::MEDIUM)
            } else {
                column![]
            },
        ]
        .spacing(styles::spacing::MEDIUM);

        container(
            column![
                section_header,
                Space::with_height(styles::spacing::MEDIUM),
                settings,
            ]
        )
        .padding(styles::spacing::MEDIUM.0)
        .width(Length::Fill)
        .into()
    }

    /// Render the settings view
    pub fn view(&self, theme: &Theme) -> Element<Message> {
        let header = row![
            button(
                text("← Back")
                    .size(14)
            )
            .on_press(Message::ShowMainView)
            .style(iced::theme::Button::Secondary),
            
            Space::with_width(styles::spacing::MEDIUM),
            
            text("Settings")
                .size(24)
                ,
            
            Space::with_width(Length::Fill),
            
            button(
                text("Save")
                    .size(14)
            )
            // Save action placeholder
            .on_press(Message::Tick)
            .style(iced::theme::Button::Primary),
            
            button(
                text("Reset")
                    .size(14)
            )
            // Reset action placeholder
            .on_press(Message::Tick)
            .style(iced::theme::Button::Secondary),
        ]
        .align_items(Alignment::Center)
        .padding(styles::spacing::MEDIUM.0);

        let content = scrollable(
            column![
                // General settings
                self.general_settings(theme),
                
                Space::with_height(styles::spacing::LARGE),
                
                // AirDrop settings
                self.airdrop_settings(theme),
                
                Space::with_height(styles::spacing::LARGE),
                
                // AirPlay settings
                self.airplay_settings(theme),
                
                Space::with_height(styles::spacing::LARGE),
                
                // Network settings
                self.network_settings(theme),
                
                Space::with_height(styles::spacing::LARGE),
                
                // Advanced settings
                self.advanced_settings(theme),
                
                Space::with_height(styles::spacing::LARGE),
            ]
            .spacing(0)
        )
        .height(Length::Fill);

        container(
            column![
                header,
                horizontal_rule(1),
                content,
            ]
        )
        .padding(styles::spacing::MEDIUM.0)
        .into()
    }

    /// General settings section
    fn general_settings(&self, _theme: &Theme) -> Element<Message> {
        let section_header = text("General")
            .size(18);

        let settings = column![
            // Auto discovery
            row![
                checkbox(
                    "Automatic device discovery",
                    self.auto_discovery
                )
                .on_toggle(|_| Message::Tick),
            ],
            
            // Discovery interval
            if self.auto_discovery {
                column![
                    text(format!("Scan interval: {} seconds", self.discovery_interval))
                        .size(14)
                        ,
                    
                    slider(
                        5..=60,
                        self.discovery_interval,
                        |_| Message::Tick
                    )
                    ,
                ]
                .spacing(styles::spacing::SMALL)
            } else {
                column![]
            },
            
            // Notifications
            checkbox(
                "Show notifications",
                self.show_notifications
            )
            .on_toggle(|_| Message::Tick),
            
            // Minimize to tray
            checkbox(
                "Minimize to system tray",
                self.minimize_to_tray
            )
            .on_toggle(|_| Message::Tick),
        ]
        .spacing(styles::spacing::MEDIUM);

        container(
            column![
                section_header,
                Space::with_height(styles::spacing::MEDIUM),
                settings,
            ]
        )
        .padding(styles::spacing::MEDIUM.0)
        .width(Length::Fill)
        .into()
    }

    /// AirDrop settings section
    fn airdrop_settings(&self, _theme: &Theme) -> Element<Message> {
        let section_header = text("AirDrop")
            .size(18);

        let settings = column![
            // AirDrop abilitato
            checkbox(
                "Enable AirDrop",
                self.airdrop_enabled
            )
            .on_toggle(|_| Message::Tick),
            
            if self.airdrop_enabled {
                column![
                    // Visibility
                    row![
                        text("Visibility:")
                            .size(14)
                            
                            .width(Length::FillPortion(1)),
                        
                        pick_list(
                            &AIRDROP_VISIBILITIES[..],
                            Some(self.airdrop_visibility.clone()),
                            |_| Message::Tick
                        )
                        
                        .width(Length::FillPortion(2)),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(styles::spacing::MEDIUM),
                    
                    // Auto-accept from contacts
                    checkbox(
                        "Automatically accept from contacts",
                        self.auto_accept_from_contacts
                    )
                    .on_toggle(|_| Message::Tick),
                ]
                .spacing(styles::spacing::MEDIUM)
            } else {
                column![]
            },
        ]
        .spacing(styles::spacing::MEDIUM);

        container(
            column![
                section_header,
                Space::with_height(styles::spacing::MEDIUM),
                settings,
            ]
        )
        .padding(styles::spacing::MEDIUM.0)
        .width(Length::Fill)
        .into()
    }

    /// Network settings section
    fn network_settings(&self, _theme: &Theme) -> Element<Message> {
        let section_header = text("Network")
            .size(18);

        let settings = column![
            // Network interface
            row![
                text("Network interface:")
                    .size(14)
                    
                    .width(Length::FillPortion(1)),
                
                pick_list(
                    &EMPTY_INTERFACES[..],
                    None::<&str>,
                    |_| Message::Tick
                )
                .placeholder("Automatic")
                
                .width(Length::FillPortion(2)),
            ]
            .align_items(Alignment::Center)
            .spacing(styles::spacing::MEDIUM),
            
            // Custom port
            row![
                text("Custom port:")
                    .size(14)
                    
                    .width(Length::FillPortion(1)),
                
                text_input(
                    "Automatic",
                    ""
                )
                .on_input(|_| Message::Tick)
                .width(Length::FillPortion(2)),
            ]
            .align_items(Alignment::Center)
            .spacing(styles::spacing::MEDIUM),
        ]
        .spacing(styles::spacing::MEDIUM);

        container(
            column![
                section_header,
                Space::with_height(styles::spacing::MEDIUM),
                settings,
            ]
        )
        .padding(styles::spacing::MEDIUM.0)
        .width(Length::Fill)
        .into()
    }

    /// Advanced settings section
    fn advanced_settings(&self, _theme: &Theme) -> Element<Message> {
        let section_header = text("Advanced")
            .size(18);

        let settings = column![
            // Debug mode
            checkbox(
                "Debug mode",
                self.debug_mode
            )
            .on_toggle(|_| Message::ToggleDebugMode)
            ,
            
            // Log level
            row![
                text("Log level:")
                    .size(14)
                    
                    .width(Length::FillPortion(1)),
                
                pick_list(
                    &LOG_LEVELS[..],
                    Some(self.log_level.clone()),
                    |_| Message::Tick
                )
                
                .width(Length::FillPortion(2)),
            ]
            .align_items(Alignment::Center)
            .spacing(styles::spacing::MEDIUM),
            
            // Max concurrent transfers
            column![
                text(format!("Concurrent transfers: {}", self.max_concurrent_transfers))
                    .size(14)
                    ,
                
                slider(
                    1..=10,
                    self.max_concurrent_transfers,
                    |_| Message::Tick
                )
                ,
            ]
            .spacing(styles::spacing::SMALL),
            
            // Advanced actions
            row![
                button(
                    text("Open Log Folder")
                        .size(14)
                )
                .on_press(Message::OpenLogFolder)
                .style(iced::theme::Button::Secondary),
                
                button(
                    text("Clear Cache")
                        .size(14)
                )
                .on_press(Message::ClearCache)
                .style(iced::theme::Button::Secondary),
                
                button(
                    text("Diagnostics")
                        .size(14)
                )
                .on_press(Message::RunDiagnostics)
                .style(iced::theme::Button::Secondary),
            ]
            .spacing(styles::spacing::MEDIUM),
        ]
        .spacing(styles::spacing::MEDIUM);

        container(
            column![
                section_header,
                Space::with_height(styles::spacing::MEDIUM),
                settings,
            ]
        )
        .padding(styles::spacing::MEDIUM.0)
        .width(Length::Fill)
        .into()
    }
}