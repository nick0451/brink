# Voice hooks from the crisis passes (backlog)

Threads that crisis passes 2–5 (D62–D102) turned into real simulation state. Each one is a candidate trigger for the satirical voice under CLAUDE.md principle 14 and the depth limit (principle 15, "voiced"). They are **drafts for the voice library** (`data/voice/lines.ron`, V3); none is wired yet.

Every hook names its **trigger** (the sim state or event that fires it), the **contradiction** the Ledger exposes (the Ledger is the straight man), and a **draft line** in the Black voice. Guardrails (design/behaviour-and-voice.md §2.3) apply: the joke targets offices and institutions, never victims, peoples or faiths; no lines on casualties, atrocities, famine or nuclear use; nuclear moments stay silent.

| # | Hook | Trigger (sim) | Contradiction | Draft line (Black) |
|---|---|---|---|---|
| 1 | The guarantee to the aggressor | A guarantee accepted for a state with a recorded `WarDeclared` or a pressed claim (now rare after D78; fires when it still happens) | We guaranteed the peace of the country most likely to break it | "The pledge stands: we will defend them from the consequences of their own plans." |
| 2 | Sanctions nobody remembers starting | A sanction in force 20+ turns whose original cause has faded (D80 counterfactual says re-impose = no, kept by goal/solidarity) | The policy outlived its reason | "Asked why the measures continue, the ministry cited the measures." |
| 3 | Loyalty sanctions | An ally joins a sanction with `shared_grievance` ≈ 0 (D69: rare now; still fires via reflexes, e.g. JPN `provisional_solidarity`) | Punishing a country we have no quarrel with, on principle | "Our grievance with Havana is that Washington has one." |
| 4 | The forgiven debt | `DebtForgiven` (D96): the creditor writes off a war loan to a state that still threatens a third party | Generosity as containment | "Riyadh forgives the debt. The debt was the cheapest army it ever bought." |
| 5 | The held debt | `DebtHeld` against a debtor in distress (D96/D98), the grudge deepening each quarter | Insisting on repayment from the only buyer of your protection | "Kuwait reminds Baghdad of its obligations. Baghdad is making a list of its own." |
| 6 | The cartel's prisoner's dilemma | A producer breaks ranks under money pressure (D102 reason "we need the cash now") and the others answer | Everyone loses together, on schedule | "OPEC reaffirms quota discipline, effective immediately after this quarter's exceptions." |
| 7 | The arms race that renews itself | `ArmsBuildUp` renewal (D82/D83): each side's build-up renews the other's grudge | Defence spending justified by the other side's defence spending | "The threat assessment has been revised upward, following our previous revision upward." |
| 8 | The war fought to punish | D91 "punishing the aggressor": a defender refuses a white peace while on the aggressor's soil | The defended becomes the invader, for justice | "Having been invaded, we feel it is only fair to return the visit." |
| 9 | The purge that saves the regime | `took_power` purge (D89): command collapses after a revolution; the army is weaker, the regime safer | Security bought by disarming yourself | "Loyalty in the officer corps has never been higher. Neither has the vacancy rate." |
| 10 | The enemy of my enemy's invoice | `fund_war` (D94): a state funds the war against its main threat, then holds the loan | Charity with interest | "The Kingdom's support for Iraq's struggle was unconditional. The repayment schedule is attached." |
| 11 | The grudge that won't fade | A historical grudge pinned at the bloc floor −10 (D81) after decades of nothing | Hostility with no remaining cause but membership | "Relations remain frosty for reasons both sides consider too obvious to state." |
| 12 | The free rider's dividend | D79/D16 context: a client whose patron's guarantee lets it cut defence | Security outsourced, savings declared | "Defence cuts announced. The Americans have been informed they are increasing theirs." |
| 13 | The war loan as weapon | A creditor's held claim on a state that later attacks it (watch for it; D98 + D96) | Lending to the army that will invade you | "The loan was secured against the borrower's future conduct. The borrower has defaulted on the conduct." |

## Notes for the voice pass
- **Ledger first:** hooks 1, 2, 4, 5 and 10 are direct ledger contradictions; prefer them (principle 14).
- **Statistics:** hooks 6 and 7 work best with a number from the sim (quota exceptions this year; consecutive upward revisions).
- **Silence:** none of these touch casualty screens; hook 8 must not appear on a G3+ casualty screen (behaviour-and-voice.md gravity gate).
- **Liveness check (principle 15):** before writing final lines, confirm each trigger fires in measured runs (`brink diagnose`), e.g. hook 5 fired in 1 of 20 runs at D98, hook 13 not yet observed.
