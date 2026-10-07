# BRINK — v0.1 Implementation Plan

*Concrete plan for the v0.1 engine milestone (DESIGN §22.1). Progress is judged by **gates, not features**.*

*Priority, never reversed: working game → interesting decisions → coherent simulation → historical richness → additional complexity.*

---

## 1. Workspace layout

```
war games/
  Cargo.toml                 workspace
  crates/
    sim-core/                Simulation/Core Agent — canonical state, turn loop, RNG, economy,
                             stability, opinion, (later) ledger, treaties, trade, transfers, wars.
                             Pure: no I/O, no threads, no wall-clock, no OS randomness.
    ai/                      AI Agent — consumes ObserverView only; produces Orders + reasoning.
    scenario/                Scenario/Data Agent — RON schemas → sim-core WorldState. No behaviour.
    headless/                Test/Simulation Agent — CLI runner, batch runs (rayon), summaries.
  data/
    fixtures/                small deterministic test worlds (four_actor.ron, ...)
    scenarios/1980/          full scenario data (build step 9)
  design/                    design docs (no code)
```

**Dependency direction:** `headless → {scenario, ai} → sim-core`. `sim-core` depends on nothing in the workspace. A future UI depends on `sim-core` and is never depended on.

**External crates (minimal):**
- `serde`
- `ron`
- `serde_json` (snapshots and logs)
- `rand_core` + `rand_chacha` (seeded RNG)
- `rayon` (batch runs only, in `headless`)

---

## 2. Core architectural rules (enforced by code and tests)

| Rule | Mechanism |
|---|---|
| Determinism | One `ChaCha8Rng` in `WorldState`, seeded from the scenario seed. All iteration in `CountryId` order (`Vec`/`BTreeMap`, never `HashMap` iteration). Same seed → byte-identical JSON snapshot |
| Fog | The AI receives `ObserverView` built by `sim-core::view::observe(&WorldState, observer)`. The `ai` crate never takes `&WorldState`. This is enforced by function signatures |
| No identity branching | Lint test scanning `sim-core` and `ai` sources for country-code string literals, plus a data-swap test (step 5) |
| Simultaneous turns | Orders are collected from every controller against the *start-of-turn* views, then resolved in the fixed DESIGN §4.3 phase order |
| Explainability | Every AI decision returns `Vec<ReasonLine>` whose values sum to the score (step 5) |
| Gates over features | Each build step ends with named acceptance tests. A step isn't done until its tests pass |

---

## 3. Build steps, deliverables and acceptance tests

| Step | Owner(s) | Deliverables | Acceptance (tests) | Gate |
|---|---|---|---|---|
| **1 Skeleton** | Core, Data, Test | Workspace, `WorldState`, ids, RNG, budget lines + inertia, deficit/debt, revenue, economy-lite growth, Stability (4 drivers, government weights), Initiative (base, bands, banking), Opinion (decaying modifiers), turn loop, RON fixture loader, `headless run` | 80 turns run; **same seed → identical snapshot**; invariants (budget shares sum to 1, stability 0–100, GDP > 0, initiative 0–cap); batch identical at 1 vs N threads; no-identity lint | — |
| **2 Fog** | Core, AI | `ObserverView` (own state exact; others: public fields + noisy estimates by coverage), `Controller` trait, trivial AI | AI crate compiles only against `ObserverView`; estimate error shrinks with coverage | — |
| **3 Diplomacy & economics** | Core | Treaties (Trade shallow/deep, Alliance, Guarantee, Basing, Arms Supply, NAP), actions with Initiative costs, proposal accept/refuse, Sanction + substitution + lost-trade Prosperity modifier, standing streams, bilateral + power-weighted global tension | Initiative accounting; accepting a commitment costs 1; sanction erodes ~10%/turn and adaptation persists after lifting; deep trade raises the sanction cost | — |
| **4 Ledger & reputation** | Core (+AI for reads) | `EventLedger`, visibility/`seen_by`, commitment tests + grading, shadow commitments, both-sides rule, per-observer `Exp_k` read, third-party reaction, implied norms with instrumentation logs | Observer divergence; fog respect; fake-honour → Partial; withdraw-before-test → Abandoned; norm logs emitted | — |
| **5 Four-actor AI** | AI, Test | Utility framework, `ReasonLine`s, decisions: accept aid, request guarantee, join sanctions, abandon bloc (hysteresis); four-actor fixture scripts | Past behaviour changes decisions; explanation lines sum to score; data-swap mirror test; **forced-branch Pareto test** | **1, 2** |
| **6 Military & transfers** | Core | Force pools (land/naval/air, quality, readiness), threat inputs, Arms Supply transfers (permanent, quality cap), arms outflow → defence burden | Former client keeps transferred strength after relations flip | **4** |
| **7 War-lite** | Core, AI | `War → Vec<Front>` (one instantiated), simple combat, war aims incl. `Punitive`, Involvement Bands, proxy routes/conduits, band-choice utility, hold/withdraw/escalate | Pareto test over the 5 bands at a fixture crisis; covert exit cheaper than public exit | **3** |
| **8 Reassessment** | AI, Core | Strategic reassessment cadence, goal rescoring, treaty re-evaluation, defence-burden tolerance, war-of-choice weariness, balancing (intent-gated), trade depth effects | Delete-the-rival fixture: goal mix shifts; ≥ 3 strategies above 10% across seeds; alliances wobble with warning windows | **5** |
| **9 Scenario integration** | Data, Test | 1980 data (16 playable, majors, minors as restricted-action AIs), ~6 event templates, batch runner with statistics (incl. norm statistics) | 200-seed batch: Gate 6 behaviour frequencies; plausibility targets reported (non-blocking) | **6** |

**Parallelism:** steps 1–4 touch core structures and run **sequentially under one owner** at a time. From step 5, the AI work (ai crate) and the Core work (sim-core military/war) can run in parallel, because the `ObserverView`/`Orders` interface will be stable. Scenario data (step 9) can start in parallel once the step-3 schemas are frozen.

---

## 4. Step 1 numeric spec (economy-lite; first-pass constants, all in data where sensible)

**Units:** GDP is an index (USA 1980 = 100). One turn = one quarter (the monthly sub-tick is deferred until combat needs it).

| Quantity | Rule |
|---|---|
| Budget shares | `actual += (target − actual) × 0.5` per turn (inertia). Lines: military, development, welfare, intelligence |
| Revenue | `gdp × tax_rate` |
| Debt service | `debt × interest_rate`, where `interest_rate = base_rate × (1 − financial_weight)` (quarterly) |
| Spending pool | `revenue × (1 + deficit_ratio) − debt_service`; `deficit_ratio` is a standing setting 0–0.3 |
| Debt | `debt += revenue × deficit_ratio`; debt ratio = `debt / (gdp × 4)` |
| Growth (quarterly) | `g = catch_up + 0.02·dev_share − 0.004·max(0, debt_ratio − 0.6) + (stability − 50)/20000 + shock`; `catch_up = 0.006·(1 − gdp_pc / gdp_pc_max)`; `shock ~ U(−0.002, 0.002)` seeded |
| Military strength | `mil += spend_mil × 0.5 − mil × 0.02` (upkeep/decay) |
| Prosperity driver | `50 + 1500·g + 60·(welfare_share − 0.25) − 30·max(0, debt_ratio − 0.8)`, clamped 0–100 |
| Security driver | 50 in step 1; since step 6: 10 + 65 × D/(D+H) (DESIGN §9.1, D32) |
| Legitimacy | Slow: moves 5% per turn toward `(prosperity + security)/2`. Events adjust it later |
| War weariness | 0 in step 1 |
| Stability target | Weighted mean of drivers by government type (Democracy: P .45 S .20 L .35; Authoritarian: P .30 S .35 L .35; Revolutionary: P .20 S .30 L .50) − war weariness. `stability += (target − stability) × 0.25` |
| Initiative | `base 3 (+trait)`, +1 if stability ≥ 70, −1 if < 40, −1 if < 25. Unspent banks up to +2 |
| Opinion | Sum of modifiers `{value, decay_per_turn}`; each decays toward 0; clamped ±100 |

The Step 1 AI holds a constant budget. Real decisions start at step 5.

---

## 5. Data schema (step 1 subset, RON)

```ron
Scenario(
  name: "four_actor",
  seed: 1980,
  start_year: 1980,
  turns: 80,
  base_interest_rate: 0.012,
  countries: [
    Country(
      id: "MAJ", name: "Major Power", government: Democracy,
      population: 230.0, gdp: 100.0, tax_rate: 0.20, debt: 120.0,
      financial_weight: 0.55,
      budget: (military: 0.30, development: 0.20, welfare: 0.40, intelligence: 0.10),
      deficit_ratio: 0.05, military: 60.0, stability: 58.0, legitimacy: 60.0,
      initiative_base: 3,
    ),
    // ...
  ],
  opinions: [ (from: "MAJ", to: "ALY", value: 40.0) ],
)
```

**Fixture ids are deliberately neutral** (`MAJ`, `ALY`, `RIV`, `NEU`), so tests never depend on real-country identity.

---

## 6. Out of scope for v0.1 (don't build)

- map rendering or UI beyond debug output
- regions/sea zones as geometry (adjacency data arrives with step 7 proxy routes and step 9 data)
- the full intelligence-operation catalogue (only coverage → estimate noise)
- the full nuclear system (v0.1: arsenal level, deterrence term, one use action, penalty package, added with step 7/8)
- P5, P10, P3, P2 (v0.2)
- anything in DESIGN §22.3
