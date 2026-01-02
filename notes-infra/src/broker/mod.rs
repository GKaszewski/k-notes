//! Message broker adapters for various backends.
//!
//! This module provides implementations of the `MessageBroker` port
//! for different messaging backends.

#[cfg(feature = "broker-nats")]
pub mod nats;
