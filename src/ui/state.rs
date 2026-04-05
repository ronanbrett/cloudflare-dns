/// Application state management.
///
/// This module contains the core state types used throughout the application,
/// including the shared AppState and iocraft Props definitions.
use iocraft::prelude::*;
use std::sync::{Arc, Mutex};

use crate::api::{CloudflareClient, DnsCache, DnsRecord};

/// Shared application state wrapped in Arc for thread-safe access.
///
/// This state is shared across multiple async tasks and UI components.
pub struct AppState {
    /// Cloudflare API client
    pub client: Arc<CloudflareClient>,
    /// Zone name (resolved from zone ID on startup)
    pub zone_name: Mutex<String>,
    /// Current DNS records
    pub records: Mutex<Vec<DnsRecord>>,
    /// Unique IPs extracted from A/AAAA records (for IP selector)
    pub existing_ips: Mutex<Vec<String>>,
    /// Cache for DNS records to reduce redundant API calls
    pub dns_cache: Mutex<DnsCache>,
}

impl AppState {
    /// Create a new AppState instance.
    pub fn new(api_token: String, zone_id: String) -> Self {
        Self {
            client: Arc::new(CloudflareClient::new(api_token, zone_id.clone())),
            zone_name: Mutex::new(zone_id),
            records: Mutex::new(Vec::new()),
            existing_ips: Mutex::new(Vec::new()),
            dns_cache: Mutex::new(DnsCache::with_default_ttl()),
        }
    }
}

/// Props passed to the root App component.
#[derive(Props, Clone)]
pub struct AppProps {
    /// Shared application state
    pub state: Arc<AppState>,
}

impl Default for AppProps {
    fn default() -> Self {
        Self {
            state: Arc::new(AppState::new(String::new(), String::new())),
        }
    }
}

/// Props for the FormField component.
#[derive(Default, Props)]
pub struct FormFieldProps {
    /// Field label
    pub label: String,
    /// Field value (state binding)
    pub value: Option<State<String>>,
    /// Whether this field has focus
    pub has_focus: bool,
    /// Optional suffix text (e.g., domain suffix)
    pub suffix: String,
    /// Whether this field should be editable as text input (false for cycled fields like Type/Proxied)
    pub is_editable: bool,
}

/// Props for the StatusBar component.
#[derive(Default, Props)]
pub struct StatusBarProps {
    /// Status message text
    pub message: String,
}

/// Represents the current view mode of the application.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum AppView {
    /// Main record list view
    #[default]
    List,
    /// Create new record form
    Create,
    /// Edit existing record form
    Edit,
    /// Delete confirmation dialog
    Delete,
    /// IP address selector
    IpSelect,
}
