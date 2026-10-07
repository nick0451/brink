# Brief: Historical Behaviour and Dark Political Satire

*User direction, 2026-10-03 (paraphrased; the user's words are quoted).*

> "AI actions (in the spirit of gameplay) should follow similar actions their respective nations would've taken and reacted as… there should be a dark political satire/humor underneath the decisions in spirit of the game."

## Requirements

1. **Historically plausible behaviour.** AI countries act and react the way their real nations tended to, 1980–2000, *in the spirit of gameplay*: tendencies and reflexes, not scripts. History is still unscripted (DESIGN §20, scenario E.1).
2. **Dark political satire underneath decisions.** The tone is gallows humour about power, bureaucracy, doublespeak and the gap between stated principle and action. The simulation stays serious; the humour lives in presentation.

## Constraints carried forward (unchanged)

- **No country-identity branching** in engine or AI code (CLAUDE.md principle 11). National behaviour and voice must be **data**: personality vectors, reflex rules, spheres and red lines, voice templates.
- **No morality meter** (principle 13). The satire must not become a moral scoreboard; observers still judge via the Event Ledger.
- **Gameplay over history.** A historical tendency is a weighted bias, never a forced outcome.

## Proposed approach (pending research)

- **Reflexes:** per-country data rules of the form `WHEN <generic condition> THEN <labelled utility term>`, evaluated by generic AI code.
- **Spheres and red lines:** data references to countries, regions and chokepoints.
- **Voice layer:** per-country "register" plus template lines attached to DecisionRecords, briefing headlines, the ticker and ledger causes.
- **Satire guardrails:** punch up at institutions; never mock victims, ethnic or racial groups, or religions; refer to offices, not named individuals; human cost on screen lowers the humour volume.

## Research dispatched

Five agents: four regional behaviour researchers (Superpowers and Europe, Middle East, Asia, Africa/Americas/Tier-2) and one satire tone and voice-system designer. Their output gets reconciled into a behaviour-and-voice design for user approval.
