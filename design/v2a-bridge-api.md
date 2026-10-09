# BrinkSim player API (V-2a part 1)

The contract between `crates/brink-godot` and the Godot client. Missing values are `null`. Codes are country codes ("USA"). Doc comments in `crates/brink-godot/src/lib.rs` repeat this.

**Fog rule.** Once `set_player` succeeds, every player-facing answer comes from the player's own `ObserverView` (`player.rs`). Foreign values are estimates `[value, low, high]`. Values the player's coverage can't see are `null`. Narration and events drop facts the player couldn't know. Without a player, the bridge is the old omniscient spectator.

## Lifecycle
1. `load(scenario, lines, seed) -> bool`
2. `playable()`: show the start screen, then call `set_player(code)`.
3. Each turn:
   1. read `player_state()`, `proposals()`, `advice()`, `foreign(code)`, `countries()`, `wars()`
   2. queue orders with `queue_order(spec)`; use `pending()` / `unqueue(i)` to review them
   3. call `step()`
4. Stop when `step().game_over` is true. After that, `step` resolves nothing.

## Methods

| Method | Returns |
|---|---|
| `playable()` | `[{code, name, selectable: bool}]`: every active Playable country. Only `selectable` ones can be chosen. This is scenario data (`selectable:` in `1980.ron`): USA and SOV for now. |
| `set_player(code)` | `{ok: bool, reason: String}`. Clears the queue. Refused once the game is over. |
| `player_state()` | `{}` with no player. Otherwise the player's own state (exact), listed below. |
| `foreign(code)` | `{}` when unknown. Otherwise a foreign country as the player sees it, listed below. |
| `proposals()` | `[{id, from, from_name, kind, turn, accept_cost, text}]`. Each lapses unless answered this turn. To answer, queue `{kind:"respond", id, flag}`. |
| `queue_order(spec)` | `{ok: bool, cost: int, reason: String, left: int}`. The whole queue is dry-run on a copy of the world. A refusal carries the simulation's own reason, e.g. `"not enough Initiative"` or `"no such sanction"`. `left` is the Initiative still free. |
| `unqueue(index)` | `bool` |
| `pending()` | `[{text, spec (dict or null)}]` |
| `set_delegate(on)` | When on, the cabinet plays the player's orders at resolution and the queue is ignored. |
| `advice()` | The cabinet's recommended orders: `[{text, spec (dict or null if outside the V-2a set), cost, subject (String or null), score (float or null), reasons: [{label, term, value}]}]`, largest reason first. It's the player's own government AI running on the player's view, computed once per turn. Showing or hiding it is a client toggle (D105 #2). To follow a recommendation, pass its `spec` to `queue_order`. |
| `step()` | `{turn, events: [{kind, a, b, value, gravity}], narration: [{fact, gravity, heading, speaker, text, speaker_code, portrait}], rejected: [{text, reason, spec}], game_over: bool, game_over_reason: "Collapse"\|"Coup"\|"Extinct"\|""}`. `rejected` holds only the player's own refused orders, with the simulation's reason. |
| `countries()` | Spectator mode: unchanged. Player mode adds `own: bool`, `stability_band` (String or null) and `power_band: [low, high]`. For foreign rows, `stability` is null, `power` is the estimate and `arsenal` is the visible arsenal. |
| `wars()` | Now includes `id` (the war id for `join_war`, `offer_peace` and `leave_war`). |
| `tension(a, b)` | In player mode, read from the player's view (it's public anyway). |

### `player_state()`

Values:
- `code, name, government, turn, year`
- `gdp, growth, debt_ratio, deficit, reserves`
- `stability, prosperity, security, legitimacy, war_weariness`
- `initiative {allowance, banked, available, left}`
- `budget` and `budget_target`, each `{military, development, welfare, intelligence}`
- `mobilization, energy_policy, energy_capacity, energy_net_exports`
- `military, arsenal, at_war`

Status flags:
- `crisis_pending`: a transition crisis is open; answer it with `reform` or `crackdown`.
- `crisis_since` (int or null): an unanswered crisis defaults to Crackdown.
- `reserve_holder`: only the holder may set `monetary_stance`.
- `delegate, game_over`

Relations:
- `treaties [{id, kind, with, ours}]`
- `sanctioning [codes], sanctioned_by [codes]`

### `foreign(code)`

Identity and public values:
- `code, name, government, alignment, area, coverage, gdp`
- `energy_policy, energy_capacity, energy_net_exports, arms_industry, military_tech`

Estimates and coverage-gated values:
- `military, land, naval, air`: each `[v, l, h]`
- `debt_ratio`: `[v, l, h]`, or null below coverage 20
- `stability_band`: String, or null below 20
- `budget`: `{…}`, or null below 60
- `arsenal`: visible arsenal only
- `programme`: float, or null if hidden

Opinion and reputation:
- `their_opinion_of_us, our_opinion_of_them, tension`
- `credibility_back, credibility_threat, credibility_norm, trust`

Relations:
- `at_war, at_war_with_us, treaties_with_us [String], we_sanction, sanctions_us`

## Order spec (`queue_order`)
`{kind, ...}` uses only the keys that kind needs:

| kind | keys | Initiative |
|---|---|---|
| `budget` | `military, development, welfare, intelligence` (normalised) | 0 |
| `deficit` | `amount` (share of revenue, clamped 0..max) | 0 |
| `mobilization` | `level`: Peacetime / Partial / Full / Total | 1 to step up, 0 to step down |
| `energy_policy` | `level`: Restrain / Normal / Flood | 0 |
| `monetary_stance` | `level`: Tight / Neutral / Loose (reserve holder only) | 0 |
| `respond` | `id` (proposal), `flag` (accept) | 1 to accept a commitment treaty, else 0 |
| `reform`, `crackdown` | — (needs `crisis_pending`) | 0 |
| `propose_treaty` | `target`, `treaty`: Trade / NonAggression / DefensiveAlliance | 1 |
| `guarantee`, `sanction`, `denounce` | `target` | 1 |
| `lift_sanction` | `target` | 0 |
| `aid` | `target`, `amount` | 1 |
| `cancel_treaty` | `id` (treaty, from `player_state().treaties`) | 1 |
| `declare_war` | `target`, `aim`: Punitive / Limited / Major | 2 |
| `join_war` | `id` (war), `side`: Attacker / Defender, `band`: 3 or 4 (default 3) | 1 |
| `offer_peace`, `leave_war` | `id` (war) | 0 |

Deferred in V-2a:
- streams and arms sales
- Mediate, Basing
- ForgiveDebt / HoldDebt
- programmes
- NuclearStrike

The advisor may still recommend these orders. They then come with `spec: null` (text only), and the player can let the cabinet play them through `set_delegate(true)`.

## Notes for part 2
- Proposals the AI answers come back one turn later (D105 #6).
- A queued order can still be refused at resolution, because other countries' simultaneous orders resolve first in id order. When that happens it shows up in `step().rejected`.
- Narration in player mode can be quieter than spectator mode. That's correct: covert facts are filtered out.

## Narration fog: what is filtered, and known leaks
Filtered in player mode:
- events: covert stream starts and stops the player can't see; unseen covert programmes completing; **third-party proposals** (proposed, refused, lapsed: only the parties see them)
- ledger developments: entries not in the player's view; norm creations not seen; **norm tests**, unless the player is a party or saw the norm claimed
- the player's own rejected orders only
- narration, kept only when the player is a party:
  - private reasoning: StrategyReversed, GuaranteeRefused, RepeatedRequestRefused, StatusQuoRefusal, ProtectionFromDenounced, CoalitionByDependence
  - exact values: NormIgnoredByFriend, CutMilitarySpendingThenThreatened, SanctionsHurtSender, SanctionsSubstituted, SanctionsReimposed, DebtBrake, OutOfInitiative
- other narration needs a stream the player can see: RefusedPatron, ArmedClient, TransferredArmsUsedAgainstSupplier

Known leaks (V-2b):
- **ArmsSoldToPatronsRival** `{funds}`: the objector's funding share counts covert streams to the seller. The objection itself is public.
- **TokenForces**: prints exact power figures.
- **GuarantorSentSanctions**: can say "arms shipments" for a covert stream.
- **ArmsToBothSides**: a Public ledger entry by the sim's own rule, even when the streams are covert.
