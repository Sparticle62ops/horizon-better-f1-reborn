// Better F1 Reborn ported to Horizon (1:1 core, mod API layout).
// Upstream: github.com/MovTery/Better-F1-Reborn (commit 62824ad, 2026-07-21), MIT (c) 2024 MovTery.
// Ported 1:1 from common/src/main/java/com/movtery/betterf1.

use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

/// com.movtery.betterf1.client.HUDState: the three-state F1 cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudState {
    #[serde(rename = "ALL_VISIBLE")]
    AllVisible,
    #[serde(rename = "NO_HUD")]
    NoHud,
    #[serde(rename = "ALL_HIDDEN")]
    AllHidden,
}

impl HudState {
    /// HUDState.next(): ALL_VISIBLE -> NO_HUD -> ALL_HIDDEN -> ALL_VISIBLE.
    pub fn next(self) -> Self {
        match self {
            HudState::AllVisible => HudState::NoHud,
            HudState::NoHud => HudState::AllHidden,
            HudState::AllHidden => HudState::AllVisible,
        }
    }

    /// KeyboardMixin sets `options.hideGui = !BetterF1.state.equals(HUDState.ALL_VISIBLE)`.
    pub fn hides_gui(self) -> bool {
        self != HudState::AllVisible
    }

    /// GameRendererMixin.onRenderHand / MixinHandRenderer.canRender redirect:
    /// `instance.hideGui && BetterF1.state.equals(HUDState.ALL_HIDDEN)`.
    pub fn hand_hidden(self, options_hide_gui: bool) -> bool {
        options_hide_gui && self == HudState::AllHidden
    }
}

/// com.movtery.betterf1.BetterF1.state: the mod's global HUD state.
#[derive(Debug)]
pub struct BetterF1 {
    pub state: HudState,
}

impl Default for BetterF1 {
    fn default() -> Self {
        // BetterF1.state starts at HUDState.ALL_VISIBLE.
        Self { state: HudState::AllVisible }
    }
}

impl BetterF1 {
    /// KeyboardMixin.onF1Key: advance the state, then set hideGui to follow it and
    /// cancel vanilla's own toggle. Horizon seam: native_input_key (API 121) is a
    /// post-handling, non-cancellable notice, so the port tracks the desired hideGui
    /// value and returns it for game glue to apply instead of intercepting the key.
    pub fn on_f1_key(&mut self) -> bool {
        self.state = self.state.next();
        self.state.hides_gui()
    }
}

// ----- BetterF1.java's live game-side state -----
// Upstream's `public static HUDState state` on the BetterF1 class; client_setup
// is its 1:1 initializer and game glue reads it.
static INSTANCE: OnceLock<Mutex<BetterF1>> = OnceLock::new();

pub fn init() {
    INSTANCE.get_or_init(|| Mutex::new(BetterF1::default()));
}

pub fn state() -> HudState {
    INSTANCE
        .get()
        .and_then(|cell| cell.lock().ok())
        .map(|f1| f1.state)
        .unwrap_or(HudState::AllVisible)
}

/// KeyboardMixin.onF1Key through the global state: advance the state, then
/// return the desired hideGui value for game glue to apply (the pinned API's
/// input notice is post-handling and non-cancellable, so vanilla's own toggle
/// cannot be cancelled here).
pub fn on_f1_key() -> bool {
    INSTANCE
        .get()
        .and_then(|cell| cell.lock().ok())
        .map(|mut f1| f1.on_f1_key())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn global_state_starts_all_visible_then_advances() {
        // one sequential test: the tests share the process-global INSTANCE,
        // so parallel tests would race on its state
        init();
        assert_eq!(state(), HudState::AllVisible);
        assert_eq!(on_f1_key(), true);
        assert_eq!(state(), HudState::NoHud);
    }
    use super::*;

    #[test]
    fn state_starts_all_visible() {
        assert_eq!(BetterF1::default().state, HudState::AllVisible);
    }

    #[test]
    fn next_cycles_through_three_states() {
        let mut s = HudState::AllVisible;
        let seen = [s.next(), s.next().next(), s.next().next().next()];
        assert_eq!(seen, [HudState::NoHud, HudState::AllHidden, HudState::AllVisible]);
        s = s.next().next().next();
        assert_eq!(s, HudState::AllVisible, "full cycle returns to the start");
    }

    #[test]
    fn hides_gui_matches_upstream_mapping() {
        assert!(!HudState::AllVisible.hides_gui());
        assert!(HudState::NoHud.hides_gui());
        assert!(HudState::AllHidden.hides_gui());
    }

    #[test]
    fn hand_hidden_only_in_all_hidden() {
        // upstream: instance.hideGui && state == ALL_HIDDEN
        assert!(!HudState::AllVisible.hand_hidden(false));
        assert!(!HudState::AllVisible.hand_hidden(true));
        assert!(!HudState::NoHud.hand_hidden(true));
        assert!(HudState::AllHidden.hand_hidden(true));
        assert!(!HudState::AllHidden.hand_hidden(false));
    }

    #[test]
    fn on_f1_key_advances_and_returns_hide_gui() {
        let mut f1 = BetterF1::default();
        // press 1: ALL_VISIBLE -> NO_HUD, hideGui true (HUD hidden, hands kept)
        assert!(f1.on_f1_key());
        assert_eq!(f1.state, HudState::NoHud);
        // press 2: NO_HUD -> ALL_HIDDEN, hideGui true (everything hidden)
        assert!(f1.on_f1_key());
        assert_eq!(f1.state, HudState::AllHidden);
        // press 3: ALL_HIDDEN -> ALL_VISIBLE, hideGui false
        assert!(!f1.on_f1_key());
        assert_eq!(f1.state, HudState::AllVisible);
        // full press cycle returns to vanilla-off
        assert!(f1.on_f1_key());
        assert!(f1.on_f1_key());
        assert!(!f1.on_f1_key());
    }

    #[test]
    fn serde_names_match_enum_names() {
        assert_eq!(serde_json::to_string(&HudState::NoHud).unwrap(), r#""NO_HUD""#);
        assert_eq!(serde_json::from_str::<HudState>(r#""ALL_HIDDEN""#).unwrap(), HudState::AllHidden);
    }
}
