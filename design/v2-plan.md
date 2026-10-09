# V-2 plan: making BRINK playable

Drafted 2026-10-08 by a planning agent from the code (bridge, client, orders, view, AI reasoning) and CLAUDE.md principles 7, 9 and 15. Rulings on §6 are recorded in STATE.md.

## 1. Current state
- **Bridge** (`crates/brink-godot/src/session.rs`): `Session { world, ais, narrator }`; `step()` runs `Strategist::decide` for every country, then `resolve_turn` and `Narrator::narrate`, returning `StepOutput { turn, events, narration }`. `BrinkSim` (lib.rs) exposes `load`, `step`, `countries`, `wars`, `tension`, `global_tension`, `year`, `interest_rate`, `energy_price`. **All read the true `WorldState`** (fine for a spectator; breaks principle 7 for a player). `TurnReport.rejected` and `ledger_log` are discarded.
- **Client** (`client/main.gd`, 540 lines, scene built in code): holo map, capitals, tension arcs, event markers, city damage, NES popups, one-line HUD. Controls: Space (next turn), P (autoplay), camera keys. No mouse picking, no panels.
- **Missing for a human:** country choice; holding the AI back for one country; order entry with Initiative pre-checks; answering proposals and transition crises (`Reform`/`Crackdown`); rejection feedback; a fog-filtered view; every principle-15 marker (opinion breakdowns: `OpinionModifier` `Display` already prints "renewed turn N by X"; ledger; reserves; debt/loans; sanctions; war odds).

## 2. V-2a: the smallest playable slice
**`Session` (pure Rust):** new fields `player: Option<CountryId>`, `pending: Vec<Order>`, `advice: Option<ai::Decision>`, `delegate: bool`.
- `set_player(code) -> Result<CountryId, String>` (active `Tier::Playable` only).
- `begin_turn()`: runs the player's own `Strategist::decide(&observe(world, p))` once per turn and caches `advice` (keeps the Strategist's counters in step with headless, so delegate mode is deterministic).
- `player_view() -> Option<ObserverView>` = `observe(&world, p)` only.
- `queue(o) -> Result<u8, String>`: cost from `diplomacy::initiative_cost`; reject past `own.initiative.available()`; optional dry-run of `diplomacy::apply` on a clone for the sim's own error string.
- `unqueue(i)`, `pending()`, `initiative_left()`.
- `step()`: others unchanged; the player's `OrderSet` is the pending orders (or `advice.orders` when delegating); the player adds nothing to `decisions` (the narrator must not voice reasoning the human never did); `StepOutput` gains `rejected` (filtered to the player) and `game_over`.
- `resolve_turn` needs no change (sorted by `CountryId`, per-order Initiative checks, rejections recorded; simultaneity kept because every view is taken before resolution).

**GDExtension (`BrinkSim`):** `playable()`, `set_player(code)`, `player_state()` (gdp, debt ratio, deficit, reserves; stability, legitimacy, war weariness; Initiative; budget/target, mobilization, energy policy, pending transition), `foreign(code)` (a flattened `ForeignView`: estimates as `[value, low, high]`, hidden fields null, opinion, trust, credibility), `proposals()`, `queue_order(spec) -> {ok, cost, reason, left}` (spec parsed to `Order` in Rust), `unqueue`, `pending`, `set_delegate`; `step()` adds `rejected` and `game_over`. With a player set, `countries()` and `tension()` answer from `player_view()`.

**Fog rule:** all player-facing functions live in `player.rs`, taking `&ObserverView`; a source-lint test forbids `WorldState` there. Narration is still omniscient (V-2a known gap; filter in V-2b).

**V-2a orders** (each moves a currency or a visible marker):
- Free standing: `SetBudget` (4 sliders), `SetDeficit`, `SetMobilization` (stepping up costs 1), `SetEnergyPolicy` (producers), `SetMonetaryStance` (reserve holder).
- Forced: `Respond { accept }` (proposals lapse otherwise); `Reform`/`Crackdown` (a pending crisis is the main way to lose).
- Core diplomacy (1 Initiative): `ProposeTreaty` (Trade, NonAggression, DefensiveAlliance), `IssueGuarantee`, `Sanction`, `Denounce`, `Aid`.
- Free undo: `LiftSanction`, `CancelTreaty`.
- War: `DeclareWar { aim }`, `JoinWar`, `OfferPeace`, `LeaveWar`.
- Deferred: streams/arms sales, `Mediate`, `Basing`, `ForgiveDebt`/`HoldDebt` (need the loans panel), programmes, `NuclearStrike` (DESIGN §11.4 two-step preview first).

**Client:** split UI into `ui/start.gd` (country list), `ui/orders.gd`, `ui/country.gd` (built in code); pick a country on the map by projecting capitals (`camera.unproject_position`, nearest within a radius); after each turn show events, narration and a red "REJECTED: reason" list.

## 3. V-2b: legibility (principle 15), most useful first
1. **Opinion/grudge breakdown per country** (`opinion_lines(code)` from `world.opinions.modifiers(p, f)` and `(f, p)`; reveals no more than `their_opinion_of_us`, already in the view).
2. **Own markers with their last cause:** stability (`domestic::prosperity_terms`, `security_breakdown`), war weariness, reserves (`commodity::reserve_quarters`), debt and `view.loans`, `view.sanctions`.
3. **Ledger panel:** `view.ledger` (filtered by `seen_by`), newest first; this-turn tab from `TurnReport.ledger_log`.
4. **"The cabinet's view" consequence preview:** the AI's own evaluators on the player's view (fog-safe): `impose_sanction`, `ai::war::declare_war` + `war_odds`, `propose_trade`, `propose_alliance`, `offer_guarantee`, `energy_policy`, `reform`; show `Score::sorted()` lines and totals beside queued orders; mark the advisor's picks. Later: per-observer `RepDelta` (DESIGN §21.7; needs a helper extracted from `ai::war::band`).
5. **Map overlays** from the player's view: tension, alliances and sanctions, estimated military with confidence bands, coverage.

## 4. V-2c: a full first campaign, most valuable first
1. Game-over and end screens (regime collapse = game over; 20-year end with a ledger summary).
2. Save/load as a replay log `{scenario, seed, player, per-turn orders}` (deterministic; replays in seconds).
3. Delegate toggle ("let the cabinet run this turn").
4. Remaining orders (streams/arms sales, debt forgive/hold, programmes, nuclear with a two-step preview).
5. National Objectives (DESIGN §15.2): not implemented anywhere; new sim scoring + data; must pass the five-question test first.

## 5. Testing (`crates/brink-godot/tests/player.rs`, no Godot)
- No player: `session_matches_headless_final_state` still passes.
- Delegating player = headless (seeds 1 and 7, 24 turns): proves the player path has no side effects.
- Idle player: matches a do-nothing `Controller` loop and **differs** from headless (the player really controls the country).
- Replay: same seed + order log twice → byte-identical worlds.
- Initiative/rejection: over-budget `queue` → Err; valid order → no rejection; invalid → the sim's own reason.
- Fog: lint on `player.rs`; low-coverage foreign country → null hidden fields.
- Game over: a collapsing-stability fixture sets `game_over`.

## 6. Open questions (user rulings → STATE.md)
1. What ends the game (Collapse, Coup; the player's own Reform? a lost Major war? secession)?
2. The advisor ("cabinet recommends"): always on, a toggle, or by difficulty?
3. Other countries' reasoning: hidden, debug-only, or shown after the fact?
4. "Will they accept?" previews: proxies only (opinion, trust, credibility)?
5. Narration in player mode: filter covert facts (recommended) or keep omniscient narration as part of the joke?
6. Proposal timing: AI answers one turn later; acceptable?
7. Which countries are playable at first: all 16 `Playable`, or a recommended few (IRQ, IRN, USA, SOV)?

## Sizes (working sessions)
| Slice | Size | Content |
|---|---|---|
| V-2a | 2–3 | bridge ~500 Rust lines + tests; client ~600 GDScript lines |
| V-2b | 3 | breakdown + ledger panels 1; evaluator preview 1; overlays 1 |
| V-2c | 6–9 | end/game-over 1; save as replay 1; delegate 0.5; remaining orders 1–2; Objectives 3–5 |
