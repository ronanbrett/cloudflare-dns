#![allow(dead_code)]

/// Background task definitions for async operations.
///
/// This module contains all the async tasks that interact with the Cloudflare API
/// and update application state. Parameter objects are used to reduce function
/// argument counts and improve readability.
pub mod delete_task;
pub mod fetch_task;
pub mod submit_task;

#[allow(unused_imports)]
pub use delete_task::{DeleteParams, delete_task};
#[allow(unused_imports)]
pub use fetch_task::{fetch_all, refresh_task};
#[allow(unused_imports)]
pub use submit_task::{SubmitParams, submit_task};
