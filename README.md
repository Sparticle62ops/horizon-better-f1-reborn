# Better F1 Reborn for Horizon

Native Rust port of [Better F1 Reborn](https://github.com/MovTery/Better-F1-Reborn)
(Modrinth slug `better-f1-reborn`) for the [Horizon](https://github.com/horizon-rust/horizon) client.

Upstream commit `62824ad` (2026-07-21), MIT (c) 2024 MovTery. `LICENSE` carried verbatim.

## Slice 1: logic core (this repo)

Ported 1:1 from `common/src/main/java/com/movtery/betterf1`:

- `client/HUDState.java`: the three-state F1 cycle
  ALL_VISIBLE -> NO_HUD -> ALL_HIDDEN -> ALL_VISIBLE (`next()`)
- `BetterF1.java`: the global state, starting at ALL_VISIBLE
- `mixin/KeyboardMixin`: on F1, advance the state and set `options.hideGui = state != ALL_VISIBLE`
  (modeled as `on_f1_key()` returning the hideGui value the state wants)
- `mixin/GameRendererMixin` + `mixin/compat_iris/MixinHandRenderer`: the hand-render redirect
  `instance.hideGui && state == ALL_HIDDEN` (modeled as `HudState::hand_hidden`)

`cargo test` 6/6 green.

## Seams (game-side, tracked on horizon-rust/horizon #11123)

- `native_input_key` (API 121) is post-handling and non-cancellable, so the port cannot
  intercept F1 before vanilla toggles its hide flag; game glue applies the returned value.
- `native_render_hand` (API 87) runs after the hidden-HUD gate, so hands cannot be forced
  visible while the HUD is hidden; the NO_HUD state's hand visibility needs that gate to be
  mode-aware (proposal in #11123).
- The Iris hand compat mixin maps to the same hand-gate seam (no separate shader-pack hook).
