use iocraft::prelude::*;
use std::sync::{Arc, Mutex};

use crate::cloudflare::{CloudflareClient, DnsRecord};

pub struct AppState {
    pub client: Arc<CloudflareClient>,
    pub records: Mutex<Vec<DnsRecord>>,
    pub existing_ips: Mutex<Vec<String>>,
}

impl AppState {
    pub fn new(api_token: String, zone_id: String) -> Self {
        Self {
            client: Arc::new(CloudflareClient::new(api_token, zone_id)),
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
}

#[derive(Default, Props)]
pub struct StatusBarProps {
    pub message: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AppView {
    List,
    Create,
    Edit,
    Delete,
    IpSelect,
}

impl Default for AppView {
    fn default() -> Self {
        Self::List
    }
}

#[derive(Clone)]
pub struct FormState {
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: String,
    pub proxied: String,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            record_type: "A".to_string(),
            name: String::new(),
            content: String::new(),
            ttl: "1".to_string(),
            proxied: "false".to_string(),
        }
    }
}
