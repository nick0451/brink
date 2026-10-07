# The Networked Hegemon: Consolidated Mechanics Report (v0.1)

*Status: **APPROVED 2026-10-03, with changes**, and merged into DESIGN.md §6–§11, §14, §21 and §22. Answers [briefs/us-networked-hegemon-brief.md](briefs/us-networked-hegemon-brief.md).*

**The user's changes on approval:**
- **§13 decisions:**
  - Trade depth: kept.
  - Implied norms: kept, but on probation, with mandatory instrumentation (DESIGN §21.4).
  - Nuclear, fronts and national-project trims: accepted. Fronts use a `War → Fronts` data model, with one front instantiated in v0.1.
  - The v0.2 order changed to **P5 → P10 → P3 → P2**.
- **Hard invariant:** no branching on country identity.

*Where this report and DESIGN.md differ, DESIGN.md wins. §12 (Rejected) remains the authoritative do-not-re-propose list.*

*Built from six agents:*
- *Phase 1 (independent): Core Systems, US Gameplay, AI.*
- *Phase 2 (reviewing phase 1): Generalisation, Minimal Implementation, Adversarial.*

*The agents disagreed in places; resolutions are marked **⚖**. Nothing here is code. Nothing here is US-specific engine logic.*

---

## 1. Design Thesis

The United States is the most **connected** actor in the world, not the most righteous one. It has a treaty, client, base, trade tie or intelligence channel nearly everywhere, so it can usually do *something* about any crisis. Its limits are attention, money, domestic patience, and memory: every country it helps or fails to help remembers what happened. Each shortcut it takes (backing a convenient dictator, staying out of an ally's war, sanctioning a partner's customer) writes a record that observers weigh against their *own* situation. One ally concludes "they protect friends like us." Another concludes "they drop clients like us once we stop mattering." Both are right about the cases they saw.

The game never judges. It records, and every other country draws its own conclusions. Ruthless, restrained, isolationist, interventionist and trade-focused play are all viable. None is free, and none is automatically optimal.

---

## 2. Existing Mechanics Reused

**Most of the US experience already exists in DESIGN.md.** The table shows how the brief's six candidate mechanics map onto what's there.

| Brief's candidate mechanic | Built from (existing DESIGN.md) | Extension needed |
|---|---|---|
| **1. Strategic Expediency** | Aid, Arms Supply (§7.2–7.3), proxy support (§11.5), intel ops and exposure (§8.4), Opinion modifiers (§7.1), Legitimacy (§9) | Third-party reaction rule (§3, N3); exposure scaled by `openness` |
| **2. Selective Enforcement** | Guarantees and alliances as contingency orders (§4.1), Threaten/Ultimatum (§7.3), AI memory (§14.6) | **Event Ledger** (N1) replaces private memory and global Credibility |
| **3. Economic Coercion** | Sanction, Embargo (§7.3), interdependence loop (§3.3), World Market (§2.4), Prosperity driver (§9) | Substitution/adaptation (N5); visible lost-trade Prosperity modifier |
| **4. Intervention Doctrine** | Escalation ladder (§11.2), proxy (§11.5), war aims (§6.6), occupation and resistance (§6.7), mobilization (§6.2) | Five **Involvement Bands** grouping existing actions (N4), plus one new war aim (`Punitive`) |
| **5. Blowback** | Arms Supply transfers strength (§7.2) | Explicit rule: **transfers are permanent** (N6). No event |
| **6. Credibility without a hypocrisy meter** | Opinion, Trust, Credibility, Alignment, Legitimacy | Credibility becomes **derived per observer** from the ledger (N2) |

**What already constrains the US with no changes:**
- **Initiative** (3–5 actions a turn) means the US can act anywhere but not everywhere. Choosing which crises to engage *is* selective enforcement.
- **Budget, Debt and democratic Stability weights** create the domestic pressure.
- **War weariness and occupation resistance** already make "winning the operation was the cheap part" emerge.
- **Tension** is already a shared cost that alarms everyone.

---

## 3. New Generic Mechanics (survived all three reviews)

Every item below passes the generalisation test: the USSR, China, France, Saudi Arabia, Iran and a WW1/WW2 scenario can all use it. Each one is needed to pass at least one implementation gate.

### N1. Event Ledger (the one new data structure)

A single append-only log, owned by the simulation. It replaces the private per-AI memory in DESIGN §14.6.

| Field | Meaning |
|---|---|
| `actor`, `beneficiary_or_target` | Who acted, for or against whom |
| `kind` | **Back** (defence, guarantee, support stream, involvement), **Threat** (ultimatum, red line), **Norm** (an implied principle; see N2), **Coercion** (sanction, punitive strike, intervention against someone) |
| `outcome` | Honoured / Partial / Abandoned / Lapsed (neutral) / Pending |
| `cost_paid` | 0–1. Costly honouring counts for more |
| `case` | Beneficiary region, alignment, government type, whether the actor's rival was involved, norm tag |
| `visibility` | Public / Covert / Exposed, plus a 16-bit `seen_by` mask |
| `turn` | Used for recency |

- **Fog:** an observer reads an entry only if it's Public or Exposed, or if it had coverage ≥ 60 of the actor that turn. ⚖ *Agent 4 proposed 40, but at 40 allied intel-sharing made most covert acts effectively public. The value is a per-era data parameter.*
- **Size:** about 40 bytes per entry, a few hundred entries per campaign. Old entries fold into the priors.

### N2. Commitment tests and grading

The simulation opens a **test** when a commitment is called on:
- an ally or guaranteed state is attacked
- a threat deadline passes
- a support stream is cut while the client is in need
- a norm the actor holds is violated by anyone

**Grading** ⚖ *(adopts the adversarial fix F1; replaces Agent 1's "25% of usable pool" rule, which superpowers couldn't meet)*:

| Outcome | Condition |
|---|---|
| **Honoured** | Forces committed to the front of at least 0.5× the attacker's strength there, **or** the beneficiary keeps all its core regions |
| **Partial** (scores 0.5) | A declaration without forces, a Punitive strike only, or sanctions only |
| **Abandoned** | No response, or withdrawal while the beneficiary is losing |

**Rules that close exploits:**
- **Shadow commitments** (F3). A commitment withdrawn or lapsed stays in shadow for 8 turns. If the beneficiary is attacked during that time, the test is graded Abandoned ×0.75. A calm withdrawal costs reputation only with the former beneficiary.
- **Implied norms** (F8). Any action at band 3+ against an aggressor, or a public Ultimatum citing a crisis type (aggression, proliferation, chokepoint closure), writes a **Norm** claim for the actor. Later violations of that norm by *anyone* open a test against that actor. Selective enforcement is therefore recorded **only for those who have enforced before**. A country that never enforces has no norm to betray, but it also never gains enforcement credibility.
- **Arming both sides** of the same war (F11) writes a Partial Back entry with each belligerent and cancels involvement credit.
- **Involvement commitments.** Acting at band 2+ for a side auto-creates a Back commitment, staked by band (see N4). It **takes the visibility of the action that created it**. ⚖ *This reconciles Agent 1's entanglement rule with Agent 4's cheap covert exits: covert proxy support can be dropped quietly, while public intervention locks you in.*

### N2b. Per-observer reputation read (replaces global Credibility)

⚖ **One formula.** It is a simplified form of Agent 4's weighted average (Agent 5's v0.1 version). Agent 1's additive formula is rejected: it could be farmed to 100, and it let partial responses *raise* credibility.

```
Exp_k(E,P) = (n0·prior_k(P) + Σ w_i·s_i) / (n0 + Σ w_i)        k ∈ {Back, Threat, Norm}
  s_i  = 1 honoured · 0.5 partial · 0 abandoned        (entries E can see)
  w_i  = 0.5^(age/20) × rel(E,i) × (1 + cost_paid_i) × (abandoned ? 2 : 1)
  rel  = 1.0 if E was beneficiary/target · 0.5 if E shares region OR alignment with the beneficiary · 0.2 otherwise
  n0   = 3; priors from scenario data (and seeded historical ledger entries)
```

| Derived value | Definition |
|---|---|
| **Credibility(E,P)** | 100 × Exp_Back (for "will they defend me") or Exp_Threat (for "will they follow through"), depending on the decision |
| **Trust(E,P)** | The same query with rel limited to entries where E was beneficiary or target |
| **Global Credibility** | Display-only power-weighted average. **The AI never reads it** |
| **Deterrence value of a guarantee** (F4) | The *potential attacker's* Credibility read about the guarantor, scaled by the guarantor's available force. No longer a single global number |

**Observer divergence emerges by itself.** A US that defends a Gulf client and abandons an African one is read differently in each region.

### N3. Third-party reaction rule

When P acts toward B, every observer O who sees it adjusts its Opinion of P:

`ΔOpinion(O→P) = magnitude × [0.6·Opinion(O→B)/100 + 0.4·ideology(O)·affinity(O,B)]`

- Backing an authoritarian ally against a common rival pleases its friends and alarms its enemies, and democracies with high ideology dislike it.
- **Exposure** of covert acts costs domestic Legitimacy scaled by the actor's **`openness`**. This is a 0–1 per-country value, defaulting from government type. ⚖ *It replaces every "if democracy" switch (Agents 3 and 6).*

### N4. Involvement Bands (the intervention ladder)

⚖ **Five bands for decisions; a `rung` attribute on every action for cost and tension.** Agent 4's 11 levels collapse into these. DESIGN §11.2's seven rungs become display labels.

| Band | Concrete existing actions | Involvement stake | Typical cost profile |
|---|---|---|---|
| **0 None** | (optionally Mediate) | 0 | Cheapest now; may write an Abandoned entry if a commitment existed |
| **1 Coerce** | Threaten, Ultimatum, Sanction, Embargo | 0.3 (an ultimatum creates a Threat test) | Money, trade, allied friction |
| **2 Proxy** | Arms, advisors, volunteers; covert aid; fund insurgents | 0.5 (visibility follows the action) | Budget, exposure risk, persistent transfers |
| **3 Strike / Limited** | `Punitive` war aim; limited-aims war | 1.0 | Casualties, tension, an implied norm |
| **4 Major** | Coalition war; regime-change aims; occupation | 1.5 | Mobilization, war weariness, occupation resistance |

**New: `Punitive` war aim** (generic, any domain). A short operation against military or programme targets. ⚖ *Adopts F6:*
- It ends only if the target accepts peace. If not, it becomes a limited war.
- It leaves a **Coercion** entry.
- It absorbs P9 (counter-proliferation strike) as a special case.

**Proxy supply needs a route** (F10): your own adjacency, a **consenting conduit state** (one that accepted the arrangement), or sea control into the recipient. Volunteer casualties count toward the supplier's war weariness.
- This makes Pakistan-type conduit states mechanically valuable without special code.

### N5. Sanction substitution

One generic trade-cut function, shared by Sanction, Embargo and chokepoint/naval closure.
- Each turn a sanction is in force, the target reroutes +10% of the lost trade to non-participants. It caps out once all its trade has been rerouted.
- Adaptation **persists after lifting** (F7), so on/off sanction cycling erodes the weapon.
- The sanctioner's own lost exports appear as a **visible Prosperity modifier**, scaled by a per-commodity **export exposure weight** (data). This is how Breadbasket and "domestic winners and losers" work, without a sector model.

### N6. Permanent transfers (blowback with no event)

- Arms Supply moves strength from the supplier's pool to the recipient's.
- Quality is a weighted average, capped at the recipient's Military tech + 2.
- Advisors are a temporary quality bonus.
- **Transferred strength never returns**, whatever happens to relations later.
- Arms outflows count toward the supplier's **defence-burden** and arms-control limits (F13), so you can't dodge budget pressure by arming clients.

### N7. Recurring support streams (replaces scenario P4 "Patronage")

Aid or Arms Supply can be set as a **standing stream**:
- Setting it up costs 1 Initiative. Maintaining it costs 0. Cutting it while the client is in need opens a Back test.
- Dependence (stream as a share of the client's budget) falls out naturally.
- Cuba, Vietnam, Egypt and Pakistan all work this way with no special treaty type.

### N8. Domestic and systemic terms (four one-line rules)

| Term | Rule | Produces |
|---|---|---|
| **Defence-burden tolerance** | Legitimacy penalty when military spending is above what the biggest perceived threat justifies | Peace-dividend pressure appears automatically when a rival collapses |
| **War-of-choice weariness** | War weariness × `(1 + c·(1 − threat_from_enemy))`; `c` is data (democracy default 0.5) | Wars of choice hurt more than defensive wars. ⚖ *Replaces Vietnam Syndrome ×1.5 (F14: it double-counted)* |
| **Power-weighted Global Tension** | Bilateral tensions weighted by the two countries' power shares, not a fixed "great power" list | Works for every era and adapts as powers rise and fall |
| **Treaty re-evaluation** | At each strategic reassessment the AI re-scores its treaties. Exit requires the score to stay past the threshold for **2 consecutive assessments** | Alliances wobble when their purpose fades, with a visible warning window |

### N9. Proposal acceptance costs Initiative (F5)

- **Accepting** an AI proposal that creates a commitment (alliance, guarantee, basing, stream) costs **1 Initiative**.
- **Refusing** costs only Opinion, unless an *existing* commitment covers the request, in which case it's a test.
- ⚖ *This closes free alliance stacking. It also overrides Agent 2's "refusal costs Trust" idea, which made accept-everything optimal.*
- One-off Aid costs 1 Initiative, as DESIGN §7.3 says. The §17 example turn that treats it as free gets corrected.

### N10. Trade Agreement `depth` (shallow / deep)

⚖ *Included in v0.1 against Agent 5's deferral, because the brief and Gate 5 both need trade to matter after the Cold War.*
- Deep agreements multiply trade value.
- They make sanctions or war between the partners much costlier.
- They impose a temporary adjustment shock on the Prosperity of each partner's exposed commodity category, measured against **that category, not total GDP** (F12). Big economies don't get integration for free.
- **NAFTA is scenario data**: one deep agreement between three countries.

---

## 4. US Traits (three)

⚖ **Agent 2 proposed four traits; the generalisation review kept three, and every one is a data parameter on a generic primitive.** "Global Commitments" becomes starting data (treaties and bases). SDI is a national-project instance.

| Trait | Generic parameter | Upside | Downside | Decision it shapes |
|---|---|---|---|---|
| **Reserve Currency** | `financial_weight` = 0.55 (UK in 1914 ≈ 0.5) | Very cheap credit, so deficits are sustainable | Easy borrowing tempts debt-funded wars and buildups. *(The world interest-rate lever and finance sanctions are deferred; see §10)* | Pay for power now, or stay solvent |
| **War-of-Choice Aversion** (flavour name: "Vietnam Syndrome") | `c` = 1.0 (democracy default 0.5) | Defensive wars and defence of attacked allies carry no extra penalty | Long wars of choice burn Stability fast | Short limited aims, proxies, or nothing at all |
| **Breadbasket** | Food surplus + food export-exposure weight 2.0 | Food leverage over importers; opinion from food aid | Food embargoes hit US Prosperity hard | Grain as a weapon versus grain as a constituency |

**Explicitly not traits:**
- **Global reach:** comes from geography plus starting bases.
- **Intelligence superiority:** comes from Intel tech 8.
- **Financial dominance:** a parameter.

---

## 5. US Strategic Actions (what the player actually does)

All of these are generic actions available to every country. The US simply has more starting treaties, bases, money and reach to use them with.

| Category | Actions | Band |
|---|---|---|
| **Commit** | Defensive Alliance, Guarantee, Basing, Arms Supply, standing streams, accept or refuse requests | — (create tests) |
| **Coerce** | Threaten / Ultimatum, Sanction (trade or commodity scope), Embargo, Denounce | 1 |
| **Support covertly** | Expand network, Fund opposition, Arm insurgents, covert arms, Disinformation | 2 |
| **Support openly** | Arms transfers, advisors, volunteers (needs a route or conduit) | 2 |
| **Strike** | `Punitive` war aim; limited-aims war | 3 |
| **Go big** | Coalition war (allies join via contingency orders and Aid), regime-change aims, occupation | 4 |
| **De-escalate** | Mediate, Arms Control, lift sanctions, withdraw (watch the shadow-commitment rule) | — |
| **Integrate** | Trade Agreement (shallow or deep), Aid | — |
| **Standing settings** | Budget sliders, mobilization, nuclear posture, theater postures | — |

**Before committing, the player sees a consequence preview**, generated by the same calculation the AI uses:

```
Respond at Band 3 (Punitive strike) to the attack on Kuwait-region minor:
  Test grade if chosen: PARTIAL (no forces ≥ 0.5× attacker on front)
  Saudi Arabia  Credibility(Back) 71 → 69   "partial response, but a response"
  Gulf States   Credibility(Back) 64 → 61
  Iraq-like     Credibility(Threat) 40 → 52  "they will strike"
  Implied norm written: AGGRESSION (future violations by anyone open tests on you)
  Tension +12 · War weariness +low · Budget −4
```

---

## 6. AI Reasoning

### 6.1 Inputs (v0.1 set, all generic)

| Input | Definition |
|---|---|
| `Opinion` | Existing |
| `Trust`, `Credibility` | From the ledger (N2b) |
| `Threat(E,X)` | From estimates (§14.3) |
| `SharedRival(E,P)` | Overlap between E's threats and P's |
| `Dependence(E→P)` | Trade share or stream share |
| `Leverage` | Asymmetric dependence |
| `IdeoDist` | One shared government-type / alignment matrix (scenario data) |
| `Need` | Budget, stability or commodity gap |
| `PowerShare_E(P)` | Global share × a crude reach factor |
| `IntentEst(E,P)` | Rate of P's **Coercion** entries |

- **IntentEst (F9):** responses to someone else's aggression count ×0.5, so a reliable defender isn't balanced against like an aggressor.
- **Personality** vector and active **goals** complete the input set.

**No input names a country.** A lint check plus a data-swap test enforce this.

### 6.2 Decisions other countries make about a great power P

Each decision is a utility sum whose terms *are* the explanation lines shown to the player.

| Decision | Key terms |
|---|---|
| **Accept P's aid / stream** | Need, shared rival, dependence creep (× paranoia), ideology (bounded), angering current patrons |
| **Request P's guarantee** | Security gap × Credibility(Back), odds P accepts, provoking the threat, the price of strings |
| **Join P's sanctions** | Patron goodwill × dependence, own grievance against the target, own trade loss (× greed), retaliation risk, effectiveness (diminishing) |
| **Abandon P's bloc** | Lost purpose (security value falls when threat falls), grievance and coercion entries, alternative offers, dependence, loyalty. **2-assessment hysteresis** |
| **Oppose P's intervention** (none / denounce / arm the target) | P's power growth in my region × IntentEst, "could be me" precedent, affinity with the target, P's leverage over me |
| **Balance against P** | `max(0, PowerShare − 0.4) × IntentEst × (0.5 + paranoia)`, with diminishing returns per extra balancer |
| **Probe P's clients** | No new action. Low Credibility(Back) toward a client raises the existing **opportunity score** against it. Falklands- and Kuwait-style probes emerge from P's own record |

**Safeguards:**
- **Anti-dogpile:** balancing needs *intent*, not just strength. Power counts only within reach. Local threats outrank distant giants. Weak states can bandwagon. Each extra balancer adds less.
- **Anti-ideology-lock:** the ideology term is capped at 30 × ideology, while threat, need and dependence reach about 60. It is discounted under threat (`× (1 − 0.8·Threat/100)`). It carries full weight for alliances and bases, half for arms, a quarter for trade.

### 6.3 How a great power (any, including an AI-run US) decides

**Choose a band for each crisis.**

```
U(band) = Stake·ΔP(success)        (from estimates: fog can mislead)
        + RepDelta(band)           (run N2b on the hypothetical test grade, sum over observers P values)
        − Cost·(0.5+greed) − DomesticCost(war-of-choice, weariness, openness)
        − Tension·(1−aggression) − EscalationRisk·(1−risk) − Entanglement·(1−risk)
```

**Ongoing involvement** is re-scored each assessment as hold, withdraw or escalate, **looking forward only** (sunk costs are excluded). Withdrawal writes Abandoned or Partial entries weighted by the band's stake and visibility. As a result:
- covert involvement is cheap to exit
- public, high-band involvement locks you in
- peripheral clients can be dropped at local cost; core allies can't

**Economic coercion** uses a leverage term. Coercing a *friend* is allowed when leverage is high, but it writes Coercion entries that raise IntentEst among similar states.

**Variety across runs** (Gate 6) comes from:
- seeded intelligence estimates
- a path-dependent ledger
- crises competing for Initiative
- domestic state
- seeded ±0.1 personality jitter at campaign start
- softmax choice among near-tied options only

---

## 7. Post-Cold-War Transition

**No "Cold War Victory" flag and no separate ruleset.** When the primary rival collapses or fades, ordinary terms shift:

| Mechanic | What changes | What the player feels |
|---|---|---|
| Threat map (§14.3) | US top threat drops | Strategic reassessment rescores goals |
| **Defence-burden tolerance** (N8) | The same military budget now costs Legitimacy | Peace-dividend pressure, with no event |
| **Treaty re-evaluation** (N8) | Allies' "lost purpose" term rises | Allies wobble and warning windows appear. Keep them with trade, aid or a new shared purpose, or let them go |
| Clients' value | Streams to clients look expensive; cutting them opens tests | Each cut writes a record observers read: "they drop clients once the rival is gone" |
| **Power-weighted tension + balancing** | US power share rises. *Intent* decides whether others balance | A coercive US gets balanced against; a restrained one mostly doesn't |
| War-of-choice weariness | Fewer wars are defensive, so `(1 − threat)` rises | Intervention is militarily easier and domestically harder |
| **Trade depth** (N10) | Deep agreements become relatively more valuable | Integration competes with alliances as the organising tool |
| AI goal rescoring | Other powers re-pick goals | Regional powers grow independent. A rising power may become the next rival, or not |

**"What is American power for now?"** has no menu answer. Retrenchment, coalition leadership, integration, primacy and selective intervention each emerge from different slider settings, treaty choices and responses to crises. The AI-run US picks among them through the same goal rescoring. Test target: no single strategy in more than 60% of runs, and at least 3 strategies in more than 10%.

---

## 8. Example Turns

### Turn A — Spring 1981: the conduit

**State:**
- The Soviet occupation of Afghanistan is meeting resistance.
- Pakistan (Opinion of USSR −45) offers to act as a conduit. It wants a standing Arms stream.
- Its suspected arsenal programme sits at coverage 45: "progress 20–60%, low confidence."
- US Initiative: 3.

**Options:**

| Option | Cost | Effect | Second-order |
|---|---|---|---|
| **Accept the conduit + standing Arms stream to Pakistan + covert Arms to insurgents through it** | 2 Initiative, Budget −3/turn | Soviet war weariness rises. Pakistan Opinion +20 | Arms to Pakistan are **permanent** (N6). Insurgent factions keep their arms. India (Opinion of US) −18 via the third-party rule. A covert Back commitment to the insurgents (stake 0.5, covert) |
| **Covert arms only, no stream** | 1 Initiative | Smaller effect | Pakistan's request refused: Opinion −10 only (no existing commitment) |
| **Stay out, Mediate** | 1 Initiative | Tension −, India +, non-aligned + | The USSR consolidates and frees forces elsewhere |

**Player chooses** the first option and banks 1 Initiative. There's no "ignore the bomb" button. The player simply chose not to act on the programme. Nothing is recorded, because the US has never made a non-proliferation claim (no implied Norm). That changes the day the US strikes someone's programme.

### Turn B — Autumn 1990: the seizure

**State:**
- The USSR has reformed and is weak (Stability 34).
- Iraq-like actor: Debt 85%, army enlarged by earlier transfers from the USSR, France *and* US credits. It annexes Kuwait.
- Saudi Arabia requests a Guarantee plus Basing. Oil +60%.
- US Credibility(Back) as read by Saudi Arabia: 71. As read by the Gulf States: 64.

**Options:**

| Band | Choice | Immediate | Ledger and AI |
|---|---|---|---|
| 0 | Accept the fait accompli | No cost | No test, since there was no commitment to Kuwait. But Iraq's **probe** succeeded: other actors' opportunity scores against weakly backed clients rise. Saudi Credibility read drops via "similar case" relevance |
| 1 | Sanctions + embargo coalition | Allies join (their own trade loss vs goodwill) | Substitution erodes it by ~10%/turn. The USSR abstains for credits |
| 1+guarantee | Guarantee Saudi Arabia + Basing (accepting costs 1 Initiative) | Deterrence via Iraq's Credibility read of the US × force available | A new Back test is now pending on Saudi soil. Foreign-presence cost hits Saudi Legitimacy (Custodian trait) |
| 3 | `Punitive` strikes + Ultimatum | Iraq refuses peace, so it becomes a limited war | Implied **Aggression norm** written |
| 4 | Coalition war, limited aims (liberate) | High cost, 2 Initiative, partial mobilization | Honoured test with cost_paid 0.8, so every observer near the Gulf reads the US as reliable for years. Iraq survives, wounded, with a revenge goal |

**Player chooses** the guarantee now (1 Initiative), sanctions (1 Initiative), and banks 1 Initiative for next turn's escalation decision. The preview showed band 4 maximising Gulf credibility but bringing an 8-turn entanglement risk. Iraq's AI now sees a guarantee backed by six divisions, and its probe utility against Saudi Arabia turns negative.

### Turn C — Spring 1994: what is it for?

**State:**
- The USSR collapsed in 1992 into 5 successors. The US top threat is 31.
- Defence-burden tolerance is now costing −4 Legitimacy per turn at the current military spending.
- Mexico-like partner proposes a **deep** Trade Agreement. The US manufacturing export-exposure category takes a −6 adjustment shock for 6 turns.
- West Germany's re-evaluation shows "reconsidering alliance commitments (1st assessment)."
- A multi-ethnic state has fragmented with atrocities. European allies ask the US to lead. The US holds no commitment to any party.

**Options, mixable within 3 Initiative:**

| Option | Cost | Gain | Risk |
|---|---|---|---|
| **Cut military 25% → Welfare and Development** | 0 Initiative, takes effect over 3 turns | Legitimacy recovers, growth | Allies' lost-purpose term rises faster. Rising powers' opportunity scores tick up |
| **Sign the deep Trade Agreement** | 1 Initiative | Growth, deep interdependence | Manufacturing shock before an election. Partner's later crisis becomes the US's crisis |
| **Lead in the civil war** (Punitive + Ultimatum) | 1–2 Initiative | Allied Trust +, alliance purpose renewed | Writes an implied norm. Ultimatum ignored → escalate or abandon |
| **Stay out of the civil war** | 0 | No cost | **No credibility loss, because there was no commitment.** Only Opinion −10 with the asking allies |
| **Offer West Germany a deep trade deal** instead of a military rationale | 1 Initiative | Alliance re-scored on dependence and trade | Economic competition replaces a security rationale |

Any combination is defensible. The game doesn't say which is right; observers record what happened.

---

## 9. v0.1 Implementation List

**Scope of change versus DESIGN.md:**
- 1 data structure (ledger)
- 1 read function
- 1 opinion rule
- 1 war aim
- 1 trade function
- 1 standing-order type
- 4 one-line terms
- 1 treaty parameter

No new currencies. No US-specific code.

| # | Item | Unlocks gate |
|---|---|---|
| C1 | Event Ledger + visibility/fog (N1) | 1 |
| C2 | Commitment tests, grading, shadow commitments, implied norms, both-sides rule (N2) | 1, 2 |
| C3 | Per-observer reputation read; global Credibility display-only (N2b) | 1 |
| C4 | Third-party reaction + `openness` exposure (N3) | 2 |
| C5 | Five Involvement Bands + `rung` on actions + `Punitive` aim + proxy route/conduit (N4) | 3 |
| C6 | Sanction substitution + lost-trade Prosperity modifier with exposure weights (N5) | 2, 3 |
| C7 | Permanent transfers + arms outflows count toward burden (N6) | 4 |
| C8 | Standing support streams (N7) | 2, 5 |
| C9 | Defence-burden, war-of-choice, power-weighted tension, treaty re-evaluation (N8) | 5 |
| C10 | Accepting a commitment costs Initiative (N9) | 1 (exploit) |
| C11 | Trade Agreement depth (N10) | 5 |
| C12 | AI: v0.1 inputs, the 7 decisions in §6.2, band choice, hold/withdraw/escalate, softmax, safeguards, explanation lines = terms | 1–6 |
| C13 | Reasoning log with top-3 precedents per decision (the UI panel comes later) | all |

### Build order (with gates)

```
1  Skeleton: Cargo workspace, seeded RNG, RON schemas, quarterly loop, Budget/Initiative/Stability,
   economy-lite, opinion lists — 80 deterministic empty turns
2  Fog view type (AI compiles against it from day one)
3  Treaties, rung-tagged actions, tension, sanctions with substitution (C6), streams (C8), C10
4  Ledger, tests, reputation read, third-party reaction (C1–C4, C13)
5  AI framework + 4-actor fixture (USA, ally, rival, neutral)            → GATE 1, then GATE 2
6  Force pools + permanent transfers + threat inputs (C7)                → GATE 4  (cheaper than 3: do first)
7  War-lite + bands + Punitive + proxy routes (C5)                      → GATE 3
8  Reassessment, treaty re-eval, balancing, N8 terms, trade depth (C9, C11) → GATE 5 (delete-the-rival fixture)
9  Full 1980 data, minors as restricted-action AIs, ~6 event templates, batch runner → GATE 6
```

### v0.1 test suite (12 tests)

**Kept from the AI design:**
1. No identity branching (lint + data swap)
2. Observer divergence
3. Fog respect
4. Past behaviour changes decisions (Gate 1)
5. Explanation lines sum to the score
6. Blowback with no event (Gate 4)
7. Post-rival rescore (200 seeds, ≥ 3 strategies above 10%)
8. Gate 6 variety (200 runs; each behaviour in ≥ 20% of runs, none in more than 95%)
9. Determinism across thread counts

**New:**

10. **Forced-branch Pareto test** (Gates 2 and 3). Clone the world at a fixture crisis, apply each option, and run 8 turns. Every option must be best on at least one metric, and none best on all.
11. **Fake-honour exploit:** a declaration without forces grades Partial.
12. **Withdraw-before-test exploit:** an attack inside the shadow window grades Abandoned.

---

## 10. Deferred Mechanics (good, not yet)

Each item is pulled in only when headless runs show the need.

| Deferred | Pull in when… |
|---|---|
| ExpAbandon and Selectivity queries; value-to-patron relevance feature | Players can't tell why two observers disagree |
| Threat-trend term ("contain the rising power") | The Gate 5 strategy mix lacks primacy/containment |
| Declared Public Stances (explicit norm claims) | Implied norms prove unreadable |
| Finance and technology sanction scopes; `financial_weight` → world interest rate (scenario P7) | Debt and Latin-crisis plausibility targets miss |
| Conditional packages / pledges ("aid if you freeze the programme") | Proliferation play feels flat |
| Supplier dependency ("spare parts") | Arms clients feel too independent |
| Host resentment of bases (beyond the foreign-presence cost) | Basing feels free |
| Denounce exposing a covert entry; propaganda raising salience | Covert play has too little counterplay |
| Election-calendar amplifier | Elections feel irrelevant |
| Personality drift via leadership change (scenario P11) | AI behaviour is too static within a run |
| Reputation-by-observer UI panel and consequence preview UI | UI phase |
| Standing response policies / preset honour levels (anti-tedium) | Playtests show commitment tests become chores |
| Scenario candidates P2 (secession), P3 (transition), P5 (production policy), P6 (contested chokepoints), P10 (covert programmes) | **See the open decision in §13: several playable countries' identities depend on these** |

---

## 11. Scenario-Layer Content (data, not engine)

**Countries and starting state**
- US traits as parameters (§4); SDI as a project instance.
- Starting treaties, bases, standing streams, `openness`, `c`, exposure weights, priors.
- **Seeded historical ledger entries** for 1980:
  - South Vietnam 1975
  - Hungary 1956, Czechoslovakia 1968
  - Suez 1956
  - Iran 1953
  - Camp David

**Era data**
- The ideology/alignment distance matrix, defined per era.
- The coverage threshold for seeing covert acts, per era.
- NAFTA-like agreements as deep Trade Agreement instances.

**Situations and events**
- Active situations (the hostage crisis as a standing Legitimacy drain).
- About 6 event templates for v0.1.
- Agent 2's ten dilemmas become **crisis fixtures** for tests and examples, keyed by role (aggressor, victim, patron, conduit), never by country.

**Flavour only:** names such as Reagan Doctrine, Iran-Contra, Desert Storm, Peace Dividend, Vietnam Syndrome, NAFTA, no-fly zone.

---

## 12. Rejected Mechanics (do not re-propose)

| Rejected | Why |
|---|---|
| Hypocrisy / Morality / Goodness meter | Reputation emerges from the ledger per observer |
| Stored global Credibility value | The brief requires no single truth. Display-only average instead |
| Agent 1's additive credibility formula | Farmable to 100; partial responses raised credibility (E2) |
| Ten-button intervention doctrine / 11-level AI ladder | Five bands over existing actions do the same job |
| Scripted blowback events | Permanent transfers produce blowback |
| Separate Patronage treaty (scenario P4) | Standing streams |
| Export-control, financial-sanction and "expose" as separate actions | One Sanction with scopes; Denounce |
| Sector-level domestic economy | Exposure weights on commodity categories |
| Entanglement meter | Involvement commitments plus shadow tests |
| Influence / commitment currency | Initiative already prices attention |
| Cold War Victory flag; post-Cold-War ruleset; Peace Dividend as a mechanic | Ordinary terms shift on their own |
| "Global Commitments" and "Expeditionary Democracy" as traits | Starting data, and a duplicate of the democracy credit rule |
| Vietnam Syndrome ×1.5 multiplier | Double-counted war-of-choice weariness |
| "If democracy" exposure switches | `openness` scalar |
| Refusing requests costs Trust | Made accepting everything optimal |
| Training as a separate bonus from advisors | Duplicate |
| AssocCost as a separate AI input | Merged into the bounded ideology term |
| Separate minor-country rules engine (scenario P1) | Minors run the same AI with a restricted action set |
| Ownership tags on transferred weapons | Nothing reads them |
| Using Agent 2's observer slots (DA/AC/R/NA) as AI categories | Analysis labels only |

---

## 13. Decisions Needed From You

1. **Approve this report** (or mark changes). Once approved, I'll merge it into DESIGN.md and update the scenario document, including the US archetype rename.
2. **Two of my calls that overrode the minimal-implementation reviewer:**
   - (a) **Trade depth** stays in v0.1, because the brief and Gate 5 need post-Cold-War trade.
   - (b) **Implied norms** stay in v0.1, because without them "similar cases treated differently" isn't modelled and the adversarial review found it exploitable.

   Keep both, or defer them?
3. **Foundation trims** proposed by the minimal-implementation reviewer, touching things you already confirmed:
   - (a) Nuclear in v0.1 becomes the arsenal as a deterrence input plus one "use" action carrying the full penalty package. The three strike scales and the Nuclear Winter meter come later.
   - (b) One front per pair of belligerents, not per border segment.
   - (c) National projects limited to a few data instances.

   Accept, or keep the full versions?
4. **Scenario identity gap.** Deferring P5 (production policy), P10 (covert programmes), P2 (secession) and P3 (transition) keeps the engine small. It also weakens the signature play of **Saudi Arabia, Israel, South Africa, Pakistan, North Korea, India, Yugoslavia and the USSR**.
   - My recommendation: treat v0.1 as the *engine* milestone (Gates 1–6), then a **v0.2 "scenario completeness" milestone** that adds P5 and P10 first, then P2 and P3, before the first human playtest. OK?
