// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Linux input implementation via evdev.
//!
//! Tier: T3 (σ Sequence + ∃ Existence — Linux evdev-specific)
//!
//! Reads input events from `/dev/input/event*` devices.
//! Phase 1: Basic polling from a virtual event queue.
//! Phase 2: Direct evdev file descriptor reading.

use nexcore_pal::Input;
use nexcore_pal::error::InputError;
use nexcore_pal::types::{InputEvent, KeyCode, KeyState, Modifiers};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

/// Linux input subsystem backed by evdev.
///
/// Tier: T3 (Linux-specific input implementation)
pub struct LinuxInput {
    /// Pending events queue.
    events: VecDeque<InputEvent>,
    /// Discovered input device paths.
    device_paths: Vec<PathBuf>,
    /// Current modifier state (will be used when evdev event processing is added).
    _modifiers: Modifiers,
}

impl LinuxInput {
    /// Create a new Linux input subsystem.
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            device_paths: Vec::new(),
            _modifiers: Modifiers::NONE,
        }
    }

    /// Probe for available input devices.
    pub fn probe() -> Result<Self, InputError> {
        let mut input = Self::new();

        let input_dir = PathBuf::from("/dev/input");
        if !input_dir.exists() {
            return Ok(input); // No input devices, but not an error
        }

        if let Ok(entries) = fs::read_dir(&input_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("event") {
                    input.device_paths.push(entry.path());
                }
            }
        }

        input.device_paths.sort();
        Ok(input)
    }

    /// Inject an event into the queue (for testing or virtual input).
    pub fn inject_event(&mut self, event: InputEvent) {
        self.events.push_back(event);
    }

    /// Get the number of discovered input devices.
    pub fn device_count(&self) -> usize {
        self.device_paths.len()
    }

    /// Map a Linux keycode to platform-agnostic KeyCode.
    ///
    /// Based on linux/input-event-codes.h values.
    pub fn map_keycode(linux_code: u16) -> KeyCode {
        match linux_code {
            1 => KeyCode::Escape,
            2 => KeyCode::Key1,
            3 => KeyCode::Key2,
            4 => KeyCode::Key3,
            5 => KeyCode::Key4,
            6 => KeyCode::Key5,
            7 => KeyCode::Key6,
            8 => KeyCode::Key7,
            9 => KeyCode::Key8,
            10 => KeyCode::Key9,
            11 => KeyCode::Key0,
            14 => KeyCode::Backspace,
            15 => KeyCode::Tab,
            16 => KeyCode::Q,
            17 => KeyCode::W,
            18 => KeyCode::E,
            19 => KeyCode::R,
            20 => KeyCode::T,
            21 => KeyCode::Y,
            22 => KeyCode::U,
            23 => KeyCode::I,
            24 => KeyCode::O,
            25 => KeyCode::P,
            28 => KeyCode::Enter,
            29 => KeyCode::LeftCtrl,
            30 => KeyCode::A,
            31 => KeyCode::S,
            32 => KeyCode::D,
            33 => KeyCode::F,
            34 => KeyCode::G,
            35 => KeyCode::H,
            36 => KeyCode::J,
            37 => KeyCode::K,
            38 => KeyCode::L,
            42 => KeyCode::LeftShift,
            44 => KeyCode::Z,
            45 => KeyCode::X,
            46 => KeyCode::C,
            47 => KeyCode::V,
            48 => KeyCode::B,
            49 => KeyCode::N,
            50 => KeyCode::M,
            54 => KeyCode::RightShift,
            56 => KeyCode::LeftAlt,
            57 => KeyCode::Space,
            59 => KeyCode::F1,
            60 => KeyCode::F2,
            61 => KeyCode::F3,
            62 => KeyCode::F4,
            63 => KeyCode::F5,
            64 => KeyCode::F6,
            65 => KeyCode::F7,
            66 => KeyCode::F8,
            67 => KeyCode::F9,
            68 => KeyCode::F10,
            87 => KeyCode::F11,
            88 => KeyCode::F12,
            100 => KeyCode::RightAlt,
            103 => KeyCode::Up,
            105 => KeyCode::Left,
            106 => KeyCode::Right,
            108 => KeyCode::Down,
            102 => KeyCode::Home,
            107 => KeyCode::End,
            104 => KeyCode::PageUp,
            109 => KeyCode::PageDown,
            111 => KeyCode::Delete,
            125 => KeyCode::LeftSuper,
            126 => KeyCode::RightSuper,
            other => KeyCode::Unknown(u32::from(other)),
        }
    }

    /// Convert evdev value to KeyState.
    pub fn map_key_state(value: i32) -> KeyState {
        match value {
            0 => KeyState::Released,
            2 => KeyState::Repeat,
            _ => KeyState::Pressed,
        }
    }
}

impl Default for LinuxInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for LinuxInput {
    fn poll_events(&mut self) -> Result<Vec<InputEvent>, InputError> {
        // In a real implementation, we'd read from evdev file descriptors
        // using epoll. For now, drain the injected event queue.
        let events: Vec<InputEvent> = self.events.drain(..).collect();
        Ok(events)
    }

    fn poll_event(&mut self) -> Result<Option<InputEvent>, InputError> {
        Ok(self.events.pop_front())
    }

    fn has_events(&self) -> bool {
        !self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_pal::types::{KeyEvent, TouchEvent, TouchPhase};

    #[test]
    fn input_creation() {
        let input = LinuxInput::new();
        assert!(!input.has_events());
        assert_eq!(input.device_count(), 0);
    }

    #[test]
    fn inject_and_poll() {
        let mut input = LinuxInput::new();

        let event = InputEvent::Touch(TouchEvent {
            id: 0,
            x: 100.0,
            y: 200.0,
            pressure: 1.0,
            phase: TouchPhase::Started,
        });

        input.inject_event(event.clone());
        assert!(input.has_events());

        let polled = input.poll_event();
        assert!(polled.is_ok());
        let polled = polled.ok().flatten();
        assert!(polled.is_some());
    }

    #[test]
    fn poll_events_drains() {
        let mut input = LinuxInput::new();
        input.inject_event(InputEvent::Key(KeyEvent {
            code: KeyCode::A,
            state: KeyState::Pressed,
            modifiers: Modifiers::NONE,
        }));
        input.inject_event(InputEvent::Key(KeyEvent {
            code: KeyCode::A,
            state: KeyState::Released,
            modifiers: Modifiers::NONE,
        }));

        let events = input.poll_events();
        assert!(events.is_ok());
        let events = events.unwrap_or_default();
        assert_eq!(events.len(), 2);
        assert!(!input.has_events());
    }

    #[test]
    fn keycode_mapping() {
        assert_eq!(LinuxInput::map_keycode(30), KeyCode::A);
        assert_eq!(LinuxInput::map_keycode(57), KeyCode::Space);
        assert_eq!(LinuxInput::map_keycode(28), KeyCode::Enter);
        assert_eq!(LinuxInput::map_keycode(1), KeyCode::Escape);
    }

    #[test]
    fn unknown_keycode() {
        match LinuxInput::map_keycode(999) {
            KeyCode::Unknown(code) => assert_eq!(code, 999),
            _ => assert!(false, "Expected Unknown variant"),
        }
    }

    #[test]
    fn key_state_mapping() {
        assert_eq!(LinuxInput::map_key_state(0), KeyState::Released);
        assert_eq!(LinuxInput::map_key_state(1), KeyState::Pressed);
        assert_eq!(LinuxInput::map_key_state(2), KeyState::Repeat);
    }
}
