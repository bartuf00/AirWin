//! Main user-interface module (Iced)
//!
//! Contains the complete UI implementation built on Iced,
//! with a modern, responsive design.

use iced::{
    executor,
    window, Application, Command, Element, Settings, Subscription, Theme as IcedTheme,
};

use std::time::Duration;

pub mod components;
pub mod messages;
pub mod styles;
pub mod views;
pub mod widgets;

// Re-export of main types
pub use messages::Message;
 
/// Application theme (used by `styles` for custom styles)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}
 
/// Main AirWin application struct
#[derive(Debug)]
pub struct AirWinApp {
    /// Current application state
    current_view: AppView,
    
    /// Devices discovered on the network
    discovered_devices: Vec<crate::network::DiscoveredDevice>,
    
    /// Currently selected device
    selected_device: Option<crate::network::DiscoveredDevice>,
    
    /// Scan state
    is_scanning: bool,
    
    /// AirPlay state
    airplay_status: crate::protocols::airplay::AirPlayStatus,
    
    /// AirDrop state
    airdrop_status: crate::protocols::airdrop::AirDropStatus,
    
    /// File transfer progress (0.0-100.0)
    file_transfer_progress: Option<f32>,
    
    /// Active notifications
    notifications: Vec<messages::NotificationMessage>,
    
    /// Current theme
    theme: Theme,
    
    /// Persisted settings view to avoid lifetime issues
    settings_view: views::settings_view::SettingsView,
    
    /// Persisted about view to avoid lifetime issues
    about_view: views::about_view::AboutView,
    
    /// State of the send-link dialog
    show_link_dialog: bool,
    
    /// URL to send as a link
    link_url: String,
    
    /// General loading state
    is_loading: bool,
    
    /// Status message
    status_message: String,
} 

/// Views available in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppView {
    /// Main view with device list and action panel
    Main,
    /// Settings view
    Settings,
    /// About view
    About,
    /// Initial loading view
    Loading,
}

impl Default for AppView {
    fn default() -> Self {
        Self::Loading
    }
}

impl Application for AirWinApp {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let app = Self {
            current_view: AppView::Loading,
            status_message: "Initializing...".to_string(),
            is_loading: true,
            theme: Theme::default(),
            settings_view: views::settings_view::SettingsView::new(
                true,                // enable_auto_discovery
                15,                  // discovery_interval
                true,                // show_notifications
                false,               // minimize_to_tray
                true,                // airdrop_enabled
                views::settings_view::AirDropVisibility::Everyone,
                false,               // auto_accept_from_contacts
                true,                // airplay_enabled
                views::settings_view::AirPlayQuality::Auto,
                false,               // airplay_audio_only
                None,                // network_interface
                Vec::new(),          // available_interfaces
                None,                // custom_port
                false,               // debug_mode
                views::settings_view::LogLevel::Info,
                2,                   // max_concurrent_transfers
            ),
            about_view: views::about_view::AboutView::new(
                "0.1.0".to_string(),
                "unknown".to_string(),
                None,
            ),
            discovered_devices: Vec::new(),
            selected_device: None,
            is_scanning: false,
            airplay_status: crate::protocols::airplay::AirPlayStatus::Idle,
            airdrop_status: crate::protocols::airdrop::AirDropStatus::Idle,
            file_transfer_progress: None,
            notifications: Vec::new(),
            show_link_dialog: false,
            link_url: String::new(),
        };

        let command = Command::perform(
            async {
                // Simulated initialization
                tokio::time::sleep(Duration::from_secs(2)).await;
            },
            |_| Message::InitializationComplete,
        );

        (app, command)
    }

    fn title(&self) -> String {
        match self.current_view {
            AppView::Main => "AirWin - Apple Sharing".to_string(),
            AppView::Settings => "AirWin - Settings".to_string(),
            AppView::About => "AirWin - About".to_string(),
            AppView::Loading => "AirWin - Loading".to_string(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::InitializationComplete => {
                self.current_view = AppView::Main;
                self.is_loading = false;
                self.status_message = "Ready".to_string();
                
                // Start automatic scanning
                Command::perform(
                    async { () },
                    |_| Message::StartScanning,
                )
            }

            Message::StartScanning => {
                self.is_scanning = true;
                self.status_message = "Scanning for devices...".to_string();
                
                Command::perform(
                    Self::scan_devices(),
                    Message::DevicesUpdated,
                )
            }

            Message::StopScanning => {
                self.is_scanning = false;
                self.status_message = "Scan stopped".to_string();
                Command::none()
            }

            Message::DevicesUpdated(devices) => {
                self.discovered_devices = devices;
                self.is_scanning = false;
                self.status_message = format!(
                    "Found {} devices",
                    self.discovered_devices.len()
                );
                
                if !self.discovered_devices.is_empty() {
                    self.add_notification(
                        "Devices found".to_string(),
                        format!(
                            "Discovered {} nearby Apple devices",
                            self.discovered_devices.len()
                        ),
                        messages::NotificationType::Success,
                    );
                }
                
                Command::none()
            }

            Message::DeviceSelected(device) => {
                self.selected_device = Some(device.clone());
                self.status_message = format!("Selected: {}", device.name);
                
                self.add_notification(
                    "Device selected".to_string(),
                    format!("You can now send content to {}", device.name),
                    messages::NotificationType::Info,
                );
                
                Command::none()
            }

            Message::SendFile(_device) => {
                if self.selected_device.is_some() {
                    self.airdrop_status = crate::protocols::airdrop::AirDropStatus::Transferring(0.0);
                    self.file_transfer_progress = Some(0.0);
                    
                    Command::perform(
                        Self::simulate_file_transfer(),
                        Message::FileSendProgress,
                    )
                } else {
                    Command::none()
                }
            }

            Message::SendLink(device, url) => {
                self.link_url = url.clone();
                self.add_notification(
                    "Sending link".to_string(),
                    format!("Sending link to {}", device.name),
                    messages::NotificationType::Info,
                );
                self.airdrop_status = crate::protocols::airdrop::AirDropStatus::Connecting;
                Command::perform(
                    async {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        Ok::<(), String>(())
                    },
                    |res| match res {
                        Ok(()) => Message::FileSendCompleted(Ok(())),
                        Err(e) => Message::FileSendCompleted(Err(e)),
                    },
                )
            }

            Message::FileSendProgress(progress) => {
                self.file_transfer_progress = Some(progress);
                self.airdrop_status = crate::protocols::airdrop::AirDropStatus::Transferring(progress);
                Command::none()
            }

            Message::FileSendCompleted(result) => {
                self.file_transfer_progress = None;
                self.airdrop_status = crate::protocols::airdrop::AirDropStatus::Idle;
                match result {
                    Ok(()) => self.add_notification(
                        "Transfer complete".to_string(),
                        "Operation completed successfully".to_string(),
                        messages::NotificationType::Success,
                    ),
                    Err(e) => self.add_notification(
                        "Transfer failed".to_string(),
                        e,
                        messages::NotificationType::Error,
                    ),
                }
                Command::none()
            }

            Message::ShowLinkDialog => {
                self.show_link_dialog = true;
                Command::none()
            }

            Message::HideLinkDialog => {
                self.show_link_dialog = false;
                self.link_url.clear();
                Command::none()
            }

            Message::LinkInputChanged(url) => {
                self.link_url = url;
                Command::none()
            }

            Message::ShowNotification(notification) => {
                self.notifications.push(notification);
                
                // Auto-remove notification after 5 seconds
                Command::perform(
                    async {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    },
                    |_| Message::HideNotification,
                )
            }

            Message::HideNotification => {
                if !self.notifications.is_empty() {
                    self.notifications.remove(0);
                }
                Command::none()
            }
            Message::StartScreenMirroring(_device) => {
                self.airplay_status = crate::protocols::airplay::AirPlayStatus::Connecting;
                Command::perform(
                    async {
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        crate::protocols::airplay::AirPlayStatus::Connected
                    },
                    Message::AirPlayStatusChanged,
                )
            }

            Message::StopScreenMirroring => {
                self.airplay_status = crate::protocols::airplay::AirPlayStatus::Idle;
                Command::none()
            }

            Message::AirPlayStatusChanged(status) => {
                self.airplay_status = status.clone();
                match status {
                    crate::protocols::airplay::AirPlayStatus::Connected => self.add_notification(
                        "AirPlay connected".to_string(),
                        "AirPlay connection established".to_string(),
                        messages::NotificationType::Success,
                    ),
                    crate::protocols::airplay::AirPlayStatus::Failed(err) => self.add_notification(
                        "AirPlay error".to_string(),
                        err,
                        messages::NotificationType::Error,
                    ),
                    _ => {}
                }
                Command::none()
            }
            
            // Remaining message variants
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        match self.current_view {
            AppView::Loading => self.loading_view(),
            AppView::Main => self.main_view(),
            AppView::Settings => self.settings_view(),
            AppView::About => self.about_view(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Periodic updates subscription
        Subscription::none()
    }

    fn theme(&self) -> Self::Theme {
        match self.theme {
            Theme::Light => IcedTheme::Light,
            Theme::Dark => IcedTheme::Dark,
        }
    }
}

impl AirWinApp {
    /// Loading view
    fn loading_view(&self) -> Element<Message> {
        components::loading_state(&self.status_message)
    }

    /// Main application view
    fn main_view(&self) -> Element<Message> {
        views::main_view::render(
            &self.discovered_devices,
            self.selected_device.as_ref(),
            self.is_scanning,
            &self.airplay_status,
            &self.airdrop_status,
            self.file_transfer_progress,
            &self.notifications,
            self.show_link_dialog,
            &self.link_url,
            &self.theme,
        )
    }
 
    /// Settings view
    fn settings_view(&self) -> Element<Message> {
        self.settings_view.view(&self.theme)
    }

    /// About view
    fn about_view(&self) -> Element<Message> {
        self.about_view.view(&self.theme)
    }
  
    /// Simulated network device scan
    async fn scan_devices() -> Vec<crate::network::DiscoveredDevice> {
        // Simulated scan delay
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        // Sample devices for testing
        vec![
            crate::network::DiscoveredDevice {
                name: "Marco's iPhone".to_string(),
                address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(192,168,1,100)),
                port: 8771,
                service_type: crate::network::ServiceType::AirDrop,
                txt_records: std::collections::HashMap::new(),
            },
            crate::network::DiscoveredDevice {
                name: "iPad Pro".to_string(),
                address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(192,168,1,101)),
                port: 7100,
                service_type: crate::network::ServiceType::AirPlay,
                txt_records: std::collections::HashMap::new(),
            },
            crate::network::DiscoveredDevice {
                name: "MacBook Pro".to_string(),
                address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(192,168,1,102)),
                port: 8771,
                service_type: crate::network::ServiceType::AirDrop,
                txt_records: std::collections::HashMap::new(),
            },
        ]
    }

    /// Simulated file transfer
    async fn simulate_file_transfer() -> f32 {
        for progress in (0..=100).step_by(10) {
            tokio::time::sleep(Duration::from_millis(200)).await;
            if progress == 100 {
                return 100.0;
            }
        }
        100.0
    }

    /// Push a notification onto the list
    fn add_notification(
        &mut self,
        title: String,
        message: String,
        notification_type: messages::NotificationType,
    ) {
        let notification = messages::NotificationMessage {
            title,
            content: message,
            notification_type,
            duration_ms: Some(3000),
        };
        
        self.notifications.push(notification);
        
        // Keep only the last 5 notifications
        if self.notifications.len() > 5 {
            self.notifications.remove(0);
        }
    }
}

/// Window icon decoded from the embedded PNG at startup.
fn window_icon() -> Option<window::Icon> {
    let img = image::load_from_memory(include_bytes!("../../assets/icon.png")).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    window::icon::from_rgba(img.into_raw(), w, h).ok()
}

/// Main entry point that launches the application
pub fn run() -> iced::Result {
    std::env::set_var("WGPU_BACKEND", "dx12");
    std::env::set_var("WGPU_VALIDATION", "0");

    let settings = Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            position: iced::window::Position::Centered,
            resizable: true,
            decorations: true,
            transparent: false,
            icon: window_icon(),
            ..Default::default()
        },
        default_font: iced::Font::DEFAULT,
        default_text_size: iced::Pixels(14.0),
        antialiasing: true,
        ..Default::default()
    };

    AirWinApp::run(settings)
}

/// Launch the AirWin application with the provided services
pub async fn run_app(
    _services: std::sync::Arc<crate::AirWinServices>,
) -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("WGPU_BACKEND", "dx12");
    std::env::set_var("WGPU_VALIDATION", "0");

    let settings = Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1200.0, 800.0),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            position: iced::window::Position::Centered,
            resizable: true,
            decorations: true,
            transparent: false,
            icon: window_icon(),
            ..Default::default()
        },
        antialiasing: true,
        default_font: iced::Font::DEFAULT,
        default_text_size: iced::Pixels(14.0),
        ..Default::default()
    };
    
    AirWinApp::run(settings)?;
    Ok(())
}

/// Helper macro: column with spacing
#[macro_export]
macro_rules! spaced {
    ($spacing:expr, $($element:expr),+ $(,)?) => {
        iced::widget::column![$($element),+].spacing($spacing)
    };
}

/// Helper macro: row with spacing
#[macro_export]
macro_rules! spaced_row {
    ($spacing:expr, $($element:expr),+ $(,)?) => {
        iced::widget::row![$($element),+].spacing($spacing)
    };
}
