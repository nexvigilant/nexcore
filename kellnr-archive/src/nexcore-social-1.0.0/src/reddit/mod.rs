// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Reddit API client for value signal detection.
//!
//! ## Authentication
//!
//! Uses OAuth2 "script" flow for personal use applications.
//! Register app at: https://www.reddit.com/prefs/apps
//!
//! ## Rate Limits
//!
//! Free tier: 60 requests/minute (3600/hour)
//!
//! ## Primitive Grounding
//!
//! - API Call → (causality)
//! - Response → μ (mapping)
//! - Posts → σ (sequence)
//! - Score → N (quantity)
//! - Timestamp → σ + N

pub mod client;
pub mod types;

pub use client::RedditClient;
pub use types::{Comment, Post, RedditConfig, Subreddit};
