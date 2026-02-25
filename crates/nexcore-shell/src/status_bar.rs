// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Status bar — system status indicators across form factors.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  Watch:   [Clock]                        [Battery]      │
//! │  Phone:   [Clock] [Network]   [Services] [Battery]      │
//! │  Desktop: [Clock] [Security] [Services] [Network] [Bat] │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Primitive Grounding
//!
//! - ν Frequency: Clock ticks, refresh rate
//! - N Quantity: Battery percentage, service count
//! - ς State: Power state, network state, security level
//! - ∂ Boundary: Layout constraints per form factor
//! - λ Location: Indicator positions in bar

use nexcore_compositor::surface::Rect;
use nexcore_pal::{FormFactor, PowerState};

/// Individual status indicator in the bar.
///
/// Tier: T2-P (ς State — each indicator represents system state)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusIndicator {
    /// Clock — displays current time.
    Clock {
        /// Hours (0-23).
        hours: u8,
        /// Minutes (0-59).
        minutes: u8,
        /// Whether to use 24-hour format.
        format_24h: bool,
    },
    /// Battery — power state indicator.
    Battery {
        /// Battery percentage (0-100), None if no battery.
        percent: Option<u8>,
        /// Whether the device is charging.
        charging: bool,
        /// Whether the battery is critically low.
        critical: bool,
    },
    /// Network — connectivity status.
    Network {
        /// Whether connected to a network.
        connected: bool,
        /// Signal strength (0-4 bars), None if wired/unknown.
        signal_bars: Option<u8>,
        /// Network name (SSID or "Ethernet").
        name: Option<String>,
    },
    /// Running services count.
    Services {
        /// Number of running system services.
        running: u32,
        /// Number of total registered services.
        total: u32,
    },
    /// Security classification level.
    Security {
        /// Current security level label.
        level: String,
        /// Numeric level (0-4, higher = more secure).
        ordinal: u8,
    },
    /// Notification count badge.
    NotificationBadge {
        /// Number of unread notifications.
        count: u32,
    },
}

impl StatusIndicator {
    /// Get the indicator kind name.
    pub fn kind_name(&self) -> &str {
        match self {
            Self::Clock { .. } => "clock",
            Self::Battery { .. } => "battery",
            Self::Network { .. } => "network",
            Self::Services { .. } => "services",
            Self::Security { .. } => "security",
            Self::NotificationBadge { .. } => "notification_badge",
        }
    }

    /// Get a text representation of the indicator.
    pub fn display_text(&self) -> String {
        match self {
            Self::Clock {
                hours,
                minutes,
                format_24h,
            } => {
                if *format_24h {
                    format!("{hours:02}:{minutes:02}")
                } else {
                    let (h, ampm) = if *hours == 0 {
                        (12, "AM")
                    } else if *hours < 12 {
                        (*hours, "AM")
                    } else if *hours == 12 {
                        (12, "PM")
                    } else {
                        (hours - 12, "PM")
                    };
                    format!("{h}:{minutes:02} {ampm}")
                }
            }
            Self::Battery {
                percent,
                charging,
                critical,
            } => {
                let pct = percent.map_or_else(|| "AC".to_string(), |p| format!("{p}%"));
                if *critical {
                    format!("!{pct}")
                } else if *charging {
                    format!("+{pct}")
                } else {
                    pct
                }
            }
            Self::Network {
                connected,
                signal_bars,
                name,
            } => {
                if !connected {
                    return "No Network".to_string();
                }
                let bars = signal_bars.map_or(String::new(), |b| {
                    let filled = b.min(4);
                    let empty = 4 - filled;
                    format!(
                        "{}{}",
                        "|".repeat(filled as usize),
                        ".".repeat(empty as usize)
                    )
                });
                match name {
                    Some(n) => format!("{bars} {n}"),
                    None => bars,
                }
            }
            Self::Services { running, total } => format!("{running}/{total} svc"),
            Self::Security { level, .. } => level.clone(),
            Self::NotificationBadge { count } => {
                if *count > 99 {
                    "99+".to_string()
                } else {
                    format!("{count}")
                }
            }
        }
    }

    /// Create a clock indicator from hours and minutes.
    pub const fn clock(hours: u8, minutes: u8, format_24h: bool) -> Self {
        Self::Clock {
            hours,
            minutes,
            format_24h,
        }
    }

    /// Create a battery indicator from a `PowerState`.
    pub fn from_power_state(state: PowerState) -> Self {
        Self::Battery {
            percent: state.battery_pct(),
            charging: state.is_charging(),
            critical: state.is_critical(),
        }
    }

    /// Create a services indicator.
    pub const fn services(running: u32, total: u32) -> Self {
        Self::Services { running, total }
    }

    /// Create a security indicator.
    pub fn security(level: impl Into<String>, ordinal: u8) -> Self {
        Self::Security {
            level: level.into(),
            ordinal,
        }
    }

    /// Create a network indicator.
    pub fn network(connected: bool, signal_bars: Option<u8>, name: Option<String>) -> Self {
        Self::Network {
            connected,
            signal_bars,
            name,
        }
    }

    /// Create a notification badge.
    pub const fn notification_badge(count: u32) -> Self {
        Self::NotificationBadge { count }
    }
}

/// Positioned indicator in the status bar.
///
/// Tier: T2-C (λ + ∂ — positioned element within bar bounds)
#[derive(Debug, Clone)]
pub struct PositionedIndicator {
    /// The indicator.
    pub indicator: StatusIndicator,
    /// Position within the status bar.
    pub bounds: Rect,
}

/// Status bar configuration for each form factor.
///
/// Tier: T2-C (∂ + N — which indicators appear, in what order)
#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    /// Which indicators to show (in order, left to right).
    pub indicator_kinds: Vec<IndicatorSlot>,
    /// Bar height in pixels.
    pub height: u32,
    /// Form factor this config targets.
    pub form_factor: FormFactor,
}

/// Slot defining what appears in the status bar.
///
/// Tier: T2-P (σ + ∂ — ordered bounded slot)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorSlot {
    /// Clock display.
    Clock,
    /// Battery indicator.
    Battery,
    /// Network status.
    Network,
    /// Running services.
    Services,
    /// Security level.
    Security,
    /// Notification badge.
    Notifications,
}

impl StatusBarConfig {
    /// Watch config: compact bar with clock + battery.
    pub fn watch() -> Self {
        Self {
            indicator_kinds: vec![IndicatorSlot::Clock, IndicatorSlot::Battery],
            height: 40,
            form_factor: FormFactor::Watch,
        }
    }

    /// Phone config: clock, network, services, battery.
    pub fn phone() -> Self {
        Self {
            indicator_kinds: vec![
                IndicatorSlot::Clock,
                IndicatorSlot::Notifications,
                IndicatorSlot::Network,
                IndicatorSlot::Battery,
            ],
            height: 80,
            form_factor: FormFactor::Phone,
        }
    }

    /// Desktop config: clock, security, services, network, battery.
    pub fn desktop() -> Self {
        Self {
            indicator_kinds: vec![
                IndicatorSlot::Clock,
                IndicatorSlot::Security,
                IndicatorSlot::Services,
                IndicatorSlot::Network,
                IndicatorSlot::Battery,
            ],
            height: 48,
            form_factor: FormFactor::Desktop,
        }
    }

    /// Select config for a form factor.
    pub fn for_form_factor(ff: FormFactor) -> Self {
        match ff {
            FormFactor::Watch => Self::watch(),
            FormFactor::Phone => Self::phone(),
            FormFactor::Desktop => Self::desktop(),
            _ => Self::desktop(),
        }
    }
}

/// The status bar — renders system status indicators.
///
/// Tier: T3 (ν + N + ς + ∂ + λ — full status bar integration)
///
/// Reads system state and positions indicators within the bar region.
pub struct StatusBar {
    /// Configuration for this form factor.
    config: StatusBarConfig,
    /// Current indicator values.
    indicators: Vec<StatusIndicator>,
    /// Bar bounds on screen.
    bounds: Rect,
    /// Whether time is in 24-hour format.
    use_24h: bool,
}

impl StatusBar {
    /// Create a new status bar for a form factor with given bounds.
    pub fn new(form_factor: FormFactor, bounds: Rect) -> Self {
        let config = StatusBarConfig::for_form_factor(form_factor);
        let indicators = Vec::new();
        Self {
            config,
            indicators,
            bounds,
            use_24h: true,
        }
    }

    /// Set whether to use 24-hour time format.
    #[must_use]
    pub const fn with_24h(mut self, use_24h: bool) -> Self {
        self.use_24h = use_24h;
        self
    }

    /// Update the clock to the given time.
    pub fn update_clock(&mut self, hours: u8, minutes: u8) {
        self.set_indicator(StatusIndicator::clock(hours, minutes, self.use_24h));
    }

    /// Update the battery from a `PowerState`.
    pub fn update_battery(&mut self, state: PowerState) {
        self.set_indicator(StatusIndicator::from_power_state(state));
    }

    /// Update the network status.
    pub fn update_network(
        &mut self,
        connected: bool,
        signal_bars: Option<u8>,
        name: Option<String>,
    ) {
        self.set_indicator(StatusIndicator::network(connected, signal_bars, name));
    }

    /// Update running services count.
    pub fn update_services(&mut self, running: u32, total: u32) {
        self.set_indicator(StatusIndicator::services(running, total));
    }

    /// Update security level.
    pub fn update_security(&mut self, level: impl Into<String>, ordinal: u8) {
        self.set_indicator(StatusIndicator::security(level, ordinal));
    }

    /// Update notification badge count.
    pub fn update_notifications(&mut self, count: u32) {
        self.set_indicator(StatusIndicator::notification_badge(count));
    }

    /// Set or replace an indicator by kind.
    fn set_indicator(&mut self, indicator: StatusIndicator) {
        let kind = indicator.kind_name();
        if let Some(existing) = self.indicators.iter_mut().find(|i| i.kind_name() == kind) {
            *existing = indicator;
        } else {
            self.indicators.push(indicator);
        }
    }

    /// Get an indicator by kind name.
    pub fn get_indicator(&self, kind: &str) -> Option<&StatusIndicator> {
        self.indicators.iter().find(|i| i.kind_name() == kind)
    }

    /// Get all current indicators.
    pub fn indicators(&self) -> &[StatusIndicator] {
        &self.indicators
    }

    /// Get the number of indicators currently set.
    pub fn indicator_count(&self) -> usize {
        self.indicators.len()
    }

    /// Compute positioned indicators for rendering.
    ///
    /// Distributes indicators evenly across the bar width,
    /// following the config's slot order.
    #[allow(clippy::cast_possible_wrap)]
    pub fn layout_indicators(&self) -> Vec<PositionedIndicator> {
        let slots = &self.config.indicator_kinds;
        if slots.is_empty() {
            return Vec::new();
        }

        let slot_width = self.bounds.width / slots.len() as u32;
        let mut positioned = Vec::new();

        for (idx, slot) in slots.iter().enumerate() {
            let indicator = self.indicator_for_slot(*slot);
            if let Some(ind) = indicator {
                let x = self.bounds.x + (idx as u32 * slot_width) as i32;
                positioned.push(PositionedIndicator {
                    indicator: ind,
                    bounds: Rect::new(x, self.bounds.y, slot_width, self.bounds.height),
                });
            }
        }

        positioned
    }

    /// Find the current indicator matching a slot kind.
    fn indicator_for_slot(&self, slot: IndicatorSlot) -> Option<StatusIndicator> {
        let kind_name = match slot {
            IndicatorSlot::Clock => "clock",
            IndicatorSlot::Battery => "battery",
            IndicatorSlot::Network => "network",
            IndicatorSlot::Services => "services",
            IndicatorSlot::Security => "security",
            IndicatorSlot::Notifications => "notification_badge",
        };
        self.indicators
            .iter()
            .find(|i| i.kind_name() == kind_name)
            .cloned()
    }

    /// Get the bar bounds.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Get the form factor.
    pub fn form_factor(&self) -> FormFactor {
        self.config.form_factor
    }

    /// Get the bar height.
    pub fn height(&self) -> u32 {
        self.config.height
    }

    /// Get the configured indicator slots.
    pub fn slots(&self) -> &[IndicatorSlot] {
        &self.config.indicator_kinds
    }

    /// Get the display text for all set indicators.
    pub fn display_texts(&self) -> Vec<(&str, String)> {
        self.indicators
            .iter()
            .map(|i| (i.kind_name(), i.display_text()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── StatusIndicator display tests ──

    #[test]
    fn clock_24h_format() {
        let clock = StatusIndicator::clock(14, 30, true);
        assert_eq!(clock.display_text(), "14:30");
    }

    #[test]
    fn clock_12h_format_pm() {
        let clock = StatusIndicator::clock(14, 30, false);
        assert_eq!(clock.display_text(), "2:30 PM");
    }

    #[test]
    fn clock_12h_format_am() {
        let clock = StatusIndicator::clock(9, 5, false);
        assert_eq!(clock.display_text(), "9:05 AM");
    }

    #[test]
    fn clock_12h_noon() {
        let clock = StatusIndicator::clock(12, 0, false);
        assert_eq!(clock.display_text(), "12:00 PM");
    }

    #[test]
    fn clock_12h_midnight() {
        let clock = StatusIndicator::clock(0, 0, false);
        assert_eq!(clock.display_text(), "12:00 AM");
    }

    #[test]
    fn battery_normal() {
        let batt = StatusIndicator::from_power_state(PowerState::Battery { percent: 65 });
        assert_eq!(batt.display_text(), "65%");
    }

    #[test]
    fn battery_charging() {
        let batt = StatusIndicator::from_power_state(PowerState::Charging { percent: 80 });
        assert_eq!(batt.display_text(), "+80%");
    }

    #[test]
    fn battery_critical() {
        let batt = StatusIndicator::from_power_state(PowerState::Battery { percent: 5 });
        assert_eq!(batt.display_text(), "!5%");
    }

    #[test]
    fn battery_full() {
        let batt = StatusIndicator::from_power_state(PowerState::Full);
        assert_eq!(batt.display_text(), "+100%");
    }

    #[test]
    fn battery_ac_power() {
        let batt = StatusIndicator::from_power_state(PowerState::AcPower);
        assert_eq!(batt.display_text(), "AC");
    }

    #[test]
    fn network_connected() {
        let net = StatusIndicator::network(true, Some(3), Some("HomeWifi".to_string()));
        let text = net.display_text();
        assert!(text.contains("HomeWifi"));
        assert!(text.contains("|||."));
    }

    #[test]
    fn network_disconnected() {
        let net = StatusIndicator::network(false, None, None);
        assert_eq!(net.display_text(), "No Network");
    }

    #[test]
    fn network_full_signal() {
        let net = StatusIndicator::network(true, Some(4), None);
        assert_eq!(net.display_text(), "||||");
    }

    #[test]
    fn services_display() {
        let svc = StatusIndicator::services(5, 8);
        assert_eq!(svc.display_text(), "5/8 svc");
    }

    #[test]
    fn security_display() {
        let sec = StatusIndicator::security("Internal", 1);
        assert_eq!(sec.display_text(), "Internal");
    }

    #[test]
    fn notification_badge_normal() {
        let badge = StatusIndicator::notification_badge(7);
        assert_eq!(badge.display_text(), "7");
    }

    #[test]
    fn notification_badge_overflow() {
        let badge = StatusIndicator::notification_badge(150);
        assert_eq!(badge.display_text(), "99+");
    }

    // ── StatusIndicator kind names ──

    #[test]
    fn kind_names() {
        assert_eq!(StatusIndicator::clock(0, 0, true).kind_name(), "clock");
        assert_eq!(
            StatusIndicator::from_power_state(PowerState::Full).kind_name(),
            "battery"
        );
        assert_eq!(
            StatusIndicator::network(true, None, None).kind_name(),
            "network"
        );
        assert_eq!(StatusIndicator::services(0, 0).kind_name(), "services");
        assert_eq!(StatusIndicator::security("X", 0).kind_name(), "security");
        assert_eq!(
            StatusIndicator::notification_badge(0).kind_name(),
            "notification_badge"
        );
    }

    // ── StatusBarConfig tests ──

    #[test]
    fn watch_config() {
        let config = StatusBarConfig::watch();
        assert_eq!(config.indicator_kinds.len(), 2);
        assert_eq!(config.height, 40);
        assert_eq!(config.form_factor, FormFactor::Watch);
    }

    #[test]
    fn phone_config() {
        let config = StatusBarConfig::phone();
        assert_eq!(config.indicator_kinds.len(), 4);
        assert_eq!(config.height, 80);
    }

    #[test]
    fn desktop_config() {
        let config = StatusBarConfig::desktop();
        assert_eq!(config.indicator_kinds.len(), 5);
        assert_eq!(config.height, 48);
    }

    // ── StatusBar tests ──

    #[test]
    fn status_bar_creation() {
        let bar = StatusBar::new(FormFactor::Watch, Rect::new(0, 0, 450, 40));
        assert_eq!(bar.form_factor(), FormFactor::Watch);
        assert_eq!(bar.height(), 40);
        assert_eq!(bar.indicator_count(), 0);
    }

    #[test]
    fn update_clock() {
        let mut bar = StatusBar::new(FormFactor::Watch, Rect::new(0, 0, 450, 40));
        bar.update_clock(14, 30);
        assert_eq!(bar.indicator_count(), 1);

        let clock = bar.get_indicator("clock");
        assert!(clock.is_some());
        assert_eq!(
            clock.map(StatusIndicator::display_text),
            Some("14:30".to_string())
        );
    }

    #[test]
    fn update_battery() {
        let mut bar = StatusBar::new(FormFactor::Phone, Rect::new(0, 0, 1080, 80));
        bar.update_battery(PowerState::Battery { percent: 42 });
        assert_eq!(bar.indicator_count(), 1);

        let batt = bar.get_indicator("battery");
        assert!(batt.is_some());
        assert_eq!(
            batt.map(StatusIndicator::display_text),
            Some("42%".to_string())
        );
    }

    #[test]
    fn update_replaces_existing() {
        let mut bar = StatusBar::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 48));
        bar.update_clock(10, 0);
        bar.update_clock(10, 1);
        assert_eq!(bar.indicator_count(), 1); // replaced, not added

        let clock = bar.get_indicator("clock");
        assert_eq!(
            clock.map(StatusIndicator::display_text),
            Some("10:01".to_string())
        );
    }

    #[test]
    fn multiple_indicators() {
        let mut bar = StatusBar::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 48));
        bar.update_clock(9, 15);
        bar.update_battery(PowerState::Charging { percent: 80 });
        bar.update_network(true, Some(4), Some("Office".to_string()));
        bar.update_services(3, 5);
        bar.update_security("Confidential", 2);
        assert_eq!(bar.indicator_count(), 5);
    }

    #[test]
    fn layout_indicators_positioned() {
        let mut bar = StatusBar::new(FormFactor::Watch, Rect::new(0, 0, 450, 40));
        bar.update_clock(12, 0);
        bar.update_battery(PowerState::Battery { percent: 75 });

        let positioned = bar.layout_indicators();
        assert_eq!(positioned.len(), 2);

        // First slot = clock
        assert_eq!(positioned[0].indicator.kind_name(), "clock");
        assert_eq!(positioned[0].bounds.x, 0);

        // Second slot = battery
        assert_eq!(positioned[1].indicator.kind_name(), "battery");
        assert!(positioned[1].bounds.x > 0);
    }

    #[test]
    fn layout_empty_bar() {
        let bar = StatusBar::new(FormFactor::Watch, Rect::new(0, 0, 450, 40));
        let positioned = bar.layout_indicators();
        assert!(positioned.is_empty());
    }

    #[test]
    fn display_texts() {
        let mut bar = StatusBar::new(FormFactor::Watch, Rect::new(0, 0, 450, 40));
        bar.update_clock(8, 30);
        bar.update_battery(PowerState::Battery { percent: 90 });

        let texts = bar.display_texts();
        assert_eq!(texts.len(), 2);
        assert!(texts.iter().any(|(k, _)| *k == "clock"));
        assert!(texts.iter().any(|(k, _)| *k == "battery"));
    }

    #[test]
    fn use_12h_format() {
        let mut bar = StatusBar::new(FormFactor::Phone, Rect::new(0, 0, 1080, 80)).with_24h(false);
        bar.update_clock(15, 45);

        let clock = bar.get_indicator("clock");
        assert_eq!(
            clock.map(StatusIndicator::display_text),
            Some("3:45 PM".to_string())
        );
    }

    #[test]
    fn notification_badge() {
        let mut bar = StatusBar::new(FormFactor::Phone, Rect::new(0, 0, 1080, 80));
        bar.update_notifications(3);

        let badge = bar.get_indicator("notification_badge");
        assert!(badge.is_some());
        assert_eq!(
            badge.map(StatusIndicator::display_text),
            Some("3".to_string())
        );
    }

    #[test]
    fn desktop_all_slots() {
        let bar = StatusBar::new(FormFactor::Desktop, Rect::new(0, 0, 1920, 48));
        assert_eq!(bar.slots().len(), 5);
        assert_eq!(bar.slots()[0], IndicatorSlot::Clock);
        assert_eq!(bar.slots()[1], IndicatorSlot::Security);
        assert_eq!(bar.slots()[2], IndicatorSlot::Services);
        assert_eq!(bar.slots()[3], IndicatorSlot::Network);
        assert_eq!(bar.slots()[4], IndicatorSlot::Battery);
    }

    #[test]
    fn bounds_preserved() {
        let bounds = Rect::new(10, 20, 300, 40);
        let bar = StatusBar::new(FormFactor::Watch, bounds);
        assert_eq!(bar.bounds(), bounds);
    }
}
