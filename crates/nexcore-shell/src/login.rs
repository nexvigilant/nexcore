// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Login screen — authentication gate between Locked and Running states.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           LoginScreen                    │
//! │                                          │
//! │  ┌─────────┐    ┌──────────┐            │
//! │  │ UserList │───►│ Credential│           │
//! │  │ Select  │    │  Entry    │           │
//! │  └─────────┘    └────┬─────┘           │
//! │                      │                  │
//! │                      ▼                  │
//! │              ┌──────────────┐           │
//! │              │ UserManager  │           │
//! │              │   .login()   │           │
//! │              └──────┬───────┘           │
//! │                     │                   │
//! │            ┌────────┴────────┐          │
//! │            ▼                 ▼          │
//! │       [Success]          [Failed]       │
//! │    → Shell::Running    show error       │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Form Factor Input Methods
//!
//! | Device | Method | Digits | Input |
//! |--------|--------|--------|-------|
//! | Watch | PIN | 4 | Touch numpad |
//! | Phone | PIN | 6 | Touch numpad |
//! | Desktop | Password | Unlimited | Keyboard |
//!
//! ## Primitive Grounding
//!
//! - ∂ Boundary: Security gate (allowed / denied)
//! - ς State: Login state machine (Select → Entry → Auth → Result)
//! - μ Mapping: Input → credential characters
//! - κ Comparison: Credential validation (hash comparison)
//! - → Causality: Auth result → shell state transition

use nexcore_compositor::surface::Rect;
use nexcore_os::user::{AuthError, UserManager, UserRole, UserSummary};
use nexcore_pal::FormFactor;

/// Login screen state machine.
///
/// Tier: T2-P (ς State — authentication flow lifecycle)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginState {
    /// Showing the user selection list.
    UserSelection,
    /// Entering a PIN (watch/phone).
    PinEntry {
        /// Selected username.
        username: String,
        /// PIN digits entered so far (masked).
        digits: Vec<char>,
        /// Maximum digits for this form factor.
        max_digits: usize,
    },
    /// Entering a password (desktop).
    PasswordEntry {
        /// Selected username.
        username: String,
        /// Password characters entered so far.
        chars: Vec<char>,
    },
    /// Authenticating against UserManager.
    Authenticating {
        /// Username being authenticated.
        username: String,
    },
    /// Authentication succeeded — session returned.
    AuthSuccess {
        /// The authenticated session token.
        session_token: String,
        /// Username that authenticated.
        username: String,
        /// User role.
        role: UserRole,
    },
    /// Authentication failed — show error.
    AuthFailed {
        /// Username that failed.
        username: String,
        /// Error message to display.
        error_message: String,
        /// Remaining attempts before lockout (None = unknown).
        remaining_attempts: Option<u32>,
    },
}

/// Authentication method based on form factor.
///
/// Tier: T2-P (∂ Boundary — input method constraint)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMethod {
    /// 4-digit PIN for watch.
    Pin4,
    /// 6-digit PIN for phone.
    Pin6,
    /// Full password for desktop.
    Password,
}

impl AuthMethod {
    /// Select the authentication method for a form factor.
    pub const fn for_form_factor(ff: FormFactor) -> Self {
        match ff {
            FormFactor::Watch => Self::Pin4,
            FormFactor::Phone => Self::Pin6,
            FormFactor::Desktop => Self::Password,
        }
    }

    /// Maximum credential length.
    pub const fn max_length(&self) -> usize {
        match self {
            Self::Pin4 => 4,
            Self::Pin6 => 6,
            Self::Password => 128,
        }
    }

    /// Whether this method accepts only numeric digits.
    pub const fn numeric_only(&self) -> bool {
        matches!(self, Self::Pin4 | Self::Pin6)
    }
}

/// Visual element in the login screen layout.
///
/// Tier: T2-C (λ + ∂ + N — positioned bounded element)
#[derive(Debug, Clone)]
pub struct LoginElement {
    /// Element kind.
    pub kind: LoginElementKind,
    /// Bounding rectangle on screen.
    pub bounds: Rect,
}

/// Types of visual elements on the login screen.
///
/// Tier: T2-P (Σ Sum — element enumeration)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginElementKind {
    /// Clock display.
    Clock,
    /// Branding / OS name.
    Branding,
    /// User avatar (placeholder circle).
    UserAvatar { username: String },
    /// User selection list item.
    UserListItem {
        username: String,
        display_name: String,
    },
    /// PIN dot indicator (filled or empty).
    PinDot { index: usize, filled: bool },
    /// Password field (shows asterisks).
    PasswordField { length: usize },
    /// Numpad button (0-9).
    NumpadButton { digit: char },
    /// Backspace button.
    BackspaceButton,
    /// Submit / enter button.
    SubmitButton,
    /// Error message display.
    ErrorMessage { text: String },
    /// "Unlock" label.
    UnlockLabel,
}

/// Login screen layout computed for a specific form factor.
///
/// Tier: T2-C (∂ + λ + N — form-factor-aware layout)
#[derive(Debug, Clone)]
pub struct LoginLayout {
    /// All visual elements with positions.
    pub elements: Vec<LoginElement>,
    /// Screen bounds.
    pub bounds: Rect,
    /// Form factor.
    pub form_factor: FormFactor,
}

impl LoginLayout {
    /// Compute login layout for watch (450x450).
    ///
    /// Layout: Clock top, avatar center, 4 PIN dots, 3x4 numpad.
    #[allow(clippy::cast_possible_wrap)]
    pub fn watch() -> Self {
        let w = 450_u32;
        let h = 450_u32;
        let mut elements = Vec::new();

        // Clock at top
        elements.push(LoginElement {
            kind: LoginElementKind::Clock,
            bounds: Rect::new(0, 0, w, 50),
        });

        // Avatar center
        elements.push(LoginElement {
            kind: LoginElementKind::UserAvatar {
                username: String::new(),
            },
            bounds: Rect::new((w / 2 - 40) as i32, 60, 80, 80),
        });

        // 4 PIN dots
        let dot_y = 155;
        let dot_size = 20;
        let dot_gap = 30;
        let total_dots_w = 4 * dot_size + 3 * dot_gap;
        let dot_start_x = (w - total_dots_w) / 2;
        for i in 0..4 {
            elements.push(LoginElement {
                kind: LoginElementKind::PinDot {
                    index: i,
                    filled: false,
                },
                bounds: Rect::new(
                    (dot_start_x + i as u32 * (dot_size + dot_gap)) as i32,
                    dot_y,
                    dot_size,
                    dot_size,
                ),
            });
        }

        // 3x4 numpad (1-9, backspace, 0, submit)
        let pad_y = 200;
        let btn_w = 100;
        let btn_h = 50;
        let btn_gap = 10;
        let pad_total_w = 3 * btn_w + 2 * btn_gap;
        let pad_x = (w - pad_total_w) / 2;
        let digits = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
        for (idx, &digit) in digits.iter().enumerate() {
            let row = idx / 3;
            let col = idx % 3;
            elements.push(LoginElement {
                kind: LoginElementKind::NumpadButton { digit },
                bounds: Rect::new(
                    (pad_x + col as u32 * (btn_w + btn_gap)) as i32,
                    pad_y + row as i32 * (btn_h as i32 + btn_gap as i32),
                    btn_w,
                    btn_h,
                ),
            });
        }
        // Bottom row: backspace, 0, submit
        let bottom_y = pad_y + 3 * (btn_h as i32 + btn_gap as i32);
        elements.push(LoginElement {
            kind: LoginElementKind::BackspaceButton,
            bounds: Rect::new(pad_x as i32, bottom_y, btn_w, btn_h),
        });
        elements.push(LoginElement {
            kind: LoginElementKind::NumpadButton { digit: '0' },
            bounds: Rect::new((pad_x + btn_w + btn_gap) as i32, bottom_y, btn_w, btn_h),
        });
        elements.push(LoginElement {
            kind: LoginElementKind::SubmitButton,
            bounds: Rect::new(
                (pad_x + 2 * (btn_w + btn_gap)) as i32,
                bottom_y,
                btn_w,
                btn_h,
            ),
        });

        Self {
            elements,
            bounds: Rect::new(0, 0, w, h),
            form_factor: FormFactor::Watch,
        }
    }

    /// Compute login layout for phone (1080x2400).
    ///
    /// Layout: Clock top, avatar, username, 6 PIN dots, 3x4 numpad, error.
    #[allow(clippy::cast_possible_wrap, clippy::vec_init_then_push)]
    pub fn phone() -> Self {
        let w = 1080_u32;
        let h = 2400_u32;
        let mut elements = Vec::new();

        // Clock
        elements.push(LoginElement {
            kind: LoginElementKind::Clock,
            bounds: Rect::new(0, 80, w, 120),
        });

        // Branding
        elements.push(LoginElement {
            kind: LoginElementKind::Branding,
            bounds: Rect::new(0, 220, w, 60),
        });

        // Avatar
        elements.push(LoginElement {
            kind: LoginElementKind::UserAvatar {
                username: String::new(),
            },
            bounds: Rect::new((w / 2 - 80) as i32, 350, 160, 160),
        });

        // Unlock label
        elements.push(LoginElement {
            kind: LoginElementKind::UnlockLabel,
            bounds: Rect::new(0, 540, w, 60),
        });

        // 6 PIN dots
        let dot_y = 650;
        let dot_size = 28;
        let dot_gap = 40;
        let total_dots_w = 6 * dot_size + 5 * dot_gap;
        let dot_start_x = (w - total_dots_w) / 2;
        for i in 0..6 {
            elements.push(LoginElement {
                kind: LoginElementKind::PinDot {
                    index: i,
                    filled: false,
                },
                bounds: Rect::new(
                    (dot_start_x + i as u32 * (dot_size + dot_gap)) as i32,
                    dot_y,
                    dot_size,
                    dot_size,
                ),
            });
        }

        // 3x4 numpad
        let pad_y = 800;
        let btn_w = 240;
        let btn_h = 120;
        let btn_gap = 30;
        let pad_total_w = 3 * btn_w + 2 * btn_gap;
        let pad_x = (w - pad_total_w) / 2;
        let digits = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
        for (idx, &digit) in digits.iter().enumerate() {
            let row = idx / 3;
            let col = idx % 3;
            elements.push(LoginElement {
                kind: LoginElementKind::NumpadButton { digit },
                bounds: Rect::new(
                    (pad_x + col as u32 * (btn_w + btn_gap)) as i32,
                    pad_y + row as i32 * (btn_h as i32 + btn_gap as i32),
                    btn_w,
                    btn_h,
                ),
            });
        }
        let bottom_y = pad_y + 3 * (btn_h as i32 + btn_gap as i32);
        elements.push(LoginElement {
            kind: LoginElementKind::BackspaceButton,
            bounds: Rect::new(pad_x as i32, bottom_y, btn_w, btn_h),
        });
        elements.push(LoginElement {
            kind: LoginElementKind::NumpadButton { digit: '0' },
            bounds: Rect::new((pad_x + btn_w + btn_gap) as i32, bottom_y, btn_w, btn_h),
        });
        elements.push(LoginElement {
            kind: LoginElementKind::SubmitButton,
            bounds: Rect::new(
                (pad_x + 2 * (btn_w + btn_gap)) as i32,
                bottom_y,
                btn_w,
                btn_h,
            ),
        });

        // Error message area
        elements.push(LoginElement {
            kind: LoginElementKind::ErrorMessage {
                text: String::new(),
            },
            bounds: Rect::new(0, 1700, w, 60),
        });

        Self {
            elements,
            bounds: Rect::new(0, 0, w, h),
            form_factor: FormFactor::Phone,
        }
    }

    /// Compute login layout for desktop (1920x1080).
    ///
    /// Layout: Clock top-right, avatar center, username, password field, error.
    #[allow(clippy::cast_possible_wrap, clippy::vec_init_then_push)]
    pub fn desktop() -> Self {
        let w = 1920_u32;
        let h = 1080_u32;
        let mut elements = Vec::new();

        // Clock top-right
        elements.push(LoginElement {
            kind: LoginElementKind::Clock,
            bounds: Rect::new((w - 300) as i32, 20, 280, 40),
        });

        // Branding top-left
        elements.push(LoginElement {
            kind: LoginElementKind::Branding,
            bounds: Rect::new(40, 20, 300, 40),
        });

        // Avatar centered
        elements.push(LoginElement {
            kind: LoginElementKind::UserAvatar {
                username: String::new(),
            },
            bounds: Rect::new((w / 2 - 60) as i32, 280, 120, 120),
        });

        // Unlock label
        elements.push(LoginElement {
            kind: LoginElementKind::UnlockLabel,
            bounds: Rect::new((w / 2 - 200) as i32, 420, 400, 40),
        });

        // Password field
        elements.push(LoginElement {
            kind: LoginElementKind::PasswordField { length: 0 },
            bounds: Rect::new((w / 2 - 200) as i32, 500, 400, 50),
        });

        // Submit button
        elements.push(LoginElement {
            kind: LoginElementKind::SubmitButton,
            bounds: Rect::new((w / 2 - 60) as i32, 580, 120, 44),
        });

        // Error message
        elements.push(LoginElement {
            kind: LoginElementKind::ErrorMessage {
                text: String::new(),
            },
            bounds: Rect::new((w / 2 - 200) as i32, 650, 400, 40),
        });

        Self {
            elements,
            bounds: Rect::new(0, 0, w, h),
            form_factor: FormFactor::Desktop,
        }
    }

    /// Select layout for a given form factor.
    pub fn for_form_factor(ff: FormFactor) -> Self {
        match ff {
            FormFactor::Watch => Self::watch(),
            FormFactor::Phone => Self::phone(),
            FormFactor::Desktop => Self::desktop(),
        }
    }

    /// Get elements of a specific kind.
    pub fn elements_of_kind(&self, kind_name: &str) -> Vec<&LoginElement> {
        self.elements
            .iter()
            .filter(|e| e.kind.kind_name() == kind_name)
            .collect()
    }

    /// Total element count.
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }
}

impl LoginElementKind {
    /// Get a string name for the element kind (for filtering).
    pub fn kind_name(&self) -> &str {
        match self {
            Self::Clock => "clock",
            Self::Branding => "branding",
            Self::UserAvatar { .. } => "user_avatar",
            Self::UserListItem { .. } => "user_list_item",
            Self::PinDot { .. } => "pin_dot",
            Self::PasswordField { .. } => "password_field",
            Self::NumpadButton { .. } => "numpad_button",
            Self::BackspaceButton => "backspace_button",
            Self::SubmitButton => "submit_button",
            Self::ErrorMessage { .. } => "error_message",
            Self::UnlockLabel => "unlock_label",
        }
    }
}

/// The login screen — authenticates users before unlocking the shell.
///
/// Tier: T3 (∂ + ς + μ + κ + → — full login integration)
///
/// Integrates with `UserManager` for credential validation and
/// provides form-factor-aware UI layouts.
pub struct LoginScreen {
    /// Current login state.
    state: LoginState,
    /// Authentication method based on form factor.
    auth_method: AuthMethod,
    /// Visual layout.
    layout: LoginLayout,
    /// Form factor.
    form_factor: FormFactor,
    /// Available users for selection.
    available_users: Vec<UserSummary>,
    /// Error display timeout (ticks remaining).
    error_timeout: u32,
}

/// Maximum auto-lock threshold from UserManager.
const MAX_FAILED_ATTEMPTS: u32 = 5;

/// Error display duration in ticks.
const ERROR_DISPLAY_TICKS: u32 = 120;

impl LoginScreen {
    /// Create a new login screen for a form factor.
    pub fn new(form_factor: FormFactor) -> Self {
        Self {
            state: LoginState::UserSelection,
            auth_method: AuthMethod::for_form_factor(form_factor),
            layout: LoginLayout::for_form_factor(form_factor),
            form_factor,
            available_users: Vec::new(),
            error_timeout: 0,
        }
    }

    /// Refresh the user list from the user manager.
    pub fn refresh_users(&mut self, user_manager: &UserManager) {
        self.available_users = user_manager.list_users();
    }

    /// Select a user to log in as.
    ///
    /// Transitions: `UserSelection` → `PinEntry` or `PasswordEntry`
    pub fn select_user(&mut self, username: &str) -> bool {
        if self.state != LoginState::UserSelection {
            return false;
        }

        // Verify user exists in available list
        let user_exists = self.available_users.iter().any(|u| u.username == username);
        if !user_exists {
            return false;
        }

        self.state = if self.auth_method.numeric_only() {
            LoginState::PinEntry {
                username: username.to_string(),
                digits: Vec::new(),
                max_digits: self.auth_method.max_length(),
            }
        } else {
            LoginState::PasswordEntry {
                username: username.to_string(),
                chars: Vec::new(),
            }
        };
        true
    }

    /// Auto-select user if there's only one.
    ///
    /// Returns `true` if auto-selection happened.
    pub fn auto_select_single_user(&mut self) -> bool {
        if self.state != LoginState::UserSelection {
            return false;
        }
        if self.available_users.len() == 1 {
            let username = self.available_users[0].username.clone();
            self.select_user(&username)
        } else {
            false
        }
    }

    /// Enter a digit (for PIN entry).
    ///
    /// Returns `true` if the digit was accepted.
    pub fn enter_digit(&mut self, digit: char) -> bool {
        if !digit.is_ascii_digit() {
            return false;
        }
        if let LoginState::PinEntry {
            digits, max_digits, ..
        } = &mut self.state
        {
            if digits.len() < *max_digits {
                digits.push(digit);
                return true;
            }
        }
        false
    }

    /// Enter a character (for password entry).
    ///
    /// Returns `true` if the character was accepted.
    pub fn enter_char(&mut self, ch: char) -> bool {
        if let LoginState::PasswordEntry { chars, .. } = &mut self.state {
            if chars.len() < self.auth_method.max_length() {
                chars.push(ch);
                return true;
            }
        }
        false
    }

    /// Delete the last entered character/digit.
    pub fn backspace(&mut self) -> bool {
        match &mut self.state {
            LoginState::PinEntry { digits, .. } => {
                digits.pop();
                true
            }
            LoginState::PasswordEntry { chars, .. } => {
                chars.pop();
                true
            }
            _ => false,
        }
    }

    /// Clear all entered credentials.
    pub fn clear_input(&mut self) {
        match &mut self.state {
            LoginState::PinEntry { digits, .. } => digits.clear(),
            LoginState::PasswordEntry { chars, .. } => chars.clear(),
            _ => {}
        }
    }

    /// Submit credentials for authentication.
    ///
    /// Transitions to `AuthSuccess` or `AuthFailed`.
    pub fn submit(&mut self, user_manager: &mut UserManager) -> bool {
        let (username, credential) = match &self.state {
            LoginState::PinEntry {
                username, digits, ..
            } => (username.clone(), digits.iter().collect::<String>()),
            LoginState::PasswordEntry { username, chars } => {
                (username.clone(), chars.iter().collect::<String>())
            }
            _ => return false,
        };

        if credential.is_empty() {
            return false;
        }

        self.state = LoginState::Authenticating {
            username: username.clone(),
        };

        match user_manager.login(&username, &credential) {
            Ok(session) => {
                self.state = LoginState::AuthSuccess {
                    session_token: session.token.clone(),
                    username: session.username.clone(),
                    role: session.role,
                };
                self.error_timeout = 0;
                true
            }
            Err(err) => {
                let (error_message, remaining) = match &err {
                    AuthError::InvalidPassword => {
                        // Check remaining attempts
                        let remaining = user_manager
                            .get_user(&username)
                            .map(|u| MAX_FAILED_ATTEMPTS.saturating_sub(u.failed_attempts));
                        ("Incorrect PIN/password".to_string(), remaining)
                    }
                    AuthError::AccountLocked(u) => (format!("Account '{u}' is locked"), None),
                    AuthError::AccountDisabled(u) => (format!("Account '{u}' is disabled"), None),
                    AuthError::UserNotFound(u) => (format!("User '{u}' not found"), None),
                    _ => (format!("Authentication error: {err}"), None),
                };

                self.state = LoginState::AuthFailed {
                    username,
                    error_message,
                    remaining_attempts: remaining,
                };
                self.error_timeout = ERROR_DISPLAY_TICKS;
                false
            }
        }
    }

    /// Go back to user selection.
    pub fn back_to_user_selection(&mut self) {
        self.state = LoginState::UserSelection;
        self.error_timeout = 0;
    }

    /// Tick the login screen (for error timeout countdown).
    pub fn tick(&mut self) {
        if self.error_timeout > 0 {
            self.error_timeout -= 1;
            if self.error_timeout == 0 {
                // After error timeout, return to credential entry
                if let LoginState::AuthFailed { username, .. } = &self.state {
                    let username = username.clone();
                    self.state = if self.auth_method.numeric_only() {
                        LoginState::PinEntry {
                            username,
                            digits: Vec::new(),
                            max_digits: self.auth_method.max_length(),
                        }
                    } else {
                        LoginState::PasswordEntry {
                            username,
                            chars: Vec::new(),
                        }
                    };
                }
            }
        }
    }

    /// Get the current login state.
    pub fn state(&self) -> &LoginState {
        &self.state
    }

    /// Whether the login screen should be visible.
    pub fn is_visible(&self) -> bool {
        !matches!(self.state, LoginState::AuthSuccess { .. })
    }

    /// Whether authentication succeeded.
    pub fn is_authenticated(&self) -> bool {
        matches!(self.state, LoginState::AuthSuccess { .. })
    }

    /// Get the session token if authenticated.
    pub fn session_token(&self) -> Option<&str> {
        if let LoginState::AuthSuccess { session_token, .. } = &self.state {
            Some(session_token)
        } else {
            None
        }
    }

    /// Get the authenticated username if authenticated.
    pub fn authenticated_user(&self) -> Option<&str> {
        if let LoginState::AuthSuccess { username, .. } = &self.state {
            Some(username)
        } else {
            None
        }
    }

    /// Get the authenticated role if authenticated.
    pub fn authenticated_role(&self) -> Option<UserRole> {
        if let LoginState::AuthSuccess { role, .. } = &self.state {
            Some(*role)
        } else {
            None
        }
    }

    /// Get the number of entered digits/chars.
    pub fn input_length(&self) -> usize {
        match &self.state {
            LoginState::PinEntry { digits, .. } => digits.len(),
            LoginState::PasswordEntry { chars, .. } => chars.len(),
            _ => 0,
        }
    }

    /// Get the current error message, if any.
    pub fn error_message(&self) -> Option<&str> {
        if let LoginState::AuthFailed { error_message, .. } = &self.state {
            Some(error_message)
        } else {
            None
        }
    }

    /// Get the authentication method.
    pub fn auth_method(&self) -> AuthMethod {
        self.auth_method
    }

    /// Get the form factor.
    pub fn form_factor(&self) -> FormFactor {
        self.form_factor
    }

    /// Get the login layout.
    pub fn layout(&self) -> &LoginLayout {
        &self.layout
    }

    /// Get available users.
    pub fn available_users(&self) -> &[UserSummary] {
        &self.available_users
    }

    /// Reset the login screen (e.g., after lock).
    pub fn reset(&mut self) {
        self.state = LoginState::UserSelection;
        self.error_timeout = 0;
    }

    /// Get the current username being authenticated (if in entry or failed state).
    pub fn current_username(&self) -> Option<&str> {
        match &self.state {
            LoginState::PinEntry { username, .. }
            | LoginState::PasswordEntry { username, .. }
            | LoginState::Authenticating { username }
            | LoginState::AuthSuccess { username, .. }
            | LoginState::AuthFailed { username, .. } => Some(username),
            LoginState::UserSelection => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_user_manager_with_owner() -> UserManager {
        let mut um = UserManager::new();
        // Use a password that meets validation: 8+ chars, upper, lower, digit
        let result = um.create_owner("admin", "Admin User", "Admin123");
        assert!(result.is_ok());
        um
    }

    fn create_user_manager_multi() -> UserManager {
        let mut um = UserManager::new();
        let r1 = um.create_owner("alice", "Alice Owner", "Alice123");
        assert!(r1.is_ok());
        let r2 = um.create_user("bob", "Bob User", "Bobpass1", UserRole::User);
        assert!(r2.is_ok());
        um
    }

    // ── Auth method tests ──

    #[test]
    fn auth_method_watch() {
        let method = AuthMethod::for_form_factor(FormFactor::Watch);
        assert_eq!(method, AuthMethod::Pin4);
        assert_eq!(method.max_length(), 4);
        assert!(method.numeric_only());
    }

    #[test]
    fn auth_method_phone() {
        let method = AuthMethod::for_form_factor(FormFactor::Phone);
        assert_eq!(method, AuthMethod::Pin6);
        assert_eq!(method.max_length(), 6);
        assert!(method.numeric_only());
    }

    #[test]
    fn auth_method_desktop() {
        let method = AuthMethod::for_form_factor(FormFactor::Desktop);
        assert_eq!(method, AuthMethod::Password);
        assert_eq!(method.max_length(), 128);
        assert!(!method.numeric_only());
    }

    // ── Login screen creation ──

    #[test]
    fn login_screen_watch() {
        let screen = LoginScreen::new(FormFactor::Watch);
        assert_eq!(screen.form_factor(), FormFactor::Watch);
        assert_eq!(screen.auth_method(), AuthMethod::Pin4);
        assert_eq!(*screen.state(), LoginState::UserSelection);
        assert!(screen.is_visible());
        assert!(!screen.is_authenticated());
    }

    #[test]
    fn login_screen_phone() {
        let screen = LoginScreen::new(FormFactor::Phone);
        assert_eq!(screen.form_factor(), FormFactor::Phone);
        assert_eq!(screen.auth_method(), AuthMethod::Pin6);
    }

    #[test]
    fn login_screen_desktop() {
        let screen = LoginScreen::new(FormFactor::Desktop);
        assert_eq!(screen.form_factor(), FormFactor::Desktop);
        assert_eq!(screen.auth_method(), AuthMethod::Password);
    }

    // ── User selection ──

    #[test]
    fn refresh_users() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        assert_eq!(screen.available_users().len(), 1);
        assert_eq!(screen.available_users()[0].username, "admin");
    }

    #[test]
    fn select_user_transitions_to_password() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);

        assert!(screen.select_user("admin"));
        assert!(matches!(
            screen.state(),
            LoginState::PasswordEntry { username, .. } if username == "admin"
        ));
    }

    #[test]
    fn select_user_transitions_to_pin() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Watch);
        screen.refresh_users(&um);

        assert!(screen.select_user("admin"));
        assert!(matches!(
            screen.state(),
            LoginState::PinEntry { username, max_digits, .. }
            if username == "admin" && *max_digits == 4
        ));
    }

    #[test]
    fn select_nonexistent_user_fails() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);

        assert!(!screen.select_user("nobody"));
        assert_eq!(*screen.state(), LoginState::UserSelection);
    }

    #[test]
    fn auto_select_single_user() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);

        assert!(screen.auto_select_single_user());
        assert_eq!(screen.current_username(), Some("admin"));
    }

    #[test]
    fn auto_select_skipped_for_multi_user() {
        let um = create_user_manager_multi();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);

        assert!(!screen.auto_select_single_user());
        assert_eq!(*screen.state(), LoginState::UserSelection);
    }

    // ── Digit / char entry ──

    #[test]
    fn enter_digits_pin() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Watch);
        screen.refresh_users(&um);
        screen.select_user("admin");

        assert!(screen.enter_digit('1'));
        assert!(screen.enter_digit('2'));
        assert!(screen.enter_digit('3'));
        assert!(screen.enter_digit('4'));
        assert_eq!(screen.input_length(), 4);

        // Max reached — reject further
        assert!(!screen.enter_digit('5'));
        assert_eq!(screen.input_length(), 4);
    }

    #[test]
    fn enter_digit_rejects_non_numeric() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Watch);
        screen.refresh_users(&um);
        screen.select_user("admin");

        assert!(!screen.enter_digit('a'));
        assert_eq!(screen.input_length(), 0);
    }

    #[test]
    fn enter_chars_password() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("admin");

        for ch in "Admin123".chars() {
            assert!(screen.enter_char(ch));
        }
        assert_eq!(screen.input_length(), 8);
    }

    #[test]
    fn backspace_removes_last() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Watch);
        screen.refresh_users(&um);
        screen.select_user("admin");

        screen.enter_digit('1');
        screen.enter_digit('2');
        assert_eq!(screen.input_length(), 2);

        assert!(screen.backspace());
        assert_eq!(screen.input_length(), 1);

        assert!(screen.backspace());
        assert_eq!(screen.input_length(), 0);
    }

    #[test]
    fn clear_input() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("admin");

        screen.enter_char('p');
        screen.enter_char('w');
        assert_eq!(screen.input_length(), 2);

        screen.clear_input();
        assert_eq!(screen.input_length(), 0);
    }

    // ── Authentication ──

    #[test]
    fn successful_login_desktop() {
        let mut um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("admin");

        for ch in "Admin123".chars() {
            screen.enter_char(ch);
        }

        assert!(screen.submit(&mut um));
        assert!(screen.is_authenticated());
        assert!(!screen.is_visible());
        assert_eq!(screen.authenticated_user(), Some("admin"));
        assert_eq!(screen.authenticated_role(), Some(UserRole::Owner));
        assert!(screen.session_token().is_some());
    }

    #[test]
    fn failed_login_wrong_password() {
        let mut um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("admin");

        for ch in "Wrong123".chars() {
            screen.enter_char(ch);
        }

        assert!(!screen.submit(&mut um));
        assert!(!screen.is_authenticated());
        assert!(screen.is_visible());
        assert!(screen.error_message().is_some());
        assert!(
            screen
                .error_message()
                .is_some_and(|m| m.contains("Incorrect"))
        );
    }

    #[test]
    fn empty_credential_rejected() {
        let mut um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("admin");

        // Submit with no input
        assert!(!screen.submit(&mut um));
        // Should still be in password entry (not transitioned)
        assert!(matches!(screen.state(), LoginState::PasswordEntry { .. }));
    }

    #[test]
    fn error_timeout_returns_to_entry() {
        let mut um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("admin");

        for ch in "Wrong123".chars() {
            screen.enter_char(ch);
        }
        screen.submit(&mut um);
        assert!(matches!(screen.state(), LoginState::AuthFailed { .. }));

        // Tick down the error timeout
        for _ in 0..ERROR_DISPLAY_TICKS {
            screen.tick();
        }

        // Should return to password entry with cleared input
        assert!(matches!(
            screen.state(),
            LoginState::PasswordEntry { chars, .. } if chars.is_empty()
        ));
    }

    #[test]
    fn back_to_user_selection() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("admin");

        screen.back_to_user_selection();
        assert_eq!(*screen.state(), LoginState::UserSelection);
    }

    #[test]
    fn reset_clears_state() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Watch);
        screen.refresh_users(&um);
        screen.select_user("admin");
        screen.enter_digit('1');
        screen.enter_digit('2');

        screen.reset();
        assert_eq!(*screen.state(), LoginState::UserSelection);
        assert_eq!(screen.input_length(), 0);
    }

    // ── Layout tests ──

    #[test]
    fn watch_layout_has_numpad() {
        let layout = LoginLayout::watch();
        assert_eq!(layout.form_factor, FormFactor::Watch);
        let numpad = layout.elements_of_kind("numpad_button");
        assert_eq!(numpad.len(), 10); // digits 0-9
    }

    #[test]
    fn watch_layout_has_4_pin_dots() {
        let layout = LoginLayout::watch();
        let dots = layout.elements_of_kind("pin_dot");
        assert_eq!(dots.len(), 4);
    }

    #[test]
    fn phone_layout_has_6_pin_dots() {
        let layout = LoginLayout::phone();
        let dots = layout.elements_of_kind("pin_dot");
        assert_eq!(dots.len(), 6);
    }

    #[test]
    fn phone_layout_has_numpad() {
        let layout = LoginLayout::phone();
        let numpad = layout.elements_of_kind("numpad_button");
        assert_eq!(numpad.len(), 10);
    }

    #[test]
    fn desktop_layout_has_password_field() {
        let layout = LoginLayout::desktop();
        let fields = layout.elements_of_kind("password_field");
        assert_eq!(fields.len(), 1);
    }

    #[test]
    fn desktop_layout_no_numpad() {
        let layout = LoginLayout::desktop();
        let numpad = layout.elements_of_kind("numpad_button");
        assert_eq!(numpad.len(), 0);
    }

    #[test]
    fn all_layouts_have_clock() {
        for ff in [FormFactor::Watch, FormFactor::Phone, FormFactor::Desktop] {
            let layout = LoginLayout::for_form_factor(ff);
            let clocks = layout.elements_of_kind("clock");
            assert_eq!(clocks.len(), 1, "Missing clock for {ff:?}");
        }
    }

    #[test]
    fn all_layouts_have_submit() {
        for ff in [FormFactor::Watch, FormFactor::Phone, FormFactor::Desktop] {
            let layout = LoginLayout::for_form_factor(ff);
            let submits = layout.elements_of_kind("submit_button");
            assert_eq!(submits.len(), 1, "Missing submit for {ff:?}");
        }
    }

    #[test]
    fn layout_element_count() {
        // Verify no empty layouts
        for ff in [FormFactor::Watch, FormFactor::Phone, FormFactor::Desktop] {
            let layout = LoginLayout::for_form_factor(ff);
            assert!(layout.element_count() > 5, "Too few elements for {ff:?}");
        }
    }

    // ── Multi-user flow ──

    #[test]
    fn multi_user_selection() {
        let um = create_user_manager_multi();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);

        assert_eq!(screen.available_users().len(), 2);
        assert!(screen.select_user("bob"));
        assert_eq!(screen.current_username(), Some("bob"));
    }

    #[test]
    fn multi_user_login_correct_user() {
        let mut um = create_user_manager_multi();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("bob");

        for ch in "Bobpass1".chars() {
            screen.enter_char(ch);
        }

        assert!(screen.submit(&mut um));
        assert_eq!(screen.authenticated_user(), Some("bob"));
        assert_eq!(screen.authenticated_role(), Some(UserRole::User));
    }

    // ── Edge cases ──

    #[test]
    fn cannot_select_user_in_wrong_state() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("admin");

        // Already in PasswordEntry — selecting again should fail
        assert!(!screen.select_user("admin"));
    }

    #[test]
    fn enter_digit_in_password_state_fails() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Desktop);
        screen.refresh_users(&um);
        screen.select_user("admin");

        // enter_digit only works in PinEntry state
        assert!(!screen.enter_digit('1'));
    }

    #[test]
    fn enter_char_in_pin_state_fails() {
        let um = create_user_manager_with_owner();
        let mut screen = LoginScreen::new(FormFactor::Watch);
        screen.refresh_users(&um);
        screen.select_user("admin");

        // enter_char only works in PasswordEntry state
        assert!(!screen.enter_char('a'));
    }

    #[test]
    fn backspace_in_user_selection_fails() {
        let screen = LoginScreen::new(FormFactor::Desktop);
        let mut screen = screen;
        assert!(!screen.backspace());
    }

    #[test]
    fn session_token_none_when_not_authenticated() {
        let screen = LoginScreen::new(FormFactor::Desktop);
        assert!(screen.session_token().is_none());
        assert!(screen.authenticated_user().is_none());
        assert!(screen.authenticated_role().is_none());
    }

    #[test]
    fn current_username_none_at_selection() {
        let screen = LoginScreen::new(FormFactor::Desktop);
        assert!(screen.current_username().is_none());
    }
}
