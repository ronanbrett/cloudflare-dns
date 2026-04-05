use crate::state::AppView;

/// Represents the type of status message
#[derive(Debug, Clone, PartialEq)]
pub enum StatusType {
    /// Transient messages that should display briefly
    Transient,
    /// Persistent contextual help based on current view
    Contextual,
}

/// Context for form field help messages
#[derive(Debug, Clone)]
pub enum FormFieldContext {
    Type,
    Name,
    Content,
    Ttl,
    Proxied,
    Submit,
}

/// All possible status message variants
#[derive(Debug, Clone)]
pub enum StatusMessage {
    /// Initial loading state
    Initializing,
    /// Loading DNS records
    LoadingRecords,
    /// Error message
    Error(String),
    /// Operation results (created, updated, deleted, etc.)
    OperationResult(String),
    /// View-specific contextual help
    ViewHelp(ViewHelpContext),
    /// Form field help
    FormFieldHelp {
        context: FormFieldContext,
        form_type: String,
        form_proxied: String,
        is_editing: bool,
    },
    /// Record list navigation help
    RecordListHelp {
        position: usize,
        total: usize,
        record_name: String,
    },
    /// Empty list help
    EmptyListHelp,
}

/// View-specific help contexts
#[derive(Debug, Clone)]
pub enum ViewHelpContext {
    DeleteConfirmation,
    IpSelector,
}

impl StatusMessage {
    /// Determine if this status should be transient (auto-clearing)
    pub fn status_type(&self) -> StatusType {
        match self {
            StatusMessage::Error(_)
            | StatusMessage::OperationResult(_)
            | StatusMessage::Initializing
            | StatusMessage::LoadingRecords => StatusType::Transient,
            _ => StatusType::Contextual,
        }
    }

    /// Check if a status string represents a transient message
    pub fn is_transient(status_str: &str) -> bool {
        status_str.starts_with("Error:")
            || status_str.starts_with("Refreshing...")
            || status_str.starts_with("Created")
            || status_str.starts_with("Updated")
            || status_str.starts_with("Deleted")
            || status_str.starts_with("Failed")
            || status_str.starts_with("Selected")
            || status_str.starts_with("Cancelled")
    }

    /// Render the status message to a display string
    pub fn render(&self) -> String {
        match self {
            StatusMessage::Initializing => "Initializing...".to_string(),
            StatusMessage::LoadingRecords => "Loading DNS records...".to_string(),
            StatusMessage::Error(msg) => format!("Error: {}", msg),
            StatusMessage::OperationResult(msg) => msg.clone(),
            StatusMessage::ViewHelp(ViewHelpContext::DeleteConfirmation) => {
                "Enter: confirm deletion | Esc: cancel".to_string()
            }
            StatusMessage::ViewHelp(ViewHelpContext::IpSelector) => {
                "↑↓: navigate | Enter: select IP | Esc: back to form".to_string()
            }
            StatusMessage::FormFieldHelp {
                context,
                form_type,
                form_proxied,
                is_editing,
            } => match context {
                FormFieldContext::Type => {
                    format!("Field 1/6 — Type: {} | Space: cycle types", form_type)
                }
                FormFieldContext::Name => {
                    "Field 2/6 — Name: e.g. app.example.com".to_string()
                }
                FormFieldContext::Content => {
                    "Field 3/6 — IP Address | Space: open selector | Type: enter IP".to_string()
                }
                FormFieldContext::Ttl => {
                    "Field 4/6 — TTL: seconds (1 = auto)".to_string()
                }
                FormFieldContext::Proxied => {
                    let proxied_status = if form_proxied == "true" {
                        "Orange cloud ON"
                    } else {
                        "Grey cloud OFF"
                    };
                    format!("Field 5/6 — Proxied: {}", proxied_status)
                }
                FormFieldContext::Submit => {
                    let action = if *is_editing { "Save" } else { "Create" };
                    format!("Field 6/6 — Press Enter to {} record", action)
                }
            },
            StatusMessage::RecordListHelp {
                position,
                total,
                record_name,
            } => {
                format!(
                    "{} of {} — {} | E: edit | D: delete | R: refresh | C: create | Q: quit",
                    position, total, record_name
                )
            }
            StatusMessage::EmptyListHelp => {
                "No records | C: create your first DNS record | Q: quit".to_string()
            }
        }
    }
}

/// Generate the appropriate contextual status message based on current state
pub fn generate_contextual_status(
    view: &AppView,
    form_focus: usize,
    form_type: &str,
    form_proxied: &str,
    is_editing: bool,
    record_count: usize,
    selected_record_idx: usize,
    selected_record_name: Option<&str>,
) -> StatusMessage {
    match view {
        AppView::Delete => {
            StatusMessage::ViewHelp(ViewHelpContext::DeleteConfirmation)
        }
        AppView::IpSelect => {
            StatusMessage::ViewHelp(ViewHelpContext::IpSelector)
        }
        AppView::Create | AppView::Edit => {
            let context = match form_focus {
                0 => FormFieldContext::Type,
                1 => FormFieldContext::Name,
                2 => FormFieldContext::Content,
                3 => FormFieldContext::Ttl,
                4 => FormFieldContext::Proxied,
                5 => FormFieldContext::Submit,
                _ => FormFieldContext::Name, // fallback
            };
            StatusMessage::FormFieldHelp {
                context,
                form_type: form_type.to_string(),
                form_proxied: form_proxied.to_string(),
                is_editing,
            }
        }
        AppView::List => {
            if record_count > 0 && selected_record_idx < record_count {
                StatusMessage::RecordListHelp {
                    position: selected_record_idx + 1,
                    total: record_count,
                    record_name: selected_record_name.unwrap_or("Unknown").to_string(),
                }
            } else {
                StatusMessage::EmptyListHelp
            }
        }
    }
}
