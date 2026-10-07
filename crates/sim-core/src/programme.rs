//! Covert arsenal programmes and declaration (scenario P10, v0.2).
//!
//! - A programme costs a share of the spending pool every turn and advances
//!   with Military tech; on completion it yields an **undeclared** arsenal.
//! - Undeclared arsenals and active programmes are hidden from observers
//!   whose coverage of the owner is below the era's covert threshold, so
//!   they deter only those who can see them (ambiguity vs clarity).
//! - Hostile observers who can see a programme may expose it publicly: a
//!   Proliferation norm violation (claimants face tests) and an opinion hit.
//! - Declaring makes an arsenal public; Disclose & Dismantle gives it up for
//!   trust. A punitive strike sets a programme back.

use crate::diplomacy::DiplomaticEvent;
use crate::ids::CountryId;
use crate::intel;
use crate::ledger::{self, CauseCode, EntryKind, Grade, LedgerLog, NormTag, Visibility};
use crate::opinion::OpinionModifier;
use crate::orders::Order;
use crate::world::WorldState;

/// Progress per turn at Military tech 5 with an economy of at least
/// [`CAPABLE_GDP`] (100 = a new arsenal level).
pub const PROGRESS_RATE: f64 = 2.5;
/// Economy size at which a programme runs at full speed; smaller economies
/// progress at √(GDP ÷ this): a bomb needs an industrial base.
pub const CAPABLE_GDP: f64 = 3.0;

/// How fast a country can run a programme, 0–1: tech and industrial base.
pub fn capability(c: &crate::country::Country) -> f64 {
    (c.military_tech as f64 / 5.0) * (c.gdp / CAPABLE_GDP).sqrt().min(1.0)
}
/// Share of the spending pool a programme consumes each turn.
pub const PROGRAMME_COST: f64 = 0.04;
/// Per-turn chance that a hostile observer who can see a programme or an
/// undeclared arsenal makes it public.
pub const EXPOSURE_CHANCE: f64 = 0.1;
/// Programme progress destroyed by each turn of punitive strike against it.
pub const STRIKE_SETBACK: f64 = 25.0;

fn modifier(source: &str, value: f64, decay: f64) -> OpinionModifier {
    OpinionModifier {
        source: source.into(),
        value,
        decay,
        memory: None,
    }
}

/// Can `observer` see `owner`'s covert nuclear activity?
pub fn sees(state: &WorldState, observer: CountryId, owner: CountryId) -> bool {
    observer == owner
        || state.country(owner).programme_exposed
        || intel::coverage(state.country(observer), state.country(owner)) >= state.params.covert_visibility
}

/// The arsenal level `observer` believes `owner` has.
pub fn visible_arsenal(state: &WorldState, observer: CountryId, owner: CountryId) -> u8 {
    let c = state.country(owner);
    if c.arsenal_declared || sees(state, observer, owner) {
        c.arsenal
    } else {
        0
    }
}

/// Programme progress `observer` can see (None if none or hidden).
pub fn visible_programme(state: &WorldState, observer: CountryId, owner: CountryId) -> Option<f64> {
    let c = state.country(owner);
    c.programme.filter(|_| sees(state, observer, owner))
}

/// Orders: start or stop a programme, declare, disclose and dismantle.
pub fn apply(state: &mut WorldState, actor: CountryId, order: Order) -> Result<Option<DiplomaticEvent>, String> {
    match order {
        Order::StartProgramme => {
            let c = state.country_mut(actor);
            if c.programme.is_some() || c.arsenal >= 3 {
                return Err("programme already running or arsenal at maximum".into());
            }
            c.programme = Some(0.0);
            Ok(None) // Covert: no public event.
        }
        Order::StopProgramme => {
            state.country_mut(actor).programme = None;
            Ok(None)
        }
        Order::DeclareArsenal => {
            let c = state.country_mut(actor);
            if c.arsenal == 0 || c.arsenal_declared {
                return Err("no undeclared arsenal".into());
            }
            c.arsenal_declared = true;
            c.programme_exposed = true;
            let ids: Vec<CountryId> = state.ids().filter(|&o| o != actor).collect();
            for o in ids {
                state
                    .opinions
                    .add(o, actor, modifier("declared a nuclear arsenal", -20.0, 0.5));
                if crate::domestic::hostility(state, o, actor) >= 0.2 {
                    state.tension.add(actor, o, 10.0);
                }
            }
            Ok(Some(DiplomaticEvent::ArsenalDeclared { country: actor }))
        }
        Order::DiscloseAndDismantle => {
            let c = state.country_mut(actor);
            if c.arsenal == 0 && c.programme.is_none() {
                return Err("nothing to dismantle".into());
            }
            c.arsenal = 0;
            c.strike_capacity = 0.0;
            c.programme = None;
            c.arsenal_declared = true;
            c.programme_exposed = true;
            let ids: Vec<CountryId> = state.ids().filter(|&o| o != actor).collect();
            for o in ids {
                state.opinions.add(o, actor, modifier("gave up its bomb", 20.0, 0.2));
            }
            Ok(Some(DiplomaticEvent::ArsenalDismantled { country: actor }))
        }
        _ => Err("not a programme order".into()),
    }
}

/// Programme progress (its cost is taken from the spending pool by the
/// economy); completion yields an undeclared arsenal level.
pub fn advance(state: &mut WorldState, events: &mut Vec<DiplomaticEvent>) {
    for c in state.countries.iter_mut() {
        let Some(p) = c.programme else { continue };
        let p = p + PROGRESS_RATE * capability(c);
        if p >= 100.0 {
            let first = c.arsenal == 0;
            c.arsenal = (c.arsenal + 1).min(3);
            c.strike_capacity = crate::nuclear::full_capacity(c.arsenal);
            if first {
                c.arsenal_declared = false;
            }
            c.programme = None;
            events.push(DiplomaticEvent::ProgrammeCompleted { country: c.id });
        } else {
            c.programme = Some(p);
        }
    }
}

/// Exposure: a hostile observer who can see covert nuclear activity may make
/// it public. Uses the world RNG only when something is there to expose.
pub fn expose(state: &mut WorldState, events: &mut Vec<DiplomaticEvent>) {
    let ids: Vec<CountryId> = state.ids().collect();
    for &owner in &ids {
        let c = state.country(owner);
        let covert = !c.programme_exposed && (c.programme.is_some() || (c.arsenal > 0 && !c.arsenal_declared));
        if !covert {
            continue;
        }
        let watcher = ids.iter().copied().find(|&o| {
            o != owner
                && state.opinions.opinion(o, owner) < 0.0
                && intel::coverage(state.country(o), state.country(owner)) >= state.params.covert_visibility
        });
        let Some(by) = watcher else { continue };
        if state.roll_unit() >= EXPOSURE_CHANCE {
            continue;
        }
        state.country_mut(owner).programme_exposed = true;
        for &o in &ids {
            if o != owner {
                state
                    .opinions
                    .add(o, owner, modifier("secret weapons programme", -15.0, 0.5));
            }
        }
        events.push(DiplomaticEvent::ProgrammeExposed { country: owner, by });
    }
}

/// Ledger effects: exposure is a public Proliferation violation (norm
/// claimants face tests); dismantling is an honoured Proliferation entry
/// that raises trust in the dismantler.
pub fn process_ledger(state: &mut WorldState, events: &[DiplomaticEvent], log: &mut Vec<LedgerLog>) {
    for e in events {
        match *e {
            DiplomaticEvent::ProgrammeExposed { country, by } => {
                ledger::record(
                    state,
                    country,
                    by,
                    EntryKind::Coercion,
                    None,
                    0.0,
                    0.5,
                    false,
                    Visibility::Public,
                    CauseCode::ProgrammeExposed,
                    "secret weapons programme exposed",
                    log,
                );
                ledger::report_violation(state, country, NormTag::Proliferation, log);
            }
            DiplomaticEvent::ArsenalDismantled { country } => {
                ledger::record(
                    state,
                    country,
                    country,
                    EntryKind::Norm(NormTag::Proliferation),
                    Some(Grade::Honoured),
                    1.0,
                    1.5,
                    false,
                    Visibility::Public,
                    CauseCode::ArsenalDismantled,
                    "disclosed and dismantled its arsenal",
                    log,
                );
            }
            // A punitive strike on a programme sets it back and claims the
            // norm for the striker (counter-proliferation, P9 → Punitive).
            DiplomaticEvent::FrontReport { war, .. } => {
                let Some(w) = state.wars.find(war).cloned() else {
                    continue;
                };
                if w.aim != crate::war::WarAim::Punitive {
                    continue;
                }
                if let Some(p) = state.country(w.defender).programme {
                    state.country_mut(w.defender).programme = Some((p - STRIKE_SETBACK).max(0.0));
                    ledger::create_norm(
                        state,
                        w.attacker,
                        NormTag::Proliferation,
                        w.defender,
                        "struck a weapons programme",
                        log,
                    );
                }
            }
            _ => {}
        }
    }
}
