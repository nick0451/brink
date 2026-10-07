# Agent 3: Generalization / Anti-Special-Case Review

**Verdict.** The three phase-1 outputs are already mostly identity-free, so I'm not rejecting anything outright. The real problems are of two kinds:
- **Duplication.** The agents propose three escalation ladders, two credibility formulas, two ledger record types and two sanction-erosion rules.
- **US-shaped residue.** Some proposals carry US assumptions, including Reserve Currency "sets the world rate", "Expeditionary Democracy", Exposed Sectors, Global Commitments, SDI, and the "democracy" booleans in exposure rules.

Every item below is phrased as `(actor P, observer E, target T)`. The test for each primitive is: does it work unchanged for the USSR, China, France, Saudi Arabia, Iran, Britain in 1914 and Germany in 1938? Where the answer was no, I rewrote it.

---

## 1. Generic engine primitives

### G1. Event Ledger (one store)
**Definition.** One append-only, sim-owned log of acts. Each observer reads it through its own fog.

**Unifies:**
- A1 `LedgerEntry` and A4 `CommitmentRecord`
- DESIGN §14.6 memory
- the seed of A2's "historical memory"

**Unified schema:**
- `id, turn, actor, kind, target, beneficiary, level (G5), magnitude, cost_paid (0–1)`
- `visibility ∈ {Public, Covert, Exposed}`, `seen_by` bitmask
- `commitment_ref?`, `outcome?`
- `case_features` snapshot: value-to-actor, alignment, government type, region, rival involved, norm tag

**Visibility rule.** An observer sees a Covert entry if it is the target and detection succeeded, or if its Coverage on the actor or the target is at least `ledger_observe_coverage`. That threshold is one era-rule parameter, default 60.

**Example uses:**
- US Stingers to the mujahideen (Covert, later Exposed)
- Soviet invasion of Czechoslovakia in 1968 (a starting Public entry)
- Britain's 1914 entry for Belgium (Honoured)
- Iranian funding of Hezbollah-like proxies

### G2. Commitment and Test
**Definition.** A ledger entry that makes a promise conditional on a trigger. When the trigger fires, the simulation opens a test lasting K = 2 turns and grades the response.

**Unified kinds (A4's four semantic kinds; A1's kinds become `source`):**
- `Defend`: alliance, guarantee, basing host, involvement
- `Threat`: ultimatum, nuclear posture
- `Support`: recurring aid or arms stream
- `Norm`: a declared stance, which absorbs A1 N2 "Public Stances"

**Outcomes:** Honoured = 1, Partial = 0.5, Abandoned = 0. Lapsed and withdrawn-in-calm are neutral and are not counted.

**Grading:** A1's thresholds.
- Honoured: war, or at least 25% of the usable pool committed.
- Partial: a G5 level from 2 to 4.
- Withdrawing a commitment under tension > 60 counts as Abandoned × 0.5.

**Pledges.** A2's *conditional packages* are a Commitment written by the recipient (`Norm` kind, `source: pledge`, e.g. "freeze arsenal programme"). Breach is detected through coverage. The patron's announced response is a `Threat` commitment. Arms Control verification and Non-Aggression Pacts use the same pattern, so this is not a new mechanic.

**Example uses:**
- US guarantees to Gulf states
- Brezhnev Doctrine (a seeded bloc `Defend` + `Norm`)
- Britain's 1939 guarantee to Poland
- the French Africa garrisons
- Chinese "One China" claims tested by a foreign guarantee to Taiwan
- French loans to Russia in 1913, conditioned on strategic railways (a pledge)

### G3. Reputation queries (derived, per observer, never stored)
**Definition.** A4's weighted beta-mean over the G2 records an observer has seen: `Expect_k(E,P|ctx)`.

**A1 elements folded in:**
- `rel(O,t)` becomes salience.
- `stake × (1+cost_paid)` becomes the record weight.
- A1's "fast to lose" becomes a loss multiplier λ = 2 on Abandoned records. This is a data parameter.

**Derived values:**
- `Credibility(E,P)` = 100·ExpThreat for threats, 100·ExpDefend for guarantees.
- `Trust(E,P)` = the same query with salience restricted to records where E was beneficiary or target, plus treaty-break entries.
- `Selectivity` and `ExpAbandon`, as in A4.
- Global Credibility is only a power-weighted display mean. The AI never reads it.

**Unifies:** A1 §2.2, A1 §2.4, A4 §1.2, and A2's "global × per-observer adjustment" proposal (which this rejects).

**Rule kept from A2.** Credibility moves only through Commitment tests. An act made without a prior commitment changes Opinion and IntentEst, but not Credibility. This rule is what keeps restraint viable in D10.

**Example uses:**
- Thailand reading "the US defends allies" while Zaire reads "the US drops clients"
- Romania in 1939 reading the Anglo-French guarantee to Poland
- Syria and Egypt reading Soviet reliability differently after 1972

### G4. Observer Reaction and Exposure
**Definition.** When a ledger entry by A on B becomes visible to O, O's Opinion of A changes by A1's formula: `m(kind)·[α·Opinion(O→B) + β·ideology(O)·affinity(O,B)]`.

**Single matrix.** `affinity(O,B) = 1 − 2·IdeoDist(O,B)`, where IdeoDist is A4's matrix. A1's separate government-type/alignment test is dropped, leaving one data matrix per era.

**Exposure (generalized).** The domestic hit is `base × openness(actor) × (1 + govDistance)`. It is not "if democracy", because authoritarian elites also punish embarrassing exposures, just less.

**Denounce.** Denounce citing an entry does one of two things:
- If the entry is Covert and known to the denouncer, it becomes Public (A1).
- If the entry is already Public, its salience rises ×1.5 for N turns for chosen audiences (A4 §2.8).

That makes one action, not "Expose" plus "Propaganda".

**Example uses:**
- Iran-Contra (a US op exposed)
- the Soviet Krasnoyarsk radar exposed
- the French *Rainbow Warrior*
- the Zimmermann Telegram (a German pledge to Mexico exposed by Britain in 1917)

### G5. Involvement Scale (one ladder, as data rows)
**Definition.** One table of levels **L0–L10**: none, pressure, sanctions, covert aid, arms, advisors, proxy forces, punitive strike, limited intervention, coalition war, occupation.

**Columns:** `cost (budget, initiative, manpower), visibility, tension, commitment_stake, entanglement_hazard, effect, era_enabled`.

**How the pieces fit:**
- Every existing action carries a `level` field that points at a row. This is A1's mapping table.
- The AI's per-crisis choice (A4 §3.1) iterates over the same rows.
- DESIGN §11.2's seven tension rungs become **UI bands** derived from the rows' tension column. They are no longer a separate ladder.
- Nuclear use stays under §11.4 and is outside the scale.

**Entanglement.** An action at L ≥ 4 on behalf of B auto-creates a `Defend/involvement` commitment with `stake = row.commitment_stake` (A1). The commitment inherits the action's visibility, so a covert exit stays cheap until it is exposed (A4 §3.2). Both agents get what they wanted.

**`Punitive` war aim (L7).** Generalized from A1's version:
- any domain, not only air and naval (China's 1979 "pedagogical war" was a land war);
- no region transfer;
- auto-ends after N ticks;
- also absorbs P9 counter-proliferation strikes.

**Example uses:**
- US Libya 1986 (L7)
- USSR Afghanistan (L10)
- China–Vietnam 1979 (L7, land)
- Pershing's 1916 Punitive Expedition
- Israel at Osirak (L7)
- Germany's L6 Condor Legion in Spain

### G6. Crisis (decision container)
**Definition.** An open situation, such as a war, an aggression, an insurgency, a proliferation reveal, a chokepoint closure or a crackdown. It generates one involvement decision per interested power, and Commitment tests attach to it.

**Unifies:**
- A4's crisis
- A1's per-crisis "Involvement readout"
- A2's dilemma triggers, which become crisis templates keyed by role (aggressor, victim, patron, conduit), never by country

**Example uses:**
- the Afghan occupation
- an Iran–Iraq-like war
- the 1914 July Crisis
- the 1938 Sudeten crisis
- a 2025 Taiwan Strait crisis

### G7. Persistent Transferred Capability
**Definition.** Arms transfers move pool strength and blended quality permanently. Quality is capped at the recipient's tech + 2. There are no supplier tags, and there is no reclaim (A1 §2.6, A2, A4 test 14).
- Advisors give a temporary quality bonus.
- Training gives a decaying bonus.
- Earmarked infrastructure aid persists.
- Supplier dependency (A1 N3, "spare parts") is deferred, but it fits this schema.

**Example uses:**
- US arms to the Shah, kept by the revolutionary Iran
- French Exocets used by Argentina in 1982
- Soviet arms to Iraq and Somalia (Somalia later switched sides)
- Lend-Lease to the USSR

### G8. Trade Cut with Substitution
**Definition.** One shared function for any interruption of a trade flow:

```
bite_t = Σ blocked share × (1 − sub_t);  sub_t += 0.1/turn, capped by non-participants' spare capacity
```

Non-participants who keep trading gain sanction-breaker Opinion from the target.

**Unifies:**
- A1 §2.7 adaptation
- A2's "Sanction substitution/fatigue" (the same rule, so one copy)
- A4 §3.3 leverage term

**Callers:**
- **Sanction**, with a `scope` field:
  - `trade`
  - `commodity`: today's Embargo, including the "to everyone" variant
  - `technology_arms`: merges A1's technology scope with A2's "arms embargo on all parties"
  - `finance`: strength ∝ the sanctioner's `financial_weight`, see G12
- **Chokepoint Contested/Closed** (P6)
- **naval blockade** through sea control

**Secondary sanctions** are just a Sanction on a third party (A2).

**Example uses:**
- the US grain embargo, with Argentina substituting
- the 1973 OAPEC oil embargo
- the British blockade of Germany in 1914–18 and the U-boat counter-blockade
- the US oil embargo on Japan in 1941
- Soviet energy cutoffs to Yugoslavia in 1948

### G9. Domestic Trade Exposure
**Definition.** Any change in a country's commodity-export or partner-trade flows (from G8, from G10, or from war) produces a visible, decaying Prosperity modifier. It is proportional to Δflow/GDP, multiplied by the country's `exposure_weights` (data: `food_exports`, `energy_exports`, `materials_exports`, `import_competition`). It is then amplified by the generic **election-proximity multiplier** for government types with elections.

**Unifies:**
- A2's Exposed Sectors (with no sector model, which saves state)
- A1's adjustment modifier
- A1's "Breadbasket multiplies"

**Example uses:**
- US farmers under the grain embargo
- French farmers at GATT
- the Soviet oil budget under a 1986 price crash
- Germany's 1879 "iron and rye" tariff coalition
- Argentine beef under the 1932 Imperial Preference

### G10. Trade Agreement `depth` 1–3 (integration treaty)
**Definition.** A1 §2.8 unchanged.

**Effects scaling with depth:**
- trade value and growth
- the G9 adjustment shock (import competition)
- sanction and war cost between partners × (1 + depth)
- the weight of Debt-Default contagion along the link (A2's "financial contagion link")
- exit = a `treaty_broken` ledger entry (A2's "costly exit")

**Example uses:**
- NAFTA-like instances
- the EC
- Comecon (depth 2, managed)
- the Zollverein
- Ottawa Imperial Preference
- the Co-Prosperity Sphere (coerced depth)

### G11. Recurring Support stream (patronage)
**Definition.** Aid or Arms Supply with a `recurring` flag. The client's dependence `Dep = stream ÷ client revenue` is derived. Cutting the stream while the client is in need writes a `Support` test (A4). There is no separate P4 treaty, which follows A1's rejection.

**Example uses:**
- USSR to Cuba
- US to Egypt
- Saudi Arabia to Iraq in the 1980s
- France to Francophone Africa
- British subsidies and loans to Russia in 1914–17

### G12. Financial weight and the world rate (generalizes Reserve Currency, P7)
**Definition.** Every country has a data parameter `financial_weight ∈ [0,1]`, with weights summing to at most 1. It does three jobs:
1. **Credit discount.** It lowers the country's own borrowing cost.
2. **Monetary influence.** The World Rate = era base + tension term + Σ (`financial_weight` × that country's monetary stance). Any country with weight above about 0.1 gets the Tight / Neutral / Loose setting.
3. **Finance-scope sanction potency** (G8).

This replaces A1's "requires the Financial Hub or creditor trait" gate. Nobody "sets" the rate; heavy weights dominate it. P7 is still deferred, but the parameter is designed now so it stays generic.

**Example weights:**

| Scenario | Weights |
|---|---|
| 1980 | US 0.55, West Germany 0.15, Japan 0.1 |
| 1914 | UK 0.5, France 0.2, Germany 0.15 |
| 1936 | US 0.35, UK 0.3, France (gold bloc) 0.15 |
| 2025 | US 0.5, EU 0.2, China 0.1 |

### G13. War-of-choice weariness and the rally term
**Definition.** A1's multiplier, with a core-attack override:

```
war-weariness multiplier = 1 + c·(1 − threat_from_enemy/100);  threat term forced to 100 if own core attacked
```

- `c` = government-type default × country override.
- A generic **rally** applies to every government type: a quick victory gives +Legitimacy scaled by openness, and the weariness memory decays.
- Optional generic drift: each own long war of choice ending in Partial or Abandoned raises `c`, which then decays toward the default. With this, "Vietnam Syndrome" can emerge from play instead of being imposed.

**Unifies:** A2's "Expeditionary Democracy", scenario B.1 "Vietnam Syndrome", and A1 §2.10.

**Example uses:**
- the US after Vietnam
- the USSR's "Afghan syndrome"
- France after Algeria
- Britain's interwar "never again"
- Russia in 1916–17

### G14. Defence-burden tolerance
**Definition.** A1 §2.10. Legitimacy drifts down when military share exceeds `base(gov) + k·max_threat`. Peace-dividend pressure emerges from it.

**Example uses:**
- the US and Russia in the 1990s
- Britain's 1919 Ten-Year Rule
- interwar France

### G15. Treaty re-evaluation with hysteresis
**Definition.** At each strategic reassessment, every AI re-scores each treaty using A4 §2.6's utility. It exits only if U > θ_exit for two consecutive assessments.

**Unifies:** A1 §2.9 and A4 §2.6.

**Requests.** A2's "allies ask for help" is a generic AI proposal that can carry any action. Declining:
- writes a test **only if** a Commitment already exists;
- otherwise costs a small Opinion modifier.

**Example uses:**
- Italy leaving the Triple Alliance in 1915
- Egypt expelling Soviet advisors in 1972
- France leaving NATO's command structure in 1966
- Iran in 1979

### G16. Foreign-presence cost
**Definition.** Hosting a foreign force drains the host's Legitimacy by `AssocCost = IdeoDist × openness × presence`. This is A4 §2.2's sovereignty term made into a Stability driver.

**Unifies:** A2's deferred "host resentment" and Saudi Arabia's Custodian downside. With this primitive, the Custodian trait just multiplies the drain.

**Example uses:**
- Saudi Arabia in 1990
- the Philippines bases in 1991
- Soviet troops in Egypt
- British troops in Egypt in the 1930s–50s

### G17. Project observer alarm
**Definition.** A National Project field `observer_alarm`. When the project starts, every observer whose Threat score toward the owner exceeds X gets a paranoia/threat spike.

**Example uses:**
- SDI
- the Tirpitz naval laws
- Sputnik
- a 2025 carrier or hypersonic programme

**Already generic (keep as-is):** Initiative, Budget/Debt, Coverage and estimates, AI regime collapse, P11 personality drift, and Global Tension weighted by power share (A1 §2.11).

---

## 2. US-specific traits that remain justified (3, and a 4th slot left empty on purpose)

| Trait | Data parameters on generic primitives | Decision it changes |
|---|---|---|
| **Reserve Currency** | `financial_weight` 0.55 (G12): credit discount, dominant pull on the rate, strongest finance sanctions | Tighten money to fight inflation at home, or protect debtor clients? Use the finance weapon now, or save it? |
| **War-of-Choice Aversion** (flavour name: Vietnam Syndrome) | `c` = 1.0 against a democracy default of 0.5 (G13). Starting drift memory seeded from a Vietnam ledger entry. Rally scale unchanged | Short, limited-aim wars, covert levels, or nothing |
| **Breadbasket** | Food-surplus endowment, plus `exposure_weights.food_exports = 2.0` (G9) | Grain as a weapon, or grain as a constituency |

**What A2 proposed that is not a trait:**
- **Global Commitments** is not a trait. It is starting treaties, basing and outposts (§3). "Rivals probe the weakest commitment" is already A4's generic Probe rule.
- **Expeditionary Democracy's "cheap credit"** duplicates the democracy credit rule (§2.2), so remove it.
- **Reserve Currency's "repeated sanctions lose force"** is G8 applied to everyone, so it is not a trait downside.
- **SDI** is a project instance (G17), not a trait.

**The empty 4th slot.** It stays empty because US "reach" already emerges from geography, basing, a Maritime doctrine and Coverage. The same is true of "oceans keep the core safe": that feeds G13's threat term with no trait.

**The same primitives cover non-US traits:**
- **Brezhnev Doctrine** = a seeded bloc `Defend/Norm` commitment, plus `commitment_legitimacy_coupling` (Legitimacy moves with that commitment's test outcomes).
- **Petro-Empire** = `exposure_weights.energy_exports` plus G11 streams.
- **Custodian** = a G16 multiplier.

---

## 3. Scenario data (starting values, treaties, events, relationships)

**Priors and matrices:**
- G3 priors `(α, β)` per government type as an era rule, overridable per country and per pair.
- The IdeoDist/alignment matrix per era: Cold War blocs; WW2's fascist / communist / democratic split; 1914 monarchy and alliance blocs.

**Starting ledger (historical memory as data):**

| Scenario | Seeded entries |
|---|---|
| 1980 | South Vietnam 1975 (US `Defend` Abandoned, Public); Hungary 1956 and Czechoslovakia 1968 (USSR bloc `Defend/Norm` Honoured); Suez 1956 (UK/France); Iranian grievance from the 1953 coup |
| 1938 | Rhineland and Abyssinia (League `Norm` Abandoned) |

**Starting commitments and treaties:**
- NATO, the Warsaw Pact, US–Japan, US–South Korea, ANZUS
- the Brezhnev Doctrine
- G11 streams: US→Egypt and Israel; USSR→Cuba, Vietnam and Syria
- G10 instances: the EC and Comecon
- NAFTA as a proposable treaty instance through a crisis template, never forced

**Country parameters:**
- `financial_weight` per country
- `c` values
- `exposure_weights`
- `openness`
- `election_interval`, with an era rule that suspends elections in total war (Britain 1910–18)
- personality vectors
- Coverage sharing

**Era switches and rows:**
- G5 level rows and `era_enabled` (e.g. no satellite-dependent effects in WW1)
- the `ledger_observe_coverage` threshold
- the λ loss multiplier and the recency half-life H = 20

**Outposts** (D.5, if P8 survives) and basing at start.

**Active situations at start:**
- the hostage crisis
- the US grain embargo, as an active G8 instance with its G9 modifier
- the Afghan occupation, as an open G6 Crisis
- Iranian arms-embargo entries

**Projects and templates:**
- SDI as a project instance with `observer_alarm`
- A2's dilemmas D1–D10 as **role-keyed crisis templates and event triggers** (conduit, seizure, integration offer, post-victory uprising)

---

## 4. Historical flavor only (names and text, no mechanics)

- **Labels:** "Networked Hegemon" and the hook line; "Vietnam Syndrome"; "Reagan Doctrine"; "Brezhnev Doctrine" (as a label on a commitment); "Star Wars/SDI"; "Iran-Contra" (a scandal headline on an exposed op); "Desert Storm"; "no-fly zone" (a label for a sustained L2+Basing posture); "Peace Dividend" (headline text; the mechanic is G14); "NAFTA"; "Pressler"-style headlines on a pledge breach.
- **A2's observer slots** (Democratic Ally, Authoritarian Client, Rival, Non-aligned) are **analysis labels only**. The AI must never use them as categories. Reactions come only from G3, G4 and A4's terms.
- **Event text** for Coalition Restraint, Patron Abandonment and Thermidor.

---

## Conflicts resolved

| # | Conflict | Resolution |
|---|---|---|
| 1 | **A1 additive `Cred(O,A)` vs A4 beta-mean** | **Beta-mean (A4)**, which is bounded and has principled priors and a similarity kernel. A1's `rel` becomes salience, A1's stake becomes the weight, and A1's 25-vs-12 asymmetry becomes λ = 2 on Abandoned records. One formula serves Credibility and Trust. |
| 2 | **A1 `LedgerEntry` vs A4 `CommitmentRecord`** | One G1 entry type. A Commitment is an entry with a trigger plus test outcomes. `case_features` (A4) are added to every entry. |
| 3 | **A1 commitment kinds vs A4 kinds** | A4's four semantic kinds; A1's kinds become `source`. |
| 4 | **Partial honour score** | 0.5 (A4) instead of 0.4 (A1), for simplicity. |
| 5 | **Coverage to observe a covert entry: 60 (A1) vs 40 (A4)** | One era parameter, default 60. At 40, allied coverage sharing would make most covert acts effectively public among major powers. |
| 6 | **Three ladders (DESIGN §11.2 7 rungs; A1 `rung` 0–7 with a 1–10 mapping; A4 L0–10 rows)** | One G5 table, L0–10, as data rows. Actions carry `level`, and §11.2 becomes derived UI bands. A1's `LedgerEntry.rung` becomes `level`. Nuclear stays in §11.4. |
| 7 | **A1 entanglement at rung ≥ 4 vs A4 "covert exits cheap"** | Both. The involvement commitment inherits the action's visibility. |
| 8 | **Punitive aim limited to air/naval (A1)** | Any domain, no region transfer. |
| 9 | **A2 sanction substitution vs A1 adaptation vs A4 leverage** | One G8 function, shared with chokepoints and blockade. Scopes merge into `technology_arms`. Finance potency is read from `financial_weight`, with no trait gate. |
| 10 | **A2 conditional packages vs (nothing) in A1** | Pledge commitments (G2), the same machinery as Arms Control verification. No new mechanic. |
| 11 | **A2 Exposed Sectors vs A1 Adjustment modifier + Breadbasket** | G9 with `exposure_weights`. No sector state. The election amplifier is generic. |
| 12 | **A2 P4 patronage vs A1 rejection** | G11 recurring flag plus a `Support` test. No treaty type. |
| 13 | **Reserve Currency "sets the world rate"** | G12 weighted contributions. It is assignable to Britain in 1914 or to an EU/China mix in 2025. |
| 14 | **Expeditionary Democracy vs Vietnam Syndrome vs A1 war-of-choice** | G13. The trait is just `c`, and the rally term is generic. |
| 15 | **A1 Denounce-exposes vs A4 propaganda salience boost** | One Denounce with both effects. |
| 16 | **A1 treaty re-evaluation vs A4 abandon-bloc utility** | A4's utility with hysteresis, run on A1's schedule. |
| 17 | **A2 "Requests" vs A2's D10 rule** | Declining writes a test only if a Commitment exists. Otherwise it is an Opinion cost only. |
| 18 | **"If democracy" booleans in exposure and host-resentment rules** | Replaced by the continuous `openness` value, so authoritarian regimes take smaller but nonzero costs. |
