# Agent 4: AI Designer. Generic utility inputs and reasoning rules for great-power behaviour

**Scope.** Inputs and reasoning only, with no scripts. Every rule below takes `(observer E, actor P, ...)`. Nothing branches on country identity. Country-specific behaviour comes from **data**: personality vector (B.18), traits, treaties, starting ledger and starting estimates.

---

## 1. Utility inputs

### 1.1 The Commitment Ledger (the one new data structure)

DESIGN §14.6 keeps memory per AI and per country, and §7.1 keeps Credibility as a single global number. The brief needs observers to disagree: country A can believe "P protects allies" while country B believes "P drops partners once they stop being useful." The cheapest way to get that is **one shared, append-only event log plus a per-observer read function**. Reputation is never stored per observer. It is *computed* from what each observer saw, weighted by what that observer cares about.

**`CommitmentRecord`** (around 40 bytes, at most a few hundred per campaign):

| Field | Meaning |
|---|---|
| `actor` | Who made or held the commitment (P) |
| `beneficiary` / `target` | Who it protected or threatened |
| `kind` | `Defend` (alliance, guarantee, base host), `Threat` (ultimatum, declared red line), `Support` (patronage, aid, arms), `Norm` (a public claim such as "aggression will be punished"; see below) |
| `outcome` | `Honoured`, `Partial` (responded below the implied level), `Abandoned`, `Lapsed` (ended by mutual treaty, which is neutral), `Pending` |
| `cost_paid` | What honouring cost P, normalised 0–1 (budget, casualties, tension). Costly honouring counts for more |
| `case_features` | A snapshot when the test happened: beneficiary's value to P (0–1, from P's goals), beneficiary's alignment and government type, region, whether P's primary rival was involved, the norm tag |
| `visibility` | `Public`, `Exposed` (was covert, now revealed), `Covert` |
| `turn` | For recency |

**When records are written.** Records are written by the simulation, not the AI, whenever a *test* occurs:
- an ally or guaranteed state is attacked (a `Defend` test);
- a threat deadline passes (a `Threat` test);
- a patronage or aid stream is cut while the client is still in need (`Support` → `Abandoned`), or is kept through a client crisis (`Honoured`);
- a **norm violation occurs anywhere** (an invasion, a proliferation reveal, a chokepoint closure) while P holds a matching public `Norm` claim. P's response is graded against its claim and recorded *regardless of who the violator is*. That is how "similar cases treated differently" gets into the data without a hypocrisy meter: the ledger simply holds two `Norm:aggression` tests, one `Honoured` (hostile violator) and one `Abandoned` (friendly violator).

**Fog.** Observer E reads a record only if it is `Public`, or `Exposed`, or E had Coverage ≥ 40 on P at that turn (store a compact `seen_by` bitmask, 16 bits). A covert betrayal nobody saw does not hurt reputation until it is exposed. Exposure then flips its visibility and it becomes very salient.

### 1.2 Per-observer expectations (computed, cached per strategic assessment)

For each observer E, actor P and kind k, the expectation is a weighted beta-mean:

```
Expect_k(E,P | ctx) = (α_k + Σ w_i · s_i) / (α_k + β_k + Σ w_i)
  s_i = 1 Honoured, 0.5 Partial, 0 Abandoned   (records of kind k that E has seen)
  w_i = recency(turn) × similarity(ctx, case_i) × salience(E, i) × (1 + cost_paid_i)
  recency    = 0.5 ^ (age / H), with H ≈ 20 turns (data)
  similarity = Π over features of a kernel: value-to-P band, same alignment, same region,
               rival involved, same norm tag. Each feature mismatch ×0.5
  salience   = 3 if E was the beneficiary, 1.5 if beneficiary was E's ally, neighbour or same alignment, else 1
  α, β       = prior from P's government type + treaty count (data); also how a new country starts
```

Every derived belief is a query of the form "how did P behave in cases *like mine*?":

| Belief | Query |
|---|---|
| **Will P defend me?** `ExpDefend(E,P)` | `Defend` records, ctx = E's own features (E's value to P, alignment, region) |
| **Will P follow through on this threat?** `ExpThreat(E,P)` | `Threat` records, ctx = the current threat's target features |
| **Will P abandon me when I stop being useful?** `ExpAbandon(E,P)` | 1 − `ExpDefend ∪ Support` with ctx's value-to-P set to E's *projected* value (for example, after P's rival collapses). A P that honours only high-value clients produces a steep gap between the high-value and low-value queries. Low-value observers perceive that gap as abandonment risk |
| **Does P apply its norm consistently?** `Selectivity(E,P,norm)` | \|honour rate when violator aligned with P − honour rate when violator hostile to P\| over `Norm` records E saw. 0 = consistent, 1 = purely factional |

**Cost.** 16 × 16 × 4 kinds ≈ 1,000 cached floats, recomputed every ~4 turns or when a new record lands. Each recompute scans fewer than 500 records. Old records (weight < 0.02) fold into α/β and are dropped.

**Relation to existing values.**
- **Trust(E,P)** stays the bilateral record (records where E is beneficiary or target). Implement it as the ledger query with salience restricted to E.
- **Global Credibility** becomes a **derived display value**: the power-weighted mean of `ExpThreat`/`ExpDefend` across all observers, shown in the UI as "how the world sees you on average". AI decisions **never read the global value**. Threat strength in §7.3 becomes `perceived power × ExpThreat(target, threatener)`, which is per observer.
- The UI gets a "Reputation by observer" panel: for a selected country, the top 3 weighted records behind its belief ("Remembers: you let Hesperia fall, 1986 (−); you defended Qasr, 1983 (+)").

### 1.3 The full input set

| Input | Definition | Source | Range |
|---|---|---|---|
| `Opinion(E→P)` | Existing modifier sum | §7.1 | −100..100 |
| `Trust(E,P)` | Bilateral ledger query | Ledger | 0..100 |
| `ExpDefend`, `ExpThreat`, `ExpAbandon`, `Selectivity` | §1.2 | Ledger + fog | 0..1 |
| `Threat(E,X)` | Estimated military in reach × hostility × aggression memory × paranoia, plus a **trend term** (projected growth over 10 turns × paranoia) | §14.3 on estimates | 0..100 |
| `SharedRival(E,P)` | max over X of min(`Threat(E,X)`, E's estimate of `Threat(P,X)`) | Derived | 0..100 |
| `PowerShare_E(P)` | P's estimated power projectable into E's region ÷ total estimated projectable power there | Estimates, bases, naval reach | 0..1 |
| `IntentEst(E,P)` | Hostile intent: weighted rate of P's coercion, intervention and threat records against states similar to E, plus negative Opinion and Tension | Ledger + §11 | 0..1 |
| `Dep(E→P)` | Max of: trade share with P, aid ÷ budget, arms share of inventory, patronage share, critical commodity share | Economy, treaties | 0..1 |
| `Leverage(P over E)` | `Dep(E→P) − Dep(P→E)` | Derived | −1..1 |
| `IdeoDist(E,P)` | Government-type distance plus alignment-tag mismatch (data matrix) | Data | 0..1 |
| `AssocCost(E,P)` | Domestic Legitimacy cost of visible closeness: `IdeoDist × openness(E) × (exposed P acts that contradict E's own norm claims)` | Stability model | 0..1 |
| `ValueToP_E(E)` | E's *estimate* of how useful it is to P (P's goals touching E's region, commodity or chokepoint, rival proximity). It is exact only if Coverage(E,P) ≥ 80 reveals P's goals | Estimates | 0..1 |
| `Entrap(E,P)` | Risk of being drawn into P's wars: P's max tension with others × E's exposure (bases, proximity) | §11 | 0..1 |
| `Treaties(E,P)` | Set of active treaties, with age | §7.2 | — |
| `Goals(E)` | Active strategic goals and their weights | §14.4 | — |
| `Pers(E)` | aggression, risk, paranoia, loyalty, greed, ideology, opportunism | B.18 | 0..1 |
| `Need(E)` | Budget gap, Stability deficit, commodity deficit | §2, §9 | 0..1 |

---

## 2. Utility considerations for E's decisions about patron P

**Shared form.** `U = Σ c_j · m_j(Pers) · term_j`, with each term scaled to points (about −60..+60). E acts if `U > θ`, where θ is a status-quo bias (default 10; it is higher for treaty exits, see hysteresis). The **explanation list is exactly the term list**, sorted by \|value\| and showing the top 5. The displayed lines must sum to the shown score, apart from a single "other" line. Multipliers `m` use the personality vector; `(·)` below means "multiplied by".

### 2.1 Accept P's aid
| Term | Formula | Example line |
|---|---|---|
| Need | `40·Need(E)·(0.5+greed)` | `+22 budget shortfall` |
| Shared rival | `0.3·SharedRival` | `+15 shared rival: Northern Federation` |
| Dependence creep | `−30·Dep(E→P)·paranoia` (only if Dep > 0.3) | `−12 would deepen dependence (61% of arms from P)` |
| Association | `−25·AssocCost·ideology` | `−9 unpopular association at home` |
| Other patron's reaction | `−Δopinion of E's other patrons and P's rivals·Dep(E→them)` | `−8 angers current supplier` |
| Strings | Value of conditions attached (bases, votes) treated as their own decisions | `−6 requires basing rights` |

### 2.2 Accept P's bases
| Term | Formula | Line |
|---|---|---|
| Deterrence | `0.6·Threat(E,X*)·ExpDefend(E,P)·reachGain` | `+24 deters primary threat` |
| **Tripwire** | `+20·(1−ExpDefend)·(1−paranoia·IntentEst)`: hosting raises P's own cost of abandoning E, so states that doubt P *gain* more from bases | `+10 binds P to our defence` |
| Rent and aid | Money term, as in 2.1 | `+8 basing payments` |
| Sovereignty | `−30·IdeoDist·ideology − 10·(history of occupation by P, from the ledger)` | `−14 foreign troops on our soil` |
| Target risk | `−40·Entrap·(1−risk)` | `−11 becomes target for P's rivals` |
| Rival reaction | `−Threat(E,X*)·0.2·paranoia(X*) estimated` | `−7 provokes neighbour` |

### 2.3 Join P's sanctions against T
| Term | Formula | Line |
|---|---|---|
| Patron goodwill | `20·Dep(E→P)·loyalty + 0.1·Opinion(E→P)` | `+12 maintain standing with P` |
| Own grievance | `0.4·Threat(E,T) + memory(T vs E)` | `+9 T's 1984 border incursion` |
| Norm fit | `+15 if E holds the same norm claim and T violated it`, `× (1 − Selectivity(E,P,norm))` | `+11 punishes aggression (we agree)` |
| Own trade loss | `−50·tradeShare(E,T)·greed` | `−17 loses 9% of our trade` |
| Retaliation | `−30·ExpThreat(E,T)·reach(T→E)` | `−6 T may retaliate` |
| Efficacy | `+10·marginal effect`, which diminishes once coalition share passes 60% or stays negligible below 10% | `−4 sanctions unlikely to bite` |
| Selectivity | `−20·Selectivity(E,P,norm)·ideology·openness(E)` | `−8 P ignored the same act by its friend` |

### 2.4 Oppose P's intervention in T
E chooses a level from {none, denounce, deny overflight or bases, join sanctions on P, arm T, defensive pact with T}. Each level gets its own utility; E picks the maximum.

| Term | Formula | Line |
|---|---|---|
| Power growth | `40·ΔPowerShare_E(P)·IntentEst·paranoia` | `+16 P's reach in our region grows` |
| Precedent ("could be me") | `30·similarity(E,T)·IntentEst` (similarity: government type, alignment, prior norm violations) | `+13 states like us are next` |
| Affinity with T | `0.3·Opinion(E→T) + 15·(1−IdeoDist(E,T))·ideology` | `+8 fellow non-aligned state` |
| Sovereignty norm | `+10 if E holds the claim`, `× Selectivity` bonus | `+6 sovereignty must be respected` |
| P's punishment | `−40·ExpThreat(E,P)·Leverage(P over E)·(1−risk)` | `−21 P controls 40% of our exports` |
| Shared rival | `−0.5·Threat(E,T)` if T is E's enemy | `−18 T is our rival too` |

### 2.5 Request a guarantee from P
| Term | Formula | Line |
|---|---|---|
| Security gap | `0.8·Threat(E,X*)·(1 − selfDefence)·ExpDefend(E,P)` | `+28 cannot hold alone against X` |
| Acceptance odds | E's estimate that P will agree, from `ValueToP_E(E)` and P's past acceptance records. Low odds cost Initiative and opinion if refused | `−5 P may refuse` |
| Provocation | `−0.3·paranoia(X)est·Threat(E,X*)` | `−9 X will see it as encirclement` |
| Price | Expected strings (bases, alignment), scored as in 2.2 | `−7 will demand basing` |
| Entrapment | `−25·Entrap·(1−aggression)` | `−6 dragged into P's quarrels` |
| No alternatives | `+10` if no other candidate guarantor scores within 50% | `+10 no other protector` |

### 2.6 Abandon P's bloc (exit a treaty or switch alignment)
| Term | Formula | Line |
|---|---|---|
| Abandonment fear | `40·ExpAbandon(E,P)·paranoia` | `+17 P dropped Hesperia when it stopped mattering` |
| Lost purpose | `−(current security value of the bloc)`. This is the deterrence term from 2.2 recomputed. It falls when `Threat(E,X*)` falls, for example after the rival collapses | `+20 Northern Federation no longer threatens us` |
| Grievance | `0.4·max(0,−Opinion) + coercion records by P against E` | `+12 P sanctioned us in 1991` |
| Alternative offer | Best competing patron's offer utility | `+9 Federation offers arms at cost` |
| Dependence | `−50·Dep(E→P)` | `−25 60% of trade with bloc` |
| Punishment | `−30·ExpThreat(E,P)·Leverage` | `−10 P will retaliate economically` |
| Loyalty and own reputation | `−30·loyalty·min(1, treatyAge/20)`, plus a projected hit to *E's own* ledger (exit is a recorded `Abandoned`) | `−18 we keep our word` |

**Hysteresis:** E exits only if U > θ_exit (25) for **two consecutive assessments**. This prevents flip-flopping and gives P a visible warning window ("Assessment: Qasr reconsidering alliance").

### 2.7 Balance against excessive power of P
`BalanceDrive(E,P) = 100 · max(0, PowerShare_E(P) − θ_power) · IntentEst(E,P) · (0.5 + paranoia)`, with θ_power = 0.4 (data). The drive feeds the existing `Balance Against Hegemon` goal. Its candidate actions are: arm internally, ally with other balancers, deny bases, refuse P's sanctions, and arms-control demands. Safeguards are in §4. Lines: `+14 P holds 55% of power in our region`, `×0.3 P has not coerced states like us`.

### 2.8 Exploit P's inconsistency
This is an opportunistic action family, scored as `opportunism · Selectivity · audience`:
- **Propaganda or denounce.** Value = Σ over third parties A of `ideology(A)·(norm salience to A)·Selectivity`. Mechanically, it raises the **salience weight** of the relevant ledger records for those audiences for N turns. It does not invent records. Line: `+11 P's double standard on aggression resonates with 5 states`.
- **Probe.** Act against a P client where `ExpDefend(self, P | ctx = that client)` is low: an opportunistic invasion, a claim or a chokepoint squeeze. Score = opportunity × (1 − ExpDefend for that client's feature band). This is the Falklands or Kuwait logic, and it emerges from P's own record.
- **Bid up loyalty.** Threaten to realign if `ValueToP_E(self)` is high and P has records of paying to keep wavering clients. Line: `+9 P has paid to keep clients before`.

### 2.9 Remain aligned with P despite ideological disagreement
This is the same evaluation as 2.6. The key rule is the **ideology discount under threat**:
`IdeoPenalty = −30 · IdeoDist · ideology · (1 − 0.8·Threat(E,X*)/100)`
Shared danger therefore crowds out ideological friction. Example readout:
```
Qasr STAYS aligned with Atlantic Commonwealth (score +31)
  +30 shared rival: Kavaran
  +18 P defended states like us (Hesperia 1983)
  +12 arms dependence
   −7 ideological distance (monarchy vs democracy), discounted by threat
  −22 domestic unpopularity of foreign troops
```

---

## 3. Great-power AI using the same framework

### 3.1 Intervention as one choice over a ladder
This is not ten buttons. Each **Crisis** (a war, aggression against an aligned state, an insurgency, a proliferation reveal) generates one decision per interested power: **choose a level L ∈ {0 none, 1 pressure, 2 sanctions, 3 covert aid, 4 arms, 5 advisors, 6 proxy forces, 7 punitive strike, 8 limited intervention, 9 coalition war, 10 occupation}**. Each level is a data row: `cost(budget, initiative, manpower)`, `visibility`, `tension`, `commitmentStake`, `entanglementHazard`, `effect`.

```
U(L) = Stake · ΔPsuccess(L)                        goal value × estimated change in outcome
     + RepDelta(L)                                 projected ledger effect (below)
     − Cost(L)·(0.5+greed)
     − DomesticCost(L)                             war weariness × trait multipliers × duration estimate
     − Tension(L)·(1−aggression)
     − EscalationRisk(L)·(1−risk)                  rival's estimated escalation tolerance and arsenal
     − EntanglementHazard(L)·horizon·(1−risk)      "winning the operation is the cheap part"
```

**`RepDelta(L)` is the key coupling.** P runs the §1.2 expectation function on a *hypothetical* record ("if I respond at level L, the test is graded Honoured, Partial or Abandoned") and sums the change in every observer's `ExpDefend`/`ExpThreat`/`Selectivity`, weighted by how much P values that observer's alignment (from P's goals). The same calculation drives the player's consequence preview ("Doing nothing: Qasr −0.21 ExpDefend, Hesperia −0.15, Kavaran +0.18 probe incentive"). Because similarity weighting is built in, abandoning a **peripheral, dissimilar** client costs little reputation, while abandoning a **core** ally costs a lot. Selective commitment becomes a rational, readable strategy with real but localised costs.

**Effectiveness.** `ΔPsuccess` comes from estimated force ratios, so it is subject to fog. P can over- or under-intervene from bad intelligence. Higher levels can also *lower* Psuccess against resistance-prone targets (occupation feeds §6.7 resistance), which is how quagmire emerges.

### 3.2 Accepting losses versus escalating
Each assessment, an ongoing intervention is re-scored over {withdraw, hold, escalate one level}, **forward-looking only**. Sunk costs are excluded explicitly. Losses already taken matter only through what they did to the ledger and to domestic war weariness. Withdrawal writes an `Abandoned` or `Partial` record whose weight scales with `visibility × commitmentStake` of the current level. The emergent consequences:
- **Covert levels (3–5) keep exits cheap.** P has an incentive to start low and stay deniable.
- **Public high levels lock P in.** Withdrawing from level 8 is a salient record. The decision reads: escalate if `U(escalate) − U(withdraw) > 0`. Withdrawal wins when DomesticCost (weariness compounds per turn) plus EscalationRisk exceeds RepDelta, which happens when the case is peripheral, the observers who care are few, or the rival's resolve is estimated as high.
- **Loyalty** raises the weight P places on its *own* reputation (a multiplier on RepDelta of 0.5 + loyalty), so loyal P "honours commitments at a cost" exactly as §14.5 states.

### 3.3 Economic coercion
Sanctions, embargo, aid cut and market access are levels 1–2 of the same ladder, but they get their own leverage term: `ΔPsuccess(sanction) = f(Leverage(P over T), coalitionShare, T's substitution options via P's rivals)`. Costs include P's own trade loss (with greed and sector-pressure weighting via the Stability Prosperity driver), and **allied cost**: Σ allies' trade with T × their resulting Opinion drop toward P. Coercing a *friend* is allowed and sometimes optimal when Leverage is high, but it writes a coercion record that raises `IntentEst` and `ExpAbandon` among observers similar to that friend. Coercion therefore spends reputation, not just money.

### 3.4 After the primary rival collapses
There is no special ruleset. The strategic assessment reruns, the threat map changes, and the §14.4 goals are rescored:

| Emergent strategy | What makes it score highest |
|---|---|
| **Retrenchment** (`Develop Economy` + `Restore Stability`, lower military slider) | Low residual threat, high debt, high domestic war weariness, high greed, low aggression. Peace-dividend pressure appears through the Security driver no longer being satisfied by a named enemy |
| **Coalition leadership** (`Build Bloc` kept alive by trade and norms) | High loyalty, many allies with low `ExpAbandon` toward P, ally trade dependence |
| **Integration** (`Develop Economy` via trade treaties) | High greed, partners with high trade complementarity |
| **Primacy** (`Contain [rising X]`) | High paranoia × the **trend term** on X's projected power. Containment of an *emerging* rival is threat-trend driven, not current-power driven |
| **Selective intervention** (`Protect Client`) | Clients with high Stake (commodity, chokepoint) and a credibility record P wants to protect |
| **Democracy promotion** (`Spread Alignment`) | High ideology, low threat, high freedom of action |

Meanwhile every ally's 2.6 "lost purpose" term rises and their `ValueToP` estimates fall. Allies therefore wobble unless P pays (trade, aid) or finds a new shared purpose. That is the strategic identity problem, produced by ordinary terms.

### 3.5 Sources of variety (gate 6)
1. **Fog.** Each power's estimates differ per run (seeded intel outcomes), so Stake and Psuccess differ.
2. **Personality drift.** Election and leadership events (P11, seeded) shift the vector, for example aggression ±0.15 and risk ±0.1.
3. **Path-dependent ledger.** Early honours or abandonments change every later RepDelta.
4. **Attention scarcity.** Simultaneous crises compete for Initiative and Budget. The same crisis gets L0 in a busy year and L4 in a quiet one.
5. **Domestic state.** War weariness, Stability and the electoral calendar.
6. **Bounded decision noise.** Choose by softmax over the top-3 levels with temperature τ = 0.05·(1 − |U gap| normalised)·(0.5 + opportunism), seeded via `rand_chacha`. Noise only matters among near-ties, so the AI never makes absurd choices.

---

## 4. Safeguards

### 4.1 Anti-dogpile (strength alone does not trigger balancing)
1. **Threat = capability × intent.** `BalanceDrive` multiplies by `IntentEst`, and `IntentEst` comes from P's coercion and aggression *records against states similar to E*. A strong P with a benign record (IntentEst ≈ 0.1) draws little balancing. A coercive P draws a lot. The player controls this through behaviour.
2. **Reach-local power.** `PowerShare_E` counts only power projectable into E's region. Distant giants matter less than local rivals.
3. **Local threats dominate.** The goal scorer ranks `Balance Against Hegemon` against E's other threats. A neighbour's threat usually outranks a distant hegemon's.
4. **Bandwagoning is evaluated too.** For weak E with high `ExpDefend(E,P)` and low IntentEst, aligning *with* P scores higher than balancing.
5. **Diminishing marginal contribution.** The value of E's balancing action is `marginal change in P's PowerShare` ÷ the number of existing balancers. This is generic free-riding, and it stops everyone piling on.
6. **Hostile action costs scale per target, not per attacker.** No coordination bonus exists for attacking the leader. The nuclear "automatic coalition" in §11.4 stays the only scripted-looking pile-on, and it is triggered by an act, not by power.

### 4.2 Anti-ideology-lock
1. **Bounded ideology term.** \|ideology term\| ≤ 30·ideology. Threat, dependence and need terms can reach about 60, so pragmatism can always win when stakes are high.
2. **Ideology discount under threat** (§2.9).
3. **Scope by decision.** Full weight applies on alignment, bases and guarantees, half weight on arms purchases, and quarter weight on trade agreements, where cooperation across ideological lines is normal.
4. **Ideology becomes a real cost.** Most ideological friction runs through `AssocCost` (domestic Legitimacy), so it is a price the AI pays, not a veto.
5. **Extremes are allowed but not absolute.** Ideology 0.95 regimes can still buy arms from an enemy's ally when `Need` is critical (the Iran-Contra-like case) and the deal is covert (AssocCost × visibility).

---

## 5. Headless deterministic AI behaviour tests

All tests use fixed seeds and minimal 4-actor fixtures (Patron, Ally, Rival, Neutral) unless noted.

1. **No identity branching (lint plus permutation).** CI greps the `ai` crate for country IDs or tag literals. Swapping the data of two actors (names, flags) produces mirrored decisions bit-for-bit.
2. **Observer divergence.** P honours a guarantee to high-value A and abandons low-value B. Assert `ExpDefend(A,P) > 0.7`, `ExpDefend(B,P) < 0.4`, and that a third low-value C has `ExpAbandon(C,P)` > `ExpAbandon(high-value D,P)`.
3. **Fog respect.** A covert abandonment unseen by E leaves `ExpDefend(E,P)` unchanged. After exposure it drops by more than the public-equivalent amount × 0.9.
4. **Recency.** An abandonment 40 turns ago moves expectations less than 25% as much as a fresh one.
5. **Selective enforcement.** P sanctions a hostile aggressor and ignores a friendly one. `Selectivity(E,P,aggression)` > 0.4, and high-ideology democracies' join-sanctions scores carry a negative selectivity line.
6. **Gate 1, past behaviour matters.** The same aid offer is accepted after P honoured a guarantee and rejected after P abandoned one (other inputs equal).
7. **Explanation integrity.** For every evaluated decision, the displayed lines sum to the score within ±1 point, and every line maps to a named term.
8. **Ideology discount.** An ideologically distant ally with Threat 80 stays aligned. The same ally with Threat 10 exits within 3 assessments if other terms are neutral.
9. **No ideology lock.** An ideology-0.95 actor with critical Need accepts a covert arms deal from an opposed-alignment supplier in the fixture.
10. **No dogpile.** A dominant P (PowerShare 0.6) with no coercion records in a 16-actor run: fewer than 3 actors adopt `Balance Against Hegemon` across 200 seeds on average. With repeated coercion records, at least 5 do.
11. **Free-riding.** The marginal balancing utility of the 6th balancer is less than half that of the 1st.
12. **Sunk-cost neutrality.** Two interventions identical in forward state but with different past losses get the same continue/withdraw decision (excluding ledger and weariness effects).
13. **Covert exit is cheaper.** Withdrawing from level 4 (covert arms) costs less RepDelta than withdrawing from level 8. Assert that AIs with risk < 0.4 start at a covert level more often.
14. **Blowback, no events.** Arm a client to +20 strength, flip relations through the ledger. The client keeps its strength, and its 2.4/2.6 scores against the patron use it.
15. **Post-rival rescore.** Remove the peer rival. Within 2 assessments the patron's goal set changes in ≥ 90% of seeds, and the distribution over {retrenchment, coalition, integration, primacy, selective} has no option above 60% across 500 seeds and ≥ 3 options above 10%.
16. **Gate 6 variety.** An AI US in 500 full runs. Each of {intervened at L ≥ 7, restrained at L ≤ 2 in a qualifying crisis, applied sanctions/embargo, withdrew from an intervention} occurs in ≥ 20% of runs, and none is universal (≤ 95%).
17. **Generic patron check.** Running tests 2, 6, 13 and 16 with USSR, China, France-like and Saudi-like data as P must pass with only data changes.
18. **Determinism.** The same seed gives identical reasoning logs across 3 runs and across thread counts (rayon).

---

**Interface notes for other agents.**
- Simulation owns writing ledger records and the test-grading rules.
- AI owns the expectation queries and caching.
- UI owns the per-observer reputation panel and the "RepDelta" preview.
- Proposed DESIGN changes: Credibility (§7.1) becomes derived from the ledger, threat strength (§7.3) uses `ExpThreat` per observer, and §14.3 gains the threat-trend term.
