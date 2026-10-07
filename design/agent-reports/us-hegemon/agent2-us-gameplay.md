# Agent 2: US Gameplay Designer. The Networked Hegemon

*Design pass only. Uses DESIGN.md vocabulary. "NEW?" marks anything not in DESIGN.md, so reviewers can attack it. P1–P12 are scenario-1980.md Part C candidates, which are not yet approved.*

---

## A. Playing the US

### What the US player does each turn
1. **Reads the Briefing.** With Intel tech 8 and allies sharing coverage, the US sees more than anyone else, but its picture of closed states (USSR, Iran, North Korea, Iraq) is still estimates with wide bands. The US is rarely blind. It is often wrong with confidence.
2. **Triages requests.** Allies and clients send proposals: arms, aid, guarantees, sanctions against their enemies. Accepting costs no Initiative (§2.3). Accepting also adds commitments. Refusing is free today and costs Trust with the requester.
3. **Spends 3 Initiative**, sometimes 4 with Stability over 70 or saved Initiative, on maybe two foreign moves and one domestic or intel move. **Initiative is the binding constraint, not money.** The US can afford to do almost anything. It cannot pay attention to everything. Each crisis it joins is a crisis it can't shape elsewhere.
4. **Adjusts four budget sliders** (§2.1) with 2–3 turns of inertia. A Military increase shows up on rival intel (security dilemma loop, §3.3).
5. **Sets standing postures**: nuclear posture (§11.4), monetary stance (P7, if adopted), contingency orders (guarantees and alliances, §4.1).

### Main levers (all existing)
- **Money:** Aid, Sanction, Embargo commodity (grain), Arms Supply treaty, patronage (P4), interest rate (P7).
- **Commitments:** Defensive Alliance, Guarantee, Basing/Access, Threaten/Ultimatum.
- **Covert:** Expand network, Fund opposition, Arm insurgents, Disinformation, Coup (§8.4); proxy arms, advisors and volunteers (§11.5).
- **Force:** limited war aims vs. regime-change aims (§6.6), and expeditionary fronts that need basing and sea-lane control (§6.3–6.4).

### What constrains the US
- **Domestic:** a democracy (§9.1). Prosperity and War Weariness carry heavy weight, elections come every ~16 turns, exposed operations trigger Spy scandal (§12.1), and the US is open, so rivals read it easily (§8.2).
- **Budget:** money is rarely scarce, but every Military point is a point not spent on Welfare during an election run-up.
- **Initiative:** see above. This is the opportunity cost in the game's design.
- **Credibility:** the US holds more guarantees and alliances than anyone. Each one is a public contingency order, so rivals' Opportunity scores (§14.3) probe the weakest one.
- **Overextension:** bases, clients and occupations each pull on budget and attention. They also give rivals places to hurt the US cheaply through proxies (§11.5, §6.7).

### Proposed US traits (four, consolidating B.1)

| Trait | Upside | Downside | Decision it changes |
|---|---|---|---|
| **Reserve Currency** (keep, ★) | Near-free borrowing. Sets the World Interest Rate (P7). US Sanctions are stronger because of financial centrality | A tight rate raises every debtor's debt service, which drives Debt contagion and debtor Opinion −. **NEW?** Each sanction against a target pushes its trade toward non-sanctioning partners, so repeated sanctions lose force | Fight inflation at home, or protect debtor allies? Sanction now, or hold the weapon for when it matters? |
| **Global Commitments** (merges Imperial Outposts + "many commitments") | Starts with the largest set of treaties and outposts (P8). Supply and air range nearly everywhere. Allies share coverage | Every guarantee is a live credibility test. Rivals' Opportunity score rises against the weakest-defended commitment | Which commitments to reinforce, which to let lapse quietly, and which to never make |
| **Expeditionary Democracy** (replaces "Vietnam Syndrome": same mechanic, generic name) | Cheap credit. A short victory gives a big Legitimacy rally | War Weariness ×1.5 in wars where US core regions weren't attacked. The modifier decays after a quick victory. Exposed operations cost the most Legitimacy here | Short, sharp wars with limited aims, or none. Covert action carries a real domestic tail |
| **Breadbasket** (keep as data. It is an instance of **Exposed Sectors**, NEW?, see C) | Food leverage. Agricultural exports earn Opinion with food importers | Food embargoes and trade deals that hurt the farm sector hit Prosperity and Legitimacy, ×2 in the 4 turns before an election | Grain as a weapon vs. grain as a constituency |

**Cut from B.1:** none lost. Imperial Outposts folded into Global Commitments. SDI stays the national project. It is not a trait.

---

## B. Strategic dilemmas

Each dilemma states its **trigger conditions**. No dilemma fires on a date. The "AI reactions" use four observer slots: **Democratic ally (DA)**, **Authoritarian client (AC)**, **Rival (R)**, **Non-aligned (NA)**. How each observer reacts comes from its own relationship, interests and memory (§14.6). Nothing checks whether the actor is the US.

---

### D1. The conduit (early 1980s: proxies, authoritarian partner, blowback seed)

**Trigger:** the rival occupies a non-core region with Resistance > 30 (§6.7). The only supply route runs through a neighbouring authoritarian state (Pakistan-like), and that state's opinion of the rival is below −30. The neighbour has a **suspected covert arsenal programme** (P10).

**Situation:** the neighbour offers to channel aid to the insurgents. In return it wants patronage (P4) and arms. Implicitly, it wants the US to stop asking about its programme.

**Information:** rival casualty rate estimated at "moderate–high" (coverage 45, ±25%). Neighbour's programme progress is "20–60%, confidence low." Insurgents' alignment is "fragmented, several factions **hostile to all outsiders** (confidence medium)."

| Option | Immediate benefit | Immediate cost | Second-order risk |
|---|---|---|---|
| **a. Full patronage + arms via conduit, ignore the programme** | Rival's War Weariness and casualties rise. The neighbour aligns | Budget (patronage). 1 Initiative. DA Opinion − if exposed | Neighbour's arsenal matures under US cover. The insurgents keep the arms (transferred capability) |
| **b. Arms aid conditioned on a programme freeze** (NEW? conditional package) | Same proxy effect, smaller. Non-proliferation stays credible | Neighbour may reject (−Opinion) or cheat. Verification needs coverage 60+ | A detected breach forces a choice: enforce (lose the conduit) or ignore (credibility hit with everyone who saw the condition) |
| **c. Covert Arm insurgents only, small** | Low tension. Deniable | Weak effect. The rival can win the occupation | If exposed, the incident comes without the strategic payoff |
| **d. Transfer advanced weapons (quality, not just strength)** | Insurgent effectiveness jumps. A fast drain on the rival | Large tension (Proxy rung, §11.2). High exposure chance | High-quality arms persist after the war, in factions that later turn hostile |
| **e. Stay out; Mediate** | Initiative saved. Lower tension. NA Opinion + | The rival consolidates. AC reads the US as an unreliable patron | Rival frees up forces for elsewhere |

**Systems:** Proxy (§11.5), patronage (P4), Arms Supply (§7.2), covert programmes (P10), Intel (§8.4), Trust, Credibility.

**AI reactions:** **AC (the neighbour)** values (a) highly, accepts (b) only if its paranoia is below its greed for aid, and reads (e) as abandonment. **DA (West Germany)** is mildly negative on (a)/(d) only after exposure, and positive on (b). **R** gains a propaganda target. Its threat score for the US rises sharply under (d), and it may escalate in a third theatre. **NA (India)**, the neighbour's rival, takes Opinion −20 under (a)/(d), drifts toward the rival, and is positive on (e).

---

### D2. The pipeline and the grain (early 1980s: economic coercion that hurts allies and a domestic sector)

**Trigger:** a US sanction or food embargo on the rival is active. A democratic ally holds a Trade Agreement with the rival that includes energy imports. A rival bloc member is in crisis (Stability < 30, crackdown under way). A US election falls within 6 turns.

**Situation:** the embargo bleeds US farm income (Breadbasket). Other food exporters are filling the gap (substitution). The crackdown creates pressure to "do something."

**Information:** the embargo's effect on the rival is "food deficit cut by 10–30%, confidence medium." Substitution by a third exporter is **visible in trade data** (high confidence). Ally dependence on rival energy is known precisely, because the ally is open.

| Option | Benefit | Cost | Second-order risk |
|---|---|---|---|
| **a. Escalate: Sanction the ally's trade with the rival (secondary pressure)** | Raises pressure on the rival's hard-currency income | DA Opinion −25, Trust −. US trade loss. 1 Initiative | Ally starts hedging, and later sanction calls get less uptake |
| **b. Hold the embargo, propose multilateral sanctions** | Signals resolve. Sanctions strengthen if allies join (§7.3) | Farm Prosperity and Legitimacy keep falling before the election | Allies refuse, and the refusal is visible. The sanction weakens anyway |
| **c. Lift the grain embargo; keep high-tech export bans** | Prosperity recovers. Farm lobby is satisfied | Credibility − (a public measure abandoned) | Rival learns that US coercion has a domestic expiry date |
| **d. Swap coercion for Denounce + Aid to the crisis state's opposition (Fund opposition)** | Cheap. Targets the bloc's weak point | Network level 3 required. Exposure means a major incident | Bloc crisis may escalate into rival intervention, raising Global Tension |

**Systems:** Sanction/Embargo (§7.3), interdependence loop (§3.3), Prosperity driver (§9.1), Election event, Exposed Sectors (NEW?), Credibility.

**AI reactions:** **DA (West Germany)**, which is energy-dependent with high greed and loyalty, resists (a) hard and may sign more with the rival. It joins (b) only if its tension with the rival is above 50. **AC (Saudi Arabia)** is indifferent but notes the US paying costs for allies. **R** offers better terms to the ally and to substitute exporters. **NA (the substitute exporter, e.g. Argentina or India)** profits from the embargo and gains Opinion of the rival.

---

### D3. Paying for the buildup (early 1980s: buildup, alliances, rival perception)

**Trigger:** US threat score toward the rival is above 60, US Military readiness is below 60%, and US Prosperity is below 50 with inflation pressure (World Rate rising).

**Situation:** the buildup is wanted, but there are three ways to fund it, and each one feeds a different loop.

**Information:** rival military spending "rising, ±25%." The rival's leadership personality is "paranoid (confidence high)." Debtor allies' debt is known.

| Option | Benefit | Cost | Second-order risk |
|---|---|---|---|
| **a. Deficit buildup + Tight money** | Fast readiness. US inflation tamed | Debt rises. The World Rate climbs and debtors' service jumps | Debt contagion in debtor states, some of them US clients. Rival paranoia spikes |
| **b. Buildup funded by Welfare cuts** | No debt | Prosperity −. Election risk | Domestic crisis drops Initiative by 1 (§2.3) at the worst moment |
| **c. Start SDI (national project) and keep spending flat** | Long-term edge. Development spend | Rival paranoia jumps immediately (B.1) | Rival either overspends (oil-dependent, so its budget cracks) or adopts a desperate posture |
| **d. Offer Arms Control from strength** | Tension −. Budget freed | DA/AC who wanted reassurance read weakness. Verification depends on coverage | Rival cheats in an area below US coverage |

**Systems:** Budget/Debt (§2.1–2.2), World Interest Rate (P7), security dilemma loop (§3.3), National Project (§10.2), Arms Control (§7.2).

**AI reactions:** **DA** prefers (d), and its fear of entrapment rises under (a)+(c). **AC (Saudi Arabia)**, which has security needs, prefers (a). **R**, with paranoia 0.85, meets (a)/(c) with counter-buildup or a posture shift. A reformer leadership might accept (d). **NA debtors (Poland, Latin bloc minor)** take Opinion − under (a) and risk default, which spreads to their trade partners.

---

### D4. The reformer in Moscow (late 1980s: bloc reform or escalation)

**Trigger:** a Leadership change (P11) produces a rival personality with ideology below 0.5 and aggression below 0.4. Rival Stability is below 45 and its oil revenue is falling. The rival proposes Arms Control. A bloc minor is in a Transition Crisis (P3).

**Situation:** the rival is weakening. Help it reform, squeeze it, or exploit the bloc?

**Information:** "Leadership assessment: reformist, **confidence 55%**" (coverage ~55; intentions require 80+). Hardliner coup risk is unknown to the US. Arsenal security in peripheral regions is "uncertain."

| Option | Benefit | Cost | Second-order risk |
|---|---|---|---|
| **a. Accept Arms Control + credits (Aid)** | Tension −. Budget freed. Trust with rival + | Hawks: Legitimacy − if the rival cheats. Money to a rival | Stabilises a rival that may later revert. Or that the reformer is ousted anyway |
| **b. Squeeze: keep the buildup, Fund opposition in bloc minors** | Accelerates bloc fragmentation. Opportunity in Europe | Tension stays high. Exposure risk | Rival Stability < 25 brings a hardliner coup or disorderly collapse, with loose arsenals and successor states (P2) |
| **c. Conditional engagement: Arms Control only, no credits, Denounce crackdowns** | Middle path. Keeps leverage | Satisfies no one fully | Bloc minors' transitions go faster or slower depending on rival reaction, which the US doesn't control |
| **d. Disinformation: appear weaker/more conciliatory to lower rival paranoia** | Reduces security spiral | Credibility hit if exposed | Allies misread US resolve too |

**Systems:** Leadership change (P11), Transition (P3), Arms Control, Aid, Fund opposition, Region Loyalty (P2), Global Tension.

**AI reactions:** **DA (West Germany)** strongly prefers (a), which favours reunification-path stability. Under (b) it fears instability on its border. **AC (Pakistan, Saudi Arabia)** worries that détente lowers its value to the US (patron-value decline). **R** gains a domestic position for its reformers under (a). Hardliner factions gain under (b). **NA (Yugoslavia)** benefits from low tension under (a), and Region Loyalty stress rises under (b).

---

### D5. Tilting in someone else's war (late 1980s: authoritarian partner, chokepoint, blowback)

**Trigger:** two regional powers are at war (Iran–Iraq-like). The side with the higher-opinion relationship to US clients is losing War Score. The Tanker War event is active, so the chokepoint is Contested (P6). The revolutionary side holds a grievance against the US.

**Situation:** Gulf clients ask the US to save the losing side. The losing side is authoritarian, rival-armed and aggressive. The winning side is revolutionary and hostile.

**Information:** front strength ratio "1.3–2.0 in favour of the revolutionary side (confidence medium)." The losing side's arsenal programme is "suspected (low confidence)." Chokepoint transit cost +25% (certain).

| Option | Benefit | Cost | Second-order risk |
|---|---|---|---|
| **a. Intelligence sharing + credits to the losing side** | Prevents a revolutionary regional hegemon. AC Opinion + | Ties the US to an aggressive regime | The client keeps its improved capability and debt, and becomes a dangerous, indebted, armed state |
| **b. Escort the clients' tankers (naval deployment, Basing)** | Chokepoint reopens. Energy price falls | Naval incidents with the revolutionary side. Tension | Gives the revolutionary side a casus belli against the US. Casualties at sea |
| **c. Covert Arms Supply to the revolutionary side (opening / hostages)** | Possible leverage over a hostile state | Contradicts a public arms embargo | If exposed: Spy scandal, Legitimacy crash, Credibility − with every AC |
| **d. Mediate** | NA Opinion +. Tension − | Clients feel unheard. The war may continue anyway | Low |
| **e. Let both bleed** | No cost | Chokepoint stays Contested. Prices high | Whichever side wins emerges as an exhausted, armed, resentful regional power |

**Systems:** Proxy, Arms Supply, Basing, chokepoints (P6), commodity prices (§2.4), Spy scandal (§12.1), Trust per observer.

**AI reactions:** **AC (Saudi Arabia)** values (a)/(b) and treats (e) as abandonment, so its opinion drops and it hedges toward the rival. **DA (Japan)**, which is energy-dependent, supports (b) and pays rather than fights. **R** supplies the same client (competition), or courts the revolutionary side if the US tilts. **NA (India)** prefers (d) and resents (b) as great-power policing.

---

### D6. The seizure (early 1990s: regional aggression, rival weak)

**Trigger:** an authoritarian state with Debt > 60%, Stability < 45 and a large army (often enlarged by earlier transfers: see D1/D5) has a non-core claim on a weakly defended energy-rich neighbour (a bloc minor). It invades (Diversionary Claim / Opportunistic Invasion). The rival is weak: Stability < 40 or recently reformed, so its threat score for the US is below 40 and it won't defend the invader.

**Situation:** the invader now holds a large share of world energy. Its forces border a US client (Saudi Arabia-like).

**Information:** invader forces "±25%, 15–40% larger than client defences." **Intentions unknown** (coverage 40; intent requires 80). Energy price +60% (certain). Rival response "likely abstain (confidence medium)."

| Option | Benefit | Cost | Second-order risk |
|---|---|---|---|
| **a. Sanctions + embargo coalition only** | Low casualties. Broad support | Invader keeps the prize. Energy shock persists | Sanctions erode by substitution. Invader entrenches |
| **b. Guarantee the client + deploy defensively** | Deters further advance. Energy partly stabilises | Basing on the client's soil. Readiness costs | Open-ended deployment. Host population resentment (NEW?, deferred) |
| **c. Coalition war, limited aims (liberate the neighbour)** | Restores status quo. Huge Legitimacy and Credibility if quick | Initiative, casualties, mobilization. Needs Coalition Restraint management (E.2) | **The invader survives, wounded and unstable: see D7** |
| **d. Coalition war, regime-change aims** | Removes the threat permanently | Bigger aims: neutrals lean against, coalition members may quit, occupation (§6.7) | Resistance, long garrison, War Weariness ×1.5 |
| **e. Accept the fait accompli; negotiate prices** | No war. Initiative saved | Credibility − with every small state under a US umbrella. AC Trust collapses | Other revisionists' Opportunity scores rise globally |

**Systems:** War aims (§6.6), Guarantee/Basing, Embargo, Coalition (alliance contingency + Aid from allies), Credibility, chokepoint, prices.

**AI reactions:** **AC (the threatened client)** requests a Guarantee and Basing. It accepts despite its ideology, because the threat outweighs it. **DA (Japan, West Germany)** joins sanctions and pays Aid rather than troops (greed high, aggression low). Under (d) it hesitates. **R (weakened)** abstains under (c) in exchange for credits, and opposes (d), which sets a precedent against its own clients. **NA (India, Jordan-like minor)** joins (a), opposes (d), and is split on (c) depending on its opinion of the invader.

---

### D7. After the victory (early 1990s: early win, costly entanglement)

**Trigger:** the US ended a war with limited aims and a high War Score. The defeated regime survives with Stability < 25. Low-loyalty regions (P2) revolt. US forces remain on Basing in the host.

**Situation:** winning the military operation was the cheap part. The defeated regime crushes uprisings that US rhetoric encouraged.

**Information:** rebel strength "low, ±50%." Regime "likely survives (confidence 60%)." Arsenal programme remnants "unknown."

| Option | Benefit | Cost | Second-order risk |
|---|---|---|---|
| **a. Full withdrawal** | War Weariness decays. Budget freed | Trust − with the rebels and with observers who heard the encouragement | Regime recovers and nurses a revenge goal (§14.4) |
| **b. Containment: sanctions + air exclusion zone (sustained deployment)** | Contains without occupation | Standing readiness cost, a Basing tension tax, and Initiative each time it's renewed | Sanctions' humanitarian toll drives NA Opinion − over time. Coalition fatigue |
| **c. Arm insurgents** | Keeps the regime weak | Major-incident risk | Rebel factions gain persistent capability and their own aims (P2 secession) |
| **d. Resume war for regime change** | Ends the problem | New war, occupation, Resistance | Turns a short war into a long one. Expeditionary Democracy penalty returns |
| **e. Quiet deal with the regime** | Stability and oil flow | Credibility with rebels and DAs − | Regime outlasts the deal |

**Systems:** Occupation/Resistance, Region Loyalty, Sanction, Basing, War Weariness, Arm insurgents, revenge goal.

**AI reactions:** **AC (Saudi Arabia)** prefers (b), and fears (d) because it would destabilise the region. It dislikes indefinite US troops on its own soil. **DA (France-like minor / West Germany)** backs (b) early and drifts toward (a) over time. **R** opposes (b)/(d) and trades with the regime. **NA (Iran-like neighbour)** exploits (c) by sponsoring friendly factions.

---

### D8. What is American power for? (early–mid 1990s: post-rival)

**Trigger:** the rival collapses into successor states, or its Primacy share falls below 50% of the US share. **No country's threat score for the US exceeds 40.** The Peace Dividend event is active (E.2). Allies' utility for alliance treaties is falling, because the top threat is gone (§7.5 AI).

**Situation:** there's no enemy to organise around. The same tools now buy different things.

**Information:** successor arsenals "3 states hold weapons, custody confidence low." Former bloc minors are requesting alliance membership. China's growth is "high, ±30%," and its intentions are opaque.

| Option | Benefit | Cost | Second-order risk |
|---|---|---|---|
| **a. Retrench: shift Military into Welfare/Development, release outposts** | Prosperity, growth, Legitimacy | Credibility − where outposts close. Allies rearm or hedge | Regional powers fill vacuums. Some balance against each other |
| **b. Expand the alliance to former bloc states** | Locks in gains. Their Opinion of the US + | New guarantees (credibility exposure). Successor-state grievance | The weakened rival gains a revenge goal and slowly rebuilds |
| **c. Coalition leadership: Mediate, Aid, Arms Control on successor arsenals** | Low tension. Trust broadly + | High Initiative per turn. Slow | Free riding: allies let the US pay |
| **d. Primacy: hold the budget, Contain the next riser** | Keeps the lead | No peace dividend. Election pressure | Creates the rival it fears. Balance Against Hegemon goals trigger (§14.4) |
| **e. Conditional Aid for political reform (democracy promotion)** | Possible new aligned states | Money. Intervention in transitions | Failed transitions, and resentment from recipients who see the conditions as tutelage |

**Systems:** Budget, Peace Dividend event, treaty re-evaluation, Guarantee, Aid, Transition (P3), Balance-of-power goal (§14.4), objectives.

**AI reactions:** **DA (Germany, Japan)** welcomes (c), is torn on (a) (it wants autonomy, but fears the vacuum), and under (d) rising economic competition makes it less deferential. **AC (Gulf, Pakistan)** loses value as a client in every option. **R (successor)** reacts to (b) with lasting grievance and accepts (c) for credits. **NA (China, India)** is relaxed under (a), and (d) triggers Balance Against Hegemon goals.

---

### D9. The integration treaty (mid/late 1990s: trade winners and losers)

**Trigger:** a neighbouring, lower-cost economy (a playable country or minor) with high opinion of the US and recent debt stress proposes an **Integration Treaty** (NEW?: a deeper Trade Agreement). Within the US, Exposed Sectors overlap with the partner's competitive sectors.

**Situation:** aggregate gains and concentrated losses. An election is within 8 turns.

**Information:** GDP projection "+0.5–1.5% over 8 turns (confidence medium)." Manufacturing sector "−3 to −8%" (confidence medium). Partner currency peg "fragile (confidence low)."

| Option | Benefit | Cost | Second-order risk |
|---|---|---|---|
| **a. Sign full integration** | Growth. Partner Opinion and Trust +. Sanctions against the partner become very costly | Manufacturing sector Prosperity hit, concentrated in some regions | Partner's later crisis becomes the US's crisis (financial contagion). Hard to exit |
| **b. Sign with carve-outs** (sector protections) | Smaller losses | Smaller gains. Partner Opinion lower | Other partners demand the same exceptions |
| **c. Plain Trade Agreement instead** | Low risk | Little interdependence | A rival or other bloc builds its own deeper network |
| **d. Reject** | Sector protected. Election safe | Partner Opinion −. Migration/instability pressure on the border (Refugee Wave trigger) | Partner turns elsewhere |
| **e. Sign, then bail out when the peg breaks** (follow-up choice) | Partner stabilised. Trust high | Budget. Domestic Legitimacy − ("why pay?") | Others expect bailouts |

**Systems:** Trade Agreement (§7.2), interdependence loop (§3.3), Exposed Sectors (NEW?), Election, Debt contagion, Aid.

**AI reactions:** **DA (Japan)** reads (a) as a regional bloc that may exclude it, and seeks its own agreements. **AC** has little stake. **R (China)** prefers (d)/(c), since a fragmented trade order suits it. **NA (the partner)** values (a) most and takes a large Opinion hit under (d).

---

### D10. Somebody else's war (mid/late 1990s: restraint genuinely viable)

**Trigger:** a multi-ethnic state fragments (Region Loyalty secession, P2) into a civil war with atrocities. Refugee Wave events hit European allies. **The US holds no Guarantee or treaty with any party.** A former rival sympathises with one faction. Allies ask the US to lead.

**Situation:** the war affects Europe more than the US. Global Tension is low.

**Information:** faction strengths "±50%." Atrocity reports "partly confirmed." Rival arms flows "suspected."

| Option | Benefit | Cost | Second-order risk |
|---|---|---|---|
| **a. Stay out** | No casualties, no Initiative. **No Credibility loss, because no commitment was made** | Trust − with the allies who asked | Allies build their own capacity, so US leverage over them falls |
| **b. Arms embargo on all parties** | Low cost. Appears neutral | Freezes the stronger side's advantage | Seen as complicity by the weaker side's sympathisers |
| **c. Lift the embargo for the weaker side (Arms Supply)** | Shifts the balance without troops | Tension with the faction's sympathiser | Armed faction keeps weapons and its own war aims |
| **d. Punitive air campaign + Ultimatum** | Fast. Low casualties | Credibility now committed. Tension with the sympathiser | Ultimatum ignored means escalate or lose Credibility |
| **e. Peacekeeping ground force** | Ends fighting | Long Basing, readiness drain | Indefinite deployment. War Weariness if attacked |

**Systems:** Region Loyalty (P2), Refugee Wave, Arms embargo (Embargo + Arms Supply), Threaten/Ultimatum, Credibility (commitment-scoped), Trust.

**AI reactions:** **DA (Germany)** asks for (d)/(e), but under (a) its alliance Trust falls only moderately, and it gains autonomy in exchange. **AC** has little stake. **R (successor)** opposes (c)/(d) and tension rises, while (a) improves relations. **NA (India)** prefers (a)/(b) and treats (d) without a mandate as precedent-setting.

**Design note:** restraint is viable here because **Credibility moves only on public commitments** (§3.3 credibility loop). Reviewers should defend that rule. Without it, restraint becomes a hidden penalty.

---

## C. Mechanics the dilemmas depend on

| Mechanic | Status | Notes |
|---|---|---|
| Initiative, banking, Stability bonus | EXISTING §2.3 | US binding constraint |
| 4-line Budget, inertia, observable buildup | EXISTING §2.1 | D3 |
| Debt, debt service, default and contagion | EXISTING §2.2, §12.1 | D3, D9 |
| Stability drivers, democracy weights, elections | EXISTING §9.1, §12.1 | All |
| Opinion (with modifier lists), Trust, Credibility, Alignment | EXISTING §7.1 | All |
| Treaties: Trade, Alliance, Guarantee, Basing, Arms Supply, Arms Control | EXISTING §7.2 | All |
| Aid, Sanction, Embargo, Threaten/Ultimatum, Denounce, Mediate | EXISTING §7.3 | All |
| Package negotiation with visible AI score | EXISTING §7.4 | D1, D9 |
| Coverage, estimate bands, intentions at 80+ | EXISTING §8.2–8.3 | All "Information" fields |
| Intel ops, detection, Incidents | EXISTING §8.4 | D1, D2, D5, D7 |
| Proxy war: arms, advisors, volunteers | EXISTING §11.5 | D1, D5 |
| Escalation ladder + tension | EXISTING §11.2–11.3 | Covers the brief's intervention ladder without new buttons |
| War aims, War Score, peace | EXISTING §6.6 | D6, D7 |
| Occupation and Resistance | EXISTING §6.7 | D7 |
| Expeditionary fronts, supply, sea-lane control | EXISTING §6.3–6.4 | D6 |
| Mobilization | EXISTING §6.2 | D6 |
| National project (SDI) | EXISTING §10.2, B.1 | D3 |
| Contingency orders = alliances/guarantees | EXISTING §4.1 | D6, D8, D10 |
| AI goals incl. Balance Against Hegemon, Revenge, Protect Client | EXISTING §14.4 | D7, D8 |
| AI memory of broken treaties, aid, exposed ops | EXISTING §14.6 | All |
| Events: Spy scandal, Refugee Wave, Peace Dividend, Diversionary Claim, Tanker War, Coalition Restraint, Transition Crisis | EXISTING §12.1 / scenario E.2 | D2, D5–D10 |
| Patronage treaty | CANDIDATE P4 | D1, D5, D8 |
| Chokepoint Open/Contested/Closed | CANDIDATE P6 | D5, D6 |
| World Interest Rate / monetary stance | CANDIDATE P7 | D3, Reserve Currency |
| Outposts | CANDIDATE P8 | D8, Global Commitments |
| Covert programmes | CANDIDATE P10 | D1, D5 |
| Leadership change | CANDIDATE P11 | D4 |
| Region Loyalty & Secession | CANDIDATE P2 | D4, D7, D10 |
| Regime Transition | CANDIDATE P3 | D4, D8 |
| **Transferred capability persists** | EXISTING §7.2 Arms Supply transfers strength. **Needs explicit rule:** transferred strength/quality stays with the recipient whatever its later relations; no reclaim | D1, D5, D6, D7, D10. This is all the blowback the design needs |
| **Credibility moves only on public commitments** | EXISTING-ish §3.3 loop. **Needs explicit rule** | D10 restraint viability |
| **Observer-specific Credibility reading** | NEW? (modification) | Brief says "no single global truth." Proposal: global Credibility × per-observer adjustment from §14.6 memory (honoured/abandoned commitments *toward that observer or similar states*). Agent 1/3 call |
| **Exposed Sectors** | NEW? | 2–3 sector tags per country (data: farm, manufacturing, energy) with weights. Sanctions, embargoes and trade treaties shift sector income, which feeds Prosperity, amplified near elections. Generalises Breadbasket. Needed for D2, D9. Cheapest fallback: hand-authored opinion-style modifiers on Prosperity per treaty |
| **Integration Treaty (deep Trade Agreement)** | NEW? (could be a `depth` parameter on Trade Agreement) | More trade, sector shifts, large sanction-cost multiplier against the partner, financial contagion link, costly exit. Treaty instance = scenario data |
| **Conditional packages** | NEW? (extension of §7.4) | Attach a demand ("freeze programme," "stop crackdown") to Aid/Arms/Patronage. A breach, if detected via coverage, is recorded in memory. The patron then chooses to enforce or ignore. D1, D8 |
| **Sanction substitution / fatigue** | NEW? | Sanctions lose force as the target's trade reroutes to non-participants. Prevents sanction spam. D2, D6, D7 |
| **Requests from allies (refuse = Trust cost)** | NEW? (small: a proposal type) | Allies propose "help me"; declining records in memory. Makes commitments pull |
| **Host resentment of foreign bases** | NEW?, propose DEFER | D6, D7 flavour; can be faked with an opinion modifier on Basing |
| **Secondary sanctions** | EXISTING. **No new mechanic.** Just Sanction an ally | D2 |
| **Coalition burden-sharing** | EXISTING. Allies send Aid to the war leader | D6 |
| **Air exclusion / containment** | EXISTING. Sustained Basing + readiness + Sanction | D7 |
