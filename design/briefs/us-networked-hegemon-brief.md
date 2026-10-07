# Brief: The United States as a Networked Hegemon (mechanics consolidation)

*User brief, 2026-10-03. Kept verbatim in substance. These are requirements and design context; they are not an instruction to implement every proposed feature immediately.*

## Immediate goal

Design the smallest reusable mechanics needed to make the United States an interesting, morally and strategically ambiguous playable power. Then work out how to implement those mechanics without turning them into US-only scripting.

**Don't write production code** unless explicitly asked after this design pass.

## Core design correction

The US must NOT function as the game's default "good guy." It also must not be turned into the designated villain.

The player experiences the US as a **networked hegemon**:
- enormous reach
- conflicting commitments
- competing domestic pressures
- stated principles
- strategic interests
- frequent incentives to break its own preferred norms

**Central US question:** You can usually influence the situation. What are you willing to spend, compromise, tolerate, threaten, destabilise, or commit to in order to get the outcome you want?

The game does not judge the answer morally. The simulation models the consequences. Other countries form their own judgements through:
- Trust
- Opinion
- Credibility
- Alignment
- threat perception
- historical memory
- economic dependence
- intelligence estimates

The player can behave consistently, idealistically, cynically, opportunistically, isolationistically, aggressively, diplomatically, or any mix. All should be mechanically viable in the right circumstances.

## Historical range (1980–2000, not just the Reagan years)

The US strategic environment can evolve through broad phases analogous to:
- late Cold War confrontation
- a Reagan-era buildup and proxy competition
- détente or renewed escalation
- Soviet reform, survival or collapse
- post-Cold War uncertainty
- coalition warfare (a Desert Storm-like environment)
- 1990s economic integration and trade politics
- a potentially unipolar system

**None of these is guaranteed.** The simulation creates the conditions from which analogous situations can emerge.

## Revised US archetype

- **Replace:** "Banker, policeman and arms dealer under domestic constraints"
- **With:** **Networked Hegemon — unmatched reach, conflicting commitments.**
- **Hook:** *You have more ways to shape the world than anyone else. Every shortcut teaches the world what your promises are worth.*

Exceptional tools, each exposing the US to more commitments, contradictions and second-order consequences.

## Candidate reusable mechanics (not automatically approved)

### 1. Strategic Expediency

Supporting governments, insurgencies, proxies or factions that advance strategic goals even when they conflict with stated principles. Examples of the actions involved:
- covert aid
- military aid
- proxy funding
- intelligence sharing
- sanctions relief
- diplomatic protection
- patronage
- support for opposition groups
- support for an authoritarian ally against a common rival

This must NOT be "America hypocrisy points." Different observers react differently:
- a friendly authoritarian values the support
- a democratic ally dislikes it
- a rival uses it as propaganda
- domestic consequences depend on exposure

**Objective:** the strategically easiest choice may create future political, diplomatic or security costs. It must be generic: the USSR, China, France, Iran, Saudi Arabia and others can behave the same way.

### 2. Rules-Based Leadership / Selective Enforcement

Countries make public commitments, guarantees and normative claims:
- sovereignty respected
- aggression punished
- allies defended
- proliferation unacceptable
- terrorism not tolerated
- trade kept open

They may then act consistently or inconsistently with them.

**No morality meter.** Observers remember:
- promises made
- guarantees honoured or abandoned
- similar cases treated differently
- allies protected or ignored
- threats followed through or abandoned

This feeds Credibility and Trust. The same action is interpreted differently depending on relationship and interests.

**Emergent result:** a player can enforce principles selectively for strategic reasons, but it gradually changes what others expect.

### 3. Economic Coercion

Unusually strong non-military tools:
- sanctions
- export controls
- aid
- market access
- lending
- debt restructuring
- trade agreements
- commodity embargoes
- technology restrictions
- financial pressure

These are **not free weapons**. They create:
- domestic winners and losers
- allied costs
- target adaptation
- retaliation
- substitution toward rival markets
- long-term dependency or resentment

Damaging an enemy can also damage an ally or a domestic industry.

### 4. Intervention Doctrine

Not WAR / NO WAR. A reusable escalation ladder:
1. diplomatic pressure
2. sanctions
3. covert aid
4. arms supply
5. advisors
6. proxy forces
7. punitive strike
8. limited intervention
9. coalition war
10. occupation / regime change

These need not be ten buttons; find the smallest mechanic that produces the behaviours.

Each higher level generally increases:
- cost
- visibility
- escalation
- casualties
- domestic risk
- credibility commitment
- chance of entanglement

The player should sometimes discover that *winning the initial military operation was the cheap part.*

### 5. Arms Dealer / Blowback

Transferred weapons, training, intelligence capability and financed infrastructure persist in the world state. If relations change, the former client keeps them.

**No scripted blowback event.** Blowback emerges from changed capabilities. This applies to every supplier.

### 6. Credibility without a "Hypocrisy Meter"

No Hypocrisy, Morality, Goodness or Evil resource. Use existing systems instead:
- Trust
- Credibility
- Opinion
- grievance/memory
- Alignment
- domestic Legitimacy

Reputation emerges from actions and remembered precedents. Country A may believe "the US protects allies" while Country B believes "the US abandons partners once they stop being useful." Both views exist at once. **There is no single global truth value for US trustworthiness.**

## Post-Cold War transformation

If the USSR collapses or stops being a peer, the US does **not** simply get "Cold War Victory." The rival's disappearance creates a **strategic identity problem**. Things to consider:
- allies question old alliances
- peace-dividend pressure
- military budget pressure
- clients become less valuable
- regional powers grow more independent
- US freedom of action rises
- balancing against the US may emerge
- intervention becomes militarily easier but harder to justify
- trade matters more
- economic competition with allies can rise as military rivalry falls

**Genuine choice: what is American power for now?** Possible emergent strategies:
- retrenchment
- coalition leadership
- economic integration
- unilateral primacy
- regional balancing
- democracy promotion
- selective intervention
- continued containment of emerging rivals

Not rigid doctrine trees unless necessary. Prefer consequences from ordinary mechanics.

## 1990s trade and integration

Not "+GDP because free trade." A regional trade agreement may:
- raise trade volume and aggregate growth
- deepen interdependence and specialisation
- disrupt specific domestic sectors
- make sanctions against partners more costly
- create pressure from affected domestic groups
- raise the economic cost of future conflict

It must be able to represent NAFTA-like outcomes **without hardcoding NAFTA**. The treaty itself is scenario data.

## Mechanic test (every mechanic must pass)

1. What decision does it create?
2. What does it cost?
3. What other systems does it interact with?
4. How does the AI reason about it?
5. How does the player understand what happened?
6. Can an existing mechanic already produce the same outcome? → **if yes, merge**
7. Does it need to exist in v0.1? → **if no, defer**

Also:
- Don't expand the roster.
- Don't research more historical edge cases unless a concrete mechanics question needs it.
- Don't add a mechanic just because something happened historically.

## Multi-agent workflow (as briefed)

**Agent 1 — Core Systems Designer.** Which existing BRINK mechanics already support the desired US gameplay? Examine:
- Initiative, Budget, Stability
- Trust, Credibility, Opinion, AI memory
- patronage, sanctions, trade
- alliances, guarantees, proxy support
- military escalation, world tension

Output four lists: sufficient / needs modification / truly new / rejected. Strongly prefer extending existing systems over new resources.

**Agent 2 — US Gameplay Designer.** Design the playable US with the smallest mechanics. Write 6–10 dilemmas across four periods:
- early 1980s
- late 1980s
- early 1990s (rival collapse plus a regional aggression crisis)
- mid/late 1990s (trade, instability, alliances, unipolarity)

For each dilemma give:
- situation
- information available
- 3–5 options
- immediate benefit
- immediate cost
- second-order risk
- systems affected
- AI reactions

Multiple sensible answers each; no designer-approved moral answer.

**Agent 3 — Generalisation / Anti-Special-Case Reviewer.** For every "American" mechanic, ask whether the USSR, China, France, Saudi Arabia, Iran, or a WW1/WW2 scenario could use it. If so, generalise it. For example:
- hypocrisy → commitment consistency + observer memory
- blowback → persistent transferred capabilities
- intervention doctrine → generic escalation scale
- NAFTA → economic integration treaty

Output: generic engine primitives / justified US-specific traits / scenario data / historical flavour only.

**Agent 4 — AI Designer.** No `if actor == USA` logic. Reactions emerge from:
- relationship
- past behaviour
- interests
- ideology/alignment
- dependence
- threat perception
- personality
- treaties
- intel estimates

Design utility considerations for:
- accepting US aid
- accepting US bases
- joining sanctions
- opposing US intervention
- requesting a guarantee
- abandoning the US bloc
- balancing against excessive US power
- exploiting US inconsistency
- staying aligned despite ideological disagreement

Model the US AI with the same framework. Output utility inputs and reasoning rules, not scripts.

**Agent 5 — Minimal-Implementation Reviewer.** Goal: a complete playable state as fast as reasonably possible. Classify each mechanic:
- **V0.1 CORE**
- **DEFERRED**
- **SCENARIO LAYER** (traits, events, treaties, data)
- **CUT**

Be ruthless. Output a dependency graph.

**Agent 6 — Adversarial Design Reviewer.** Try to break the design. Look for:
- dominant US strategies
- consequence-free intervention
- sanction spam
- free proxy war
- infinite alliance stacking
- credibility exploits
- economic coercion exploits
- uncounterable US power
- AI dogpiling the US just for being strong
- AI refusing rational cooperation for ideological reasons
- hidden moral assumptions
- historical flavour overriding agency

Test whether a ruthless, restrained, isolationist, interventionist and trade-focused US are each viable, with none automatically optimal. Output:
- exploits
- missing costs
- missing counterplay
- tedium
- fixes (fixes only for demonstrated problems)

## Reconciliation: one consolidated report (not concatenated)

1. Design thesis (one paragraph)
2. Existing mechanics reused
3. New generic mechanics (only those surviving the reviews)
4. US traits (about 2–4)
5. US strategic actions
6. AI reasoning
7. Post-Cold War transition
8. Example turns (≥3 concrete US turns)
9. V0.1 implementation list
10. Deferred mechanics
11. Scenario-layer content
12. Rejected mechanics, recorded so future agents don't re-propose them

## Later: implementation planning (separate pass, after report approval)

**Ownership:**
- **Simulation:** state, turn resolution, economy, reputation/memory, transferred capabilities.
- **AI:** utility, memory, reactions, goals, behaviour tests.
- **Scenario/Data:** US values, traits, treaties, bases, events, relationships.
- **UI:** actions, previews, reputation feedback, "because" chains, memory display.
- **Test:** headless scenarios, deterministic fixtures, exploit detection, statistics.

Agents may propose interface changes across boundaries but don't redesign another agent's subsystem.

## Implementation gates

1. **One interaction works.** USA, an ally, a rival and a neutral. The US can aid, sanction, threaten, guarantee and support a proxy. *Pass:* past behaviour measurably affects future diplomatic decisions.
2. **Contradictory incentives.** Supporting a useful partner helps security, hurts another relationship, costs money, and shifts credibility or trust. *Pass:* no universally superior answer.
3. **Intervention escalation.** Choices: no action / sanctions / proxy / limited military / major intervention. *Pass:* each option solves some problems and creates others.
4. **Blowback without scripting.** Arm a client, then change relations. *Pass:* the former client remains stronger with no special event.
5. **Post-rival world.** Collapse the peer competitor. *Pass:* still strategically interesting, with changed incentives and no separate ruleset.
6. **AI plays both sides.** Repeated headless games with an AI US. *Pass:* it sometimes intervenes, sometimes restrains itself, sometimes coerces economically, and sometimes accepts losses rather than escalating.

## Primary constraint

The goal is a playable strategy game whose systems can generate recognisable geopolitical behaviour, not a perfect simulation of US foreign policy. **Historical plausibility informs the systems but never overrides gameplay. When in doubt, build the simpler system, make it playable, see what breaks, and add complexity only where the game proves it needs it.**
