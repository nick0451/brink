# Agent 1: Core Systems Designer. US as Networked Hegemon

**Bottom line.** Most of the desired US gameplay already exists in DESIGN.md. Three things are missing:
- a **shared memory of what countries did**;
- **per-observer interpretation** of that memory;
- a few parameter extensions to existing actions.

Only one genuinely new engine primitive is needed for v0.1: the **Event Ledger with Commitment records**. That ledger replaces the private per-AI memory in §14.6 and the stored global Credibility value in §7.1. Everything else is a field or a formula on a system that already exists. Nothing below is US-specific.

---

## 1. Existing systems sufficient

| System | US behaviour it already produces |
|---|---|
| **Initiative (§2.3)** | Reach is not the same as attention. The US can affect almost every crisis but acts in only 3–5 per turn, which produces conflicting commitments and selective engagement without any extra rule. "Which crisis do I ignore this turn?" is the core US decision, and ignoring a crisis is what produces selective enforcement (see the ledger in §2). |
| **Budget, four lines (§2.1) + Debt (§2.2)** | Aid, arms and intervention all compete with Welfare, which is Prosperity, which is Stability for a democracy. Guns vs. butter is the domestic constraint. Cheap credit for democracies and financial hubs lets the US run deficits for a buildup and pay later. |
| **Stability: Legitimacy and War Weariness drivers, Democracy weights (§9.1)** | Domestic consequences already depend on outcomes: casualties, long wars and exposed scandals (the Spy scandal event in §12). "Winning the first operation was the cheap part" already emerges from **Occupation/Resistance (§6.7)**, which is sponsorable by rivals, plus war weariness that keeps rising. |
| **Intelligence ops + detection → Incident (§8.4)** | Covert aid, funding the opposition, arming insurgents and coups all exist with exposure risk. Domestic cost depends on exposure, so the "Iran-Contra" situation is just an exposed op in a democracy. |
| **Proxy war: Arms / Advisors / Volunteers (§11.5)** | Rungs 3–6 of the brief's ladder already exist as actions with rising detection and impact. |
| **Bilateral and Global Tension (§11)** | Escalation is a shared cost: tension raises prices, alarms neutrals and drives balancing. Gating war on tension or a casus belli (§11.3) already makes intervention from a calm baseline costly. |
| **War aims + War Score + regime-change aim (§6.6)** | Covers limited intervention, coalition war (allies joining through contingency orders) and occupation/regime change. Bigger aims already mean more domestic resistance and more neutrals leaning against you. |
| **Sanction / Embargo (§7.3)** | Already costs the sanctioner, already scales with allies joining, and already raises tension. The Interdependence loop (§3.3 #6) already makes trade partners vulnerable to each other. |
| **Opinion as listed modifiers (§7.1)** | Per pair and directional. The visible modifier list is the "because" chain for reputation. It needs one new input (§2.3 below). |
| **Trust, per pair (§7.1)** | Bilateral record of kept and broken deals. It stays per pair, as the brief wants. |
| **Alignment tag + government type** | Supplies the "friendly authoritarian vs. democratic ally" axis that observer reactions need (§2.3 below). No new ideology system. |
| **AI threat/opportunity scores, goal library, personality (§14.3–14.5)** | `Balance Against Hegemon` and the power-balance input already exist, so balancing against excessive US power emerges. `Protect Client`, `Contain [X]` and `Build Bloc` drive patron behaviour. The `Loyalty` and `Opportunism` dimensions decide whether an AI US honours costly commitments. Strategic reassessment every ~4 turns or on a shock re-picks goals when the threat map changes, and that change is the engine of the post-Cold-War transition. |
| **AI regime collapse (§9.2, Stability < 10 → new government type and personality)** | Enough to collapse the peer rival for Gate 5. P3 (managed Transition) and P2 (Secession) aren't needed in v0.1. |
| **Events read state (§12)** | Peace-dividend or "Coalition Restraint" flavour can trigger from state. No scripted outcomes are needed. |
| **Global Tension → prices and trade (§11.3)** | When the rival disappears, great-power tension falls, prices ease and trade grows. "Trade matters more after the Cold War" emerges from this. |

---

## 2. Existing systems needing modification

### 2.1 AI Memory (§14.6) → one shared **Event Ledger** (sim-owned), read through each observer's fog

- **Change.** Replace the per-AI private record with one append-only ledger in `sim-core`. Each AI reads only the entries it is entitled to see, through its fog-filtered view.
- **Why.** "Observer memory" needs one set of facts that different observers know differently and value differently. Private per-AI memories would duplicate state and drift apart.
- **Data: `LedgerEntry`**
  - `id`, `turn`, `actor`, `kind` (aid, arms_transfer, sanction, strike, war, op_type, treaty_signed, treaty_broken, commitment_test…), `target`, `beneficiary`, `rung` (0–7 from §11.2), `magnitude`
  - `visibility`: `Public | Covert`, plus `known_by: Set<CountryId>`
  - `commitment_ref: Option<CommitmentId>`
- **Visibility rule.**
  - Public entries are known to everyone.
  - Covert entries are known to the actor, plus the target if detection succeeds, plus each observer O with Coverage(O→actor or O→target) ≥ 60, rolled at creation.
  - Exposure (a failed detection roll, or a Denounce, see 2.3) sets the entry to Public.
- **Writers:** resolution steps 1, 2, 4 and 9.
- **Readers:** Opinion, Trust, Credibility, Threat, and the UI.
- **Retention.** Prune entries whose decayed weight falls below 0.05. This keeps the ledger small (a few hundred entries).

### 2.2 Credibility (§7.1): from a stored global scalar → **derived per observer** from Commitment records

The brief rules out a single global truth, while DESIGN.md currently stores one Credibility value per country. Reconcile them as follows:

- **Credibility is not stored.**
- `Cred(O, A)` = how much observer O believes A's threats and guarantees. It is **computed** from the ledger entries that O knows about.
- The UI's "global credibility" becomes a **derived summary**: the opinion-weighted mean over observers, used only for display. The AI never reads it.

**Data: `Commitment`** (a ledger-linked record)

- `id`, `owner`, `kind` ∈ {alliance, guarantee, ultimatum, nuclear_posture, involvement}, `beneficiary`, `target?`
- `trigger` (reuses the contingency-order predicate from §4.1), `created_turn`
- `status` ∈ {active, honoured, abandoned, withdrawn_calm, lapsed}, `stake` (kind weight × cost paid)

**Test rule.** When a trigger fires, a **commitment test** opens for K = 2 turns. It resolves as:

| Result | Condition | Outcome value |
|---|---|---|
| Honoured | The owner declares war, or commits a front allocation of at least 25% of its usable pool | 1.0 |
| Partially honoured | Rung 2–4 actions only (sanctions, arms) | 0.4 |
| Abandoned | Nothing | −1.0 |

- Withdrawing a commitment while bilateral tension(beneficiary, likely attacker) > 60 counts as **abandoned × 0.5**.
- Withdrawing in calm is `withdrawn_calm`, with a negligible effect.
- Ultimatums resolve the same way: follow through if the target defies, or abandon.

**Formula**

```
Cred(O,A) = clamp( prior(O,A) + Σ_t  vis(O,t) · rel(O,t) · out(t) · w · stake(t) · 0.5^(age/20) , 0, 100)
  w = +12 if out>0, +25 if out<0           // fast to lose
  rel(O,t) = 1.0  if O was the beneficiary or target
           = 0.75 if O holds an active commitment of the same kind from A
           = 0.5  if O shares region OR alignment OR government type with the beneficiary
           = 0.25 otherwise
  stake: alliance/guarantee 1.0, posture 1.2, ultimatum 0.6, involvement 0.3×rung; ×1.5 if honoured at high cost (casualties/budget)
```

- `prior` is scenario data (a starting value per country, optionally overridden per pair).
- **Result.** A protected NATO ally believes "the US protects allies." A client in the same region as an abandoned client believes "the US drops partners." Both views are computed from the same facts. There is no hypocrisy meter: inconsistency is simply a set of differently-weighted precedents.

**Readers**
- Threaten/Ultimatum strength = perceived power × `Cred(target, threatener)`. This changes §7.3, which currently uses the stored global value.
- The AI opportunity score's third-party term = treaties × `Cred(evaluator, guarantor)` (§14.3).
- Requests for guarantees (Agent 4) read the same value.

**Feedback.** A "How the world rates your word" overlay colours each country by `Cred(O, you)`. Clicking a country shows the top 3 ledger entries driving that number ("−18: you abandoned Zaire, 1987 (same region)").

### 2.3 Opinion (§7.1): add the **third-party reaction** rule + merge "expose" into Denounce

- **Change.** When A acts on B with a ledger entry known to O, O receives a decaying opinion modifier on A.
- **Formula.** `ΔOpinion(O→A) = m(kind, magnitude) × [ α·Opinion(O→B)/100 + β·ideology(O)·affinity(O,B) ]`
  - `affinity` = +1 if O and B share government type or alignment, −1 if they are opposed, 0 otherwise.
  - `m` is signed by kind: positive for support (aid, arms, protection), negative for hostile acts (sanction, strike, war).
  - Defaults: α = 0.6, β = 0.4.
- **What this produces.** Arming a friendly authoritarian pleases its friends, annoys high-ideology democracies, and angers its rivals. The same formula covers the USSR, France or Saudi Arabia.
- **Propaganda.** **Denounce** (an existing action) may cite a Covert entry the denouncer knows about. That makes the entry Public and applies the reaction rule to every observer. This is how a rival "uses it as propaganda." No new action is added.
- **Domestic exposure.** When an entry by a democracy becomes Public, the actor's Legitimacy hit = base × (1 + govDistance(actor, beneficiary)). This extends the existing Spy-scandal effect.

### 2.4 Trust (§7.1): fed by the ledger, still per pair

- **Change.** The Trust(B, A) update now reads the commitment tests where B was the beneficiary, plus treaty-break entries.
- **Division of labour:** Trust = "will A keep its deals *with me*." Credibility = "will A act on its word, in general, as I've seen it." The two read the same ledger with different weightings, so no new store is needed.

### 2.5 Escalation ladder (§11.2) + actions → one generic **intervention scale** (not ten buttons)

- **Change.** Give every coercive or support action a `rung` attribute and a shared cost profile, instead of adding a doctrine mechanic.
- **Mapping from the brief's ladder to existing actions:**

| Brief rung | Existing action |
|---|---|
| 1 Pressure | Threaten, Denounce |
| 2 Sanctions | Sanction, Embargo |
| 3 Covert aid | Fund opposition, Arm insurgents |
| 4 Arms supply | Proxy Arms, Arms Supply treaty |
| 5 Advisors | Advisors |
| 6 Proxy forces | Volunteers |
| 7 Punitive strike | **New war aim `Punitive`** |
| 8 Limited intervention | Limited war |
| 9 Coalition war | Limited war + co-belligerents through contingency/request |
| 10 Occupation / regime change | Regime-change aim + Occupation |

- **`Punitive` war aim (data).** Air and naval only, no region transfer. The war auto-ends after 1–2 ticks. War Score comes from damage. It also absorbs **P9 counter-proliferation strike** (target = an arsenal programme, a force pool or industry in one region). One aim instead of two mechanics.
- **Entanglement.** Any action at rung ≥ 4 on behalf of B in a conflict auto-creates (or raises) a `Commitment{kind: involvement, stake: 0.3×rung}`. Stopping while B is still losing is an abandonment test. The higher you climbed, the more credibility is at stake, so "chance of entanglement" emerges.
- **Cost profile.** Initiative, budget, detection, tension, war-weariness exposure and involvement stake all scale with rung. These are already the existing per-action costs, now tabulated.
- **UI.** A per-crisis "Involvement" readout shows your current rung and the preview cost of the next rung.

### 2.6 Arms Supply / proxy Arms → transferred capability = **strength moved between pools** (yes, simply)

- **Change.** A transfer is executed as `supplier.pool[p] −= s` (or bought from production) and `recipient.pool[p] += s`. Ownership passes completely, with no supplier tag.
- **Quality.** Recipient pool quality becomes `(Q_r·S_r + min(Q_s, Q_r+2)·s)/(S_r+s)`, capped by an absorption limit of recipient tech + 2.
- **Advisors** are a temporary quality bonus that ends on withdrawal: they are people, not hardware.
- **Training** leaves +1 quality that decays over 8 turns.
- **Financed infrastructure.** Aid earmarked to a region's infrastructure (an Aid option) raises that region's Infrastructure, which persists.
- **Intelligence capability.** A one-off "network transfer" isn't needed. Allied intel sharing (§8.2) already exists and ends when relations break.
- **Blowback** follows with no event: when relations turn, the client still holds the strength. That passes Gate 4 by construction.

### 2.7 Sanction / Embargo (§7.3) → one action with a `scope` parameter + adaptation

- **Scope** ∈ {trade, commodity (the existing embargo), technology, finance}.
  - `technology` blocks research catch-up diffusion (§10.1) and Arms Supply from participants.
  - `finance` raises the target's debt service by +X. It requires the sanctioner to hold a Financial Hub or creditor trait (trait data, not US-only).
- **Adaptation.**

```
bite_t = Σ participants' share of target trade × (1 − sub_t);  sub_t += 0.1/turn
```

  - `sub_t` is capped by non-participants' spare trade capacity.
  - Each non-participant that keeps trading gains "+sanction-breaker" opinion from the target. That is substitution toward rival markets, with dependency shifting as a result.
- **Domestic losers.** The sanctioner's loss on a commodity or market it exports feeds its **Prosperity** driver with a visible modifier ("−4 farm exports lost"). The Breadbasket trait only multiplies this effect.
- **Allied cost** emerges because joining sanctions costs each joiner its own trade share. The AI's join decision reads that share (Agent 4).

### 2.8 Trade Agreement (§7.2) → add `depth` (1–3) = **economic integration treaty**

| Effect | Rule |
|---|---|
| Trade value | +trade value ∝ depth |
| Growth | +growth ∝ trade/GDP |
| Adjustment | A Prosperity modifier of −(Δtrade/GDP × k × depth), decaying over 8 turns. These are the visible "affected sectors," weighted ×1.5 for democracies |
| Sanctions between partners | Cost ×(1+depth) |
| War between partners | Loses trade value at the same multiplier |

- **Data:** `Treaty{type: trade, depth, members[]}`.
- NAFTA- or EC-like outcomes are scenario treaty instances, not hardcoded.
- **Patronage (P4)** merges into this family: patronage = recurring Aid + Arms Supply. The client's dependence = aid ÷ client revenue, which the AI can read. Cutting the aid produces a revenue shock that emerges on its own. No separate patronage treaty type.

### 2.9 Treaties → **periodic re-evaluation**

- **Change.** At each strategic reassessment, every AI re-scores its alliances, basing agreements and client relationships using the same utility it used to sign them (§7.5). If utility < −θ, it proposes downgrading or exits. Exiting is a ledger entry and costs Trust.
- **Why.** "Allies question old alliances" and "clients become less valuable" when the common threat disappears. Both need re-evaluation, not a post-Cold-War ruleset.

### 2.10 Stability (§9.1): two generic terms for the transition

- **Defence burden.** Legitimacy drifts down when `military_share > tolerance`, where `tolerance = base(gov) + k × max_threat_score/100`. When the rival collapses, threat falls, tolerance falls, and peace-dividend pressure appears without any event.
- **War of choice.**

```
war-weariness multiplier = 1 + c × (1 − threat_from_enemy/100)
```

  - `c` is a government-type default. The **Vietnam Syndrome** trait just raises `c`.
  - This makes intervention "militarily easier but harder to justify" in a unipolar world.

### 2.11 Global Tension weighting (§11.1)

- **Change.** Weight bilateral tensions by the product of the two countries' power shares, not by a "great power" list.
- **Why.** When the USSR shrinks, its weight shrinks automatically. This is era-safe for a WW1 scenario.

---

## 3. Truly new mechanics required

### N1. Event Ledger + Commitment records (v0.1 CORE)

Specified in 2.1–2.2. Listed here because it is a new data structure, even though it *replaces* §14.6 memory and the stored Credibility value.

1. **Decision.** Every commitment, and every honour or abandon choice, now has a visible future price that depends on *who is watching*.
2. **Cost.** No new currency. It costs Initiative and Budget to honour, and Credibility or Trust with the relevant observers to abandon.
3. **Interacts with.** Opinion, Trust, Credibility, Threat, Deterrence, Intel (visibility) and Stability (exposure).
4. **AI.** Reads `Cred(self, X)` in deterrence and opportunity calculations, and reads its own exposure when choosing whether to honour (a loyalty-weighted cost of abandoning).
5. **Player.** A credibility overlay with the "top 3 precedents" for each observer, and commitment tests as briefing items ("Test of your guarantee to Thailand: honour within 2 turns").
6. **Existing mechanic?** It *is* the merge of AI memory, Trust and Credibility into one store. Nothing smaller works, because per-observer views need shared facts.
7. **v0.1?** Yes. Gates 1, 2 and 4 depend on it.

### N2. Public Stances: declared principles as commitments with general triggers (DEFERRED to v0.2; designed now)

`Commitment{kind: stance, trigger ∈ {cross-border aggression, arsenal programme, chokepoint closure}, beneficiary: any}`, declared for 1 Initiative. The stance is tested whenever any country trips its trigger. Honoured = a rung ≥ 2 action against the violator within K turns. This is how "similar cases treated differently" is modelled.

1. **Decision.** Claim a principle for deterrence across the board, at the risk of being tested where you don't care.
2. **Cost.** 1 Initiative, plus repeated small tests.
3. **Interacts with.** Ledger, Credibility and Opinion (victims weigh tests heavily).
4. **AI.** Opportunists probe stance holders where the holder's interests are low.
5. **Player.** Stance tests in the briefing, plus an "enforced / ignored" history.
6. **Existing mechanic?** Mostly guarantees plus the reaction rule. Ship v0.1 without stances and add them only if Gate 2 shows selective enforcement isn't legible enough.
7. **v0.1?** No.

### N3. Supplier dependency, the "spare parts" lever (DEFERRED)

- **Data:** `pool.supplied_share[supplier]`. If the supplier embargoes the recipient, the recipient's readiness cap falls by `share × 30%`, decaying as the share is replaced through own production.
- **What it does.** Creates the counter-lever to blowback: arms both empower a client and bind it.
- **Test.**
  1. Decision: whether to buy a capability or build your own.
  2. Cost: none; it is a constraint, not a spend.
  3. Interacts with: sanctions.
  4. AI: values supplier diversity.
  5. Player: a readiness modifier with its cause shown.
  6. Existing mechanic? It overlaps with the technology-scope sanction.
  7. v0.1? No.
- **Defer reason.** Gate 4 passes without it.

**That is the full list.** Everything else in the brief is covered by sections 1–2.

---

## 4. Mechanics explicitly rejected

**Rejected outright**

| Mechanic | Reason |
|---|---|
| **Hypocrisy / Morality / Goodness meter** | The brief forbids it. Observer-weighted precedents (N1 + 2.3) produce the same consequences without a designer verdict. |
| **Stored global Credibility scalar** | Contradicts "no single global truth." Replaced by `Cred(O, A)`, with the global number kept only as a display average. |
| **Ten-button intervention doctrine / doctrine tree** | The rung attribute over existing actions plus one `Punitive` war aim covers all ten levels. |
| **Separate "Influence" or "Commitment" currency** | DESIGN §18.2 already cut it. Initiative and the ledger cover it. |
| **Scripted blowback events** | Blowback emerges from persistent pool transfers (2.6). |
| **Supplier-ownership tags on transferred weapons** | Ownership passes completely. Tags add state without adding decisions (N3 is the only justified residue, and it is deferred). |
| **Separate Patronage treaty (P4)** | Recurring Aid + Arms Supply with derived dependence does the same job. |
| **Separate Export-Control and Financial-Sanction actions** | They become `scope` values on Sanction. |
| **Separate "Expose / Propaganda" action** | Merged into Denounce citing a known covert entry. |
| **"Cold War Victory" flag or a post-Cold-War ruleset** | The transition emerges from threat recomputation, treaty re-evaluation (2.9), defence-burden tolerance and war of choice (2.10), and tension re-weighting (2.11). |
| **Peace-dividend as a mechanic** | Replaced by the defence-burden rule. It can stay as a flavour event that reads state. |
| **Sectoral or industry-level domestic economy** | The Adjustment Prosperity modifier on integration treaties and sanctions is enough to make winners and losers visible. |
| **Separate Entanglement meter** | It falls out of involvement commitments (2.5). |
| **Separate Alignment-drift meter for minors** | Opinion + Trust + Credibility already do this job. P1's "leaning bar" should *display* Opinion, not store a new value. |

**Not rejected, deferred past v0.1**
- **P7 World Interest Rate.** A new global market variable and a standing setting. Revisit only if Debt proves to be a real decision (it is on probation in §18.4).
- **P3 managed Transition** and **P2 Secession.** AI regime collapse covers Gate 5.
- **P8 Outposts.** Basing treaties cover reach.
- **N2 Public Stances.**
- **N3 Supplier dependency.**
