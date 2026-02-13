// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Example: Fetch posts from Reddit using nexcore-social.
//!
//! ## Prerequisites
//!
//! 1. Register a Reddit app at https://www.reddit.com/prefs/apps
//!    - Select "script" type
//!    - Note the client_id (under app name) and client_secret
//!
//! 2. Set environment variables:
//!    - REDDIT_CLIENT_ID
//!    - REDDIT_CLIENT_SECRET
//!    - REDDIT_USERNAME
//!    - REDDIT_PASSWORD
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example reddit_fetch
//! cargo run --example reddit_fetch -- --subreddit investing --limit 25
//! ```

use nexcore_social::{RedditClient, RedditConfig, SocialError};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line args
    let args: Vec<String> = env::args().collect();
    let subreddit = args
        .iter()
        .position(|a| a == "--subreddit")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("wallstreetbets");
    let limit: u32 = args
        .iter()
        .position(|a| a == "--limit")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    // Load config from environment
    let config = RedditConfig::new(
        env::var("REDDIT_CLIENT_ID").map_err(|_| "REDDIT_CLIENT_ID not set")?,
        env::var("REDDIT_CLIENT_SECRET").map_err(|_| "REDDIT_CLIENT_SECRET not set")?,
        env::var("REDDIT_USERNAME").map_err(|_| "REDDIT_USERNAME not set")?,
        env::var("REDDIT_PASSWORD").map_err(|_| "REDDIT_PASSWORD not set")?,
        "nexcore-social:1.0.0 (by /u/nexvigilant)",
    );

    println!("🔗 Connecting to Reddit API...");
    println!("   Rate limit available: {} tokens", 60);

    // Create client
    let mut client = RedditClient::new(config)?;

    // Authenticate
    println!("🔐 Authenticating...");
    client.authenticate().await?;
    println!("   ✅ Authenticated successfully");

    // Fetch subreddit info
    println!("\n📊 Fetching r/{} metadata...", subreddit);
    match client.get_subreddit(subreddit).await {
        Ok(sub) => {
            println!("   Subscribers: {:>12}", format_number(sub.subscribers));
            println!("   Description: {}", truncate(&sub.public_description, 60));
        }
        Err(SocialError::NotFound(_)) => {
            println!("   ⚠️  Subreddit not found");
        }
        Err(e) => return Err(e.into()),
    }

    // Fetch hot posts
    println!("\n🔥 Fetching {} hot posts from r/{}...", limit, subreddit);
    let posts = client.get_hot_posts(subreddit, limit).await?;

    println!("\n{:─<80}", "");
    println!(
        "{:<50} {:>10} {:>10} {:>8}",
        "TITLE", "SCORE", "COMMENTS", "RATIO"
    );
    println!("{:─<80}", "");

    for post in &posts {
        println!(
            "{:<50} {:>10} {:>10} {:>7.0}%",
            truncate(&post.title, 48),
            format_number(post.score),
            format_number(post.num_comments),
            post.upvote_ratio * 100.0
        );
    }

    println!("{:─<80}", "");
    println!("\n📈 Summary:");
    println!("   Posts fetched: {}", posts.len());
    println!(
        "   Avg score: {:.0}",
        posts.iter().map(|p| p.score as f64).sum::<f64>() / posts.len() as f64
    );
    println!(
        "   Avg comments: {:.0}",
        posts.iter().map(|p| p.num_comments as f64).sum::<f64>() / posts.len() as f64
    );
    println!(
        "   Rate limit remaining: {} tokens",
        client.rate_limit_available()
    );

    Ok(())
}

/// Format large numbers with K/M suffixes.
fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

/// Truncate string to max length with ellipsis.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
