//! Better F1 Reborn for Horizon: the mod API wiring of the 1:1 core.
//!
//! The 1:1 logic core lives in `runtime.rs`: the original's three-state F1
//! cycle (HUDState: ALL_VISIBLE -> NO_HUD -> ALL_HIDDEN) with its hideGui and
//! hand-render mappings, and BetterF1's global HUD state (upstream
//! `public static HUDState state`).
//!
//! Upstream has no config, so there is nothing to register as settings;
//! `client_setup` is the state initializer, 1:1 with upstream's
//! `state = ALL_VISIBLE`, and game glue drives the flows:
//!
//! - the F1 keybind seam: upstream's KeyboardMixin cancels vanilla's hideGui
//!   toggle and sets the redirected value; the pinned API's input notice is
//!   post-handling and non-cancellable, so the port tracks the desired
//!   hideGui value (runtime::on_f1_key) for game glue to apply.
//! - the hand-render redirects (GameRendererMixin.onRenderHand /
//!   MixinHandRenderer.canRender: `hideGui && state == ALL_HIDDEN`) need a
//!   render surface the pinned mod API has not got.
use mod_api::{ClientSetupEvent, ClientHost};

mod runtime;

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// 1:1 with upstream's `BetterF1.state = HUDState.ALL_VISIBLE` static.
    fn client_setup(&self, _host: &mut dyn ClientHost, _event: &mut ClientSetupEvent) {
        runtime::init();
    }
}
