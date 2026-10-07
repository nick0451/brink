# Agent 6: Adversarial Design Reviewer

**Scope.** I attacked Agent 1 (core systems), Agent 2 (US gameplay) and Agent 4 (AI) as a min-maxing player and as a skeptical designer. I checked them against DESIGN.md as it stands.

**Headline.** The ledger concept holds up. Its grading rules don't yet.

- Three rules combine into a **dominant "cheap commitment" US strategy**:
  - declaring war counts as honouring a commitment;
  - Partial responses (sanctions, arms) *raise* credibility;
  - withdrawing a commitment in calm costs almost nothing.
- Commitments can be **stacked at zero Initiative** by accepting AI proposals.
- Deterrence is read from **generic** credibility, while abandonment is priced **locally**.

Most fixes below are grading or tuning changes, not features.

**One structural conflict must be settled first.** Agent 1 and Agent 4 propose two different credibility formulas:
- **Agent 1:** an additive sum, clamped to 0–100, with outcome values +1 / +0.4 / −1.
- **Agent 4:** a weighted beta-mean.

Several exploits below exist only under the additive form. It can be farmed up to 100, and its Partial result counts as a gain.

---

## 1. Exploits found

### E1. Phony honour (critical)
- **Pattern.** Hold many guarantees. When one is tested, the contingency order auto-declares war, or I declare a `Punitive` war. I allocate nothing to the front. White peace follows.
- **Why it works.**
  - Agent 1 grades "declares war **or** commits ≥25% of usable pool" as Honoured (1.0).
  - `Punitive` auto-ends after 1–2 ticks, so a two-tick airstrike earns full honour.
  - The 25% threshold is also scaled wrongly. For a superpower it's absurdly high, so nobody will ever meet it, and declaring war becomes the only path anyone uses.
- **Severity.** Critical. Every guarantee becomes free deterrence, which collapses the whole commitment economy.

### E2. Partial counts as a win (critical, Agent 1 formula)
- **Pattern.** Answer every invasion of a guaranteed state with a sanction.
- **Why it works.** Under Agent 1, `out = +0.4 > 0` takes the gain weight `w = +12`. Sanctioning an invader of your ally therefore *raises* `Cred` with every observer. Abandonment is never chosen. Partial is always chosen, at 1 Initiative.
- **Severity.** Critical.

### E3. Lapse laundering / timing the exit (high)
- **Pattern.** Watch bilateral tension between my client and its likely attacker. At 55, withdraw the guarantee (`withdrawn_calm`, negligible). Re-guarantee after the crisis passes. Or, under Agent 4, end it by "mutual treaty" (`Lapsed`, neutral) with a dependent client the AI can't afford to refuse.
- **Why it works.**
  - The tension threshold is public and lags intent. AI attack decisions come from Opportunity scores, not from tension reaching 60.
  - Nothing looks back after the withdrawal: the attack that follows is not linked to it.
- **Severity.** High. This is the "let guarantees lapse before tests" exploit, and it works today.

### E4. Free alliance stacking via acceptance (high)
- **Pattern.** Accept every guarantee, basing, patronage and arms-supply request that AIs send (DESIGN §2.3: accepting costs 0 Initiative). Spend real Initiative elsewhere.
- **Why it works.**
  - Initiative limits the *flow* of actions the player starts, not the *stock* of commitments.
  - Agent 2 adds a Trust cost for *refusing* requests. Accepting is then free and refusing is taxed, so the gradient points to "accept everything".
  - Combined with E1 and E3, the stack never has to be paid for.
- **Severity.** High. This is the "infinite alliance stacking" case.

### E5. Generic deterrence, local abandonment cost (high)
- **Pattern.** Guarantee many peripheral, dissimilar states. They deter attackers at my *general* credibility. When one is tested, abandon it.
- **Why it works.** The two sides of the same commitment are read through different functions:

| Side | What it reads | Effect |
|---|---|---|
| Deterrence (Agent 1's Opportunity term) | `treaties × Cred(evaluator, guarantor)` | Generic credibility applies |
| Abandonment cost (Agent 4's similarity kernel) | Similarity-weighted precedents | Abandoning a peripheral, dissimilar client costs little |

  So the commitment buys deterrence at the global rate and is paid for at the local rate.
- **Note.** Agent 4's `Probe` term already uses context (`ExpDefend | ctx`). Agent 1's Opportunity term does not.
- **Severity.** High.

### E6. Credibility farming on minnows (medium)
- **Pattern.** Guarantee small states threatened by weak minors, and honour cheaply. Or issue ultimatums to states that will comply. Then spend the banked `Cred` bluffing a peer.
- **Why it works.** Under Agent 1's additive clamp, cheap honours accumulate toward 100. The ×1.5 high-cost bonus rewards costly honours but doesn't discount cheap ones. A complied ultimatum is ungraded, and probably counts as a success.
- **Severity.** Medium. Agent 4's beta-mean plus the "rival involved" similarity feature mostly contains it.

### E7. Finance-scope sanctions never adapt (high, effectively US-favoured)
- **Pattern.** Apply `finance`-scope sanctions to debtor rivals and wavering clients.
- **Why it works.**
  - The adaptation formula (`bite × (1 − sub_t)`) runs over *trade* share only. Finance scope has no substitution and no trade loss for the sanctioner.
  - It needs a Financial Hub or creditor trait, which in 1980 effectively means the US. That is the permanent, costless weapon the brief forbids.
- **Severity.** High.

### E8. Sanction pulsing (medium)
- **Pattern.** Lift the grain embargo before an election and reimpose it after.
- **Why it works.**
  - If `sub_t` resets on lift, reimposing restores full bite.
  - Agent 2 D2c claims "Credibility −" for lifting, but no rule writes that record. Lifting a sanction isn't a commitment test under either ledger design.
- **Severity.** Medium.

### E9. Arming both sides at no reputational cost (medium)
- **Pattern.** Sell Arms Supply to both belligerents in a regional war, as in D5. Collect revenue and opinion from both, and keep both bleeding.
- **Why it works.**
  - The involvement commitment is tested only when supply stops while the beneficiary is losing. Keep supplying both until peace and neither test fires.
  - What happens to an involvement commitment when the war ends is unspecified, so it presumably closes neutrally.
  - The third-party reaction gives each side a modest negative for the other side's arms, which is outweighed by its own direct positive.
- **Severity.** Medium.

### E10. Laundering strength around domestic and treaty limits (medium)
- **Pattern 1, peace-dividend dodge.** Cut the Military slider to dodge Agent 1's defence-burden Legitimacy drift. Fund allies' militaries through earmarked Aid instead. Allied pools count in deterrence through contingency entry.
- **Pattern 2, arms-control dodge.** Before signing Arms Control, transfer strength into allied pools. The caps apply to own pools only.
- **Pattern 3, free proxy war.** Use Volunteers, where "your forces fight under their flag". No rule says those casualties feed the supplier's War Weariness.
- **Severity.** Medium.

### E11. Punitive strikes with no comeback (high)
- **Pattern.** Use counter-proliferation and "honour" strikes repeatedly.
- **Why it works.** The aim auto-ends, so the *target* can't continue the war, retaliate on that front, or refuse peace. That is consequence-free intervention at rung 7.
- **Severity.** High. This is also the engine behind E1.

### E12. Integration is a free add-on for large economies (medium)
- **Pattern.** Sign depth-3 integration treaties with everyone who proposes, accepting at 0 Initiative.
- **Why it works.**
  - The adjustment cost is `−(Δtrade/GDP × k × depth)`. US GDP is huge, so the US adjustment is negligible while the growth is permanent.
  - Agent 2's contagion link is absent from Agent 1's spec.
  - The trade-focused strategy therefore isn't a choice. It stacks on top of any other strategy for free.
- **Severity.** Medium.

### E13. Selective enforcement is opt-in (medium)
- **Pattern.** Never hold a `Norm` claim, or drop scenario-preloaded ones on turn 1 at `withdrawn_calm` cost. Then intervene purely by interest.
- **Why it works.**
  - Agent 4 writes `Norm` records and computes `Selectivity` only when P holds a matching public claim. Agent 1 defers Stances (N2) entirely.
  - In v0.1, "similar cases treated differently" therefore produces **no data**.
  - The brief's core emergent result, that selective enforcement "gradually changes what others expect", is avoided by simply not declaring anything.
- **Severity.** Medium.

### E14. Turn-1 bloc dump (medium)
- **Pattern.** Isolationist US exits every inherited alliance on turn 1, where tension allows.
- **Why it works.** Agent 1 rates a calm withdrawal as negligible, while Agent 4 records a treaty exit as `Abandoned`. Under Agent 1, retrenchment is free.
- **Severity.** Medium. It makes isolationism too cheap, not dominant, because objectives still fail.

---

## 2. Missing costs

- **Stock upkeep of commitments.** Nothing scales with *how many* commitments you hold. Rivals' Opportunity scores probe the weakest one, which is the only cost of holding many, and that cost is weak while E1 and E5 stand. Beliefs should account for overstretch: what P can actually send. A P that is already fighting two wars shouldn't be believed as much as one fighting none.
- **Coercion records for Punitive strikes and complied ultimatums.** Both should raise `IntentEst` among similar states exactly as wars do.
- **Volunteer casualties.** They should count toward the supplier's War Weariness.
- **Conduit exposure.** Agent 2 D1 describes a conduit state, but no rule requires one or exposes it. Arming an insurgency in a region the supplier can't reach is currently frictionless.
- **Withdrawal in calm.** It should cost local reputation (the beneficiary and its neighbours), even though it shouldn't cost global reputation.
- **Integration adjustment for big economies.** It has to be measured per sector, not against GDP.

---

## 3. Missing counterplay

| Threat | What is missing | Counter to add |
|---|---|---|
| Punitive strikes (E11) | The target has no move | The target may refuse peace and continue the war under its own aims |
| Proxy supply | The occupier's only answer is more troops | A conduit rule: pressure, strike or bribe the conduit |
| Finance coercion | No substitution path | Rival-bloc and other-creditor lending as substitutes (the same `sub_t` mechanism) |
| A hegemon defending against aggression | Observers can't tell punishing aggression from initiating it | Weighting by who started the hostilities |

**Detail on the last row.** Agent 4's `IntentEst` counts *all* intervention records against states similar to the observer. A US that only ever honours guarantees against invaders therefore raises `IntentEst` among every state similar to the invader, exactly as a US that invades would. A restrained-but-reliable US gets balanced against like a ruthless one. That breaks Gate 2's "no universally superior answer" in favour of not honouring at all.

- **Anti-dogpile is otherwise sound.** It rests on capability × intent, reach-local power, free-riding, and test #10.
- **Anti-ideology-lock is sound.** It rests on the bounded term, the discount under threat, and decision scoping.

---

## 4. Mechanics likely to become tedious

1. **Request triage.** Up to 15 AIs send proposals every turn, each needing an accept or refuse with a Trust consequence. This is the US player's biggest chore.
2. **Commitment tests.** Each one needs explicit action within K = 2 turns, and a US with 20+ commitments faces one most turns.
3. **Per-crisis ladder picks** for every crisis the player is "interested" in.
4. **Election-calendar gaming.** Staggering integration signings and sanction lifts around elections is optimal but joyless (E8, E12).
5. **Per-observer credibility overlay.** It's useful, but it's 15 numbers with 3 precedents each. Players will want one "who doubts me and why" list.
6. **Sanction scope micro.** Four scopes × many targets. Keep scope as a dropdown on one action and default it to `trade`.

---

## 5. Strategy viability table

Columns: (a) viable under the current design? (b) dominant under the current design? (c) after the fixes, what keeps it in check?

| Strategy | Concrete play pattern | (a) | (b) | (c) |
|---|---|---|---|---|
| **Ruthless** | Accept every request. Guarantee widely. "Honour" by Punitive strike or a zero-allocation war. Withdraw before tests. Finance-sanction debtors. Arm both sides in the Gulf war. Covert-arm insurgents. Abandon peripheral clients | Yes | **Yes. E1–E5, E7, E11 make it strictly better than restraint** | Honour requires real force. Look-back regrading. `IntentEst` from coercion records drives balancing. `ExpAbandon` makes clients hedge. Exposure costs Legitimacy. It stays viable as "selective and cheap", but it pays locally |
| **Restrained** | Few public commitments, each honoured in full. Mediate. Aid. Refuse most requests. No covert ops | Yes, as in D10 | No. It is *dominated*: full honour costs more than phony honour (E1) for the same grade, and refusing requests is taxed (Agent 2) | After the fixes it is viable. `Cred` builds slowly (+ vs −), so the risk is rivals probing where the US hasn't committed, and objectives such as chokepoints and "no continental hegemon" going unmet |
| **Isolationist** | Turn 1: exit alliances in calm, move Military into Welfare/Development, no interventions | Yes. Exit is nearly free (E14) | No. US objectives (bloc share of GDP, chokepoints, no hegemon) fail, and the USSR expands | Local reputation costs on exit. Allies rearm or hedge. Opportunity scores rise against former clients. Re-entry later starts from a decayed but negative record |
| **Interventionist** | Climb to rung 7–9 in every qualifying crisis. Coalition wars. Regime change | Yes | Close to dominant only through repeated Punitive strikes (E11). Otherwise it is double-penalised (see F14) | Casus-belli/tension gate. Occupation resistance. War of choice. Coercion records raise `IntentEst` and balancing. A target that refuses peace turns a Punitive strike into a real war |
| **Trade-focused** | Depth-3 integration with every willing partner. Lift sanctions. Mediate | Yes | **Yes, as a free add-on to every other strategy (E12)** | Sector-based adjustment costs near elections. Debt contagion from unstable partners. Inability to sanction deep partners. Partners' dependence-creep refusals |
| **USSR, same tools** (non-US check) | Guarantee clients (Cuba-, Syria-, Vietnam-like). Honour by Punitive strike. Arm insurgencies abroad. Accept all client requests. Withdraw calm before tests | Yes. **E1–E5, E9–E11 work identically.** There is no US-only branch | Same exploits, so ruthless is dominant for the USSR as well. The only US-favoured piece is E7 (finance scope needs a hub trait, which is data and justified) | Same fixes. Authoritarian rule currently pays no domestic exposure cost (Agent 1: "by a democracy"), so covert action is relatively cheaper for the USSR. F15 makes that a scalar, not a flag. China trade-focused: authoritarian adjustment weight 1.0 vs 1.5 makes integration cheaper for China. That's plausible and fine if it is a scalar |

**Verdict.** Under the current spec, Ruthless + Trade-focused is the dominant combination for any great power, because of the grading exploits. With F1–F7 applied, no strategy is automatically optimal.

---

## 6. Hidden moral assumptions and flavour overriding agency

1. **Democracy-only domestic accountability.** These are coded as democracy flags:
   - exposure Legitimacy (Agent 1 2.3);
   - adjustment ×1.5;
   - Expeditionary Democracy.

   Authoritarian regimes that act against their own ideology pay no elite cost. Agent 4's `AssocCost = IdeoDist × openness × contradictions` is the generic version and should replace the flags.
2. **Double "Vietnam" penalty.**
   - Agent 2's trait applies War Weariness ×1.5 in wars of choice.
   - Agent 1 already applies `1 + c(1 − threat)` and makes the trait raise `c`.

   Stacking both penalises US intervention for flavour reasons.
3. **Dilemma consequences that no rule produces.** These are designer verdicts dressed as consequences, which is exactly the "approved answer" the brief forbids:
   - D2c: "Credibility −" for lifting an embargo.
   - D5e: "treats (e) as abandonment", where no commitment exists.
   - D6e: "Credibility − with **every** small state" and "Opportunity scores rise **globally**". This contradicts the similarity kernel.
   - D7: "uprisings US rhetoric encouraged". Rhetoric isn't a mechanic.
   - D9e: "others expect bailouts".
4. **Loaded option labels.** Examples: "ignore the programme", "quiet deal", "let both bleed". Neutral labels should describe the action.
5. **Government-type affinity matrix.** If data puts "democracy" in one cell and splits authoritarians into many, democracies get structurally larger cohesive approval in the third-party reaction rule. The matrix should be symmetric, and test #1 (permutation) should cover it.

---

## 7. Recommended fixes

Each fix targets a demonstrated problem and is the smallest change I could find.

**Credibility grading (fixes E1, E2, E6)**
- **F1 (E1, E11). Honour means force on the front.**
  - Honoured requires a front allocation ≥ 0.5× the attacker's estimated strength on the beneficiary's front within K, *or* the beneficiary losing no core region by the end of the test.
  - A war declaration with less allocation, a contingency auto-entry with no allocation, or a Punitive aim alone is graded Partial.
  - This replaces "25% of usable pool".
- **F2 (E2, E6). One formula: Agent 4's beta-mean.**
  - Partial = 0.5, so it pulls belief *down* whenever belief is above 0.5.
  - Honoured records are weighted by `(0.25 + 0.75·cost_paid)`, where `cost_paid` is relative to the attacker's power. Abandoned records take full weight.
  - Complied ultimatums are graded like honours, weighted by the same cost term.
  - Drop Agent 1's additive clamp.

**Commitment endings (fixes E3, E14)**
- **F3 (E3, E14). Look back on withdrawals.**
  - Any commitment ended unilaterally or by "mutual" lapse is regraded `Abandoned ×0.75` if, within 8 turns, the beneficiary is attacked by its top Threat at the time of withdrawal.
  - A withdrawal that stays calm is graded Partial, with salience limited to the beneficiary and same-region observers.

**Deterrence and Initiative (fixes E4, E5)**
- **F4 (E5, upkeep). Deterrence symmetry.** The Opportunity score's third-party term = `ExpDefend(attacker, P | ctx = beneficiary) × min(1, P's free usable strength in reach ÷ attacker's estimated strength)`. The function that prices abandonment also prices deterrence, and overstretch shows up as believability, not as a new upkeep cost.
- **F5 (E4). Initiative on acceptance.**
  - Accepting a proposal that makes you the *owner* of a Commitment (guarantee, alliance, or a supply stream to a belligerent) costs 1 Initiative. All other acceptances stay free.
  - Refusing a request writes only a decaying Opinion modifier, unless the request falls under an existing commitment's trigger. This replaces Agent 2's Trust-on-refusal.

**War and coercion tools (fixes E7–E11)**
- **F6 (E11). Punitive needs the target's consent to end.**
  - A Punitive aim ends after 1–2 ticks only if the target accepts. Otherwise it converts to a normal limited war, with the target's aims.
  - Punitive strikes write coercion records like any war.
- **F7 (E7, E8). Sanctions adapt and remember.**
  - `sub_t` applies to every scope. For finance, the substitutes are other hub or creditor holders plus rival-bloc lending.
  - After a lift, `sub_t` decays at −0.05 per turn instead of resetting.
  - Lifting writes a Threat record only if the sanction was attached to a public demand. Remove the claim in D2c otherwise.
- **F8 (E13). Implicit norms.** Any rung ≥ 7 action, or leading a sanction coalition, against a norm-tag violator auto-creates a `Norm` claim for that tag (stake 0.3, decaying with H). Selectivity then has data in v0.1 without the Stances UI.
- **F9 (missing counterplay). Who started it.** In `IntentEst`, records where P was *responding* to a target that initiated hostilities against P or P's beneficiary weigh ×0.5 for observers with no aggression records of their own. This is a factual feature, not a moral one.
- **F10 (E10, conduit).**
  - Proxy supply to a recipient without border or sea access requires a consenting conduit. The conduit gains tension with the target in proportion to the flow, and it is a valid casus-belli target.
  - Volunteer casualties feed the supplier's War Weariness: ×0.5 while covert, ×1.0 once exposed.
- **F11 (E9). Arming both sides.**
  - When a belligerent knows P also supplies its enemy, a Support record graded Partial is written for that belligerent, and both involvement stakes drop to 0.
  - When a war ends, involvement commitments resolve by outcome: if the beneficiary lost core regions, the commitment is graded Partial.
- **F12 (E12). Integration by sector.** Adjustment = `−k × depth × (partner's share of imports in the actor's exposed-sector tags)`, using Agent 2's sector tags as data. It is not normalised by GDP.
- **F13 (E10). Count supply as military effort.** Military-earmarked Aid and arms outflows count toward `military_share` (defence burden) and toward Arms Control caps for 8 turns.

**Cleanup**
- **F14 (§6.2).** Delete the trait's ×1.5. Expeditionary Democracy only raises `c`.
- **F15 (§6.1).** Replace democracy flags with an `openness` scalar wherever domestic exposure or adjustment is weighted.
- **F16 (§6.3–4).** Dilemma hygiene. Every consequence cell must name the ledger write or modifier that produces it. Strike the rest, or attach them to a real record (for example, Denounce plus "call to rise" creates an involvement commitment at stake 0.3).
- **F17 (tedium).**
  - Standing reply policies per request type and requester ("auto-decline arms requests from X").
  - Contingency orders pre-set the honour allocation, so most tests resolve without a prompt.
  - Show a single "who doubts me" list sorted by the size of the `ExpDefend` change.
- **F18 (consistency).** DESIGN §7.3 lists Aid as costing Initiative, but the §21 worked example shows "Food aid: 0 Initiative". Settle it. Recommendation: starting an Aid stream costs 1 Initiative and continuing it is free. If Aid is free, US aid spam buys Opinion everywhere.

**Tests to add to Agent 4's suite.**
- Phony honour graded Partial (F1).
- A withdrawal followed by an attack within 8 turns is regraded Abandoned (F3).
- Accept-all US vs selective US across 200 seeds: accept-all must not dominate on objective score.
- A finance sanction's bite halves within 10 turns when substitutes exist.
- Dual supplier: no positive Support record with either belligerent.
- Gate 2 matrix: across 500 seeds, no single strategy in the table wins more than 40% of US objective-score comparisons.
