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
                    "Field 2/6 — Name: e.g. www (subdomain)".to_string()
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
#[allow(clippy::too_many_arguments)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_type_transient_error() {
        let status = StatusMessage::Error("test error".to_string());
        assert_eq!(status.status_type(), StatusType::Transient);
    }

    #[test]
    fn test_status_type_transient_operation_result() {
        let status = StatusMessage::OperationResult("Created A for example".to_string());
        assert_eq!(status.status_type(), StatusType::Transient);
    }

    #[test]
    fn test_status_type_transient_initializing() {
        let status = StatusMessage::Initializing;
        assert_eq!(status.status_type(), StatusType::Transient);
    }

    #[test]
    fn test_status_type_transient_loading_records() {
        let status = StatusMessage::LoadingRecords;
        assert_eq!(status.status_type(), StatusType::Transient);
    }

    #[test]
    fn test_status_type_contextual_view_help() {
        let status =
            StatusMessage::ViewHelp(ViewHelpContext::DeleteConfirmation);
        assert_eq!(status.status_type(), StatusType::Contextual);
    }

    #[test]
    fn test_status_type_contextual_form_field_help() {
        let status = StatusMessage::FormFieldHelp {
            context: FormFieldContext::Name,
            form_type: "A".to_string(),
            form_proxied: "false".to_string(),
            is_editing: false,
        };
        assert_eq!(status.status_type(), StatusType::Contextual);
    }

    #[test]
    fn test_status_type_contextual_record_list_help() {
        let status = StatusMessage::RecordListHelp {
            position: 1,
            total: 5,
            record_name: "example.com".to_string(),
        };
        assert_eq!(status.status_type(), StatusType::Contextual);
    }

    #[test]
    fn test_status_type_contextual_empty_list_help() {
        let status = StatusMessage::EmptyListHelp;
        assert_eq!(status.status_type(), StatusType::Contextual);
    }

    #[test]
    fn test_is_transient_error() {
        assert!(StatusMessage::is_transient("Error: something went wrong"));
    }

    #[test]
    fn test_is_transient_created() {
        assert!(StatusMessage::is_transient("Created A for example"));
    }

    #[test]
    fn test_is_transient_updated() {
        assert!(StatusMessage::is_transient("Updated A for example"));
    }

    #[test]
    fn test_is_transient_deleted() {
        assert!(StatusMessage::is_transient("Deleted A for example"));
    }

    #[test]
    fn test_is_transient_failed() {
        assert!(StatusMessage::is_transient("Failed: API error"));
    }

    #[test]
    fn test_is_transient_selected() {
        assert!(StatusMessage::is_transient("Selected IP address"));
    }

    #[test]
    fn test_is_transient_cancelled() {
        assert!(StatusMessage::is_transient("Cancelled operation"));
    }

    #[test]
    fn test_is_not_transient_contextual() {
        assert!(!StatusMessage::is_transient(
            "Enter: confirm deletion | Esc: cancel"
        ));
    }

    #[test]
    fn test_is_not_transient_record_list() {
        assert!(!StatusMessage::is_transient(
            "1 of 5 — example.com | E: edit | D: delete | R: refresh | C: create | Q: quit"
        ));
    }

    #[test]
    fn test_render_initializing() {
        let status = StatusMessage::Initializing;
        assert_eq!(status.render(), "Initializing...");
    }

    #[test]
    fn test_render_loading_records() {
        let status = StatusMessage::LoadingRecords;
        assert_eq!(status.render(), "Loading DNS records...");
    }

    #[test]
    fn test_render_error() {
        let status = StatusMessage::Error("connection failed".to_string());
        assert_eq!(status.render(), "Error: connection failed");
    }

    #[test]
    fn test_render_operation_result() {
        let status = StatusMessage::OperationResult("Created A record".to_string());
        assert_eq!(status.render(), "Created A record");
    }

    #[test]
    fn test_render_delete_confirmation_help() {
        let status = StatusMessage::ViewHelp(ViewHelpContext::DeleteConfirmation);
        assert_eq!(
            status.render(),
            "Enter: confirm deletion | Esc: cancel"
        );
    }

    #[test]
    fn test_render_ip_selector_help() {
        let status = StatusMessage::ViewHelp(ViewHelpContext::IpSelector);
        assert_eq!(
            status.render(),
            "↑↓: navigate | Enter: select IP | Esc: back to form"
        );
    }

    #[test]
    fn test_render_form_field_type() {
        let status = StatusMessage::FormFieldHelp {
            context: FormFieldContext::Type,
            form_type: "A".to_string(),
            form_proxied: "false".to_string(),
            is_editing: false,
        };
        assert_eq!(
            status.render(),
            "Field 1/6 — Type: A | Space: cycle types"
        );
    }

    #[test]
    fn test_render_form_field_name() {
        let status = StatusMessage::FormFieldHelp {
            context: FormFieldContext::Name,
            form_type: "A".to_string(),
            form_proxied: "false".to_string(),
            is_editing: false,
        };
        assert_eq!(
            status.render(),
            "Field 2/6 — Name: e.g. www (subdomain)"
        );
    }

    #[test]
    fn test_render_form_field_content() {
        let status = StatusMessage::FormFieldHelp {
            context: FormFieldContext::Content,
            form_type: "A".to_string(),
            form_proxied: "false".to_string(),
            is_editing: false,
        };
        assert_eq!(
            status.render(),
            "Field 3/6 — IP Address | Space: open selector | Type: enter IP"
        );
    }

    #[test]
    fn test_render_form_field_ttl() {
        let status = StatusMessage::FormFieldHelp {
            context: FormFieldContext::Ttl,
            form_type: "A".to_string(),
            form_proxied: "false".to_string(),
            is_editing: false,
        };
        assert_eq!(status.render(), "Field 4/6 — TTL: seconds (1 = auto)");
    }

    #[test]
    fn test_render_form_field_proxied_true() {
        let status = StatusMessage::FormFieldHelp {
            context: FormFieldContext::Proxied,
            form_type: "A".to_string(),
            form_proxied: "true".to_string(),
            is_editing: false,
        };
        assert_eq!(
            status.render(),
            "Field 5/6 — Proxied: Orange cloud ON"
        );
    }

    #[test]
    fn test_render_form_field_proxied_false() {
        let status = StatusMessage::FormFieldHelp {
            context: FormFieldContext::Proxied,
            form_type: "A".to_string(),
            form_proxied: "false".to_string(),
            is_editing: false,
        };
        assert_eq!(
            status.render(),
            "Field 5/6 — Proxied: Grey cloud OFF"
        );
    }

    #[test]
    fn test_render_form_field_submit_create() {
        let status = StatusMessage::FormFieldHelp {
            context: FormFieldContext::Submit,
            form_type: "A".to_string(),
            form_proxied: "false".to_string(),
            is_editing: false,
        };
        assert_eq!(
            status.render(),
            "Field 6/6 — Press Enter to Create record"
        );
    }

    #[test]
    fn test_render_form_field_submit_edit() {
        let status = StatusMessage::FormFieldHelp {
            context: FormFieldContext::Submit,
            form_type: "A".to_string(),
            form_proxied: "false".to_string(),
            is_editing: true,
        };
        assert_eq!(
            status.render(),
            "Field 6/6 — Press Enter to Save record"
        );
    }

    #[test]
    fn test_render_record_list_help() {
        let status = StatusMessage::RecordListHelp {
            position: 3,
            total: 10,
            record_name: "test.example.com".to_string(),
        };
        assert_eq!(
            status.render(),
            "3 of 10 — test.example.com | E: edit | D: delete | R: refresh | C: create | Q: quit"
        );
    }

    #[test]
    fn test_render_empty_list_help() {
        let status = StatusMessage::EmptyListHelp;
        assert_eq!(
            status.render(),
            "No records | C: create your first DNS record | Q: quit"
        );
    }

    #[test]
    fn test_generate_contextual_status_delete_view() {
        let result = generate_contextual_status(
            &AppView::Delete,
            0,
            "A",
            "false",
            false,
            5,
            0,
            Some("example.com"),
        );
        assert!(matches!(
            result,
            StatusMessage::ViewHelp(ViewHelpContext::DeleteConfirmation)
        ));
    }

    #[test]
    fn test_generate_contextual_status_ip_selector_view() {
        let result = generate_contextual_status(
            &AppView::IpSelect,
            0,
            "A",
            "false",
            false,
            5,
            0,
            Some("example.com"),
        );
        assert!(matches!(
            result,
            StatusMessage::ViewHelp(ViewHelpContext::IpSelector)
        ));
    }

    #[test]
    fn test_generate_contextual_status_create_view_first_field() {
        let result = generate_contextual_status(
            &AppView::Create,
            0,
            "A",
            "false",
            false,
            5,
            0,
            Some("example.com"),
        );
        match result {
            StatusMessage::FormFieldHelp {
                context: FormFieldContext::Type,
                form_type,
                ..
            } => {
                assert_eq!(form_type, "A");
            }
            _ => panic!("Expected FormFieldHelp with Type context"),
        }
    }

    #[test]
    fn test_generate_contextual_status_create_view_name_field() {
        let result = generate_contextual_status(
            &AppView::Create,
            1,
            "AAAA",
            "true",
            false,
            5,
            0,
            Some("example.com"),
        );
        match result {
            StatusMessage::FormFieldHelp {
                context: FormFieldContext::Name,
                form_type,
                form_proxied,
                ..
            } => {
                assert_eq!(form_type, "AAAA");
                assert_eq!(form_proxied, "true");
            }
            _ => panic!("Expected FormFieldHelp with Name context"),
        }
    }

    #[test]
    fn test_generate_contextual_status_edit_view_submit_field() {
        let result = generate_contextual_status(
            &AppView::Edit,
            5,
            "CNAME",
            "false",
            true,
            5,
            0,
            Some("example.com"),
        );
        match result {
            StatusMessage::FormFieldHelp {
                context: FormFieldContext::Submit,
                is_editing,
                ..
            } => {
                assert!(is_editing);
            }
            _ => panic!("Expected FormFieldHelp with Submit context"),
        }
    }

    #[test]
    fn test_generate_contextual_status_list_view_with_records() {
        let result = generate_contextual_status(
            &AppView::List,
            0,
            "A",
            "false",
            false,
            5,
            2,
            Some("test.example.com"),
        );
        match result {
            StatusMessage::RecordListHelp {
                position,
                total,
                record_name,
            } => {
                assert_eq!(position, 3); // idx + 1
                assert_eq!(total, 5);
                assert_eq!(record_name, "test.example.com");
            }
            _ => panic!("Expected RecordListHelp"),
        }
    }

    #[test]
    fn test_generate_contextual_status_list_view_empty() {
        let result = generate_contextual_status(
            &AppView::List,
            0,
            "A",
            "false",
            false,
            0,
            0,
            None,
        );
        assert!(matches!(result, StatusMessage::EmptyListHelp));
    }

    #[test]
    fn test_generate_contextual_status_list_view_invalid_selection() {
        // Selection index beyond record count
        let result = generate_contextual_status(
            &AppView::List,
            0,
            "A",
            "false",
            false,
            3,
            5, // idx >= record_count
            Some("example.com"),
        );
        assert!(matches!(result, StatusMessage::EmptyListHelp));
    }

    #[test]
    fn test_generate_contextual_status_list_view_unknown_record_name() {
        let result = generate_contextual_status(
            &AppView::List,
            0,
            "A",
            "false",
            false,
            1,
            0,
            None, // No record name
        );
        match result {
            StatusMessage::RecordListHelp { record_name, .. } => {
                assert_eq!(record_name, "Unknown");
            }
            _ => panic!("Expected RecordListHelp"),
        }
    }

    #[test]
    fn test_generate_contextual_status_form_fallback() {
        // form_focus beyond 0-5 range should fallback to Name context
        let result = generate_contextual_status(
            &AppView::Create,
            10, // out of range
            "A",
            "false",
            false,
            5,
            0,
            Some("example.com"),
        );
        match result {
            StatusMessage::FormFieldHelp {
                context: FormFieldContext::Name,
                ..
            } => {
                // Expected fallback behavior
            }
            _ => panic!("Expected FormFieldHelp with Name context as fallback"),
        }
    }
}
