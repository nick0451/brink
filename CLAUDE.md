# CLAUDE.md — BRINK (working title)

A single-player 2D global strategy game: 1980s computer-wargame presentation, Plague Inc.-style map readability, modern systemic design. The player runs one country among ~16 AI countries on a world map. Simultaneous turns.

## Current phase: moving from DESIGN to IMPLEMENTATION

The v0.1 design is approved (DESIGN.md v0.3). Implementation follows DESIGN §22: the v0.1 engine milestone (gates 1–6), then the v0.2 scenario-completeness milestone. **Check [STATE.md](STATE.md) for open blockers before starting code.**

**Implementation philosophy (user directive):**
- Produce a **functional game**.
- Stop expanding historical content unless implementation reveals a concrete missing requirement.
- A mechanic must earn implementation by creating a meaningful decision or solving a demonstrated simulation problem.
- Prefer simple state + interacting rules + persistent consequences over scripted events and special cases.
- When in doubt, build the simpler system, see what breaks, and add complexity only where the game proves it needs it.

## Documents

| File | Purpose | When to update |
|---|---|---|
| [DESIGN.md](DESIGN.md) | The canonical game design. The source of truth for mechanics | Whenever a design decision is made or changed |
| [design/scenario-1980.md](design/scenario-1980.md) | 1980–2000 scenario: roster, country designs, map, starting situation, events, plausibility targets | When scenario content changes. Generic mechanics move to DESIGN.md once approved |
| [design/us-mechanics-report.md](design/us-mechanics-report.md) | Consolidated mechanics: Event Ledger, per-observer credibility, involvement bands, v0.1 implementation list and gates. **Its §12 Rejected list must be checked before proposing new mechanics** | When mechanics are approved, merge into DESIGN.md |
| [design/behaviour-and-voice.md](design/behaviour-and-voice.md) | Historical behaviour (reflexes, personality) and satirical voice design, including the satire guardrails | When approved, merge the mechanics into DESIGN.md |
| design/briefs/ | User briefs (requirements context, kept in substance) | Add new briefs as they arrive |
| design/agent-reports/ | Raw multi-agent outputs (reference only; superseded by consolidated reports) | Never edit |
| [design/implementation-plan-v0.1.md](design/implementation-plan-v0.1.md) | Concrete v0.1 build plan: crates, rules, 9 steps with acceptance tests | When the build plan changes |
| `data/reflexes/1980.ron` | Country reflex sets (institutional habits as data) | When behaviour research or new decisions add reflexes |
| `data/voice/lines.ron` | Satirical voice lines (trigger-bound, lint-validated) | When the voice library grows (V2 after Gate 3, V3 after Gate 6) |
| [STATE.md](STATE.md) | Project status: phase, decisions log, open questions, next steps | At the end of every working session, and when decisions are made |
| CLAUDE.md | How to work on this project | When working conventions change |

When a decision changes DESIGN.md, also record it in the STATE.md decisions log with a date and a one-line rationale.

## Design principles (non-negotiable unless the user changes them)

1. **Simple mechanics, complex interactions.** Favour a few strongly interacting systems over many shallow ones. The player should easily answer "what are my options?" and struggle with "what is the best option?"
2. **Every mechanic must pass the five-question test:**
   - What decision does it create for the player?
   - What resource or opportunity does it cost?
   - What other systems does it interact with?
   - How does the AI reason about it?
   - How does the player get feedback about the result?
   A mechanic that fails is cut or merged. Say so plainly. Don't keep it for flavour.
3. **Opportunity cost everywhere.** The three spent currencies are Budget, Initiative and Stability. New mechanics should cost one of these, not introduce a new currency.
4. **War is powerful but expensive and dangerous.** It must never be the default optimal expansion path.
5. **Geography matters.** Small countries matter through position, resources, technology, trade or diplomacy. They aren't just weaker versions of big countries.
6. **Asymmetry comes from layers** (geography, endowments, institutions, doctrine, traits, project, diplomacy, objectives), **not from character classes.**
7. **No information cheating.** The AI uses its own fog-filtered estimates. The player isn't a special target.
8. **Events read state.** World events trigger from simulation conditions and feed back into the systems. No flat random bonuses.
9. **Causality must be visible.** Every significant change needs a traceable "because." Opaque emergent complexity reads as randomness, and that is the top design risk.
10. **Data-driven.** Countries, regions, traits, events and AI personalities are defined in data files, not code.
11. **No country-identity branching (hard architectural rule).** Core mechanics and AI must never branch on country identity (`if country == USA`). Countries differ only through data and state. Historical names are flavour. Lint and data-swap tests enforce this. Country-specific UI text and scenario content are fine. Country-specific engine behaviour needs an explicitly approved exception.
12. **State persists; consequences emerge.** No scripted blowback, peace-dividend or "Cold War victory" events. Use persistent state (permanent transfers, the Event Ledger, treaty re-evaluation) instead.
13. **No morality meter.** The simulation records behaviour (Event Ledger); each observer draws its own conclusions. There's no authoritative global Credibility; the AI reads only observer-specific values (DESIGN §21).
14. **Tone: aggressively dark, cynical, bureaucratic gallows humour, and the joke must be true.** The Black voice is *1984* + *Brave New World* + modern stand-up deadpan and rant (style emulated, never copied). Proven, verified historical events are fair game for satire of institutions; victims and major groups never are.
    - Satire lives only in the presentation-only `voice` layer.
    - Every line needs a real, specific simulation trigger. No generic filler.
    - Ledger contradictions take priority; the Event Ledger is the straight man.
    - Black is the default setting.
    - Hard guardrails in design/behaviour-and-voice.md §2.3: never victims, peoples, faiths, real atrocities, famine or nuclear use; text speaks as offices, not named people.
    - **Portraits** may be recognisable caricatures of real leaders (D56), under the caricature rules in §2.3.

## Confirmed user decisions (don't revisit without being asked)

- **Real countries** on a stylised real-world map. Stats are abstracted indices, not real statistics.
- **Nuclear use is possible** but carries extremely heavy penalties (DESIGN §11.4). It must stay "rare but not never."
- **Player regime collapse = game over.**
- **Campaign = 20 years** (~80 turns), adjustable.
- **Simulation before UI.** The headless sim core and AI-vs-AI runs come first. No UI work until the sim produces interesting campaigns.
- **Approved mechanics (DESIGN §21–§22):**
  - Event Ledger + per-observer reputation
  - five Involvement Bands (don't expand them)
  - permanent transfers
  - trade depth
  - implied norms (**on probation**: instrument them and remove them if they're unintuitive)
  - `War → Fronts` model (one front in v0.1)
  - v0.2 order: P5 → P10 → P3 → P2
- **Historical behaviour = reflexes** (data, state-gated, capped ±40). Traits = structural characteristics; reflexes = institutional habits. Changed governments can stop behaving historically. See design/behaviour-and-voice.md.
- **First campaign is 1980–2000.** WW1, WW2, present day and others come later as scenarios. **Never hard-code era-specific content in engine code.** It belongs in scenario data or era-rule switches (DESIGN §20). Test: *would this break a WW1 scenario?*

## Hard scope constraints for v1

2D gameplay (3D presentation allowed for the map only; D55) · single-player · AI opponents · mouse-driven · no multiplayer · no tactical battle map · no individual units or soldiers · no detailed logistics · no large tech tree (3 tracks plus projects) · no province spam (~100 land regions, ~25 sea zones) · 16 playable countries.

## How to collaborate on design

- **Be critical.** The user explicitly wants pushback. If a requested feature adds complexity without decisions, say so and propose a cut or merge.
- When proposing a new mechanic, present it as a **mechanic card** (the five questions) and state what it replaces or interacts with.
- Prefer concrete examples (numbers, example turns, AI reasoning logs) over abstract description.
- Keep the scope table in DESIGN.md §19 honest. Adding content means saying what it costs.

## Working notes (session conventions)

- **Start every session by reading the "RESUME HERE" block in STATE.md.** Update it at the end of each working block.
- **Briefs arrive as long pasted documents.** Save them to `design/briefs/`. Treat approvals with changes as authoritative and record them in the design doc and STATE.md.
- **Multi-agent work.**
  - Research and design passes run as parallel `Agent` calls, never the `Workflow` tool unless the user explicitly asks.
  - Copy raw agent reports into `design/agent-reports/` only when they need preserving. Otherwise reconcile them into a single design doc and mark resolved disagreements with ⚖.
- **Report outcomes with real numbers** (test counts, decision scores, sample output). Be candid about limitations, e.g. the reactive AI or omniscient narration.
- **Tooling.**
  - In Git Bash, prefix `export PATH="$HOME/.cargo/bin:$PATH"` before cargo.
  - For complex edits, write a Python patch script to the scratchpad; long heredocs with mixed quotes can fail.
  - Re-read files after `cargo fmt` before patching.
- **No commits** until the user says so (no git repo yet).
- **Godot** (front-end, D54):
  - Godot 4.7.2 is installed via winget but isn't on PATH in Git Bash. Use the console executable for CLI work: `C:\Users\nick\AppData\Local\Microsoft\WinGet\Packages\GodotEngine.GodotEngine_Microsoft.Winget.Source_8wekyb3d8bbwe\Godot_v4.7.2-stable_win64_console.exe`.
  - The bridge crate is `crates/brink-godot` (gdext `godot` 0.5.5, feature `api-4-7`).
  - The bridge builds; `cargo test --workspace` covers it. The client loads `target/release/brink_godot.dll`, so run `cargo build --release -p brink-godot` before launching Godot. Close Godot first, because the running client locks the dll.
  - Client assets come from scripts: `tools/map_build.py` (map) and `tools/make_sprites.py` (placeholder art). Regenerate them; don't hand-edit the outputs.

## Glossary

- **Initiative:** per-turn action slots (government attention). Base 3; banks up to +2.
- **Coverage:** intelligence visibility of one country into another (0–100). Determines the width of the estimate band.
- **Tension:** bilateral (per pair) and global (Readiness Conditions 5→1) escalation measures.
- **Front:** an auto-generated war theater along a border between belligerents. Forces are allocated per front.
- **Contingency order:** an if-then standing order. Alliances and guarantees are public contingency orders.
- **Credibility(E,P):** what observer E believes about P's guarantees (Back) or threats (Threat), derived from the ledger entries E has seen. It's per observer. The global figure is display-only.
- **Event Ledger:** the shared, append-only, fog-filtered log of commitments and their outcomes. It's the only memory store.
- **Involvement Band:** 0 None, 1 Coerce, 2 Proxy, 3 Strike/Limited, 4 Major.
- **National Objectives:** per-country victory goals (1 public, 2 secret).

## Implementation agents (user-defined ownership)

Agents have explicit subsystem ownership. They may propose interface changes across boundaries, but they don't redesign another agent's subsystem. **Parallel agents must not independently modify the same core structures without coordination.**

- **Simulation/Core Agent:**
  - canonical game state
  - turn sequencing
  - deterministic RNG access
  - Event Ledger
  - commitment state
  - economy primitives
  - trade primitives
  - permanent transfers
  - war/front structures

  Must expose stable interfaces to other agents.
- **AI Agent:**
  - observer-state consumption
  - strategic goals
  - utility evaluation
  - crisis involvement-band selection
  - treaty re-evaluation
  - balancing
  - hold/withdraw/escalate
  - personality effects
  - reasoning output

  **Must never bypass fog by reading canonical hidden state.**
- **Scenario/Data Agent:**
  - 1980 setup
  - country parameters
  - starting relationships
  - traits
  - starting ledger history
  - treaties
  - bases
  - support streams
  - economic exposure values
  - event configuration

  **Must not add engine behaviour through data hacks.**
- **Test/Simulation Agent:**
  - deterministic fixtures
  - batch simulations
  - invariant testing
  - exploit tests
  - statistical summaries
  - regression tests
  - reproducibility across thread counts
- **UI Agent:** don't prioritise the full UI yet. For now it owns only the interfaces needed for debugging, consequence previews, reasoning output, event history and simulation inspection. Player-facing presentation comes after the simulation can complete coherent campaigns.

## Development discipline (user directive)

**Priority order, never reversed:**

working game → interesting decisions → coherent simulation → historical richness → additional complexity

**Judge progress by gates, not feature count** (DESIGN §22.1).

**Don't:**
- begin a major subsystem because a future feature might need it
- optimise prematurely
- rewrite working systems for elegance during v0.1 unless they block a gate

**When an agent discovers a missing mechanic, it must:**
1. describe the missing behaviour
2. show the failing fixture
3. demonstrate why existing mechanics can't represent it
4. propose the smallest fix
5. evaluate whether it belongs in v0.1 or later

No mechanic enters v0.1 through brainstorming alone. It enters because the current game demonstrably needs it.

## Code conventions

- **Language: Rust** (confirmed). Cargo workspace with these crates:
  - `sim-core`: pure simulation. No I/O, no rendering, no engine dependencies.
  - `scenario`: data loading via serde.
  - `ai`: AI decision-making against fog-filtered views only.
  - `headless`: CLI batch runner. Uses rayon for parallel AI-vs-AI runs and outputs logs and summary statistics.
  - Front-end (decided 2026-10-04, D54): **Godot 4 + godot-rust (gdext)**.
    - Godot project in `client/`.
    - Rust glue in `crates/brink-godot`, which depends on `sim-core`, `ai`, `scenario` and `voice`, never the other way round.
    - The simulation stays engine-free.
  - Visual direction (D54–D57):
    - a 3D holographic 1980s-wargame map with zoom bands (world → area → city)
    - NES-style pixel leader popups
    - 2D sprite markers on the map
    - city damage as presentation only (stark, no people)
- **Build the headless simulation first** (confirmed). The simulation core is separate from the UI and runnable for AI-vs-AI batch runs, with per-turn logs and summary statistics.
- Deterministic resolution given a seed (e.g. `rand_chacha`; never thread-local or OS randomness inside `sim-core`) for replays, debugging and balance testing.
- AI code receives a fog-filtered view type, not the true world state. Enforce "no information cheating" through the type system.
- The simulation ticks monthly internally; the player-decision cadence is configurable.
- An AI reasoning log on every decision, viewable in a debug mode.
