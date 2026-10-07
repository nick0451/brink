# Scenario: 1980–2000 "Brinkmanship" — design v0.2

*Status: **approved** (2026-10-03). The approval checklist was accepted by default. v0.2 also folds in the approved Networked Hegemon mechanics report ([us-mechanics-report.md](us-mechanics-report.md)):*
- *US archetype revised (A.1, B.1).*
- *Part C mechanics re-statused: some merged into DESIGN.md, some deferred to the v0.2 scenario-completeness milestone (DESIGN §22).*

*Built from four parallel research passes: Superpowers & Europe, Middle East, Asia, Africa & Latin America.*

> **Implementation note.** Where a country's signature trait depends on a mechanic scheduled for v0.2 (P5, P10, P3, P2), the v0.1 engine milestone ships that country with the trait inactive or simplified. These countries are affected:
> - Saudi Arabia
> - Israel
> - South Africa
> - Pakistan
> - North Korea
> - India
> - Yugoslavia
> - the USSR
>
> All engine behaviour is generic; country identity lives only in data (DESIGN §14.1 rule 5).

---

## How to read this (and what I need from you)

This document has five parts:

- **Part A: who's in the game.** 16 playable countries, plus how the rest of the world is handled.
- **Part B: the 16 country designs.** What makes each one play differently.
- **Part C: new engine mechanics.** The research turned up patterns that repeat across many countries. Those belong in the engine, not as one-off rules.
- **Part D: the map.** Regions, sea zones, chokepoints and bases.
- **Part E: starting situation, events, and the "does it feel like the 80s?" tests.**

You don't need game-design experience to review it. The approval checklist at the end lists the decisions as yes/no or pick-one questions. Anything you don't comment on, I'll treat as accepted.

**Two principles behind every choice:**
1. **Pick countries for the *decisions* they create, not their size.** Brazil is huge but plays passively. Cuba is tiny and is never idle.
2. **Prefer reusable rules to special cases.** If three countries need a similar rule, it becomes a generic trait or mechanic. Each playable country still gets one or two signature traits.

---

# Part A — The Roster

## A.1 The 16 playable countries

| # | Country | Archetype (what makes it unique) | Fun hook in one line |
|---|---|---|---|
| 1 | **United States** | **Networked Hegemon**: unmatched reach, conflicting commitments | You have more ways to shape the world than anyone else. Every shortcut teaches the world what your promises are worth |
| 2 | **Soviet Union** | Oil-funded empire on a clock | Reform before the oil money runs out and the empire walks away |
| 3 | **China** | Poor giant choosing between the market and the Party | Every reform buys growth and loosens your grip |
| 4 | **Japan** | Economic superpower forbidden to be a military one | Win with money, or rewrite your constitution |
| 5 | **West Germany** | Non-nuclear economic giant on the front line | Spine of NATO, tempted by reunification |
| 6 | **Poland** | The bloc's weakest link | A strike can end an empire, or bring the tanks |
| 7 | **Yugoslavia** | Non-aligned federation held together by borrowed money | Courted by both blocs; can shatter into successor states |
| 8 | **Iran** | Revolution at war | Survive an invasion with zeal, mass and the oil lanes |
| 9 | **Iraq** | Over-armed, debt-financed regional bully | Everyone pays you to fight, until the bill comes due |
| 10 | **Saudi Arabia** | The central bank of oil | Militarily weak; can crash or spike the world economy |
| 11 | **Israel** | Small tech state with an arsenal nobody admits exists | Best air force and intel, no strategic depth |
| 12 | **India** | Non-aligned giant with a basement bomb | When to reform, when to test, how to handle two rivals |
| 13 | **Pakistan** | Indispensable frontline proxy and covert proliferator | Make yourself too useful to sanction, and build the bomb meanwhile |
| 14 | **North Korea** | Garrison state that lives on blackmail | Artillery on Seoul, nothing to lose |
| 15 | **South Africa** | Pariah fortress with a secret bomb and the West's minerals | A regime with an expiry date: crack down or transition? |
| 16 | **Cuba** | Tiny island with an expeditionary army on borrowed money | Your power is a loan, and the lender can go bankrupt |

**Coverage check.** Every design archetype from DESIGN §16 has a real counterpart:

| Archetype | Countries |
|---|---|
| Continental power | USSR, China |
| Maritime | USA |
| Resource exporter | Saudi Arabia |
| Small tech state | Israel |
| Unstable regional power | Iran, Iraq |
| Fortress | South Africa, North Korea |
| Trade-dependent | Japan, West Germany |
| Rapidly developing | China, India |

The roster also adds archetypes the original list didn't have:

| New archetype | Country |
|---|---|
| Client state | Cuba |
| Fragile federation | Yugoslavia |
| Bloc weak link | Poland |
| Proxy conduit | Pakistan |

**Geographic spread:**

| Area | Playable countries |
|---|---|
| Europe | 4, plus the USSR |
| Middle East | 4 |
| Asia | 5 |
| Africa | 1 |
| Americas | 2 |

Latin America is the thin spot. Argentina is first on the bench for that reason.

## A.2 Three tiers of countries

The real world has ~170 countries. We can't fully design them all, and we shouldn't try.

| Tier | Who | Simulation | AI | Player can pick? |
|---|---|---|---|---|
| **1. Playable** | The 16 above | Full | Full strategic AI | Yes |
| **2. Major AI** | ~13 important countries (below) | Full, same engine as Tier 1 | Full strategic AI | Not at launch. **Promotion to playable is a data flag** plus polish (signature traits, project, objectives) |
| **3. Minor** | Everyone else that matters | Simplified (see Part C, P1) | Rule-based behaviours | No |
| *Unmodelled* | Tiny or strategically irrelevant states (Ireland, Albania, Jordan, Bangladesh…) | None. Neutral terrain on the map | — | No |

**Tier 2: Major AI countries**

| Country | Why it matters | Key traits (from the generic library, Part C) |
|---|---|---|
| **United Kingdom** | Nuclear (2), GIUK gap, Falklands, Hong Kong, Gibraltar | Financial Hub, Imperial Outposts, Restive Periphery (Ulster) |
| **France** | Independent nuclear spoiler (2), Françafrique, arms to anyone | Arms Bazaar, Imperial Outposts, Breadbasket; posture: Massive Retaliation |
| **East Germany** | Stasi state; trigger for West German reunification and Soviet intervention tests | Command Economy, Siege Mentality |
| **Turkey** | Owns the Bosporus (Soviet Black Sea Fleet exit), NATO hinge, Kurdish insurgency | Chokepoint Warden, Praetorian Army, Restive Periphery |
| **Egypt** | Owns Suez; switched sides to the US; food-import dependent | Chokepoint Warden, Patron's Client (US), Debtor State |
| **Syria** | Proxy master in Lebanon; Soviet client; Iran's Arab ally | Patron's Client (USSR), Proxy Master |
| **Libya** | Petrodollar troublemaker; funds insurgency worldwide; loses wars | Petro-State, Proxy Master |
| **Argentina** | Junta that may gamble on the Falklands; nuclear race with Brazil | Praetorian Army, Breadbasket, Debtor State |
| **Brazil** | Biggest Latin economy and debtor; arms exporter; secret navy nuclear programme | Debtor State, Arms Bazaar, Covert Programme |
| **Mexico** | Oil boom turned debt bomb: **the debt-contagion trigger** | Petro-State, Debtor State |
| **Vietnam** | Battle-hardened army trapped occupying Cambodia; Soviet client; China's enemy | Patron's Client (USSR), Command Economy |
| **South Korea** | Dictatorship turning into a rich democracy; US tripwire | Export Juggernaut, Frontline State |
| **Taiwan** | Exists because nobody settles the question; covert bomb programme | Export Juggernaut, Resource-Poor, Covert Programme |

**Bench (first in line for promotion to playable):** Argentina, France, United Kingdom, Egypt, Vietnam.

**Tier 3: Minor countries with special rules.** These are flagged because the scenario depends on them:

| Country | Why it matters |
|---|---|
| **Afghanistan** | Insurgency magnet. Its regime changes are driven by state, not dates |
| **Lebanon** | Multi-sponsor proxy arena |
| **Kuwait** | Rich, weak, owed money by Iraq |
| **Gulf States** (bloc) | South shore of Hormuz |
| **Yemen** | Bab-el-Mandeb; unification event |
| **Panama** | The canal, with a US Canal Zone handover due by 1999 |
| **Nicaragua / Central America** | US–Soviet proxy arena |
| **Angola** | Oil enclave plus a civil war where South Africa, Cuba, the USSR and the US collide |
| **Zaire** | Half the world's cobalt; patron-abandonment collapse |
| **Ethiopia** | Soviet client with famine and the Eritrean secession |
| **Mongolia** | Soviet garrison buffer |
| **Malaya–Singapore** | Owns Malacca |
| **Indonesia** | Sunda bypass; financial-crisis contagion |
| **Philippines** | US bases |
| **Hungary** | Can open its border and trigger an exodus from East Germany |
| **Romania** | Maverick within the Warsaw Pact |

---

# Part B — The 16 Playable Countries

Every country follows the same layout. Traits marked ★ are **unique signature traits**. The rest come from the generic trait library in Part C. Every trait has an upside and a downside that change decisions.

Starting numbers are collected in **B.17**. They're first-pass values to be tuned in the headless simulation runs.

---

### B.1 United States — Networked Hegemon

*Revised from the approved mechanics report (us-mechanics-report.md §1–§8).*

The US is the most **connected** actor in the world, not the most righteous one. It is neither the designated good guy nor the villain.
- **Advantages:** money, alliances, bases, trade, intelligence, military reach and extensive commitments.
- **Limits:** Initiative, domestic tolerance, economic cost, escalation, and the memory of other countries.

**Central question:** you can usually influence the situation. What are you willing to spend, compromise, tolerate, threaten, destabilise or commit to?

**Profile**
- **Government:** Democracy (`openness` high)
- **Doctrine:** Maritime, Air-Centric secondary
- **Arsenal:** 3, Flexible Response

**Traits** (three; each is a data parameter on a generic primitive):
- ★ **Reserve Currency** (`financial_weight` 0.55). Very cheap credit makes deficits sustainable. That also makes debt-funded wars and buildups tempting. (The world interest-rate lever and finance sanctions are deferred; DESIGN §22.3.)
- ★ **War-of-Choice Aversion** (`c` = 1.0 against the democracy default of 0.5; flavour name "Vietnam Syndrome"). Defending attacked allies carries no extra penalty. Long wars of choice burn Stability fast. Short limited aims, proxies, or nothing.
- **Breadbasket** (food surplus, food export-exposure weight 2.0). Food leverage and food-aid opinion. Food embargoes hit US Prosperity hard.

**Starting data, not traits:**
- **Global reach:** the largest starting set of treaties, bases and standing streams (Part D, D.5).
- **Intelligence:** Intel tech 8.

**National project: Strategic Defense Initiative** (data instance). It has an `observer_alarm` effect: the USSR's threat assessment jumps the moment it starts.

**What the player does:**
- triages allies' requests (accepting a commitment costs Initiative; refusing costs only Opinion)
- picks which crises to engage, at which Involvement Band (DESIGN §11.2)
- balances the budget against domestic patience
- watches each observer's reading of US credibility, which comes from the Event Ledger (DESIGN §21) and differs region by region

**Objectives pool**
- Contain the USSR (no new Soviet-aligned state in Europe, the Gulf or Latin America)
- Soviet regime change or collapse by 2000
- Keep Hormuz and Suez open
- Primacy
- *Counterfactual:* Fortress America (top GDP with no foreign war)

**How it changes after the rival:** if the USSR collapses, there's no victory flag. Defence-burden tolerance, treaty re-evaluation and goal rescoring make "what is American power for now?" a real choice: retrenchment, coalition leadership, integration, primacy or selective intervention (DESIGN §14.9).

**Event hooks**
- Hostage crisis (active at start)
- Election cycle
- Exposed covert proxy (an `openness`-scaled scandal)
- Allies wobbling after a rival collapses (emerges from treaty re-evaluation; no event needed)

**How it plays:** you can do almost anything, almost anywhere, but not everything, everywhere. Every shortcut writes a record. One ally concludes "they protect friends like us." Another concludes "they drop clients like us." Both are right about the cases they saw. Ruthless, restrained, isolationist, interventionist and trade-focused play are all viable, and none is free.

---

### B.2 Soviet Union

- **Government:** Authoritarian (ideology tag)
- **Doctrine:** Mass, Mobile secondary
- **Arsenal:** 3. Declared No First Use, but really Flexible Response. That gap is a deliberate bluff.

**Traits**
- ★ **Petro-Empire.** Oil sales earn the hard currency that pays for grain imports and client subsidies. When the oil price falls, you must choose: cut subsidies to Cuba, Vietnam and Eastern Europe (clients drift away), or cut Welfare (Prosperity crashes).
- ★ **Brezhnev Doctrine.** Gains Legitimacy by keeping Warsaw Pact members aligned. If a member defects and you don't intervene, you take a large Legitimacy and Credibility hit. **Every Polish crisis becomes "invade or lose face."**
- ★ **Gerontocracy.** Leadership changes are likely early. Each change rerolls the personality between hardliner and reformer, so nobody can count on continuity.
- **Command Economy.** Military production up, growth down. Reform is stronger than usual, but each use risks *Nationalist Awakening* in non-Russian regions (Part C, P2).

**National project: Perestroika.** Large growth and economic-tech payoff. It weakens Repression and raises the chance of secession.
- *Alternative project:* Dead Hand (guaranteed second strike).

**Objectives pool**
- Keep the Warsaw Pact intact
- Win in Afghanistan, or leave without the client falling
- Warm-water access (an aligned state on the Gulf or Indian Ocean)
- Economic tech parity with the US
- Never lose a core region
- *Counterfactual:* reconcile with China

**Event hooks**
- Nationalist Awakening
- Grain crisis
- Hardliner coup (August 1991-style)
- Afghan quagmire
- Nuclear accident (Chernobyl-type)
- Breakup into successor states (if Stability collapses for an AI USSR, or the player loses periphery regions)

**How it plays:** the richest "self-driving" story in the game. You start strong and the clock is against you. The oil price, grain imports, the client empire and the reform dilemma all pull at once. A Saudi oil flood in 1986 can do more damage than a NATO army.

---

### B.3 China

- **Government:** Authoritarian
- **Doctrine:** Mass
- **Arsenal:** 2, No First Use (a credible, long-standing posture)

**Traits**
- ★ **Reform and Opening.** Each Reform gives big growth and tech diffusion from trade partners. The cost is rising **Openness**: more foreign intel coverage on you, more inflation, more protest risk. The question is how fast to reform.
- ★ **People's War.** Huge defence and resistance at home. Offensives beyond the border take ×1.5 supply penalties until Military tech 5. Deter by denial, or invest to project power.
- ★ **One China.** Recovering Hong Kong or Taiwan peacefully gives big Legitimacy. Any foreign guarantee to Taiwan drains Legitimacy every turn until you confront it.
- **Command Economy.** Reform gradually removes this trait. That is its own decision.

**National project: Special Economic Zones.** Coastal regions get a GDP multiplier and foreign investment. They also become more prone to unrest and more exposed to espionage.

**Objectives pool**
- Recover Hong Kong and Taiwan
- Top-3 world GDP
- Remove the Three Obstacles (no Soviet forces on the border, Soviets out of Afghanistan, Vietnam out of Cambodia)
- No hegemon in Indochina
- Party survives (Stability never below 30)

**Event hooks**
- Tiananmen-style protests: Openness high, inflation, Legitimacy falling. Crackdown brings Western sanctions. Concession may change the government type.
- Pedagogical war: a limited war against Vietnam
- Taiwan Strait crisis
- Malacca dilemma: China becomes an energy importer around 1993
- Sino-Soviet thaw

**How it plays:** the swing power. You lean toward the US against the USSR while getting rich, and every turn you choose between growth and control.

---

### B.4 Japan

- **Government:** Democracy
- **Doctrine:** Maritime
- **Arsenal:** 0. Latent: Japan could build one very quickly.

**Traits**
- ★ **Article 9.** Military budget above about 1% of GDP costs Legitimacy every turn. Deploying abroad needs a Legitimacy check. Upside: neighbours feel little threat and trust you.
- ★ **Under the Umbrella.** Covered by US deterrence. US bases cause incidents, and a "Normal Nation" path opens only if opinion of the US falls.
- **Export Juggernaut.** Big trade income. A growing trade surplus erodes US opinion until the US issues a Plaza-style ultimatum.
  - Accept: a strong yen, an asset bubble, and crash risk.
  - Refuse: US tariffs.
- **Resource-Poor.** Chokepoint closures hit you double. This makes you the natural financier of Gulf stability.

**National project: Flying Geese aid programme.** Turns the trade surplus into permanent influence in China and Southeast Asia.
- *Note:* the generic Strategic Arsenal project would be very fast for Japan, but it shatters Article 9 and the US alliance.

**Objectives pool**
- Top-2 GDP all campaign
- Keep Malacca and Hormuz open
- Recover the Kurils
- Amend Article 9 while keeping Stability above 60
- *Counterfactual:* independent deterrent

**Event hooks**
- Bubble and crash ("Lost Decade")
- North Korean missile overflight
- Hokkaido scare (Soviet buildup in the Far East)

**How it plays:** the purest economic game in the roster. You have huge money, little military power, and you depend on everyone else's sea lanes.

---

### B.5 West Germany

- **Government:** Democracy
- **Doctrine:** Mobile (forward defence)
- **Arsenal:** 0. Hosts US nuclear weapons.

**Traits**
- ★ **Ostpolitik.** Offer credits and trade to Eastern Bloc states. Gains opinion and lowers tension with the USSR. US opinion falls if you go too far.
- ★ **Non-Nuclear Pledge.** Can't start an arsenal without a huge Legitimacy and alliance penalty. You depend on others' deterrence.
- **Frontline State.** Your regions are the likeliest war zone. Allied garrisons raise your Security. Any European war wrecks your industry first, so your AI always prefers de-escalation.
- **Export Juggernaut.** Big trade income. Sanctions you join hurt you more than they hurt the US.

**National project: Reunification.** Available only if East German Stability is below 25 (or the regime has collapsed) **and** the USSR is weak or consents. Annexes East Germany at half output for several turns. A "unity hangover" follows: Welfare demand spikes.

**Objectives pool**
- Reunify by 2000
- Top-3 world GDP
- No war on German soil
- Lead the European bloc
- *Counterfactual:* neutral reunification (leave NATO for unity)

**Event hooks**
- Peace movement: protests when US missiles are deployed in West Germany
- Pipeline row: US anger at a Soviet gas deal
- Wall crack: the reunification window opens

**How it plays:** you can't win wars and don't want to fight them. You win by economics and patience, and you're watching for the moment the East cracks.

---

### B.6 Poland

- **Government:** Authoritarian
- **Doctrine:** Mass
- **Arsenal:** 0

**Traits**
- ★ **Solidarity.** A parallel workers' movement. Food price rises or Austerity trigger strikes that cut output.
  - Crackdowns work, but bring Western sanctions and a credit cut-off.
  - A Round Table reform can turn you into a democracy peacefully (Part C, P3).
- ★ **Catholic Nation.** The Church is a second source of legitimacy. Repression costs more, and Western "Fund opposition" operations against you get a bonus.
- **Debtor State.** About $25B of Western debt. Western creditors can squeeze you without firing a shot.
- **Command Economy.** Military production up, growth down.

**National project: Shock Therapy.** A few turns of terrible stability, then a growth multiplier and Western trade access. A regime-defining gamble.

**Objectives pool**
- Survive without a Soviet invasion
- Become a democracy without collapse
- Debt below 40%
- Leave the Warsaw Pact
- *Counterfactual:* the China path (stay authoritarian and reach top-2 growth)

**Event hooks**
- Solidarity strikes
- "Fraternal assistance": the Soviet AI chooses between invading, demanding martial law, or doing nothing
- Papal visit
- Debt default

**How it plays:** you start in crisis (Stability 38). It's the best "small country that can topple an empire" seat. Every move is judged in Moscow, Washington and Bonn, and your geography (the Soviet supply line to East Germany runs through you) makes you matter.

---

### B.7 Yugoslavia

- **Government:** Authoritarian (non-aligned)
- **Doctrine:** Defence-in-Depth
- **Arsenal:** 0

**Traits**
- ★ **Fragile Federation.** Each region has its own loyalty (Part C, P2). A crackdown raises loyalty in one region and lowers it in the others. Below Stability 25, regions can declare independence and become new countries.
- ★ **Total National Defence.** Occupiers face double resistance. Weapons are stored regionally, so a secession becomes a war, not a vote.
- **Non-Aligned Leader.** Takes aid from both blocs. Joining either bloc forfeits that.
- **Debtor State.** High debt. IMF austerity is the only fix, and it hits the poorer regions hardest, which strains the federation.

**National project: Brotherhood and Unity.** Slowly raises every region's loyalty. Competes with paying down debt for the money.

**Objectives pool**
- Keep the federation whole through 2000
- Stay non-aligned
- Clear the debt
- Mediate peace deals
- *Counterfactual:* EC membership as a democracy

**Event hooks**
- Succession vacuum: Tito dies (turn 1–2)
- Kosovo riots
- Nationalist strongman
- Secession war, with foreign proxies and a NATO peacekeeping dilemma

**How it plays:** a juggling act. Play both blocs for money, keep the republics loyal, and pay the debt, knowing that pushing any one of these hurts the others. It's also the stress test for the successor-state system.

---

### B.8 Iran

- **Government:** Revolutionary
- **Doctrine:** Mass
- **Arsenal:** 0. Project available, with high detection risk.

**Traits**
- ★ **Revolutionary Zeal.** Casualty war weariness halved; human-wave offensives are cheap. But Legitimacy decays if you go N turns without an offensive. **The regime is pushed to keep attacking past the sensible point.**
- ★ **Embargoed Arsenal.** Readiness decays without black-market spare parts. They're expensive and need an intel op, and when exposed they embarrass buyer and seller (Iran-Contra as a mechanic).
- **Chokepoint Warden (Hormuz).** Can make transit *Contested* rather than *Closed* (Part C, P6). Oil prices rise while you keep exporting. Each use raises tension with whoever escorts the tankers.
- **Petro-State.** Revenue tracks the oil price.

**National project: Proxy Network.** Builds a permanent proxy asset in a foreign region (the Lebanon template). Cheap, deniable insurgency, and it can retaliate on your behalf.

**Objectives pool**
- Survive with the revolutionary regime intact
- Expel foreign navies from the Gulf
- Spread alignment to 2 states
- Never lose Khuzestan
- Threshold nuclear capability by 2000

**Event hooks**
- Tanker War
- Hostage crisis: active at start
- War of the Cities
- Accept the ceasefire ("drinking poison")
- Thermidor: drift toward pragmatism

**How it plays:** you start isolated, purged and likely invaded. You win on zeal and asymmetric pressure, and the hardest decision is when to stop.

---

### B.9 Iraq

- **Government:** Authoritarian
- **Doctrine:** Mass, shifting toward Mobile
- **Arsenal:** 0

**Traits**
- ★ **Bought Army.** Foreign patrons subsidise your military while you fight a common enemy. When the war ends, the subsidy becomes debt and creditors make demands. **Peace becomes a financial crisis.**
- ★ **Pipeline-Dependent.** Oil leaves through Turkey, Syria and Saudi Arabia, so every transit country holds a veto over your income. You must court or coerce them.
- ★ **Republic of Fear.** Very high counterintelligence and cheap crackdowns, but coup risk rises with each one. **Your own intel estimates carry extra error.** The leadership is lied to, so it misjudges enemies.
- **Petro-State.** Revenue tracks the oil price.

**National project: Babylon Arsenal** (reactor, supergun and chemical programme combined). A cheaper deterrent, but highly visible. Rivals with air reach get a pre-emptive strike option (Osirak, Part C, P9).

**Objectives pool**
- Annex Kuwait or secure Gulf access
- Win the war with Iran
- Lead the Arab world
- Clear the debt
- Survive sanctions

**Event hooks**
- Opportunistic invasion: fires only if Iran is weak and purged
- Creditor squeeze: Kuwait, 1990
- Pre-emptive reactor strike
- Uprising after defeat: the encourager's credibility suffers if it doesn't help
- Water cut-off (event only)

**How it plays:** the "my victory bankrupted me" story. You're strong, everyone funds you, and every win digs a deeper financial hole.

---

### B.10 Saudi Arabia

- **Government:** Authoritarian (monarchy)
- **Doctrine:** Air-Centric
- **Arsenal:** 0. Covert missile purchase possible.

**Traits**
- ★ **Swing Producer.** Spare capacity lets you **Restrain** or **Flood** oil output (Part C, P5).
  - Flood: the world price crashes, punishing rival exporters (Iran, Iraq, Libya, the USSR) and your own budget.
  - Restrain: props up the price but cedes market share.
  - **The most powerful economic lever in the game.**
- ★ **Custodian of the Two Holy Mosques.** Big Legitimacy and influence in Muslim states. **Hosting foreign non-Muslim troops steadily erodes Legitimacy** and spawns extremist events. Survival conflicts with legitimacy.
- ★ **Chequebook Diplomacy.** Turns cash into aid, proxy arms or opinion very efficiently. Manpower is capped, so you must buy protection.
- **Rentier State.** Stability tracks Welfare spending. There's no tax lever, so when oil revenue drops you choose between debt, cuts and unrest.

**National project: Strategic Petroline.** Reroutes half your exports to the Red Sea, removing Iran's Hormuz leverage over you.
- *Secret alternative:* Dongfeng Purchase (covert missiles; costly if your patron finds out).

**Objectives pool**
- Keep the oil price inside a band
- No regional hegemon
- Survive with no core region lost
- Wealth top-5
- Lead the Muslim world over Iran
- *Counterfactual:* never host foreign troops

**Event hooks**
- Price war: OPEC quota cheating
- Invitation crisis: an army on your border; do you invite foreign troops in?
- Mosque seizure or Hajj riot
- Patron discovers the missiles

**How it plays:** weak by arms, mighty by markets. Your production choice ripples through every economy on the map, from Moscow to Mexico City.

---

### B.11 Israel

- **Government:** Democracy (coalition politics)
- **Doctrine:** Air-Centric, Mobile secondary
- **Arsenal:** 1, **hidden**. Publicly "not the first to introduce."

**Traits**
- **Covert Programme (Nuclear Opacity).** Your arsenal is hidden from most estimates, but rivals factor in a *suspected* arsenal. Declaring it raises deterrence but costs opinion and US tension. Ambiguity versus clarity.
- ★ **Citizen Army.** Fast, high-quality mobilisation. Every turn at Partial or higher costs heavy GDP, and casualties count double. **Short wars are mandatory.**
- ★ **Long Arm.** Pre-emptive strikes against weapons programmes in range are cheaper. Each one costs diplomatically, and the target gains a revenge goal.
- ★ **Occupation Burden.** Holding non-core regions generates compounding resistance and loss of international opinion. Every conquest becomes a trap.

**National project: Aliyah Absorption.** Turns a migration wave into population, manpower and research. It's triggered when the USSR weakens.
- *Alternative:* Ofek reconnaissance satellite.

**Objectives pool**
- No hostile state acquires an arsenal
- Peace with 2 neighbours
- Never lose a core region
- Expel the hostile proxy from Lebanon
- Highest regional tech in 2 tracks

**Event hooks**
- Pre-emptive strike window
- Lebanon quagmire
- Scud test: retaliate and risk breaking the coalition, or hold back (1991)
- Uprising in occupied territories
- Vanunu-style exposure of the arsenal

**How it plays:** the real version of the fictional Hesperia from DESIGN §16. You see everything and strike precisely, but you can't afford long wars or occupations.

---

### B.12 India

- **Government:** Democracy
- **Doctrine:** Mass
- **Arsenal:** 0, latent

**Traits**
- ★ **License Raj.** Slow, stable growth. The Liberalisation reform is cheap only during a debt or energy crisis. It brings growth plus a Welfare and stability hit. Reform early and painfully, or wait for a crisis?
- **Covert Programme (Bomb in the Basement).** A cheap, covert level-1 arsenal that gives **no deterrence until tested**. Testing brings democracies' sanctions and **an automatic Pakistani test in response**.
- **Non-Aligned Leader.** Trades with everyone and buys Soviet arms on credit. A formal alliance costs Legitimacy.
- Restive periphery (Punjab, Kashmir) handled through region loyalty (Part C, P2).

**National project: Regional Policeman.** Intervene in South Asian minor countries at low global tension (Sri Lanka 1987), with a risk of quagmire.

**Objectives pool**
- Never lose Kashmir
- A tested, credible deterrent by 1998
- No foreign bases or proxies in South Asia
- Top-5 growth after liberalising
- Great-power recognition

**Event hooks**
- War scare from a military exercise (Brasstacks)
- Kashmir uprising
- Forced reform (a 1991-style crisis)
- Assassination

**How it plays:** two rivals (Pakistan, China), a Soviet friendship that can vanish, and two timing decisions that define the game: when to reform and when to test.

---

### B.13 Pakistan

- **Government:** Authoritarian (government type changes through coups)
- **Doctrine:** Defence-in-Depth
- **Arsenal:** 0, covert. Flexible Response (keeps first use open).

**Traits**
- ★ **Conduit State.** When a neighbour hosts a great-power war, proxy aid flows through you and you keep a share. Refugees and militancy build up in your Frontier region every turn the conduit runs.
- ★ **Protected Proliferator.** Penalties for discovery of your arsenal programme are **suspended while a patron needs the conduit**, then **hit all at once** when the war ends. Race to the bomb while you're protected.
- ★ **Strategic Depth.** Cheap opposition-funding operations in Afghanistan and Kashmir. Exposed operations bring blowback and sanctions.
- **Praetorian Army.** A coup changes the government type instead of collapsing the regime, so the army is a stability floor. **You can't cut the military budget below a threshold without coup risk.**

**National project: Kahuta.** A cheap covert arsenal. Also unlocks *Sell Technology* (cash and opinion with revolutionary states, with catastrophic tension if exposed).

**Objectives pool**
- Take or internationalise Kashmir
- A friendly regime in Kabul
- Nuclear parity with India
- No breakaway regions
- Keep US aid flowing for 20+ turns

**Event hooks**
- The conduit opens: active at start
- Pressler-style sanctions
- Forced test: India tested
- Kargil-style incursion
- Coup

**How it plays:** the most distinctive small-power seat in the game. Weak on paper, indispensable in practice. Ride the Afghan war for money and cover, and get the bomb before your usefulness expires.

---

### B.14 North Korea

- **Government:** Revolutionary
- **Doctrine:** Mass
- **Arsenal:** 0, covert

**Traits**
- ★ **Seoul Under the Guns.** Any war with you automatically devastates the Seoul region, so **everyone's war utility against you is heavily penalised**. This also keeps US forces in place and tension permanently high.
- ★ **Patron Balancing.** Aid from **both** the USSR and China, as long as opinion with each stays above a threshold. Losing one is catastrophic. A double-patron version of Patron's Client.
- ★ **Brinkmanship.** Raising tension can extract aid (freeze-for-fuel deals). Being caught cheating zeroes trust.
- **Juche Autarky.** Foreign intel coverage capped and sanctions barely bite, but there's no trade growth and a chronic food deficit.

**National project: Yongbyon–Taepodong.** Covert arsenal plus missile reach, with a missile-export option.

**Objectives pool**
- Regime survival
- Expel US forces from Korea
- A deterrent by 2000
- Extract aid
- *Counterfactual:* reunify on your terms

**Event hooks**
- Succession crisis
- Famine (patron aid lost and food deficit)
- Terror operation against the South's prestige
- Nuclear crisis (the programme is detected)

**How it plays:** you're weak in every economic sense. You survive by being too dangerous to attack and too unpredictable to ignore.

---

### B.15 South Africa

- **Government:** Authoritarian (a narrow-franchise democracy)
- **Doctrine:** Mobile
- **Arsenal:** 0, with a covert programme that can reach level 1 by around 1985

**Traits**
- ★ **Laager State.** Security weighted ×2 and crackdowns are cheap. Each crackdown adds a permanent **sanctions pressure** counter in democracies. Repression now, isolation later.
- ★ **Minerals Vault.** Platinum, chromium, manganese and gold give leverage with industrial importers, so sanctions against you cost the sanctioner too. Your Prosperity tracks mineral prices.
- ★ **Narrow Base.** Manpower and Legitimacy come from about 15% of the population. Casualties count double, and an internal unrest region is always fundable by foreign opposition operations.
- ★ **Embargo Workshop.** Self-sufficient arms industry, so arms embargoes barely bite. Military research costs 25% more because there's no tech sharing.

**National project: Threshold Bomb (covert).** Hidden unless a rival's intel coverage is 70+.
- **Unique option: Disclose & Dismantle.** A one-off move: large Legitimacy and Trust gains, faster sanctions relief, and the arsenal permanently drops to 0. It's the only real-world case of a country building a bomb and giving it up.

**Objectives pool**
- Keep the Namibia buffer until 1990
- Prevent a Soviet-aligned state on your border
- Keep the arsenal undetected until 1990
- Survive to 2000 with Stability above 40 **under any government type** (allows the reform path)
- *Counterfactual:* break the sanctions regime

**Event hooks**
- Sanctions cascade
- Debt standstill
- **Transition crisis** (Part C, P3): reform into a democracy, or total crackdown
- Test site detected

**How it plays:** the regime has an expiry date and you know it. Delay it, manage it, or turn it into a win.

---

### B.16 Cuba

- **Government:** Revolutionary
- **Doctrine:** Mass (territorial militia)
- **Arsenal:** 0

**Traits**
- **Patron's Client (USSR).** Large fixed income from the patron (about 20% of GDP), **as long as the patron can pay**. The patron's crisis becomes yours.
- ★ **Internationalist Army.** Sending volunteers abroad costs half and **gives Legitimacy**. Volunteer deaths abroad add war weariness. Intervention is how the regime stays legitimate.
- ★ **Siege Island.** Invading you is very costly. But you're under permanent embargo from the world's largest economy, so trade income is very low.
- ★ **Doctors not Dollars.** Cheap aid actions that raise opinion across the developing world. Low Prosperity triggers emigration waves (Mariel 1980). You can use them as a lever against the US, at a Legitimacy cost.

**National project: Expeditionary Corps.** Deploy forces overseas on the patron's ships without a navy. Usable as a bargaining chip: withdraw Cuban troops in exchange for Namibian independence.

**Objectives pool**
- Keep 2 allied regimes in Africa alive until 1990
- Survive independent to 2000
- Spread alignment to 1 Latin American state
- Force South Africa out of Namibia
- *Counterfactual:* replace the Soviet patron

**Event hooks**
- Patron withdrawal: the 1991 cliff, with energy collapse and emigration
- Rafter crisis
- Volunteer sinkhole: the recall dilemma

**How it plays:** the purest test of the Initiative promise that a tiny state can be as busy as a superpower. You fight in Africa, needle the US, and pray Moscow stays solvent.

---

### B.17 Starting values (first pass)

**Scales:**
- **Population:** millions, 1980.
- **GDP:** index, USA = 100 (roughly nominal, abstracted).
- **Military:** relative 0–100 (USSR land = 100, US naval and air = 100). Quality comes from the Military tech level.
- **Commodities:** "++" big surplus, "+" surplus, "=" balanced, "−" deficit, "−−" big deficit.
- **Tech:** Military / Economic / Intelligence, 1–10.

| Country | Pop | GDP | Energy | Mat. | Food | Land | Naval | Air | Tech M/E/I | Stab | Arsenal |
|---|---|---|---|---|---|---|---|---|---|---|---|
| USA | 227 | 100 | − | = | ++ | 55 | 100 | 100 | 8/8/8 | 58 | 3 |
| USSR | 265 | 45 | ++ | + | − | 100 | 60 | 75 | 7/4/7 | 55 | 3 |
| China | 981 | 7 | + | + | = | 85 | 15 | 25 | 3/3/4 | 60 | 2 |
| Japan | 117 | 38 | −− | −− | − | 12 | 25 | 20 | 6/9/4 | 78 | 0 |
| West Germany | 61 | 33 | −− | − | − | 30 | 6 | 22 | 7/8/6 | 65 | 0 |
| Poland | 36 | 5 | + | = | − | 18 | 3 | 7 | 5/4/5 | 38 | 0 |
| Yugoslavia | 22 | 3 | − | + | = | 15 | 4 | 5 | 5/4/5 | 55 | 0 |
| Iran | 39 | 3 | ++ | = | − | 25 | 4 | 8 | 3/2/3 | 45 | 0 |
| Iraq | 13 | 2 | ++ | − | − | 22 | 1 | 8 | 4/3/3 | 55 | 0 |
| Saudi Arabia | 10 | 6 | +++ | − | −− | 5 | 2 | 10 | 4/3/2 | 65 | 0 |
| Israel | 4 | 1 | − | − | = | 15 | 2 | 15 | 6/5/7 | 60 | 1 (hidden) |
| India | 697 | 6.5 | − | + | = | 40 | 6 | 12 | 4/3/4 | 55 | 0 |
| Pakistan | 80 | 1 | − | − | = | 18 | 2 | 6 | 3/2/5 | 45 | 0 |
| North Korea | 18 | 0.5 | − | = | −− | 30 | 2 | 6 | 3/1/4 | 65 | 0 |
| South Africa | 29 | 3 | − | ++ | + | 8 | 2 | 6 | 5/4/5 | 55 | 0 |
| Cuba | 10 | 1 | −− | + | − | 6 | 1 | 3 | 3/2/6 | 60 | 0 |

### B.18 AI personality vectors (each value 0–1)

*Refined 2026-10-03 from the behaviour research (design/behaviour-and-voice.md §1.4). Reflex sets for these countries live in `data/reflexes/1980.ron`.*

| Country | Aggr | Risk | Paranoia | Loyalty | Greed | Ideology | Opportunism |
|---|---|---|---|---|---|---|---|
| USA | .45 | .50 | .55 | .65 | .60 | .65 | .60 |
| USSR | .45 | .35 | .85 | .65 | .30 | .80 | .45 |
| China | .35 | .30 | .80 | .30 | .75 | .35 | .45 |
| Japan | .10 | .15 | .45 | .85 | .90 | .20 | .25 |
| West Germany | .10 | .25 | .45 | .80 | .80 | .30 | .40 |
| Poland | .10 | .25 | .75 | .35 | .60 | .30 | .60 |
| Yugoslavia | .20 | .30 | .75 | .30 | .60 | .25 | .45 |
| Iran | .60 | .70 | .90 | .70 | .30 | .90 | .40 |
| Iraq | .80 | .85 | .90 | .15 | .60 | .40 | .90 |
| Saudi Arabia | .10 | .20 | .85 | .60 | .75 | .55 | .30 |
| Israel | .50 | .55 | .90 | .45 | .40 | .40 | .55 |
| India | .30 | .30 | .70 | .50 | .40 | .45 | .35 |
| Pakistan | .50 | .70 | .90 | .35 | .50 | .55 | .80 |
| North Korea | .50 | .70 | .95 | .20 | .50 | .85 | .75 |
| South Africa | .60 | .45 | .90 | .25 | .55 | .60 | .50 |
| Cuba | .60 | .65 | .85 | .90 | .20 | .95 | .45 |

**Transition vectors (data for P3/P11):**
- USSR reformer: .25 / .35 / .55 / .40 / .55 / .45 / .45
- South Africa post-Reform: .15 / .25 / .40 / .55 / .65 / .30 / .35
- Iran post-Thermidor: ideology drifts to about .65
- India post-liberalisation: greed rises to about .60

These are **starting** vectors. Leadership changes and regime transitions shift them (Part C, P3 and P11).

---

# Part C — New Engine Mechanics This Scenario Needs

> **Status after the mechanics report (approved 2026-10-03).** This status overrides the original proposals below:
>
> | # | Status |
> |---|---|
> | P1 Tiers | **Approved, simplified.** Minors run the *same* utility AI with a restricted action set, not a separate rules engine. Bloc minors and unmodelled states are data |
> | P2 Loyalty & Secession | **v0.2, 4th** (largest blast radius) |
> | P3 Regime Transition | **v0.2, 3rd** |
> | P4 Patronage | **Cut.** Replaced by standing Aid/Arms streams (DESIGN §7.2) |
> | P5 Production Policy | **v0.2, 1st** |
> | P6 Contested chokepoints | Deferred. v0.1 has Open/Closed via sea control |
> | P7 World Interest Rate | **Built 2026-10-04** (DESIGN §11.4c), after the crisis pass showed debtor crises had no driver |
> | P8 Outposts | Data: basing treaties at start. Full objects deferred |
> | P9 Counter-proliferation strike | **Absorbed** into the generic `Punitive` war aim (DESIGN §6.6) |
> | P10 Covert Programmes | **v0.2, 2nd** |
> | P11 Leadership change | Deferred. v0.1 uses seeded personality jitter |
> | P12 Event templates | Engine in v0.1 (~6 templates); content is data |
>
> New mechanics from the report (Event Ledger, per-observer reputation, Involvement Bands, permanent transfers, trade depth, implied norms) are in DESIGN.md §7, §11, §14 and §21. References below to "Patron's Client" mean a standing support stream.

The research turned up patterns that repeat across many countries. Rather than hard-coding them per country (which would break future WW1/WW2 scenarios), I propose them as **generic engine mechanics**. Each passes the five-question test from CLAUDE.md.

| # | Mechanic | Used by | Decision it creates | Cost | AI reasoning | Feedback |
|---|---|---|---|---|---|---|
| **P1** | **Country tiers, bloc minors, unmodelled states** (A.2). Minor actors have no budget or Initiative; they run on rules: drift toward the patron giving the most aid or opinion, seek a pact when threatened, accept bases when aligned, resist occupation, host insurgencies, flip in revolutions. **Bloc minors** (e.g., Gulf States, Benelux) represent several small, similar, aligned states as one actor | Whole map | Courting minors becomes a cheap competition arena | — | Minors follow rules, not utility AI | Alignment overlay; minor's "leaning" bar |
| **P2** | **Region Loyalty & Secession.** Each region has a 0–100 loyalty to its owner. Low loyalty means unrest, cheaper foreign insurgency funding, and **declaring independence** below thresholds while the country is in crisis. Seceded regions become new minor countries (or join a neighbour) | USSR, Yugoslavia, Iran, Iraq, India, Pakistan, Ethiopia, UK, Turkey | Crackdown here or concede there; reform versus the risk of opening the lid | Welfare, garrisons, Initiative | Opportunists fund insurgency in low-loyalty enemy regions | Loyalty overlay; secession warnings |
| **P3** | **Regime Transition.** In a domestic crisis a country can be offered a **Transition** choice: *Reform* (government type changes, personality resets, sanctions may lift) or *Crackdown* (survive now, with growing coup and pariah risk). **A managed transition is not a regime collapse:** the player keeps playing under the new government type. Collapse (Stability < 10) is still game over | South Africa, Poland, USSR, Argentina, Iran, South Korea, China | The defining late-game choice for authoritarian regimes | Legitimacy, policy freedom | Personality-weighted: ideological regimes resist reform | A clear crisis prompt with projected outcomes |
| **P4** | **Patronage treaty.** A standing subsidy (money, arms, oil) from patron to client in exchange for alignment, basing and votes. The patron can cut it. The client's economy depends on it | USSR → Cuba, Vietnam, Syria, North Korea, Ethiopia; US → Egypt, Israel, Pakistan; Gulf → Iraq | Patron: is this client worth the money? Client: how far can I defy the patron? | Patron's budget | Patrons weigh strategic value against cost; cuts happen in budget crises | A patronage web on the Alliances overlay |
| **P5** | **Commodity production policy.** Countries with a large export surplus get a standing setting for their main commodity: *Restrain / Normal / Flood*. Swing-producer traits (Saudi Arabia) amplify the effect | Saudi Arabia, Iran, Iraq, USSR, Mexico, Libya | Price versus market share; punishing rivals while hurting yourself | Your own revenue | Greedy AIs maximise revenue; strategic AIs flood to hurt rivals | Price chart with a "because" link |
| **P6** | **Chokepoint states: Open / Contested / Closed.** Contested raises transit costs and prices, with the owner still trading. Closed cuts the routes and gives a near-certain casus belli to dependants | All ~10 chokepoints | A graduated pressure tool instead of on/off | Tension, the owner's own trade | Owners use Contested as leverage; dependants escort or retaliate | Chokepoint icons on the map; trade-route overlay |
| **P7** | **World Interest Rate** (a global market variable). Feeds every country's debt service. Driven by the Reserve Currency holder's monetary stance (*Tight / Neutral / Loose*, a standing US setting) plus global tension | USA (sets it); every debtor feels it | US: fight inflation at home or protect debtor allies | Debtor stability; US opinion | The US AI weighs domestic Prosperity against alliance health | Global rate shown next to the oil price |
| **P8** | **Outposts.** Bases that aren't full regions (Gibraltar, Guantánamo, Diego Garcia, Cam Ranh Bay, Djibouti, Hong Kong…). They give naval basing and supply in a sea zone, plus intel coverage. They can be lost by treaty, event or attack | US, UK, France, USSR | Commitments far from home: defend them or abandon them | Upkeep; a credibility test if attacked | Diversionary AIs target lightly defended outposts (Falklands logic) | Base icons; ticker alerts |
| **P9** | **Counter-proliferation strike.** A generic military action against a *visible* arsenal programme within air range. Sets the programme back; diplomatic penalty for the attacker; the target gains a revenge goal. Israel's *Long Arm* makes it cheaper | Israel (Osirak), anyone with air reach | Strike now, or let them finish | Initiative, opinion, tension | Weighs programme progress × hostility × own arsenal against diplomatic cost | Strike report; visible change in programme progress |
| **P10** | **Covert programmes and declaration.** Arsenal programmes and arsenals can be **undeclared**, hidden from foreign estimates below an intel-coverage threshold. Actions: *Declare* (deterrence up, opinion down) and *Disclose & Dismantle* (trust up, arsenal to 0) | Israel, South Africa, Pakistan, India, North Korea, Iraq, Taiwan, Brazil, Argentina | Ambiguity versus clarity; race versus restraint | Detection risk | The AI factors in *suspected* arsenals from its own estimates | "Suspected programme" markers with confidence |
| **P11** | **Leadership change.** Events (age, assassination, coup, election) replace a country's leader. For AI countries the personality vector drifts. For the player, effects land on Stability and Legitimacy only; the player's strategy isn't overridden | USSR (gerontocracy), Yugoslavia (Tito), North Korea (succession), India and Egypt (assassinations) | Prepare for successions; exploit rivals' transitions | — | Rivals watch for weakness during transitions | Headline plus a revised intel assessment ("New leadership appears reformist") |
| **P12** | **Event templates.** Events are parameterised families (e.g., *Transition Crisis (country)*, *Debt Default (country)*, *Patron Withdrawal (client, patron)*). About 15 templates produce ~40 concrete events, which means less content and more consistency | All | — | — | — | — |

**Considered and rejected:**
- **Water as a resource** (Turkish dams). A one-off. Becomes an event only.
- **A separate chemical-weapons system.** Folded into arsenal programmes as flavour (Iraq).
- **A refugee population simulation.** Refugee waves stay events with stability effects.
- **Treaty deadlines as a system** (the Hong Kong handover, the Panama Canal). Handled as events plus treaties.
- **A separate "Openness" stat.** Already covered by government type plus China's Reform trait.

---

# Part D — The Map

## D.1 Region count: revised from ~70 to ~100

Real geography and a three-tier world need more regions than the abstract estimate in DESIGN §5. **The player-facing complexity barely changes:**
- About 45 of the regions are single-region minors in quiet areas.
- A typical war still involves 1–4 contested region pairs.

The extra regions mainly buy **real borders** and **future WW1/WW2 reuse**.

**Trim candidates** if you want closer to 85: merge Norway and Denmark, Sweden into Alpine Neutrals, Bulgaria into Romania, Thailand into Indochina, Somalia into Ethiopia, Canada into one "Arctic North" region, Mongolia into Siberia.

## D.2 Land regions (~102)

**North America (10)**

| Owner | Regions |
|---|---|
| USA | Northeast · South & Gulf Coast · Midwest & Plains · West Coast & Mountain · Alaska |
| Canada | Canada |
| Mexico | Mexico |
| Nicaragua (T3) | Central America (rest of the isthmus abstracted) |
| Panama | Panama — **Panama Canal** |
| Cuba | Cuba |

**South America (5)**
- Gran Colombia (owner: Venezuela)
- Brazil
- Andes & Chile
- Argentina
- South Atlantic Islands (UK; the Falklands)

**Western Europe (11)**

| Owner | Regions |
|---|---|
| UK | Great Britain (Ulster as an insurgency flag; GIUK) |
| France | Northern France · Southern France |
| Spain | Iberia (UK Gibraltar outpost → **Gibraltar**) |
| Italy | Italy |
| Bloc minor | Benelux |
| Norway | Norway |
| Denmark | Denmark — **Danish Straits** |
| Sweden | Sweden |
| Bloc minor | Alpine Neutrals (Switzerland, Austria) |
| Greece | Greece |

**Central & Eastern Europe (9)**

| Owner | Regions |
|---|---|
| West Germany | North German Plain · Rhine & South |
| East Germany | East Germany |
| Poland | Poland West · Poland East |
| Others | Czechoslovakia · Hungary · Romania · Bulgaria |

**Balkans & Turkey (5)**

| Owner | Regions |
|---|---|
| Yugoslavia | Slovenia–Croatia · Bosnia–Montenegro · Serbia–Macedonia–Kosovo |
| Turkey | Thrace & Straits — **Bosporus** · Anatolia |

**Soviet Union (8)**
- Baltic–Belarus
- Ukraine
- Central Russia & Kola (Moscow, Northern Fleet)
- Volga–Urals
- Caucasus
- Central Asia
- Siberia
- Far East (Pacific Fleet)

**Middle East & North Africa (18)**

| Owner | Regions |
|---|---|
| Egypt | Nile · Sinai & Canal — **Suez** |
| Others | Libya · Maghreb (owner: Algeria) · Israel · Lebanon · Syria |
| Iraq | Basra · Baghdad · Kurdistan–Mosul |
| Kuwait | Kuwait |
| Saudi Arabia | Al-Hasa · Najd–Hejaz |
| Bloc minor | Gulf States — **Hormuz** (south shore) |
| Yemen | Yemen — **Bab-el-Mandeb** |
| Iran | Gulf Coast & Khuzestan — **Hormuz** (north shore) · Central Plateau · Northwest |

**South Asia (7)**

| Owner | Regions |
|---|---|
| Afghanistan | Afghanistan |
| Pakistan | Indus · Frontier |
| India | Punjab–Kashmir · Gangetic Plain · Bengal & Northeast · Deccan & South |

**East & Southeast Asia (17)**

| Owner | Regions |
|---|---|
| China | Manchuria · North China · Yangtze · South Coast (UK Hong Kong outpost) · Western Frontier |
| Others | Mongolia · North Korea · South Korea |
| Japan | Honshu · Kyushu–Okinawa — **Japanese Straits** |
| Others | Taiwan · Vietnam · Indochina (Cambodia–Laos, Vietnamese-occupied) · Thailand |
| Malaysia | Malaya–Singapore — **Malacca** |
| Others | Indonesia (Sunda bypass) · Philippines |

**Sub-Saharan Africa (11)**

| Owner | Regions |
|---|---|
| South Africa | Highveld · Cape |
| South Africa (occupied, non-core) | Namibia |
| Others | Angola · Zaire |
| Bloc minor | Frontline States |
| Ethiopia | Highlands · Eritrea |
| Others | Somalia · West Africa (owner: Nigeria) |
| Bloc minor | French Africa (French-garrisoned) |

**Oceania (1)**
- Australia

Region boundaries follow lasting seams (old imperial provinces, mountain ranges, historic fronts), so later scenarios can reassign ownership. For example, the Iraqi regions match the Ottoman provinces of 1914, and Germany's five regions work for every era.

## D.3 Chokepoints (10)

| Chokepoint | Owner region(s) | Who depends on it |
|---|---|---|
| **Hormuz** | Iran Gulf Coast / Gulf States | Japan, Western Europe, the Gulf exporters themselves |
| **Suez** | Egypt Sinai & Canal | Europe ↔ Asia trade, the Soviet Black Sea Fleet route south |
| **Bab-el-Mandeb** | Yemen / Ethiopia Eritrea (+ French Djibouti outpost) | Suez traffic, Saudi Petroline exports |
| **Bosporus** | Turkey Thrace | Soviet Black Sea Fleet exit |
| **Gibraltar** | UK outpost (Iberia) / Maghreb | Mediterranean access |
| **Danish Straits** | Denmark | Soviet Baltic Fleet exit |
| **Malacca** | Malaya–Singapore / Indonesia | Japan, Korea, Taiwan, and China after 1993 |
| **Panama Canal** | Panama (US Canal Zone outpost until the 1999 handover) | US two-ocean navy, Atlantic ↔ Pacific trade |
| **Japanese Straits** | Japan | Soviet Pacific Fleet exit |
| **GIUK Gap** *(naval gap, no land owner)* | Controlled by naval strength (UK, Norway, US Iceland outpost) | Soviet Northern Fleet to the Atlantic |

The **Cape route** is a sea zone, not a chokepoint. It becomes vital when Suez or Bab-el-Mandeb close, which gives South Africa value. The **Taiwan Strait** is a contested sea zone.

## D.4 Sea zones (~25)

**Atlantic and Arctic**
- North Atlantic
- Norwegian–Barents Sea
- North Sea
- Baltic Sea
- Western Atlantic
- Caribbean & Gulf of Mexico
- Mid-Atlantic
- South Atlantic West
- South Atlantic East (Cape route)

**Mediterranean and Black Sea**
- Western Mediterranean
- Eastern Mediterranean
- Black Sea

**Indian Ocean and Gulf**
- Red Sea
- Persian Gulf
- Arabian Sea
- Indian Ocean
- Bay of Bengal

**Pacific**
- Java Sea & Malacca approaches
- South China Sea
- East China & Yellow Sea
- Sea of Japan
- Okhotsk & NW Pacific
- Central Pacific
- Eastern Pacific
- Southwest Pacific

## D.5 Outposts (starting)

| Holder | Outposts |
|---|---|
| **US** | Panama Canal Zone, Guantánamo (inside Cuba!), Diego Garcia (with the UK), Subic/Clark (Philippines), Okinawa (treaty), Keflavík (Iceland), Guam/Hawaii (Pacific) |
| **UK** | Gibraltar, Hong Kong (handover treaty 1997), Cyprus bases, Falklands (a full region) |
| **France** | Djibouti, French Africa garrisons |
| **USSR** | Cam Ranh Bay (Vietnam), Lourdes SIGINT station (Cuba), Aden (South Yemen), Dahlak (Ethiopia) |

---

# Part E — Starting Situation, Events, and Tests

## E.1 Starting situation (January 1980)

**Global**
- Oil price **high** (just after the 1979 shock).
- World interest rate **rising** (US tight money).
- **Global Tension: Condition 4** (détente collapsing).

**Wars and occupations in progress**
- USSR occupying **Afghanistan** against an insurgency. Pakistan's conduit is open; US, Saudi and Chinese proxy aid is available.
- **Vietnam** occupying Cambodia. China–Vietnam tension very high.
- **South Africa** occupying Namibia, with a border war in Angola. **Cuban** troops are in Angola and Ethiopia.

**Crises**
- **Iran:** revolutionary government; **hostage crisis active** with the US (a Legitimacy drain on the US until resolved); army purged (low readiness); US arms embargo. Iraq–Iran tension 70.
- **Poland:** high debt, rising food prices, Stability 38.
- **Yugoslavia:** Tito's death fires a leadership event within the first 2 turns.

**Diplomatic landscape**
- Egypt–Israel peace in force; Egypt suspended from the Arab League.
- US grain embargo on the USSR in force.

**Treaties at start**
- NATO; Warsaw Pact
- US–Japan; US–South Korea
- China–North Korea; USSR–India friendship; USSR–Vietnam; USSR–Iraq
- Comecon; European Community
- US patronage of Egypt and Israel; USSR patronage of Cuba, Vietnam and Syria
- ANZUS

**What is deliberately *not* scripted:** the Iran–Iraq war, the Falklands, martial law in Poland, the Gulf War, the Soviet collapse, German reunification, the Yugoslav breakup and the Asian financial crisis. Each one has a **trigger** that makes it *likely* under historical conditions. None of them is guaranteed.

## E.2 Scenario event pool (from ~15 templates)

**Economy**

| Event | Trigger |
|---|---|
| Price War / Netback Flood | OPEC quota cheating plus Saudi market share falling. Saudi Arabia gets the option to flood |
| **Debt Default + Contagion** | Debt > threshold, interest rate shock, export price fall. Each trade partner and fellow debtor then rolls a default check |
| Hyperinflation | Debt > 60% and deficit for 4 turns |
| Bubble & Crash | Easy credit, strong currency, high asset growth (Japan) |
| Financial Contagion | Short-term debt plus a currency peg plus a neighbour's default (Asia 1997) |

**Politics**

| Event | Trigger |
|---|---|
| Solidarity Strikes | Poland, high food price, Stability < 40 |
| Nationalist Awakening | Low-loyalty region plus Reform, or Stability < 40 |
| **Transition Crisis** *(template)* | Stability < 35 for 6 turns, low Legitimacy. Choice: Reform or Crackdown |
| Hardliner Coup | Stability < 25, reformist leadership, unhappy military |
| Leadership Change *(template)* | Age, assassination or election |
| Thermidor | War over plus Prosperity rising (Iran) |
| Mass Protests | Openness high, inflation, falling Legitimacy (China) |
| Border Opening & Exodus | Hungary opens, East German Stability < 40 |
| Hostage Crisis | Revolutionary regime with tension > 60 against a superpower |

**Military and security**

| Event | Trigger |
|---|---|
| Opportunistic Invasion window | A neighbour that is revolutionary, purged and unstable |
| Diversionary Claim | Authoritarian, Stability < 40, a weakly held non-core claim (the Falklands) |
| Tanker War | Iran–Iraq at war, Gulf financiers backing Iraq |
| War of the Cities | Both sides have missiles and the front is static |
| Pre-emptive Strike window | A visible arsenal programme within reach of an enemy with air power |
| Exercise War Scare | Large exercise near a border with tension > 50 |
| Strait Crisis | Taiwan democratising plus a US arms sale |
| Outpost Grab | A lightly defended outpost near an unstable authoritarian state |
| Fraternal Assistance | A Warsaw Pact member's crisis while the Brezhnev Doctrine is active |
| Coalition Restraint | A coalition member is attacked by a party trying to split the coalition (the 1991 Scud test) |

**Nuclear**

| Event | Trigger |
|---|---|
| Programme Detected | Covert programme plus a rival with coverage above the threshold |
| Forced Test | A rival tests, and you hold an untested arsenal |
| Disclose & Dismantle | Offered to an undeclared arsenal state during a transition |
| False Alarm | Global Tension at Condition 1 |

**Patronage and humanitarian**

| Event | Trigger |
|---|---|
| **Patron Withdrawal** *(template)* | Patron budget crisis or reform turn. Client energy and Prosperity crash |
| Patron Abandonment | Cold War tension falls; aid stops (Zaire) |
| Peace Dividend | Main rival collapsed, low tension |
| Famine | Drought, food deficit, war or no aid |
| Emigration / Refugee Wave | Low Prosperity, or war next door |
| Nuclear Accident | Low economic tech plus high energy production |

## E.3 Plausibility targets (for headless AI-vs-AI runs)

These are **balance targets, not scripts**. Across hundreds of headless runs, the 1980s should *feel* plausible without repeating history:

| Outcome | Target frequency across runs |
|---|---|
| Iran–Iraq war breaks out | 50–80% |
| USSR ends intact / reformed / collapsed | Each ≥ 15%, none > 60% |
| Soviet intervention in Poland | 10–40% |
| German reunification | 20–60% |
| Yugoslav breakup | 30–70% |
| At least one oil price swing of ±50% | > 70% |
| Direct US–USSR war | < 10% |
| Any nuclear use | 1–5% |
| Terminal nuclear winter | < 0.5% |
| Each playable country tops the objective scores in AI-only runs | ≥ 3% each, none > 20% |
| At least one regime transition somewhere | > 90% |

If a run of 500 campaigns shows, say, the USSR collapsing 95% of the time or never, the model is wrong and gets tuned. These targets give the simulation-first approach a concrete definition of "working."

---

# Approval Checklist — resolved 2026-10-03

*The user skipped the checklist, so items 1–3 and 6–8 were accepted by default. Item 4 was superseded by the mechanics report (see the Part C status table). Item 5: P3 is approved in principle and scheduled for v0.2; only collapse is game over. The original questions are kept below for the record.*

1. **Roster:** the 16 playable countries in A.1. Swap anyone? (The bench is Argentina, France, UK, Egypt, Vietnam.)
2. **Tiers:** Playable / Major AI / Minor / Unmodelled (A.2), with promotion as a data flag.
3. **Region count** raised from ~70 to ~100 (D.1), or trim toward ~85 using the listed merges?
4. **New mechanics P1–P12** (Part C). Approve all, or reject specific ones?
5. **Regime Transition (P3):** a *managed* transition (reform) lets the player keep playing under the new government. Only *collapse* is game over. OK?
6. **Unscripted history (E.1):** no historical events are guaranteed; they're only made likely by their triggers. OK?
7. **Plausibility targets (E.3)** as the definition of "the simulation works."
8. **Starting values (B.17, B.18):** fine as a first pass to be tuned by headless runs?
