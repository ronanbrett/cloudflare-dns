use iocraft::prelude::*;
use std::sync::{Arc, Mutex};

use crate::cloudflare::{CloudflareClient, DnsRecord};

pub struct AppState {
    pub client: Arc<CloudflareClient>,
    pub zone_name: Mutex<String>,
    pub records: Mutex<Vec<DnsRecord>>,
    pub existing_ips: Mutex<Vec<String>>,
}

impl AppState {
    pub fn new(api_token: String, zone_id: String) -> Self {
        Self {
            client: Arc::new(CloudflareClient::new(api_token, zone_id.clone())),
            zone_name: Mutex::new(zone_id),
            records: Mutex::new(Vec::new()),
            existing_ips: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Props, Clone)]
pub struct AppProps {
    pub state: Arc<AppState>,
}

impl Default for AppProps {
    fn default() -> Self {
        Self {
            state: Arc::new(AppState::new(String::new(), String::new())),
        }
    }
}

#[derive(Default, Props)]
pub struct FormFieldProps {
    pub label: String,
    pub value: Option<State<String>>,
    pub has_focus: bool,
    pub suffix: String,
}

#[derive(Default, Props)]
pub struct StatusBarProps {
    pub message: String,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum AppView {
    #[default]
    List,
    Create,
    Edit,
    Delete,
    IpSelect,
}
