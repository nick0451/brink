# BRINK — Prototype Design Document

*Working title. Version 0.3 — merges the approved Networked Hegemon mechanics report ([design/us-mechanics-report.md](design/us-mechanics-report.md)): Event Ledger, per-observer reputation, Involvement Bands, permanent transfers, trade depth, implied norms, v0.1/v0.2 milestones. No code yet.*

> A 2D grand-strategy game of geopolitical competition. 1980s wargame presentation, Plague Inc.-style map readability, modern systemic design. **Simple levers, tangled consequences.**

---

## 0. The Design Thesis (read this first)

Most grand strategy games give the player too many levers. Most wargames make war the only lever. BRINK aims for a small number of levers that all push on the same few shared quantities, so every action spills into other systems.

**Three things the player spends:**

| Currency | What it is | Why it exists |
|---|---|---|
| **Budget** | Money per turn, split across four spending lines | Sets the long-term direction of the state |
| **Initiative** | 3–6 action slots per turn: the government's attention | Forces hard choices. You can't do everything, even when you're rich |
| **Stability** | The regime's capacity to absorb pain | Every bold action draws on it. If it runs out, you lose |

**Three things that connect the world:**

| Connective tissue | What it carries |
|---|---|
| **World Market** | Commodity prices. A war in one region becomes a recession somewhere else |
| **Tension** | Bilateral and global escalation. It decides what is possible and what is dangerous |
| **Estimates** | Every country, the AI included, sees the world through imperfect intelligence. Bluffs work. Surprises happen |

**The core design pattern: state persists, consequences emerge.** BRINK prefers simple state plus interacting rules plus persistent consequences over scripted events and special cases:
- Blowback comes from permanently transferred weapons.
- Peace-dividend pressure comes from military spending outrunning the threat.
- Alliances weaken because their strategic value changes.
- Reputation comes from remembered actions (§21).
- Historical crises emerge from state conditions.

**No country is the good guy or the villain.** The simulation records behaviour; each observer draws its own conclusions. There is no morality, hypocrisy, goodness or evil meter.

**The shape of play:** nearly every non-military action is ultimately an attack on, or a defence of, someone's **economy or stability**. War is the most direct and most expensive way to do the same thing. The player should usually find a cheaper indirect route, and the tension comes from knowing that sometimes there isn't one.

**The test applied to every mechanic in this document:**
1. What decision does it create?
2. What does it cost?
3. What does it touch?
4. How does the AI reason about it?
5. How does the player see the result?

A mechanic that can't answer all five is cut or merged. Section 18 lists what has already been cut or put on probation.

---

## 1. Core Gameplay Loop

### The turn loop (target: 1–3 minutes per turn in peacetime)

```
 ┌─► BRIEFING ──► ASSESS ──► DECIDE ──► COMMIT ──► RESOLUTION ──┐
 │   news ticker,  map        budget,     orders     playback on   │
 │   alerts, intel overlays,  initiatives,lock for   map, ticker,  │
 │   reports       estimates  postures    everyone   cause→effect  │
 └──────────────────────────────────────────────────────────────────┘
```

1. **Briefing.** A short prioritised list: 3–7 items, each one clickable to its map location. "Kavaran mobilization detected (confidence: moderate)." "Oil +18% — Qasr embargo." "Domestic: protests in the Southern Region."
2. **Assess.** Flip between map overlays (Tension, Stability, Trade, Military estimates, Intel coverage, Alliances, Resources). The overlays are the main way to read the game. Plague Inc. readability lives here.
3. **Decide.** Adjust standing settings only if needed (budget, mobilization, theater postures). Then spend Initiative on discrete actions.
4. **Commit.** All countries' orders lock at the same time.
5. **Resolution.** An animated playback on the map with a scrolling ticker. Every significant change carries a "why" link to its cause.

### The campaign loop (hours)

- **Early:** read the board, pick a strategic direction, build networks (trade, spies, alliances).
- **Middle:** crises cascade. Two AIs go to war, prices spike, a neighbour destabilises, a bloc forms. The player exploits, contains, or gets dragged in.
- **Late:** pursue national objectives, survive the consequences of earlier choices, manage escalation as power concentrates.

### Why it shouldn't become tedious

Most settings are **standing orders**. A peaceful turn can be "read briefing → spend 3 Initiative → end turn." Complexity rises only when the situation gets complex.

---

## 2. Resources the Player Manages

Deliberately few. Each one exists because it creates a distinct trade-off.

### 2.1 Budget (money)

`Revenue = GDP × tax rate (set by government type) + commodity export income − debt service`

Revenue is split across **four spending lines** using sliders:

| Line | Buys | Neglecting it causes |
|---|---|---|
| **Military** | Force production, upkeep, readiness | Readiness decays and forces shrink |
| **Development** | Research, regional infrastructure, national projects | Economic stagnation and tech lag |
| **Welfare** | Prosperity, which feeds stability | Unrest, especially in democracies |
| **Intelligence** | Spy networks, counterintelligence, operations | Blindness, vulnerability to subversion |

- **Decision:** guns vs. growth vs. contentment vs. information. This is the master strategic dial.
- **Cost:** zero-sum within revenue. Deficit spending is allowed, but it adds **Debt**.
- **Inertia:** changes phase in over 2–3 turns. The military budget especially can't ramp instantly, and a sudden military increase is visible to foreign intelligence as a warning sign.
- **Feedback:** a budget panel shows projected effects ("Readiness will fall to 61% in 3 turns at this level").

### 2.2 Debt

Kept because it's a real decision: **borrow to win now, pay later.**

- Debt raises debt service (a permanent revenue drain).
- Debt above roughly 60–90% of GDP (it depends on traits) raises **default risk events**, which damage trade and stability and spread through trade partners. This is the financial cascade.
- Wealthy maritime and trade economies borrow cheaply. Unstable regimes don't.
- **AI:** risk-tolerant AIs borrow for wars; economic AIs avoid debt.
- **War consumption on credit (issue 21):** a mobilized belligerent borrows its war bill (munitions, fuel, pay) automatically: 3% / 12% / 25% of GDP a year at Partial / Full / Total mobilization. Peace, or a war fought unmobilized, borrows nothing. Debt service then follows from the world rate (§11.4c). Creditors are not yet tracked (follow-up: war loans from friends as a creditor relationship).

### 2.3 Initiative (action slots)

The government's political and administrative bandwidth. **This is the key opportunity cost in the game.**

- Base 3 per turn. +1 at Stability > 70, +1 or −1 from traits or government type, −1 during a domestic crisis.
- **Costs Initiative:** diplomatic proposals, threats, sanctions, launching intelligence operations, expanding spy networks, changing mobilization level, launching offensives, starting national projects, domestic policies (reforms, crackdowns).
- **Doesn't cost Initiative:** budget sliders, theater postures, accepting AI proposals, viewing information.
- **Unspent Initiative** banks up to +2 for later turns. This lets the player save for a crisis turn, and that choice is itself a decision.
- **Why it matters:** a rich country can't simply do everything. A small country with high stability and a good trait can be as active as a superpower.

### 2.4 Strategic commodities (three)

**Energy, Materials, Food.** Each country produces some, consumes some, and trades the balance on the **World Market**.

| Commodity | Shortage effect | Who it empowers |
|---|---|---|
| **Energy** | GDP growth penalty, military readiness penalty | Petro-states, chokepoint holders |
| **Materials** | Military production and infrastructure slowed | Resource exporters |
| **Food** | Stability penalty, famine events | Agricultural exporters (an often overlooked kind of leverage) |

- Prices are set globally by supply and demand. Wars, embargoes, blockades and disasters move them.
- Imports must travel along **trade routes through sea zones**. Close a chokepoint and importers suffer even if they aren't at war.
- **Critique and safeguard:** Food is the most likely of the three to be cut. Keep it only if playtests show famine and agricultural leverage producing real decisions. The fallback is to merge it into a "Basic Goods" stability input. *Note:* nuclear winter (§11.4) works through global food production, which gives Food a second job.

*v0.2 as built (D48):*
- Energy only.
- Demand = GDP; supply = capacity × production policy (Restrain 0.85 / Normal 1 / Flood 1.2) × 0.7 at war, plus unmodelled outside supply.
- Price = base × (demand ÷ supply)^5, smoothed, with capacity investment rising with the price.
- Economies feel only the price's deviation (growth and revenue, capped).
- Materials and Food deferred.

### 2.5 Manpower

Derived, not managed: `Manpower pool = population × mobilization rate × doctrine modifier`. Casualties drain it, and it regenerates slowly. It mostly matters at Full or Total mobilization, and for low-population, high-tech states that can't afford attrition.

### 2.6 Stability (a meter, not spent)

Covered in Section 9. It acts as a budget of tolerable pain.

---

## 3. Major Strategic Systems and How They Interact

### 3.1 System list (seven systems, plus events)

1. **Economy & Trade.** GDP, growth, budget, debt, trade, World Market.
2. **Military.** Force pools, theaters, readiness, supply, mobilization.
3. **Diplomacy.** Relations, trust, treaties, credibility, coercion.
4. **Intelligence.** Coverage, estimates, operations, counterintelligence.
5. **Domestic Stability.** Stability, its drivers, government type.
6. **Research.** Three tracks plus national projects.
7. **Escalation.** Bilateral and global tension, the escalation ladder.
8. **World Events.** State-triggered, not random. They sit on top of the other systems.

### 3.2 Interaction matrix (row affects column)

| → | Economy | Military | Diplomacy | Intel | Stability | Escalation |
|---|---|---|---|---|---|---|
| **Economy** | — | Funds forces; materials and energy gate production | Trade creates interdependence and opinion | Funds networks | Prosperity driver | Sanctions and embargoes raise tension |
| **Military** | War damage, blockade, upkeep drain | — | Power shapes threats, deterrence, ally value | Buildup is observable | Casualties raise war weariness; victories raise legitimacy | Deployments and mobilization raise tension |
| **Diplomacy** | Trade agreements, aid, sanctions | Alliances drag you into wars; basing gives reach | — | Allies share intel | Popular or unpopular alliances; humiliation | Guarantees deter, or chain-react |
| **Intel** | Sabotage, tech theft | Accurate estimates enable or deter war | Exposed operations cause incidents | — | Funding the opposition | Detection raises tension; disinformation changes perceived balance |
| **Stability** | Unrest cuts output | Low stability cuts readiness; coups | Unstable states look weak (invitation) or desperate | Unstable states are easier to penetrate | — | Desperate regimes escalate ("diversionary war") |
| **Escalation** | High global tension slows trade and spikes prices | Unlocks war; drives arms races | AIs seek alliances under threat | AIs raise intel spending | Rally effect when threatened; fear | — |

### 3.3 The core feedback loops

These are the loops that make the game interesting. They're the reason these systems exist.

1. **Security dilemma loop.** I build up because I fear you → your intel sees my buildup (with error) → you build up → my threat perception rises. *Brakes:* budget cost, intel accuracy, diplomacy, and the arms-control treaty option.
2. **Resource cascade loop.** A war or embargo hits a producer → the price spikes → importers' growth falls → fragile importers' stability falls → unrest, revolutions, new opportunities for others. *This is the Plague Inc.-style spread mechanic, and you can watch it on the map.*
3. **Overextension loop.** Conquest gives regions → occupied regions resist and produce little → occupation costs military and stability → neighbours balance against you. *Brake on snowballing.*
4. **Credibility loop.** Honour a guarantee and credibility rises, so future guarantees deter more. Abandon an ally and credibility collapses worldwide, so your alliances weaken and rivals test you.
5. **Diversionary war loop.** Low stability → a regime tempted to escalate abroad for a rally effect → a risky war that either saves or destroys it. The AI uses this too, which makes unstable neighbours dangerous.
6. **Interdependence loop.** Trade raises GDP and mutual opinion, but it also creates vulnerability to sanctions and blockade. Self-sufficiency is safe but poor.

---

## 4. Turn Structure and Resolution Sequence

### 4.1 Simultaneous vs. sequential: evaluation

**Recommendation: simultaneous turns.** They're the stronger choice for this design.

| Factor | Simultaneous | Sequential |
|---|---|---|
| Value of intelligence | **High.** You must predict what others will do this turn | Low. You see their move before yours |
| First-mover advantage with 16 countries | None | Severe and arbitrary (turn order decides) |
| Feel of a "crisis" | Strong. Uncertainty, commitment | Weak. Reactive chess |
| AI interaction speed | All AIs act together; fast | 15 AI turns to sit through, or hide |
| Bluffing and deterrence | Natural | Hard |
| Clarity of resolution | **Harder.** Needs strong playback and causality | Easy |
| Ability to react to attacks | **Harder.** Needs mitigations | Natural |

**Changes recommended to make simultaneous turns work:**

1. **Nothing large is instant.** Military buildup, mobilization and redeployment take turns and are observable. Surprise attacks are possible only against poor intelligence. This turns "I couldn't react" into "I didn't read the warnings," which is the fun version.
2. **Contingency orders.** A small number of if-then standing orders: "If anyone attacks Qasr, enter the war." "If Kavaran mobilizes past Partial, mobilize to Partial." **Guarantees and alliances are contingency orders** that the diplomacy system makes public. This merges two mechanics into one.
3. **Deterministic phase order** (below). Conflicting simultaneous actions resolve in a fixed order, so outcomes are learnable.
4. **Causal playback.** Every change in the resolution ticker links to its cause.

### 4.2 Turn length

The tentative model of one turn per month gives roughly 240 turns for a 20-year campaign. That's too many decisions about slow-moving economies, and too few per war.

**Recommendation:**
- The **simulation ticks monthly** internally (combat, prices, unrest move monthly).
- The **player decides once per season (3 ticks)** by default. A 20-year campaign (for example 1980–2000) is about 80 decision turns.
- Make the cadence a configurable parameter in the prototype and **playtest both** 1-tick and 3-tick decision turns. Possible later variant: monthly turns during war, quarterly in peace.
- In the playback, the three ticks appear as three "months" of animation, so wars still feel granular.

### 4.3 Resolution sequence (per tick; orders apply at tick 1)

```
0. ORDERS LOCK        player + all AIs (AIs use only their own estimates)
1. DIPLOMACY          proposals accepted or rejected against START-of-turn state;
                      war declarations; contingency orders trigger (allies join)
2. INTELLIGENCE       network growth; operations resolve; detection rolls →
                      exposed ops become Incidents (tension + relations hit)
3. MOVEMENT           redeployments progress; mobilization levels step
4. COMBAT             each active front resolves; region control shifts;
                      sea-zone control updates (affects step 5)
5. ECONOMY            production; trade flows (blockades and chokepoints applied);
                      World Market prices clear; budget, debt
6. RESEARCH           track progress; projects advance; tech diffusion
7. DOMESTIC           stability drivers recomputed; unrest; elections/coups
8. WORLD EVENTS       trigger checks against the NEW state
9. TENSION & LEDGER   tension changes and decay; commitment tests opened/graded;
                      Event Ledger entries appended with visibility (§21)
10. REPORTING         fog-filtered reports generated per country → Briefing
```

**Why this order:** diplomacy first, so alliances triggered by this turn's attacks join before combat. Intelligence before combat, so sabotage matters. Combat before economy, so blockades bite immediately. Domestic after economy, so price shocks hit stability in the same turn. Events last, so they react to the current state.

---

## 5. Map and Region Structure

### 5.1 Granularity recommendation

**About 70 land regions and about 24 sea zones for 16 countries.**

| Country size | Regions | Example |
|---|---|---|
| Microstate / city-state | 1 | Small tech state |
| Small | 1–2 | Petro-state |
| Medium | 2–4 | Regional power |
| Large | 4–8 | Continental power |

Plus **neutral or minor regions**: unplayable minor states represented as single regions with simple AI. They work as buffers, proxy battlegrounds and sources of resources.

**Why this level:** the military decision we want is "which front, how much force, what posture." That needs 2–6 meaningful targets per war. With about 70 regions, a two-country border has 1–4 contested region pairs, which means real choices and no micromanagement. With 300 or more regions, war turns into unit-pushing. With fewer than 40, geography stops mattering.

### 5.2 Region attributes

| Attribute | Effect |
|---|---|
| **Population** | Manpower, GDP base, resistance strength when occupied |
| **Industry** | GDP and military production share; a sabotage and bombing target |
| **Resources** | Energy/Materials/Food output |
| **Terrain** | Open / Rough / Mountain / Jungle / Arctic / Urban: defence multiplier, supply penalty |
| **Infrastructure (1–5)** | Supply capacity, redeployment speed, GDP multiplier. Built with the Development budget |
| **Port** | Naval basing, trade access, amphibious target |
| **Airbase** | Air range projection |
| **Core owner** | Non-core (occupied) regions yield 25–50% and generate Resistance |

### 5.3 Sea zones and chokepoints

- Sea zones carry **trade routes** and are controlled by naval strength.
- **Chokepoints** (straits and canals, about 6–8 globally) are owned by the adjacent land region. The owner can close them, at a large tension cost, and whoever controls the region controls the strait.
- This is how **small countries matter**: a one-region state on a strait can hold a superpower's oil supply hostage.

### 5.4 Setting

**Decided: real countries on a stylised real-world map.**

What this means for the design:
- **The roster is a curation problem.** The ~16 playable countries must cover the asymmetry archetypes *and* sit in interacting regional clusters (powder kegs). Everything else is a **minor region** with simple AI.
- **Countries can be grouped or simplified where needed.** A small country that isn't playable may be represented as a single minor region. That's still a real country, just not a full actor.
- **Starting data is historically grounded but abstracted.** Values are indices, not real statistics. Plausibility matters more than accuracy, and balance can override history.
- **Real countries make readability easier**, since players already know who borders whom, who has oil and who has the bomb. The cost is that players arrive with expectations, so deviations need a reason.
- **Sensitivity.** A historical start date gives distance, and the outcome is player-driven. Events are written generically ("Revolution in X"), never as depictions of real atrocities.
- **First campaign: 1980–2000** (decided). Cold War bipolarity, oil shocks, revolutions and proxy wars match the design's systems almost one to one. Other eras (WW1, WW2, present day) come later as separate scenarios. See §20.

---

## 6. Military Model

**No unit micromanagement.** Forces are abstract **strength pools**. The player's decisions are *how big*, *how ready*, *where concentrated*, and *how aggressive*.

### 6.1 Force composition

Each country has three pools, each with strength points and a quality level:

| Pool | Role |
|---|---|
| **Land** | Holds and takes regions |
| **Naval** | Controls sea zones: trade protection, blockade, amphibious operations, overseas supply |
| **Air** | Theater multiplier (air superiority), reconnaissance (feeds intel), strategic bombing |

Plus the **Strategic Arsenal**, a deterrent level (0–3), described in Section 13.

- **Quality** = Military research level × doctrine fit.
- **Readiness** (0–100%) = the share of strength that's usable. It's funded by the Military budget and capped by Mobilization.
- **Production:** Military budget plus Materials and Energy buys strength with a **2–4 turn lead time**. Buildups are visible before they're ready.
- **v0.1 as built (D31):**
  - Power = Σ pool strength × quality/5 × (0.5 + readiness).
  - Readiness heads toward the mobilization cap × funding × **command**, where funding = (production + arms received) ÷ upkeep, capped at 1. Under-funded forces go hollow before they shrink.
  - **Command (issue 20, D88):** an army fights for a regime it trusts under officers its rulers trust. Command = min(Stability ÷ 35, turns since a revolution or a conqueror-installed regime ÷ 12), each capped at 1. A regime in crisis loses men to desertion and broken command (Russia 1917); rulers brought by a revolution (a collapse) or installed by a conqueror purge the officer corps and rebuild it over 3 years (Iran 1979–82; the Red Army 1937–42). A military coup purges no one: the army was the coup (Turkey 1980, Argentina 1976, Pakistan 1977, Korea 1979); nor does a negotiated reform. At command 0 the force fights at half value (the readiness floor). Others see it only through their own estimate of the regime's power.
  - New production enters at the country's Military tech. Lead time and commodities arrive with regions and the market.

### 6.2 Mobilization (four levels)

| Level | Readiness cap | Manpower | Economic cost | Tension | Stability |
|---|---|---|---|---|---|
| Peacetime | 50% | Base | — | — | — |
| Partial | 75% | ×1.5 | −5% GDP | + Moderate | Slight hit, or rally if threatened |
| Full | 100% | ×2.5 | −15% GDP | + Large | Needs justification |
| Total | 100%+ | ×4 | −30% GDP | + Extreme | Unsustainable for long |

- Stepping up costs 1 Initiative and takes 1 tick per level. Stepping down is free but slow.
- **Decision:** mobilize early (safe, expensive, provocative) or late (cheap, risky)?
- **AI:** mobilizes when threat perception crosses a personality-weighted threshold. Paranoid AIs mobilize early, which can trigger security spirals.

### 6.3 Theaters and fronts

- At peace: land forces are assigned to **regional deployments** (Home Reserve, border regions, overseas bases).
- At war: **fronts** are generated automatically along the borders between belligerents. A front is a group of contested region pairs.
- **Data model: `War → collection of Fronts`.** v0.1 instantiates **one front per pair of belligerents**. That's a prototype simplification, *not* an architectural assumption. Multiple theatres (a Central European front, a Far Eastern front, overseas theatres) must be addable later without redesigning the war model.
- Per front, the player sets:
  - **Force allocation** (% of the available pool)
  - **Posture:** Defend / Hold / Limited Offensive / Major Offensive
  - **Focus region** (optional): concentrate on one objective region along the front
- Redeploying between fronts takes ticks, based on distance and infrastructure. Overseas fronts need sea-lane control.

### 6.4 Supply

Each front has a **Supply Capacity** = infrastructure of the adjacent friendly regions + bonus for proximity to the home core + sea-lane supply (if naval control holds).

- Forces beyond supply capacity fight at reduced efficiency. **This prevents doomstacks and makes geography matter**: invading through mountains, or across an ocean without naval control, is hard.
- There's no detailed logistics simulation. It's one number per front.

### 6.5 Combat resolution (per front, per tick)

```
Attack = Σ(strength × quality) × readiness × supply_eff × posture_mult
         × air_superiority_mult × doctrine_mult
Defence = same × terrain_mult × fortification × (home-core bonus)
Ratio → outcome table (with variance) → casualties both sides,
        front-progress points toward the focus/contested region
Region flips when progress ≥ 100
```

- **Major Offensive:** higher progress, much higher casualties and war weariness.
- **Defend:** low casualties, no progress.
- Variance is real. Intelligence estimates of the ratio are shown before committing, *with uncertainty bands*.
- *As built (issue 6, smoothed in issue 13):* the home-core bonus (×1.5) goes to the side whose territory the front is in, not to whoever was declared on; it fades continuously over the first 25 progress into the invader's territory rather than flipping at the border (§11.4c).

### 6.6 War aims and peace

- When declaring war, the declarer selects **war aims** (e.g., take Region X, regime change, end blockade). Bigger aims mean more tension, more domestic resistance in democracies, and neutral countries leaning against you.
- **`Punitive` war aim** (any domain). A short operation against military or programme targets. It ends only if the target accepts peace; otherwise it becomes a limited war. It leaves a Coercion entry in the Event Ledger (§21). It also covers strikes on weapons programmes.
- **War Score** accumulates from regions held, casualties inflicted, blockade damage and war-weariness differential.
- **Peace deals** spend War Score: cede regions, reparations, demilitarised zones, break alliances, puppet/regime change (expensive).
- The AI accepts peace when its projected outcome of continuing is worse than the deal (its estimates, its personality).
- **Wars end.** Exhaustion forces negotiations, which prevents attrition slogs.

### 6.7 Occupation and Resistance

Occupied non-core regions accumulate **Resistance**: population × hostility × foreign support. Resistance cuts output, demands garrisons, and **can be sponsored by other countries**. That's the hook for proxy war.

### 6.8 Mechanic card: War

- **Decision:** is the gain worth the money, casualties, stability, tension, credibility and trade loss, *and* what will third parties do?
- **Costs:** Military budget; production lead time; Initiative (declare, offensives); manpower; stability (war weariness); trade (enemy plus sanctions from others); tension (balancing coalitions).
- **Interacts with:** every system.
- **AI reasoning:** expected-value calculation over *estimated* force ratios, the likely interventions of allies (from treaties and credibility), the value of war aims toward its goals, and domestic tolerance. Scaled by risk tolerance.
- **Feedback:** front lines on the map, war-score bar, casualties chart, war-weariness meter, ticker reports, neutral countries' reactions.

---

## 7. Diplomacy Model

### 7.1 Core values

| Value | Scope | Meaning |
|---|---|---|
| **Opinion** (−100..+100) | Per pair, directional | How much A likes B now. A sum of visible, decaying modifiers |
| **Trust** (0..100) | Per pair, **derived** | Track record of commitments where the observer was the beneficiary or target. Read from the Event Ledger (§21) |
| **Credibility** (0..100) | **Per observer, derived** | What *this observer* believes about a country's guarantees (Back) or threats (Threat), based on the ledger entries it has seen and how similar they are to its own case (§21.3) |
| **Alignment** | Per country | Optional bloc or ideology tag that gives opinion affinity |

Opinion modifiers are always shown as a list ("+20 trade partners, −25 border tension, −15 exposed spy ring, +10 shared rival"). No hidden diplomacy math.

**There is no authoritative global Credibility.** Different observers can rationally reach different conclusions about the same country:
- A Gulf ally trusts US guarantees because it saw the US defend its region.
- An African client doubts them because it saw peripheral clients dropped.
- A rival believes US threats but not US guarantees.

A global Credibility number exists **for display only** (a power-weighted average). **The AI never reads it.**

**Third-party reaction:** when a country acts toward B, every observer that sees it adjusts its Opinion of the actor by `magnitude × [0.6·Opinion(observer→B)/100 + 0.4·ideology(observer)·affinity(observer,B)]`. Backing a friend's enemy costs you that friend. Backing an authoritarian ally against a common rival pleases its friends and displeases ideological democracies.

### 7.2 Treaties

| Treaty | Effect | Commitment |
|---|---|---|
| **Trade Agreement** (`depth`: shallow / deep) | +trade value, +opinion. **Deep:** larger trade and growth multiplier; sanctions or war between the partners become much costlier; a temporary adjustment shock to each partner's exposed commodity category (measured against that category, not total GDP). Economic integration can become an alternative organising structure to military blocs. NAFTA-like deals are scenario data | Low (shallow) / Medium (deep) |
| **Non-Aggression Pact** | Lowers bilateral tension; breaking it is a huge trust hit | Medium |
| **Defensive Alliance** | Mutual contingency order: attack on one = war with both | High |
| **Guarantee** (unilateral) | Public contingency order: protects a weaker state | High, credibility at stake |
| **Basing / Access** | Military access; overseas supply and air range | Medium; raises tension with the host's rivals |
| **Arms Supply** | Transfers military strength for money or influence. **Transfers are permanent** (§21.6): the recipient keeps them whatever happens to relations later | Medium |
| **Arms Control** | Caps military or arsenal growth; reduces tension | Medium; verification depends on intel |

**Standing support streams.** Aid or Arms Supply can be set as a recurring stream. Setting it up costs 1 Initiative; keeping it running costs 0. Cutting it while the client is in need opens a commitment test (§21.2). Client dependence emerges naturally. This replaces a separate "Patronage" treaty.

**Accepting proposals.** Accepting an AI proposal that **creates a commitment** (alliance, guarantee, basing, stream) costs **1 Initiative**. Refusing costs only Opinion, unless an existing commitment covers the request; then the refusal is graded as a test.

### 7.3 Actions (cost Initiative)

- **Propose treaty** (one per target per turn)
- **Aid** (money → opinion, stability for the recipient; builds a sphere of influence). A one-off Aid action costs 1 Initiative.
- **Sanction** (cuts trade with the target. **Costs the sanctioner too**; gets stronger when allies join). Scope in v0.1: trade or commodity.
  - **Substitution:** each turn a sanction is in force, the target reroutes +10% of the lost trade to non-participants. This adaptation **persists after the sanction is lifted**, so on/off cycling erodes the weapon. The same trade-cut function serves Embargo and chokepoint/naval closure.
  - The sanctioner's own lost exports show as a **visible Prosperity modifier**, scaled by per-commodity export exposure weights (data). That's how domestic winners and losers appear without a sector model.
- **Embargo commodity** (stop exporting energy, materials or food to the target, or to everyone)
- **Threaten / Ultimatum** ("Withdraw from Region X or face war." Strength = your perceived power × the **target's** Credibility(Threat) reading of you. Creates a Threat test in the Event Ledger: follow through or be graded Abandoned)
- **Denounce** (cheap opinion and tension tool; signals alignment)
- **Mediate** (between two other countries: lowers their tension, gains influence. Diplomatic AIs love this)

### 7.4 Negotiation

**No haggling UI.** The player builds a single package (treaty plus optional sweeteners: money, trade terms, arms) and sees the AI's evaluation **before** sending it:

```
Kavaran will REJECT  (score −12)
  +18  shared rival: Northern Federation
  +10  trade benefit
  −25  fears drawing Commonwealth intervention
  −15  distrust: you broke NAP in 1983
```

### 7.5 Mechanic card: Diplomacy

- **Decision:** whom to bind yourself to, and at what risk of being dragged into their wars. Whether to coerce or court.
- **Costs:** Initiative; money (aid, sweeteners); freedom of action (commitments); credibility (if you fail to honour them).
- **Interacts with:** Military (allies' power counts in deterrence; contingency entry into wars), Economy (trade, sanctions), Escalation (alliances and guarantees raise rivals' tension), Intel (allies share coverage), Stability (popular or unpopular partners).
- **AI reasoning:** utility of the treaty toward its goals (security vs. its top threat, economic gain) minus risk of entrapment, weighted by trust and personality (loyal vs. opportunistic). **Balance of power:** AIs actively seek alliances against the strongest perceived threat.
- **Feedback:** alliance overlay, opinion breakdowns, credibility meter, AI reaction headlines.

---

## 8. Intelligence and Fog-of-War Model

### 8.1 Principle

**Nobody sees the truth, including the AI.** Every foreign statistic is displayed as an **estimate with a confidence band**. The AI makes decisions on its own estimates, so it can be bluffed, surprised and misled.

### 8.2 Coverage

For each observer → target pair, **Coverage** (0–100) =
- Spy network level in the target (0–5, the main source)
- \+ Reconnaissance (Intelligence research: aircraft → satellites)
- \+ Proximity (neighbours see border deployments)
- \+ Shared intelligence from allies (partial)
- \+ Target openness (democracies leak; closed states are opaque)
- − Target counterintelligence

### 8.3 What coverage reveals

| Coverage | Visible |
|---|---|
| 0–20 | GDP, rough military (±50%), government type |
| 20–40 | Military ±25%, mobilization level, stability band |
| 40–60 | Theater deployments, research tracks, treaty negotiations |
| 60–80 | Military ±10%, budget allocation, active ops against you (chance) |
| 80+ | **Intentions:** AI current strategic goals, war plans, secret objectives |

Estimates also **lag**. Low coverage shows stale data ("as of 2 turns ago").

### 8.4 Operations (cost Initiative + Intel budget; need a minimum network level)

| Op | Network | Effect | If exposed |
|---|---|---|---|
| **Expand network** | — | +1 level over several turns | Small incident |
| **Steal technology** | 2 | Progress toward the target's lead track | Incident, opinion hit |
| **Sabotage** | 2 | Damage industry or infrastructure in one region | Major incident |
| **Disinformation** | 2 | Skews the target's estimates of *you*: look stronger (deterrence) or weaker (mask buildup or bait) | Credibility hit |
| **Fund opposition** | 3 | −Stability in target | Major incident, tension |
| **Arm insurgents** | 3 | +Resistance in occupied or unstable regions | Major incident |
| **Coup** | 5 | Chance of regime change toward your alignment | Crisis-level incident |

Detection chance = base + target counterintelligence − your Intelligence tech. Exposure becomes an **Incident**: tension and opinion hit, a ticker headline, and a possible casus belli.

**Exposure has a domestic cost too.** It's scaled by the actor's **`openness`**: a 0–1 per-country value that defaults from government type and can be overridden by data. Open societies pay more when caught. This replaces every "if democracy" switch. Covert acts also produce covert Event Ledger entries, which become **Exposed** (and highly salient) when detected (§21.1).

### 8.5 Mechanic card: Intelligence

- **Decision:** where to look (you can't watch everyone), whether to act covertly (risk of exposure) or stay passive, and how much to spend defending yourself.
- **Costs:** Intel budget (instead of guns or welfare), Initiative, exposure risk (diplomatic incidents).
- **Interacts with:** Military (accurate estimates, deterrence through disinformation), Diplomacy (incidents; verifying arms control), Stability (subversion), Research (theft), Victory (discovering rivals' secret objectives).
- **AI reasoning:** AIs prioritise coverage of their top threats and targets. Paranoid AIs over-invest in counterintelligence. Opportunistic AIs fund opposition in weakening rivals. The AI computes threat on estimates, so disinformation genuinely changes its behaviour.
- **Feedback:** coverage overlay, confidence bands on all foreign stats (shown as a range bar, not a single number), an intelligence briefing section, an "accuracy reveal" when war starts (true strength is revealed on contact, which is a dramatic moment).

---

## 9. Domestic Stability Model

### 9.1 One meter, four visible drivers

**Stability (0–100)**, recomputed every tick from:

| Driver | Raised by | Lowered by |
|---|---|---|
| **Prosperity** | Growth, Welfare budget, cheap food and energy | Recession, shortages, sanctions, debt default |
| **Security** | Strong defences, alliances, being attacked (short-term rally) | Hostile buildups nearby, lost battles, terrorism and subversion |

*v0.1 Security (D32):* 10 + 65 × D/(D+H).
- D = own power + Σ defenders' power × 0.5 × own Credibility(Back) reading of each.
- H = Σ others' power × hostility × reach. Reach = 1 until regions exist.
- The breakdown is exposed as `security_breakdown`.
| **War Weariness** (negative only) | — | Casualties, long wars, mobilization level, unpopular war aims |
| **Legitimacy** (slow-moving) | Victories, kept promises, elections won, reforms | Defeats, humiliations, repression, exposed scandals |

**Government type sets the weights.** This is the main source of domestic asymmetry:

| Government | Prosperity | Security | War Weariness sensitivity | Legitimacy source | Special |
|---|---|---|---|---|---|
| **Democracy** | High | Medium | High | Elections | Elections every ~16 turns; open (leaks intel); cheap credit |
| **Authoritarian** | Medium | High | Low | Performance and order | **Repression** policy; opaque; coup risk if the military is unhappy |
| **Revolutionary** | Low | Medium | Low | Ideological success | Gains legitimacy by spreading alignment; falls if the revolution "stalls" |

**Two generic terms (all governments):**
- **Defence-burden tolerance.** A Legitimacy penalty applies when military spending (including arms transferred out) exceeds what the largest perceived threat justifies. When a rival collapses, peace-dividend pressure appears automatically, with no event.
- **War-of-choice weariness.** War weariness is multiplied by `(1 + c·(1 − threat_from_enemy))`. `c` is data, with a democracy default of 0.5. Defensive wars hurt less than wars of choice. Traits such as the US *War-of-Choice Aversion* simply raise `c`.

### 9.2 Thresholds

| Stability | Effect |
|---|---|
| 70+ | +1 Initiative, GDP bonus |
| 40–70 | Normal |
| 25–40 | Unrest events: protests and strikes (output loss), −1 Initiative |
| 10–25 | Crisis: regional uprisings, coup attempts, election collapse; foreign subversion doubled |
| <10 | **Regime collapse: game over for the player** (decided). An AI country instead gets a new regime: a new government type and personality vector |

### 9.3 Domestic policies (cost Initiative)

Kept to a small set:
- **Crackdown** (authoritarian): +Stability now, −Legitimacy, −Opinion abroad.
- **Reform:** −Stability short-term, +Legitimacy and growth long-term.
- **Propaganda campaign:** +Security/Legitimacy for a war or rivalry; raises tension with the named enemy.
- **Austerity:** cut debt faster at a stability cost.

### 9.4 Mechanic card: Stability

- **Decision:** how much pain the regime can absorb, and whether a foreign adventure will rally the nation or break it.
- **Costs:** Welfare budget (instead of guns or growth); policy choices trade short-term for long-term.
- **Interacts with:** Economy (prices and recession feed Prosperity), Military (war weariness, rally effect, coups), Intel (foreign subversion), Diplomacy (humiliation, popular alliances), Escalation (diversionary war).
- **AI reasoning:** AIs below 40 stability switch toward "Restore Stability" goals, **or**, if risk-tolerant and authoritarian or revolutionary, consider diversionary conflict. This makes unstable neighbours unpredictable.
- **Feedback:** a stability gauge with a four-driver breakdown and trend arrows, regional unrest flashes on the map, a projected-stability readout before committing to war or mobilization.

---

## 10. Research and Development

### 10.1 Three tracks plus national projects

| Track | Levels 1–10 improve |
|---|---|
| **Military** | Force quality, readiness efficiency, eventually the Strategic Arsenal ceiling |
| **Economic** | GDP multiplier, commodity efficiency (lowers consumption), growth |
| **Intelligence** | Recon coverage, detection avoidance, counterintelligence |

- Funded by the Development budget (shared with infrastructure. **That's the trade-off**).
- **Catch-up diffusion:** research is cheaper when trade partners are ahead. Technology theft adds progress.
- **Infrastructure is not research.** It's built per region with the Development budget.

### 10.2 National Projects

Big, discrete, multi-turn investments. Every country has access to a common list (3–4) plus **one unique national project**.
- Common: Strategic Arsenal (deterrent), Satellite Network (global recon), Green Revolution (+food), Energy Independence program.
- Unique examples are in Section 13.

**v0.1 scope (decided):** implement only enough projects to prove the loop works:
- projects can begin
- they consume resources over multiple turns
- they complete
- completion changes strategic state
- the AI can judge whether one is worth starting

No large catalogue. Most scenario projects stay data, to be added later.

**Critique:** a tree of hundreds of technologies is cut. Three tracks keep the trade-off clear (military edge vs. growth vs. information), and projects provide memorable milestones. If even three tracks prove shallow, merge Intelligence research into the Intel budget line.

### 10.3 Mechanic card

- **Decision:** a future edge vs. present power. Which track matters most for *your* strategy?
- **Costs:** Development budget (competes with infrastructure, and with every other line).
- **Interacts with:** Military (quality), Economy (growth), Intel (coverage), Diplomacy (tech trade), Escalation (strategic arsenal).
- **AI reasoning:** track weights follow its goals (an economic AI favours Economic). Lagging badly behind a rival triggers catch-up spending or theft.
- **Feedback:** a comparative tech chart (with fog: rivals' levels are estimates).

---

## 11. Escalation Model

### 11.1 Two layers

**Bilateral Tension** (per pair, 0–100) and **Global Tension**, a world meter shown as five **Readiness Conditions** (5 = calm → 1 = brink). Global Tension = sum of bilateral tensions **weighted by the two countries' power shares** (not a fixed "great power" list, so it works in every era and adapts as powers rise and fall), plus active wars, plus strategic arsenal posture.

### 11.2 Involvement Bands and the escalation ladder

**The player-facing decision is "how involved do I want to become?"** That's answered with **five Involvement Bands** (decided, and must not be expanded into many near-identical levels):

| Band | Concrete actions | Involvement stake |
|---|---|---|
| **0 None** | (optionally Mediate) | 0 |
| **1 Coerce** | Threaten, Ultimatum, Sanction, Embargo | 0.3 (an ultimatum creates a Threat test) |
| **2 Proxy** | Arms, advisors, volunteers, covert aid, fund insurgents | 0.5 (visibility follows the action) |
| **3 Strike / Limited** | `Punitive` war aim (§6.6), limited-aims war | 1.0 |
| **4 Major** | Coalition war, regime-change aims, occupation | 1.5 |

- Acting at band 2+ for a side automatically creates a **Back commitment** with that stake (§21.2). It **takes the visibility of the action that created it**: covert support can be dropped quietly, while public intervention locks you in.
- Each action also carries finer-grained **`rung` metadata**, used internally for tension and cost (the table below). The rung labels are display and AI detail, not extra player buttons.

| Rung | Actions | Typical tension added |
|---|---|---|
| 1 Rhetoric | Denounce, propaganda | Small |
| 2 Pressure | Sanctions, embargo, arms race | Moderate |
| 3 Posturing | Border deployments, mobilization, exercises | Moderate–Large |
| 4 Proxy | Arms to enemies, funding insurgents, volunteers | Large (smaller if covert and undetected) |
| 5 Limited war | War with limited aims | Very large |
| 6 Total war | Regime-change aims, full mobilization | Extreme |
| 7 Nuclear | Limited strike, counter-value strike, full exchange (§11.4) | Global Tension → Condition 1, immediately |

### 11.3 What tension does

- **Gates options.** War without a casus belli or adequate tension costs a heavy legitimacy, credibility and opinion penalty worldwide. You can't credibly invade from a calm baseline.
- **Drives the AI.** Threat perception rises, so alliances, armament and intel spending rise.
- **Hits economies.** High global tension raises commodity prices and cuts trade (markets panic).
- **Rally effect.** Moderate tension with a named enemy raises Security and Legitimacy. Very high tension erodes Prosperity.
- **Decays** slowly when nothing feeds it. Arms control and mediation actively reduce it.

### 11.4 Nuclear weapons: possible, catastrophic, never free

**Decided:** nuclear use is possible and carries extremely heavy penalties. It isn't an automatic game end. It's a real option whose costs almost never justify it, and that "almost" is what makes deterrence believable.

> **v0.1 scope (decided).** The purpose of v0.1 is to prove that nuclear capability changes strategic decisions. v0.1 implements only:
> - arsenal possession and level
> - the deterrence effect
> - threat calculations
> - **one** nuclear-use action
> - the major diplomatic, domestic, economic and escalation consequences of use (the penalty package below)
>
> **Deferred** (the full design below stays the target):
> - the three strike scales
> - counter-value vs. battlefield targeting
> - detailed second-strike simulation
> - the Nuclear Winter meter
> - doctrine detail beyond what deterrence needs

*v0.1 as built (D49):*
- Arsenal level with strike capacity (2 per level, regenerating).
- Deterrence: 20 power-equivalents per level in threat and security, plus an AI catastrophic-risk term scaled by global tension and the target's nuclear cover, and (D68 #5) by how far the war aim threatens the target: the aim's offensive weight relative to a Major war (Major 1, Limited ~0.4, Punitive 0.25, never below 0.25), full against a target visibly in crisis. The stability–instability paradox: a Major war on a nuclear state stays nearly unthinkable; a limited one over a claim or a border is deterred less (Falklands, Kargil).
- One use action: a limited strike on a war front.
- Penalty package: pariah opinion, tension, domestic shock, energy price shock, a weight-5 ledger entry.
- AI use only when existential.
- Postures, strike scales and Nuclear Winter deferred.

#### Arsenal

- **Strategic Arsenal level 0–3**, built through the national project. Level 1 = a small deterrent (a few strikes). Level 2 = a regional power. Level 3 = a superpower able to devastate any country.
- Each level grants a number of **strike capacity** points that regenerate slowly. Using them depletes the arsenal.
- **Second strike:** at level 2+, part of the arsenal survives any first strike. Retaliation is always possible, which is the basis of mutually assured destruction.

#### Nuclear posture (a standing declaration, costs 1 Initiative to change)

| Posture | Deterrence | Tension | Constraint |
|---|---|---|---|
| **No First Use** | Low against conventional attack | Lowers your tension with everyone | First use also breaks a public promise: Trust and Credibility collapse |
| **Flexible Response** | Medium | Neutral | — |
| **Massive Retaliation** | High (also covers allies) | Raises tension with rivals | Failing to respond when the trigger is met destroys Credibility |

Posture is public. That's the point: it's a commitment others plan around, and your Credibility determines whether they believe it.

#### Three scales of use

| Use | Target | Direct effect |
|---|---|---|
| **Limited strike** | Military forces on one front | Wipes out most enemy strength on that front. Region infrastructure −2, population −10% |
| **Counter-value strike** | One region's cities and industry | Region population −40–60%, industry and infrastructure destroyed, permanent output penalty |
| **Full exchange** | All of a rival's major regions | The target nation is effectively destroyed as a power. Retaliation is near certain |

#### Penalties (applied on ANY use, scaling with the size of the strike)

1. **Global pariah.** A permanent "Nuclear Aggressor" opinion modifier (−60 to −100) with every country. Trust drops to 0 with everyone. Every treaty you hold is reconsidered, and most allies abandon you.
2. **Automatic coalition.** Every AI evaluates sanctions and war against you with a large bonus. Non-aligned countries embargo you.
3. **Domestic shock.** Democracies: Legitimacy collapses, and Stability often goes straight into crisis. Authoritarian: a smaller hit, but coup risk rises sharply as the military and elites fear what comes next.
4. **Retaliation.** If the target or anyone guaranteeing it has an arsenal, its AI decides on retaliation. Under Massive Retaliation, retaliation is close to automatic.
5. **Market crash.** Global Tension → Condition 1. Commodity prices spike and trade value falls worldwide for many turns.
6. **Fallout and nuclear winter.** A global **Nuclear Winter** meter rises with total strikes used across the whole campaign. It lowers food production worldwide, which feeds famine, refugee and revolution events everywhere.
7. **Victory penalty.** Your objective score is halved, and you're barred from Hegemony victory for the rest of the campaign.

#### Thresholds

- **Nuclear Winter "Severe":** global food production −30%. Famine cascades. Unstable states start falling.
- **Nuclear Winter "Terminal":** the campaign ends immediately with **no winner**. A full exchange between two level-3 powers crosses this threshold. A single limited strike never does.

#### What this produces

- A **limited strike** by a desperate country can save it in the short run and wreck it in the long run. That's a dramatic, legitimate late-game gamble.
- **A full exchange between superpowers ends the world.** Smaller nuclear wars leave a damaged world where everyone else keeps playing, with famine and a crashed market. A non-nuclear player can still win in that world.
- Great powers rationally compete through proxies, intelligence, economics and diplomacy. That's the Cold War dynamic emerging from rules, not scripted.

#### AI reasoning on nuclear use

- **Considered only** under existential conditions: core regions falling, Stability < 20 because of war, or a nuclear attack received.
- Utility = survival value − (all the penalties above, estimated) − retaliation risk × the target's arsenal and posture. It's weighted heavily by risk tolerance, and only Revolutionary or extremely risk-tolerant personalities ever reach a first use.
- Attacking an arsenal state at all includes a catastrophic-risk term that grows with Global Tension and the target's posture.
- **Accident risk:** at Condition 1, every tick has a small chance of a false alarm crisis. This is an event where each arsenal holder must choose to stand down or respond.

#### Feedback

- An arsenal panel shows your capacity and posture, plus the estimated arsenals and postures of others (intelligence applies: covert programmes may be hidden).
- Before any nuclear order, a **consequence preview** lists the projected penalties and retaliation odds. It requires a two-step confirmation.
- Strikes are shown on the map as a stark, distinct visual. The Nuclear Winter meter appears permanently once it's above 0.

### 11.4b Covert programmes, regime transition, secession (v0.2 as built)

- **P10 (D50).** Programmes progress by capability (tech × √GDP) and complete as undeclared arsenals.
  - Undeclared arsenals and programmes are visible only to observers with coverage, so they deter only those who see them.
  - Hostile exposure is a Proliferation violation. Declare / Disclose & Dismantle.
  - Punitive strikes set programmes back (Osirak); nuclear powers sanction proliferating strangers; patrons can cut clients.
- **P3 (D51).** A sustained crisis (Stability < 35, Legitimacy < 40, 6 turns) forces Reform or Crackdown.
  - Reform: democracy, reform personality, may leave its bloc, ends command-economy drag.
  - Crackdowns breed coups. Collapse needs a catastrophe. 12-turn reprieve.
- **P2 (D52).** Peripheries with political loyalty; low loyalty causes unrest.
  - Secession from a vulnerable centre (crisis, war, or a fresh reform) activates a dormant successor state. The region map will replace peripheries.

### 11.5 Proxy war (a key mechanic)

Support a side in someone else's war, or an insurgency:
- **Arms** (money and materials → strength for the recipient; low detection)
- **Advisors** (quality boost; moderate detection)
- **Volunteers** (your forces fight under their flag; high detection, high impact)

Covert support adds less tension until it's exposed. Proxy wars let great powers bleed each other cheaply. They also let small states extract aid from both sides.

**Rules that keep proxy war from being free:**
- **Proxy supply needs a route:** your own adjacency, a **consenting conduit state** (one that accepted the arrangement), or sea control into the recipient. This makes conduit states such as Pakistan valuable without special code.
- **Volunteer casualties** count toward the supplier's war weariness.
- **Transferred arms are permanent** (§21.6).
- **Arming both sides** of the same war writes a Partial entry with each belligerent and cancels involvement credit (§21.2).

### 11.6 Mechanic card: Escalation

- **Decision:** how far up the ladder to push. Every rung gains leverage and risks a spiral.
- **Costs:** tension is a shared resource; pushing it raises everyone's prices, alarms neutrals, and invites balancing.
- **Interacts with:** everything. It's the game's thermostat.
- **AI reasoning:** each AI has an escalation tolerance (risk tolerance × arsenal security × stability). It responds to escalation in proportion, by personality: paranoid → over-responds; risk-averse → de-escalates or seeks allies; opportunistic → escalates against weakened targets.
- **Feedback:** a Global Tension meter always visible (the 1980s DEFCON aesthetic), a bilateral tension overlay (red lines between capitals), ticker warnings when thresholds cross.

---

### 11.4c Crisis drivers and the world interest rate (built 2026-10-04)

These came out of the crisis pass, after military calibration showed that wars and transitions had been running on inflated militaries. Each one was diagnosed with `brink diagnose` before any value changed.

**War motives (independent of army size)**
- **Subversion threat** (`ai::inputs::subversion`):
  - Applies when a hostile revolutionary state is within reach of a non-revolutionary regime.
  - Scales with the target regime's fragility (1.2 − legitimacy) and with paranoia.
  - Feeds the threat picture and the war line "ending their subversion of our regime". It uses visible government type only.
- **Claims** (`WorldState.claims`, scenario data, public):
  - Each claim is a (claimant, target, weight) quarrel.
  - The war line "pressing our claim" is worth 20 × weight.
  - Winning a Limited or Major war settles the claim; losing halves it.
- **War memory:** "we fought them not long ago" fades over 40 turns, instead of switching off after 12.
- **Counter-proliferation reach:** the "stop their weapons programme" motive now scales with the target's reach. It used to have none, so distant states struck Iran.

**Political pressure**
- **Legitimacy target** = 0.65 × prosperity + 0.35 × security − burden + rally − falling behind. Security was 50%, which made small insecure states "illegitimate".
- **Falling behind:**
  - An aligned state's growth trend (5-year EMA) is compared with the largest economy of the other bloc, × 1500. This is systemic competition.
  - Unaligned states aren't in the race.

**World interest rate (P7)** (`sim_core::money`)
- **The holder:** the country with the largest `financial_weight` ≥ 0.4 (data) sets a free standing stance: Tight, Neutral or Loose.
- **The rate:** it moves 25% per turn toward base × (2.2 / 1.0 / 0.6), plus 0.3 × base at full global tension.
- **Inflation:** a persistent world level. Its target = energy deviation × (0.3 / 1.0 / 1.5) + push (−0.2 / 0 / +0.3) + persistence (0.3 / 0.7 / 1.0) × current. It adjusts 15% per turn and is capped at 3.
- **Growth:** a worldwide effect of −0.4% / 0 / +0.2% per quarter. Tight money is a recession.
- **Prosperity terms:**
  - inflation: −15 × inflation × (0.5 for energy exporters)
  - debt service: −60 × (service ÷ revenue − 0.08)
  - Service = debt × rate × (1 − financial weight); it is paid from the spending pool.
- **Holder AI:**
  - 2 × its own prosperity change (inflation, interest and growth, at each stance's settled values) × (0.5 + greed)
  - −0.15 × its friends' estimated debt pain × (0.5 + loyalty)
  - +0.15 × hostile debtors' pain × (0.5 + aggression)
  - status quo +3
- **1980 data:** the stance opens Tight with inflation 1.0, as after the Volcker shock. The AI eases once inflation breaks, which produces the early-1980s rate peak without a script.
- **Voice:** `RatesRaised` and `RatesCut` lines; the fact line quotes the target rate.

**Repression wears out (issue 2, 2026-10-04)** (`sim_core::transition`, `ai::evaluate::reform`)
- `TransitionState.repressions` counts crackdowns in the current crisis era. A coup resets `crackdowns` (the new rulers' coup risk) but **not** `repressions`: new rulers, same state, same failed answer. The record clears after `CALM_RESET` (24) turns out of crisis, counted once the reprieve ends. Every crackdown still buys the full 12-turn reprieve (a shrinking reprieve was tried and dropped: it mainly added crises and coups).
- **The regime learns:** the reform score gains "repression has already failed" = 15 × repressions × (1 − ideology). Caretaker juntas negotiate an exit (Latin America in the 1980s debt crisis); ideological regimes read failure as a call for more.
- Grounding: O'Donnell & Schmitter (softliners gain as hardline answers fail); Geddes (military regimes exit by negotiation, single-party regimes hold on).

**Crisis pass 2 (2026-10-04, multi-agent with adversarial review)**
- **Commitments engage only for the defender** (D62, `ai::war::commitment_engaged`). An alliance or guarantee adds stake and "our word as an ally" only when the friend is attacked. This matches the engine's Back test, which only ever tests a defender's allies. It fixed guarantors co-invading on their protégé's side.
- **Bloc backing in security** (D64, `domestic::bloc_backing`). A bloc-aligned state counts a defender only if the defender shares its bloc or its kind of government, unless the defender is weaker than it (a non-patron ally always counts). A patron that reforms out of the bloc stops propping up its clients.
- **Repression wears out** (D63). See the P3 notes above.
- **Home ground follows the front** (issue 6; `War::home_side`, `War::ground_mult`). The ×1.5 home-ground multiplier and the home-front rally (half war weariness) belong to the side whose soil the front is on: the defender's until the attacker is thrown back over its own border (progress < 0). A failed invasion is therefore held near the invader's border (the front oscillates there; see the gaps below) instead of being routed to a decisive defeat, and a counter-invasion becomes the war of choice. Grounding: Iran–Iraq 1982–88 (Iran's refusal of peace and its offensives on Basra turned Iraq's failed war into a defence of home soil; Iranian offensives broke on prepared Iraqi lines), Korea 1951–53. A rout is still possible when the enemy outweighs even the home-ground bonus (more than ×2.25). The AI reads the same rule (`side_chance`, `continue_war`). *Superseded by the smoothed rule below (issue 13):* the discrete "only on home soil" reading survives only as `home_side()` for the D65 test; nothing in the sim or AI should read it.
  - *KNOWN GAPS (issue 6 review):* (1) the ×1.5 flips sharply at progress 0, so a failed invasion's front **chatters across the border** (sign crossings 2/38 wars → 132/38 over 40 seeds) and every Iran–Iraq war ends in a negotiated white peace in about 5 turns, not a stalemate on a line. (2) **Iraq re-declares after each white peace** (46 wars in 40 runs, up to 4 per run): its claim and motives survive a draw. (3) **Iran collapses 5/40** under those repeated wars (its known fragility; Revolutionary Zeal unbuilt). Net collapses still fall (IRN + IRQ 29 → 8 per 100 runs).
  - *Follow-up:* smooth the ground multiplier (e.g. ×1.5 fading over progress 0 to −25) **only after F1 is addressed** (war weariness subtracting 1:1 from the stability target). A ramp tried in review lengthened wars (median 12 turns) but made Iran collapse 103 times in 100 runs (82 at war): the chatter is currently hiding the F1 mismatch.
  - *Smoothed (issue 13, after D71 unblocked it; `War::home_share`):* defensive advantage builds with depth (prepared lines, short supply, familiar terrain); it does not appear at the border line. The declared defender holds the full ×1.5 from the first day; a thrown-back invader gains it, and the counter-attacker loses it, linearly over the first `WIN_MARGIN` (25) of progress into the invader's territory, where the counter-attacker's supply lines lengthen and the invader falls back on its own positions. `ground_mult`, the home-front rally (half weariness, by the leader's share) and the AI's "defending our territory" all read `home_share`; `home_side` remains the discrete reading. Grounding: Iran's 1982 counter-offensives broke not at the border but on Iraqi lines before Basra. A front now settles where the invader's growing home advantage balances the enemy (ratio 0.88 → about −16): gap (1) is closed (Iran–Iraq border crossings 121 → 4 over 29 wars in 40 seeds, median per war 4 → 0); gap (2) had already gone with D60's war memory and D71 (0 re-declarations); wars still end by negotiation at a median of 6 turns (was 5) because the AI's `continue_war` offers peace once "backing down so soon" expires, the open question noted under D71. *Properties (review 13):* symmetric (A→B and B→A identical); odds at declaration unchanged (progress starts at 0). The asymmetry has a cost: an attacker only 1.0–1.5× stronger now settles **inside its own border** (ratio 1.3 → about −4; under the step rule it chattered around +5), since the defender holds the full advantage from day one and the invader earns its own only with depth. Decisive attacker wins need about ×2 under both rules (D65's ×1.5, not this change). A thrown-back invader at −12.5 carries a rally of 0.75 rather than 0.5, so it wears a few points faster. The Gate 6 "abstain" row rises 1–2 points with longer wars (more involvement assessments) and straddles the 20% line (19.9–20.6%), so it is not counted as a fixed row.

- **War weariness spends the regime's margin, not raw stability** (issue 7, F1; `domestic::stability_target`). Stability target = base − (base − collapse line 10) × weariness ÷ exhaustion (60), where base is the weighted prosperity/security/legitimacy drivers. Weariness used to subtract 1:1, so regimes collapsed at weariness ~15–25 and exhaustion never ended a war. Now a long war wears a regime into crisis (crackdowns) and to the peace table; at exhaustion, when the war system forces peace, the margin would be spent. Collapse still comes from the drivers (a lost economy, lost security, defeat). Grounding: long wars mostly end by exhaustion and negotiation (Iran 1988 ceasefire); belligerent regimes that fell mid-war were already fragile or were losing catastrophically. *Measured (review 7):* exhaustion is still not reached in practice (peak weariness p90 ≈ 21–26; 0 of 214 wars ended by exhaustion before or after), so wars end by negotiation or a decisive result; whether exhaustion should end wars is an AI `continue_war` question. Two properties: weariness alone can never collapse a regime whose base is at or above the collapse line (mid-war collapse must come from the drivers; a 1917-style case shows up as a coup or reform), and regime stability is now coupled to `war::EXHAUSTION`. A co-belligerent pushed past 60 would meet the old cliff (never observed; clamp or check co-belligerents if it appears).

## 12. World Events

### 12.1 Principle: events read state, they don't roll dice in a vacuum

Each event has **trigger conditions** drawn from simulation state, a **probability weighted by how strongly those conditions hold**, and **effects that feed back into systems** (not flat bonuses).

| Event | Triggers when | Effect |
|---|---|---|
| **Famine** | Food deficit + high price + low Welfare | −Stability, refugees to neighbours, aid opportunity |
| **Oil shock** | Energy supply drop (war, embargo, chokepoint) | Price spike → recession in importers |
| **Revolution** | Stability < 15 + foreign funding + Legitimacy low | Regime change; new government type and personality |
| **Coup** | Authoritarian + low military funding + defeats | Regime change or purge |
| **Border incident** | Opposing forces adjacent + bilateral tension > 60 | Tension spike; possible casus belli |
| **Refugee crisis** | War or famine adjacent | Neighbour −Stability, −Prosperity |
| **Debt contagion** | A default + trade partners with high debt | Chain of credit shocks |
| **Natural disaster** | Random location, but impact scales with infrastructure and Welfare | Damage, aid opportunity, stability test |
| **Spy scandal** | Op exposed in a democracy | Legitimacy hit to the sponsor |
| **Election** | Democracy, schedule | Stability check; low result → −Initiative, forced policy |

Only natural disasters are truly random, and even their impact depends on state. About 40 events are enough for the prototype.

### 12.2 Feedback

Events appear as **ticker headlines with "because" links** ("Famine in Kavaran — *because*: drought + food price +40% (Qasr embargo) + Welfare cut").

---

## 13. Framework for Country Asymmetry

### 13.1 Asymmetry comes in layers, not classes

| Layer | Source of difference |
|---|---|
| **1. Geography** | Regions, terrain, coasts, neighbours, chokepoints, distance from rivals |
| **2. Endowments** | Population, GDP, industry, commodity production and consumption |
| **3. Institutions** | Government type → stability weights, Initiative, credit, openness |
| **4. Doctrine** | Military multipliers: *Mass* (manpower, defence), *Mobile* (offence, supply), *Maritime* (navy, amphibious), *Air-Centric*, *Defence-in-Depth* (terrain, resistance) |
| **5. Traits** (2–4) | Systemic modifiers, each with an upside and a downside |
| **6. National Project** | One unique big investment |
| **7. Starting Diplomacy** | Treaties, opinion, rivalries, alignment |
| **8. Objectives Pool** | What *winning* means for this country (Section 14) |

**Archetypes emerge** from combinations. Nobody is labelled "resource exporter." A country with high energy output, low population and a chokepoint simply *plays* like one.

### 13.2 Trait design rule

**Every trait must change a decision, not just a number.**

| ✗ Bad trait | ✓ Good trait |
|---|---|
| "+10% GDP" | **Oil Curse:** +energy revenue; Prosperity tied to the oil price (you want prices high, which aligns you against importers) |
| "+5 stability" | **Neutral Tradition:** +trade with all; joining a military alliance costs heavy Legitimacy |
| "+20% army" | **Conscript Nation:** cheap manpower; war weariness from casualties doubles |
| | **Financial Hub:** cheap credit, +trade income; sanctions against you hurt you double |
| | **Siege Mentality:** Security driver weighted ×2; peace makes the regime nervous |
| | **Brain Drain:** research +, but low Stability bleeds research progress |

### 13.3 Data schema (illustrative)

*Shown as YAML for readability. The real files will be serde-loaded into Rust structs, likely in RON (see STATE.md).*

```yaml
country:
  id: QSR
  name: "Emirate of Qasr"
  government: authoritarian
  doctrine: air_centric
  regions: [qasr_coast]
  population: 4.2          # millions
  gdp: 85                  # index units
  tax_rate: 0.30
  commodities:
    energy:    { produce: 60, consume: 4 }
    materials: { produce: 2,  consume: 6 }
    food:      { produce: 1,  consume: 5 }
  military: { land: 8, naval: 4, air: 10, quality: 5, arsenal: 0 }
  research: { military: 4, economic: 3, intelligence: 3 }
  stability: 62
  traits: [oil_curse, rentier_state, chokepoint_guardian]
  national_project: sovereign_wealth_fund
  treaties: [{ type: arms_supply, with: ACW }]
  opinion: { KVR: -30, ACW: +25 }
  ai_personality: { aggression: 0.2, risk: 0.3, paranoia: 0.7, loyalty: 0.4, greed: 0.8, ideology: 0.2, opportunism: 0.6 }
  objectives_pool: [survive_independent, wealth_top5, hold_chokepoint, no_regional_hegemon]
```

Region definitions are separate files: owner, core, population, industry, resources, terrain, infrastructure, port, airbase, adjacency, sea zones.

---

## 14. AI Decision-Making Architecture

### 14.1 Principles

1. **No cheating on information.** The AI uses its own intelligence estimates.
2. **The player is not special.** The AI evaluates the player exactly like any other country. No "player-targeting" bias.
3. **Goals persist.** The AI commits to strategic goals for multiple turns, so its behaviour looks coherent and can be read through intelligence.
4. **Every decision is explainable** through a reasoning log, available to debugging and (filtered) to player intelligence. The explanation lines **are** the utility terms, and they sum to the score shown.
5. **No country-identity branching (hard architectural rule).** Core mechanics and AI behaviour never branch on country identity (`if country == USA`, etc.). Countries differ only through:
   - data: scenario, geography, traits, government, personality, treaties, bases, alignments, resources, parameters
   - state: economic, military and technology state, historical ledger entries, goals, the current simulation

   Historical names are flavour. A lint check plus a data-swap test enforce this. Country-specific UI text and scenario content are fine. Country-specific engine behaviour needs an explicitly approved exception.
6. **AI reads only observer-specific values.** Credibility and Trust come from the observer's own ledger reading (§21.3). The display-only global Credibility is never an AI input.

### 14.2 Three layers

```
┌─────────────────────────────────────────────────────────┐
│ 1. STRATEGIC ASSESSMENT  (every ~4 turns or on shock)    │
│    Threat map · Opportunity map · Needs · Self-assessment│
│    → choose 1–3 STRATEGIC GOALS from goal library        │
├─────────────────────────────────────────────────────────┤
│ 2. PLANNING  (every turn)                               │
│    Each goal proposes candidate actions                  │
│    Utility = Σ goal contribution − cost − risk×aversion  │
│    Greedy fill of Initiative + Budget allocation         │
├─────────────────────────────────────────────────────────┤
│ 3. EXECUTION                                            │
│    Budget sliders · Mobilization · Theater postures ·    │
│    Diplomatic/intel actions · Contingency orders         │
└─────────────────────────────────────────────────────────┘
```

### 14.3 Strategic assessment inputs

- **Threat score** for each other country = estimated military in reach × hostility (opinion, tension) × past aggression memory × paranoia.
- **Opportunity score** = weakness of the target (estimated) × value (resources, chokepoints, objectives) × the likely intervention of third parties (their treaties × **this AI's own Credibility(Back) reading** of each protector). A protector with a poor record toward clients like the target invites **probes**: Falklands- and Kuwait-style moves emerge from the protector's own history.
- **Intent estimate** (`IntentEst`) = the rate of a country's Coercion entries in the ledger. Responses to someone else's aggression count ×0.5, so a reliable defender isn't treated like an aggressor.
- **Needs:** commodity deficits, low stability, recession, debt, tech lag.
- **Power balance:** whether any country (including the AI itself) approaches hegemony.

### 14.4 Goal library (about 12 goals)

`Contain [X]` · `Deter [X]` · `Secure [commodity]` · `Expand into [region]` · `Develop Economy` · `Restore Stability` · `Build Bloc` · `Spread Alignment` · `Balance Against Hegemon` · `Revenge on [X]` · `Protect Client [X]` · `Exploit Crisis in [X]`

Goal score = situational fit × personality weight.

### 14.5 Personality

A **vector of weights**, not a class:

| Dimension | Low end | High end |
|---|---|---|
| Aggression | Content | Expansionist |
| Risk tolerance | Cautious | Gambler |
| Paranoia | Trusting | Sees threats everywhere |
| Loyalty | Opportunistic ally | Honours commitments at a cost |
| Greed | Indifferent to wealth | Economic focus |
| Ideology | Pragmatic | Alignment-driven |
| Opportunism | Plans long-term | Exploits openings |

The descriptive personalities from the brief ("expansionist," "paranoid"...) are just names for vector regions, shown to the player through intelligence ("Assessment: Federation leadership is cautious but paranoid"). **Regime change and leader change shift the vector.**

### 14.6 Memory

Memory is the shared **Event Ledger** (§21.1), read through each observer's own fog: broken treaties, attacks, aid, honoured or abandoned guarantees, threats followed through or dropped, coercion, exposed operations. There are no private per-AI memory stores. Trust, Credibility, Threat and IntentEst are all read from it. **AIs hold grudges against each other**, which creates historical rivalries without player involvement.

### 14.7 Why AIs fight each other

AI–AI conflict emerges from geography (neighbours have higher threat and opportunity scores), resource needs (an energy-poor AI targets energy-rich regions), balance-of-power logic, memory, and ideology. The player is one of 16 actors. The player is targeted only when the player is in fact the biggest threat or the best opportunity.

### 14.8 AI action evaluation, for example "Mobilize to Partial"

```
Utility = +Deter(Kavaran)        0.6 × (threat 72)
          +Security driver gain  ...
          −GDP cost              5% × greed weight
          −Tension with KVR      × (1 − aggression)
          −Provocation risk      × paranoia of KVR (estimated)
```

### 14.9 Decisions about other powers, crisis involvement, and safeguards

**Decisions other countries make about a power P.** Each is a utility sum whose terms are the explanation lines:

| Decision | Key terms |
|---|---|
| Accept P's aid / stream | Need, shared rival, dependence creep × paranoia, bounded ideology, angering current patrons |
| Request P's guarantee | Security gap × own Credibility(Back) reading of P, odds P accepts, provoking the threat, the price of strings |
| Join P's sanctions | Patron goodwill × dependence, own grievance against the target, own trade loss × greed, retaliation risk, diminishing effectiveness |
| Abandon P's bloc / treaty re-evaluation | Lost purpose (security value falls when threat falls), grievance and coercion entries, alternative offers, dependence, loyalty. **Exit only after 2 consecutive assessments** past the threshold, which gives a visible warning window |
| Oppose P's intervention (none / denounce / arm the target) | P's power growth in my region × IntentEst, "could be me" precedent, affinity with the target, P's leverage over me |
| Balance against P | `max(0, PowerShare − 0.4) × IntentEst × (0.5 + paranoia)`, with diminishing returns per extra balancer |
| Probe P's clients | No new action: low Credibility(Back) toward a client raises the opportunity score (§14.3) |

**Crisis involvement (any great power, player-equivalent).** Choose a band (§11.2):

```
U(band) = Stake·ΔP(success)  + RepDelta(band)
        − Cost·(0.5+greed) − DomesticCost(war-of-choice, weariness, openness)
        − Tension·(1−aggression) − EscalationRisk·(1−risk) − Entanglement·(1−risk)
```

- `ΔP(success)` comes from estimates, so fog can mislead.
- `RepDelta` runs the §21.3 reputation read on the hypothetical test grade and sums it over the observers P cares about. The **same calculation produces the player's consequence preview.**
- Ongoing involvement is re-scored each assessment as **hold / withdraw / escalate**, looking **forward only** (no sunk-cost reasoning). Withdrawal writes Abandoned or Partial entries weighted by the band's stake and visibility. Covert exits are cheap; public high-band exits are not.
- Economic coercion carries a leverage term. Coercing a friend is allowed, but it writes Coercion entries that raise IntentEst among similar states.

**Safeguards**
- **Anti-dogpile:** balancing needs *intent*, not just strength. Power counts only within reach. Local threats outrank distant giants. Weak states may bandwagon. Each extra balancer adds less.
- **Anti-ideology-lock:** the ideology term is capped at 30 × ideology, while threat, need and dependence terms reach about 60. It's discounted under threat (`× (1 − 0.8·Threat/100)`). Weight is full for alliances and bases, ½ for arms, ¼ for trade.

**Variety across runs** comes from:
- seeded intel estimates
- the path-dependent ledger
- crises competing for Initiative
- domestic state
- seeded ±0.1 personality jitter at start
- softmax among near-tied options only

**After a primary rival collapses** there's no special ruleset. Strategic reassessment rescores the goals. Retrenchment, coalition leadership, integration, primacy and selective intervention each emerge from ordinary terms. The target: no single strategy in more than 60% of runs, and at least 3 strategies in more than 10%.

---

## 15. Victory Systems

### 15.1 Options explored

| System | How it works | Strength | Weakness |
|---|---|---|---|
| **A. Primacy** | Composite score: share of world GDP, military, alliance network, stability. Win by holding a threshold for N turns | Universal, clear | Snowball; a dull leaderboard; pushes everyone to grow |
| **B. National Objectives** | Each country draws 3 objectives from a situation-appropriate pool. Points per objective achieved by campaign end | Different definitions of success; small states can win | Balancing the pools takes work |
| **C. Bloc Victory** | Lead an alliance controlling X% of world GDP and population | Rewards diplomacy | Snowball through alliances; allies are passengers |
| **D. Ideological** | X% of world population under your alignment | Distinct play path | Narrow; needs a strong regime-change system |
| **E. Survival / Prosperity** | Small states: stay independent, stability > 50, top-N GDP per capita | Makes small countries viable | Could be passive |
| **F. Scenario** | Hand-authored ("Hold the Strait until 1985") | Focused, teachable | Content cost |

### 15.2 Recommendation: B, with A as one objective type, plus sudden-death conditions

- **Fixed campaign length** (about 80 turns).
- Each country receives **3 National Objectives**: 1 public, 2 secret (draws weighted by the country's pool). Examples: *Hold the chokepoint*, *Prevent any power from dominating the continent*, *Top-3 world GDP*, *Spread alignment to 3 states*, *Never lose a core region*, *Primacy*.
- **Intelligence can reveal rivals' secret objectives** (coverage 80+), so you can block them. This makes intelligence part of the victory system.
- **Sudden death:**
  - **Hegemony:** Primacy score above a high threshold for 8 turns → immediate win (rare, and resisted by balancing coalitions).
  - **Collapse:** your regime falls → loss.
  - **Nuclear Winter reaches "Terminal":** everyone loses.
  - **Nuclear use** doesn't end the game, but it halves the user's objective score and bars Hegemony victory (§11.4).
- **Scoring display:** at game end, all countries are ranked by objective points, so "winning" can mean outperforming expectations as a small country.

---

## 16. Five Example Countries

Each one plays radically differently because of its layers, not because of special rules.

> **Note (v0.2):** the setting now uses real countries. These five fictional examples remain here as **archetype demonstrations** of the asymmetry framework. Their closest real analogues around 1980: Northern Federation ≈ Soviet Union; Atlantic Commonwealth ≈ United Kingdom (with some US traits); Qasr ≈ Saudi Arabia / Gulf states; Hesperia ≈ a blend of Switzerland and Israel; Kavaran ≈ revolutionary Iran (and Dorvan ≈ Iraq). The real roster will be defined separately (see STATE.md next steps). Real countries will get their own traits; they won't be copied from these.

### 16.1 The Northern Federation — *continental fortress*

- **Geography:** 7 regions spanning a vast interior. Long land borders with 5 countries. Only one warm-water port, through a chokepoint owned by someone else.
- **Endowments:** huge population and energy self-sufficiency. Mediocre GDP per capita. Low food surplus.
- **Institutions:** Authoritarian. Opaque (hard to spy on). Repression available.
- **Doctrine:** Mass (manpower, defence-in-depth).
- **Traits:** *Siege Mentality* (Security weighted ×2), *Command Economy* (military production +, growth −), *Conscript Nation*.
- **National Project:** *Strategic Rail Network* (redeployment between fronts takes half the time).
- **Arsenal:** level 2.
- **Plays like:** a slow giant defending a huge perimeter. Too big to invade, too poor to buy friends, and paranoid by design. It needs buffer states, and it needs sea access, which pushes it toward the chokepoint owner (diplomacy or war). Its weak point is Prosperity. A rival that embargoes food or funds opposition can hollow it out from inside.
- **Objectives pool:** *Secure warm-water access*, *Buffer zone (3 friendly neighbours)*, *Never lose a core region*, *Contain the Commonwealth*.

### 16.2 The Atlantic Commonwealth — *maritime financier*

- **Geography:** 3 island and coastal regions. Ports everywhere. Separated from the continent by sea.
- **Endowments:** highest GDP per capita. Small population. Imports energy and food.
- **Institutions:** Democracy. Cheap credit. Very open (rivals see a lot).
- **Doctrine:** Maritime.
- **Traits:** *Financial Hub* (cheap debt, trade income; sanctions hurt it double), *Global Bases* (starts with basing treaties on 3 continents), *Casualty Averse* (war weariness ×1.5).
- **National Project:** *Carrier Groups* (projects air power into any sea-adjacent theater).
- **Arsenal:** level 2.
- **Plays like:** a power that rules through sea lanes and money, not armies. Its weapons are sanctions, guarantees, blockades and financing proxies. It can't win a land war of attrition and its voters won't tolerate one. Its credibility is its most valuable asset, so every guarantee it extends is a liability it may have to honour.
- **Objectives pool:** *Keep all chokepoints open*, *Bloc controls 40% of world GDP*, *No continental hegemon*, *Primacy*.

### 16.3 Emirate of Qasr — *resource leverage microstate*

- **Geography:** 1 desert-coast region on the world's busiest energy chokepoint.
- **Endowments:** tiny population. Enormous energy exports. Imports all food and materials.
- **Institutions:** Authoritarian (monarchic flavour).
- **Doctrine:** Air-Centric (can buy quality, can't field mass).
- **Traits:** *Oil Curse*, *Rentier State* (stability depends on the Welfare budget; no tax resistance), *Chokepoint Guardian* (can close the strait).
- **National Project:** *Sovereign Wealth Fund* (converts surplus into permanent income and trade-partner opinion).
- **Plays like:** a country that can't win a war against anyone it fears, but can move the world economy. It plays great powers against each other for protection and arms, uses embargoes and price manipulation as weapons, and bankrolls proxies. Its danger is that its value makes it a target. If the strait matters enough, someone may decide owning it is cheaper than renting it.
- **Objectives pool:** *Survive independent*, *Wealth top-5*, *Hold the chokepoint*, *No regional hegemon*.

### 16.4 Republic of Hesperia — *small technological state*

- **Geography:** 2 mountainous regions. Landlocked or nearly so. Surrounded by larger neighbours.
- **Endowments:** small population. High industry and tech. Few commodities.
- **Institutions:** Democracy.
- **Doctrine:** Defence-in-Depth (terrain multiplier, resistance if occupied).
- **Traits:** *Neutral Tradition*, *Brain Drain* (research bonus; low stability bleeds research), *Precision Industry* (arms exports earn trade income and opinion).
- **National Project:** *Threshold Program* (a strategic arsenal level 1 at a fraction of the normal cost; huge tension if detected while building).
- **Starting position:** Intelligence research 6 (highest in the world).
- **Plays like:** the spymaster and arms dealer. It sells technology and weapons to everyone, sees everything, and is too costly to invade (mountains plus resistance). Its decisions revolve around information: whom to tip off, whom to blackmail with intelligence, whether to go nuclear quietly. It is a tempting technology-theft target and a natural mediator.
- **Objectives pool:** *Remain neutral and unoccupied*, *Highest tech in two tracks*, *Mediate 3 peace deals*, *Threshold capability by 1990*.

### 16.5 Kavaran Republic — *unstable revolutionary regional power*

- **Geography:** 4 regions. Large and populous. Borders Qasr and 3 others. Contains ethnic periphery regions with low core loyalty.
- **Endowments:** large young population. Fast growth from a low base. Some energy.
- **Institutions:** Revolutionary.
- **Doctrine:** Mass with Mobile elements.
- **Traits:** *Revolutionary Zeal* (legitimacy from spreading alignment; stagnation erodes it), *Rapid Development* (growth bonus, Prosperity volatile), *Restive Periphery* (two regions prone to uprisings that foreign powers can arm).
- **National Project:** *Export the Revolution* (Fund-opposition operations need lower network levels).
- **Plays like:** a race between growth and collapse. The regime needs ideological wins abroad to stay legitimate at home, but every adventure risks triggering great-power intervention. Rivals will arm its periphery. It is the classic diversionary-war temptation: when stability dips, the easiest rally is the oil-rich neighbour across the border.
- **Objectives pool:** *Spread alignment to 3 states*, *Top-5 GDP growth*, *Survive the decade (stability > 30)*, *Expel foreign bases from the region*.

**Why these five matter together:** Qasr and Kavaran sit in the same region. The Commonwealth needs Qasr's oil. The Federation wants the chokepoint Kavaran threatens. Hesperia sells to all of them. **One region, five completely different games, and an obvious powder keg.**

---

## 17. Three Example Turns (playing Qasr)

### Turn 3: Spring 1980, peacetime setup

**Briefing:**
- Oil price stable at 100 (index).
- Kavaran growth +7%. A revolutionary rally in their capital is "calling on the peoples of the Gulf."
- Commonwealth proposes renewal of its arms supply treaty. Federation envoy requests talks.

**Situation:** Stability 62 · Initiative 3 · Budget: 35% Military, 15% Development, 40% Welfare, 10% Intel.

**Options considered:**

| Option | Cost | Trade-off |
|---|---|---|
| Accept Commonwealth arms renewal | 1 Initiative (accepting a commitment-creating proposal costs Initiative, §7.2) | Federation opinion −10; ties you to the Commonwealth |
| Open talks with Federation (Trade Agreement) | 1 Initiative | Hedges; Commonwealth opinion −5; Federation buys oil at a premium |
| Expand spy network in Kavaran (1→2) | 1 Initiative, Intel budget | You currently see their army at ±40% |
| Start Sovereign Wealth Fund project | 1 Initiative, Development budget for 8 turns | Long-term income vs. immediate security |
| Raise Welfare to 45% | 0 Initiative | Stability safer, military weaker |

**Player chooses:** accept the Commonwealth renewal (1). Expand the Kavaran network (1). Start the Sovereign Wealth Fund (1). The player declines the Federation talks for now, keeping the option open to use as a bargaining chip.

**Resolution feedback:** "Commonwealth arms deliveries: +2 Air strength in 3 turns." "Federation: disappointed (−10)." Network-expansion progress bar appears.

### Turn 11: Winter 1982, warning signs

**Briefing:**
- ⚠ **Kavaran mobilization raised to Partial** (confidence: moderate, network level 2).
- Estimated Kavaran land forces near the border: **140–210** (yours: 8 land, 14 air).
- Kavaran stability estimated 30–40 and falling (a food price spike).
- An intelligence fragment: "Kavaran leadership debating 'liberation of the coast.'"
- Oil 112.

**The decision:** Kavaran is either preparing to invade or posturing to rally its population. You can't tell yet.

| Option | Cost | Risk / effect |
|---|---|---|
| **Request a Commonwealth guarantee** | 1 Initiative + they'll want a basing treaty | Strong deterrent *if* Kavaran believes the Commonwealth will act (credibility 74). Basing angers Kavaran (+tension) and the Federation |
| **Mobilize to Partial** | 1 Initiative, −5% GDP | Readiness up, but raises tension and could be read as a provocation |
| **Disinformation: inflate your air strength** | 1 Initiative, Intel op | Kavaran's estimate of you rises. Risk: if exposed, credibility hit |
| **Food aid to Kavaran** | 1 Initiative, money | Eases their stability crisis and the diversionary motive. But it helps a rival |
| **Embargo oil to Kavaran's trade partners** | 1 Initiative | Squeezes Kavaran's backers, but raises the price, hurts allies, adds tension |
| **Fund Kavaran's restive periphery** | Needs network 3, not available yet | — |

**Player chooses:** spend this turn's 3 plus the 1 Initiative banked during quiet turns:
1. Request the Commonwealth guarantee and offer basing (1).
2. Disinformation to inflate air strength (1).
3. Food aid to Kavaran, quietly (1, plus money).
4. Expand the network to 3 (1).

Nothing is left in reserve, which is a risk in itself.

Mobilization is deliberately avoided so as not to look aggressive.

**Interesting tension:** the food aid lowers Kavaran's motive. The guarantee raises their *cost*. The disinformation shifts their *estimate*. Each works through a different system: Stability, Diplomacy, Intelligence. They also fail differently. If the spy op is exposed, the guarantee looks like a Commonwealth bluff.

**Resolution:** the Commonwealth accepts (the AI is protecting its Secure Energy goal). Kavaran opinion of Qasr −20 because of the basing. Kavaran's AI reasoning log (visible only to a high-coverage observer) shows its war utility on Qasr flipping negative: "Commonwealth intervention likelihood 0.7." Kavaran **redirects**: its mobilization now points at a weaker northern neighbour.

### Turn 19: Autumn 1984, the crisis elsewhere

**Briefing:**
- **Kavaran invaded Dorvan** (its northern neighbour), with limited aims: two border regions.
- The Federation (Dorvan's partner) is sending arms. Bilateral tension Federation–Commonwealth is rising. **Global Tension: Condition 3.**
- Oil 138: the war disrupted Dorvan's pipeline exports. Your revenue is +30%.
- Kavaran stability rising (rally effect).
- The Commonwealth asks you to **embargo Kavaran** to support Dorvan.

**The decision:** the war is good for your treasury and bad for the region. Your protector wants you to pick a side.

| Option | Effect |
|---|---|
| **Embargo Kavaran** | Pleases the Commonwealth (they're your guarantor). Kavaran's economy suffers. But Kavaran may see Qasr as an enemy again, and you lose its oil purchases |
| **Stay neutral, pump oil** | Maximum revenue (Sovereign Wealth Fund grows fast). Commonwealth trust −; if you seem unreliable, how firm is your guarantee? |
| **Mediate a ceasefire** | Raises your opinion with everyone; ends the price boom. Requires Initiative and good relations with both sides |
| **Arm Dorvan covertly** | Bleeds Kavaran's army so it's weaker when it eventually turns south. Exposure risk. Proxy-war entanglement |
| **Fund Kavaran's periphery** (network now 3) | Strikes at Kavaran's stability while it's committed in the north. Very effective, very dangerous if exposed during a war |

**Player chooses:** arm Dorvan covertly (1) and fund opposition in the restive periphery (1). Publicly stay neutral and pump oil. Bank 1. *The bet:* earn the boom while weakening Kavaran without the Commonwealth embargo, and accept the small risk of exposure.

**Possible consequences next turn:**
- (a) **Undetected:** Kavaran stalls in the north, a periphery uprising breaks out, Kavaran's stability collapses, and maybe a revolution produces a new, friendlier regime.
- (b) **Exposed:** "Qasr arming Dorvan" headline → Kavaran tension spikes and it has a casus belli on Qasr while already mobilized. Now the Commonwealth guarantee is being tested for real.

**This is the game:** each choice was understandable, none was obviously best, and every one worked through a different system that touches the others.

---

## 18. Design Risks and Critique

### 18.1 Mechanics most likely to become tedious

| Risk | Why | Mitigation |
|---|---|---|
| **Per-turn budget fiddling** | Four sliders × 80 turns | Sliders are standing; a projection panel shows when change is needed; advisors flag drift |
| **Briefing overload** | 16 countries generate lots of news | Cap at about 7 prioritised items; filters; "only what affects me" mode |
| **Diplomatic spam** | AIs proposing deals every turn | AI proposals cost the AI Initiative too; cooldowns per pair |
| **Peaceful turns feel empty** | Nothing to do in a stable world | The world shouldn't stay stable: AI–AI conflict, cascades, events. Fast "end turn" for quiet turns |
| **Spy network micromanagement** | Tracking networks in 15 countries | Networks are levels, not agents. Coverage overlay at a glance |
| **War attrition slogs** | Stalemated fronts | War weariness rises sharply; exhaustion forces peace talks; War Score makes ending wars attractive |
| **Region upkeep** | Infrastructure per region | Only about 70 regions; build queues persist; the player rarely touches them |

### 18.2 Mechanics likely to become unnecessarily complicated

| Risk | Recommendation |
|---|---|
| **Trade route simulation** | Do **not** simulate shipping. Trade value per pair, with a route check through sea zones (open or blocked). That's it |
| **Separate "public support" and "stability" meters** | **Merged** into one Stability meter with four drivers |
| **Separate "diplomatic influence" currency** | **Cut.** Initiative covers it. Spheres of influence come from opinion and treaties |
| **Infrastructure as a research track** | **Moved** to regional builds |
| **Unit types** (tanks, infantry, frigates…) | **Cut.** Three pools plus quality |
| **Detailed supply lines** | **Cut** to one Supply Capacity number per front |
| **Commodity count** | Hard cap at three. Food is on probation |
| **Elaborate negotiation and haggling** | **Cut.** One package with a transparent AI evaluation |
| **Hundreds of technologies** | **Cut.** Three tracks plus projects |
| **Ideology as a full system** | Kept minimal: an alignment tag for opinion affinity, the Revolutionary government type, and the ideological objective. Expand only if playtests ask for it |

### 18.3 Systemic risks

1. **Snowballing.** A successful power gets richer, which brings more power. *Countermeasures:* occupation costs, balancing coalitions in the AI, diminishing returns on growth (catch-up for poorer states), Primacy rising tension with everyone. **Must be tested early with AI-vs-AI simulations.**
2. **Opaque causality.** With interacting systems, the player won't know why things happened. *This is the single biggest risk.* Every change needs a "because" chain. Without causal feedback, emergent complexity reads as randomness.
3. **Simultaneous turns frustrate defence.** Addressed by lead times, intelligence warnings and contingency orders. **Must be playtested.**
4. **AI passivity or incoherence.** Utility AIs often do nothing or flip-flop. Goal persistence and reasoning logs are essential. Build an **AI-only headless simulation mode** from day one to watch 50-year runs.
5. **Dominant strategies.** Likely candidates: espionage spam (raise detection), max Welfare turtling (security events, and objectives require action), proxy war being too cheap. Headless runs will reveal these.
6. **Fog frustration.** Never hide everything. Always show a range. Make the range visibly shrink when you invest.
7. **Nuclear balance.** If the penalties are too weak, nukes become a war-winning button. If they're too strong, they're irrelevant and deterrence is fake. Headless runs must measure how often AIs reach the nuclear threshold. The target is "rare but not never." The limited strike is the most dangerous option to tune.
8. **Balance of 16 asymmetric countries.** Objectives scoring normalises expectations. Use AI-vs-AI tournaments to measure win rates per country. Real countries make this harder: a historically weak country still needs a viable objective set.
9. **Real-world sensitivity.** The setting uses real countries (decided). Mitigated by a historical start date, generic event wording, abstracted statistics, and player-driven outcomes.
10. **Historical expectations.** Players will expect the USSR to behave like the USSR. AI personalities should start historically plausible but must be able to drift through regime change and events.

### 18.4 Mechanics on probation (cut if they don't earn their place)

- **Food:** keep only if famine and agricultural leverage produce decisions.
- **Debt:** keep only if borrowing is a real temptation; otherwise use a simple deficit penalty.
- **Arms Control treaty:** keep only if the AI uses it meaningfully.
- **Intelligence research track:** could merge into the Intel budget.
- **Domestic policies:** four is the cap.

---

## 19. Prototype Scope Summary

| Element | Prototype target |
|---|---|
| Playable countries | 16, plus ~13 Major AI and many minor countries (design/scenario-1980.md) |
| Land regions | ~100 (revised from ~70 for real geography; see scenario D.1) |
| Sea zones | ~25, with ~10 chokepoints |
| Commodities | 3 |
| Budget lines | 4 |
| Research | 3 tracks × 10 levels + a **few** national projects in v0.1 (§10.2) |
| Diplomatic actions | ~10, plus standing streams and Trade Agreement depth |
| Intervention | 5 Involvement Bands (§11.2) |
| Intel operations | ~7 |
| Domestic policies | 4 |
| Traits | ~25 in library |
| World events | ~6 templates in v0.1, growing to ~40 events |
| AI goals | ~12 |
| Campaign | 20 years (~80 quarterly turns), adjustable |
| Nuclear | **v0.1:** arsenal level, deterrence, one use action, full penalty package. **Later:** postures detail, 3 use scales, Nuclear Winter (§11.4) |

**Build order:** see **§22** (v0.1 engine milestone → v0.2 scenario completeness). Simulation before UI is a confirmed decision.

At every step, the headless runner outputs per-turn logs and summary statistics (wars, regime changes, price history, power shares, nuclear incidents, ledger and norm statistics) so balance can be judged from data.

---

## 20. Scenarios and Future Campaigns

**Decided:** the first and only campaign for now is **1980–2000**. After the model works, other eras follow: WW1 (~1914), WW2 (~1936–39), present day (~2025), and possibly others.

This isn't designed now, but it constrains the architecture from day one. **Nothing era-specific belongs in engine code.** Everything era-specific lives in a **scenario** data package.

### 20.1 What a scenario defines

| Scenario data | 1980 example | Why it must be data |
|---|---|---|
| Start date, length, tick cadence | 1980-01, 20 years, quarterly | WW1 might want 5 years with monthly turns |
| Region ownership and cores | USSR owns the Baltics, Germany split | Borders differ wildly by era |
| Playable roster and minor states | ~16 Cold War actors | WW1 has empires, 2025 has different powers |
| Country data, traits, AI personalities | Real 1980 governments | Per era |
| Starting treaties and blocs | NATO, Warsaw Pact | Per era |
| **Era rules** | Nuclear on; satellites available; colonies off | See 20.2 |
| Research baseline and ceilings | Military tech 5–8 | Era sets the scale |
| Commodity weights | Oil dominant | Coal mattered more in 1914 |
| Event pool | Oil shocks, revolutions, proxy wars | Era-appropriate events |
| Objectives pools and victory rules | Cold War objectives | Scenario-specific victory (§15, option F) |

### 20.2 Era rules: features as switches, not forks

Some mechanics exist only in some eras. They're switched on and off per scenario, never coded as era-specific branches:

- **Nuclear weapons:** off before 1945; the arsenal project is unavailable or late-unlocking in WW2.
- **Air power and satellite reconnaissance:** scaled down or off in WW1.
- **Colonies and overseas empires:** probably needed for WW1/WW2 (a non-core "colonial region" type with resistance). Not used in 1980.
- **War weariness and mobilization scales:** WW1/WW2 tolerate Total mobilization far longer.
- **Ideology/alignment tags:** Cold War blocs vs. WW2's fascist/communist/democratic split.

### 20.3 Map implication

**Use one base geography for all eras.** Regions are stable geographic units (e.g., "East Prussia," "Baltic Coast," "Ukraine-West"), and scenarios assign ownership and cores. Region boundaries should therefore follow lasting geographic and historical seams, not 1980 borders alone. This lets one map serve 1914, 1939, 1980 and 2025. A scenario may split or merge regions where it really has to.

### 20.4 Rule for now

When building the 1980 model, any time a value or behaviour is tempting to hard-code ("the USSR does X," "nukes exist"), it goes into scenario data or an era rule instead. Check: *would this line break a WW1 scenario?*

---

## 21. Commitments, Reputation and Persistent Consequences

*Approved 2026-10-03 from [design/us-mechanics-report.md](design/us-mechanics-report.md). This is one of the central mechanics of the game.*

**Principle:** the simulation records behaviour and observers draw conclusions. There's no morality, hypocrisy, goodness or evil meter, and no authoritative global trustworthiness value.

### 21.1 Event Ledger

A single append-only log owned by the simulation. It replaces all private AI memory.

| Field | Meaning |
|---|---|
| `actor`, `beneficiary_or_target` | Who acted, for or against whom |
| `kind` | **Back** (defence, guarantee, support stream, involvement), **Threat** (ultimatum, red line), **Norm** (implied principle, §21.4), **Coercion** (sanction, punitive strike, intervention against someone) |
| `outcome` | Honoured / Partial / Abandoned / Lapsed (neutral) / Pending |
| `cost_paid` | 0–1. Costly honouring counts for more |
| `case` | Beneficiary region, alignment, government type, whether the actor's rival was involved, norm tag |
| `visibility` | Public / Covert / Exposed, plus a `seen_by` observer mask |
| `turn` | Used for recency |

- **Fog:** an observer reads an entry only if it is Public or Exposed, or if it had intel coverage of the actor at or above the era's covert-visibility threshold that turn (1980 default: 60).
- **Size:** a few hundred entries per campaign. Old entries fold into priors.
- **Seeding:** scenarios seed historical entries as priors (for 1980: South Vietnam 1975, Hungary 1956, Czechoslovakia 1968, Suez 1956, Iran 1953, Camp David).

### 21.2 Commitment tests and grading

**A test opens when a commitment is called on:**
- an ally or guaranteed state is attacked
- a threat deadline passes
- a support stream is cut while the client is in need
- a norm the actor holds is violated by anyone

| Outcome | Condition |
|---|---|
| **Honoured** (1.0) | Forces committed to the front of at least 0.5× the attacker's strength there, **or** the beneficiary keeps all its core regions |
| **Partial** (0.5) | A declaration without forces, a Punitive strike only, or sanctions only |
| **Abandoned** (0) | No response, or withdrawal while the beneficiary is losing |

**Anti-exploit rules:**
- **Shadow commitments.** A commitment that is withdrawn or lapses stays in shadow for 8 turns. If the beneficiary is attacked during that time, the test is graded Abandoned ×0.75. A calm withdrawal costs reputation only with the former beneficiary.
- **Involvement commitments.** Acting at band 2+ (§11.2) for a side auto-creates a Back commitment, staked by band. It takes the creating action's visibility.
- **Both sides.** Arming both sides of the same war writes a Partial entry with each belligerent and cancels involvement credit.

### 21.3 Per-observer reputation read

```
Exp_k(E,P) = (n0·prior_k(P) + Σ w_i·s_i) / (n0 + Σ w_i)        k ∈ {Back, Threat, Norm}
  s_i  = 1 honoured · 0.5 partial · 0 abandoned        (entries E can see)
  w_i  = 0.5^(age/20) × rel(E,i) × (1 + cost_paid_i) × (abandoned ? 2 : 1)
  rel  = 1.0 if E was beneficiary/target · 0.5 if E shares region OR alignment with the beneficiary · 0.2 otherwise
  n0   = 3; priors from scenario data
```

| Value | Definition |
|---|---|
| **Credibility(E,P)** | 100 × Exp_Back ("will they defend me?") or Exp_Threat ("will they follow through?"), depending on the decision |
| **Trust(E,P)** | The same query restricted to entries where E was beneficiary or target |
| **Deterrence of a guarantee** | The *potential attacker's* Credibility reading of the guarantor × the guarantor's available force |
| **Global Credibility** | A display-only, power-weighted average. **Never an AI input** |

Observer divergence emerges by itself. A country that defends a Gulf client and abandons an African one is read differently in each region. A rival may find its threats credible and its guarantees not. A country that never saw the events holds only its prior.

### 21.4 Implied norms (v0.1, on probation)

A country is never penalised for failing to enforce a principle it never claimed. But once it publicly acts on a principle, later behaviour is compared against that precedent. **This is precedent and expectation, not morality.**

- **Creation:** an action at band 3+ against an aggressor, or a public Ultimatum citing a crisis type (aggression, proliferation, chokepoint closure), writes a **Norm** claim for the actor.
- **Tests:** a later violation of that norm by *anyone* opens a Norm test against the actor, including violations by its friends.
- **Effect:** feeds Exp_Norm, which feeds Threat credibility and the "could be me" and selectivity considerations of observers.

**Instrumentation requirement (mandatory during development).** Every norm logs:
- the norm created
- the action that created it
- the turn
- visibility
- the observers that learned of it
- each later comparison it triggered
- the reputation effect produced

```
NORM_CREATED: AGGRESSION  actor=USA  cause=PUNITIVE_ACTION  target=X  turn=17  seen_by=[...]
NORM_TEST:    AGGRESSION  actor=USA  violator=Y(friendly)  response=NONE  grade=ABANDONED  dExp_Norm: SAU -0.06, IND -0.11 ...
```

**Headless statistics to collect:**
- norms created per campaign
- tests per norm
- % of tests that changed reputation
- most common causes
- norms that never matter
- likely-unintuitive cases

**If the system produces excessive or incomprehensible penalties, simplify or remove it.** The mechanic is not protected for being clever.

### 21.5 Third-party reaction and exposure

Covered in §7.1 (third-party reaction formula) and §8.4 (`openness`-scaled exposure).

### 21.6 Permanent transfers

When Country A transfers military strength to Country B:
- B keeps it, whatever happens to relations later.
- A never automatically regains it.
- Future conflicts may involve weapons A once supplied.

Quality is a weighted average, capped at the recipient's Military tech + 2. Advisors are a temporary quality bonus. Arms outflows count toward the supplier's defence burden and arms-control limits.

*v0.1 as built (D33–D34):*
- Arms are a stream kind (Aid / Arms) plus a one-off transfer order. At most 10% of the supplier's strength moves per turn.
- Provenance is tracked per supplier and wears at the upkeep rate.
- Arms received count toward the recipient's upkeep funding, so a client cut off from supply keeps the strength but loses readiness. That "spare parts" leverage emerges without a new mechanic.

**There are no scripted blowback events.** Ordinary persistent state produces the effect. This is the preferred BRINK pattern: *state persists; consequences emerge.*

*Arms export as commerce (D58, built 2026-10-04).* Arms can be **sold** as well as given. The paradox it models: a small state can be an arms bazaar. Israel, North Korea and South Africa sold far above their size, to isolated buyers, to both sides and to their friends' enemies, until a patron objected (the 2000 Phalcon cancellation). It isn't an Israel rule; any country can sell, and the data decides who does.
- **Data:** `arms_industry` (0–1), the export orientation of the defence industry. In the 1980 scenario: ISR 0.8, FRA 0.7, SOV/PRK 0.6, GBR/CHN 0.5, USA/ZAF 0.4, FRG 0.3, EGY 0.2.
- **Order:** `SellArms { to, amount, covert }` (1 Initiative) creates an arms stream flagged `sale`, with all the stream rules: permanent, route, covert exposure, the both-sides ledger and blowback provenance. The buyer may `CancelArmsPurchase` (free).
- **Price:** the buyer pays build value × price, where price = 1 + 0.1 per sanctioner (max +0.5) + 0.25 if at war. Isolation is lucrative.
- **Payment:** from the buyer's pool, up to 15% of it; into the seller's pool, within an order book of `arms_industry` × GDP × 0.03 per turn.
- **Cost to the seller:** only (1 − industry) of the strength sold comes out of its own forces; the industry builds the rest. Sales are commerce, so they don't count toward the defence burden.
- **Objection:** the seller's allies and patrons who are at war with the buyer, or in tension of 50 or more with it, take an opinion penalty ("armed our rival", −15). Covert sales avoid this until they are exposed.
- **AI, seller:**
  - It counts the revenue margin against its own revenue, the risk of arming a hostile buyer, and patron and ally objections, weighted by how much support it gets from them.
  - Same bloc counts +8; the other bloc −12 (export controls). It also gains goodwill.
  - It sells only better kit than the buyer fields, or equal kit to a buyer with no industry. It sells only to buyers in need (danger, war or sanctions) and not already supplied.
  - Covert vs open is scored, so Iran–Contra-style covert sales can emerge.
- **AI, patron:** `keep_stream` adds "they are arming our rival" (−25). A client selling to the patron's rival risks its aid, which is the Phalcon veto.
- **AI, buyer:** it re-scores each purchase (need, cost, own industry, supplier hostility) and cancels after two bad assessments.
- **Gift fix:** the gift rule `arm_client` gains "we could sell these instead" (−20 × industry).
- **Voice:** the `ArmsSoldToPatronsRival` trigger, plus the existing both-sides, blowback and covert-exposure triggers.

### 21.7 Player feedback

Every commitment-related choice shows a **consequence preview**, built from the same calculation the AI uses (`RepDelta`):

```
Respond at Band 3 (Punitive strike) to the attack on Kuwait-region minor:
  Test grade if chosen: PARTIAL (no forces >= 0.5x attacker on front)
  Saudi Arabia  Credibility(Back) 71 -> 69
  Iraq-like     Credibility(Threat) 40 -> 52
  Implied norm written: AGGRESSION
  Tension +12 · War weariness +low · Budget -4
```

The reasoning log shows the top 3 ledger precedents behind each AI decision. A reputation-by-observer panel comes in the UI phase.

---

## 22. Milestones, Build Order and Gates

### 22.1 v0.1 — Engine milestone

Proves the systems work, in headless simulation, through six gates:

| Gate | Pass condition |
|---|---|
| **1. Memory matters** | 4-actor fixture: major power, ally, rival, neutral. Past behaviour measurably changes future diplomatic decisions |
| **2. Contradictory incentives** | Supporting a useful partner has real costs elsewhere. No universally superior answer |
| **3. Escalation works** | No option (none / coerce / proxy / limited strike / major) dominates all others |
| **4. Blowback without scripting** | A former client stays stronger from earlier transfers, with no special event |
| **5. Post-rival world** | Removing the peer competitor keeps the game interesting, with no separate ruleset, through defence-burden pressure, treaty re-evaluation, trade, regional competition, balancing and client relationships |
| **6. AI variety** | Repeated seeded runs: AI powers don't converge on one strategy. They sometimes intervene, abstain, coerce economically, use proxies, abandon peripheral commitments, defend important ones, deepen trade, reduce military burden and escalate. **Variation must come from state, information, personality and history, not randomness alone**. Measured over 200 seeded runs for the major powers: each behaviour's pooled frequency should fall in 20–95%, except **intervene: ≥5%** (D68; the 1980 war set offers few occasions, and interventions are expected from Kuwait-type wars). Intervene means fighting (band 3–4); an arms stream to a belligerent (band 2) is reported as "armed a side" (informational, not a criterion) and doesn't count as intervening, defending or abandoning a commitment |

**Build order:**

```
1  Core skeleton: workspace, seeded RNG, RON schemas, turn loop, Budget, Initiative, Stability,
   economy-lite, Opinion; deterministic long runs
2  Fog / observer state: AI consumes observer-visible state from day one
3  Diplomacy & economics: treaties, sanctions + substitution, trade, standing streams,
   Initiative costs, tension
4  Event Ledger & reputation: ledger, tests, shadow commitments, per-observer reads,
   third-party reactions, implied norms (instrumented), reasoning logs
5  Four-actor AI fixture                                            -> GATES 1, 2
6  Military capability & permanent transfers: force pools, threat, arms transfers -> GATE 4
7  War-lite: War -> Fronts (one instantiated), bands, Punitive, proxy routes -> GATE 3
8  Strategic reassessment: treaty re-eval, defence burden, balancing, post-rival behaviour,
   shallow/deep trade                                                -> GATE 5
9  Scenario integration: 1980 state, country data, restricted-action minor AIs,
   ~6 event templates, batch runner                                   -> GATE 6
```

**v0.1 test suite (12):**
1. No identity branching (lint + data swap)
2. Observer divergence
3. Fog respect
4. Past behaviour changes decisions
5. Explanation lines sum to score
6. Blowback without events
7. Post-rival rescore
8. Gate-6 variety
9. Determinism across thread counts
10. Forced-branch Pareto test (every option best on ≥1 metric, none best on all)
11. Fake-honour exploit
12. Withdraw-before-test exploit

### 22.2 v0.2 — Scenario completeness milestone

Added in this order, before the first serious human campaign:

| Order | Mechanic | Why this order |
|---|---|---|
| 1 | **P5 Commodity Production Policy** | Gives Saudi Arabia and other exporters their defining leverage, with modest systemic complexity |
| 2 | **P10 Covert Programmes** | Strategic identity for Israel, Pakistan, India, North Korea and South Africa |
| 3 | **P3 Regime Transition** | Government transformation must be distinct from state collapse before human play |
| 4 | **P2 Region Loyalty & Secession** | Largest blast radius: dynamic country creation, ownership, military, cores, treaties, AI initialisation, economy, objectives, wars, successor relations. Only after everything else is stable |

### 22.3 Deferred beyond v0.2 (pull in only on evidence)

| Mechanic | Pull in when |
|---|---|
| ExpAbandon and Selectivity queries | Players can't tell why observers disagree |
| Threat-trend term ("contain the rising power") | Post-rival strategy mix lacks containment |
| Declared Public Stances | Implied norms prove unreadable |
| Finance/technology sanction scopes; world interest rate (P7) | Debt and financial-crisis targets miss |
| Conditional aid packages / pledges | Proliferation play feels flat |
| Supplier dependency ("spare parts") | Arms clients feel too independent |
| Host resentment of bases | Basing feels free |
| Denounce exposing covert entries; propaganda salience | Covert play lacks counterplay |
| Election-calendar amplifier | Elections feel irrelevant |
| Leadership-change personality drift (P11) | AI too static within a run |
| Contested chokepoints (P6); outposts as full objects (P8) | Chokepoint or basing play feels thin |
| Full nuclear system (§11.4) | After the basic geopolitical simulation works |
| Standing response policies (anti-tedium) | Commitment tests become chores |

### 22.4 Rejected mechanics

The authoritative list is in [design/us-mechanics-report.md §12](design/us-mechanics-report.md). **Check it before proposing a mechanic.** Highlights:
- morality/hypocrisy meter
- stored global Credibility
- a many-level intervention ladder
- scripted blowback
- a separate Patronage treaty
- a sector-level economy
- an entanglement meter
- a Cold War Victory flag or post-Cold-War ruleset
- "if democracy" switches
- refusals costing Trust
