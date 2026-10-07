//! Generic AI inputs computed from an observer view (DESIGN §14.3, §14.9).
//! Nothing here names a country; everything comes from view data.

use sim_core::view::ForeignView;
use sim_core::{CountryId, Government, ObserverView, TreatyKind};

/// Share of an ally's estimated power counted toward our side, before
/// scaling by our reading of its Credibility(Back). Mirrors the domestic
/// Security driver.
pub const ALLY_WEIGHT: f64 = 0.5;

/// Estimated reach of `f` into the observer's area, 0.2–1 (mirrors the
/// simulation's rule with estimated naval and air strength).
pub fn reach(view: &ObserverView, f: &ForeignView) -> f64 {
    reach_into(f, &view.own.area)
}

/// Estimated reach of `f` into `area` (e.g. a protector's into the area of
/// the state it protects), 0.2–1.
pub fn reach_into(f: &ForeignView, area: &Option<String>) -> f64 {
    match (&f.area, area) {
        (Some(a), Some(b)) if a != b => {
            let total = (f.forces.land.value + f.forces.naval.value + f.forces.air.value).max(1e-9);
            let projection = f.military.value * (f.forces.naval.value + f.forces.air.value) / total;
            (sim_core::domestic::REACH_FLOOR + projection / sim_core::domestic::PROJECTION_FULL)
                .clamp(sim_core::domestic::REACH_FLOOR, 1.0)
        }
        _ => 1.0,
    }
}

/// The observer's own reach into `f`'s area (its projection if elsewhere).
pub fn reach_to(view: &ObserverView, f: &ForeignView) -> f64 {
    match (&view.own.area, &f.area) {
        (Some(a), Some(b)) if a != b => sim_core::domestic::projection(&view.own.forces),
        _ => 1.0,
    }
}

/// How hostile `f` is toward the observer, 0–1: at war, its tension with
/// us or its negative opinion of us (mirrors `domestic::hostility`).
pub fn hostility_toward_us(view: &ObserverView, f: &ForeignView) -> f64 {
    if view.wars.iter().any(|w| {
        matches!((w.side_of(view.observer), w.side_of(f.id)), (Some(a), Some(b)) if a != b)
    }) {
        return 1.0;
    }
    (f.tension / 100.0).max(-f.their_opinion_of_us / 100.0).clamp(0.0, 1.0)
}

/// Our side's military power: own power plus allies and guarantors, each
/// discounted by how much we believe they would come and by their own
/// hostility toward us (a defender at odds with us is paper; mirrors the
/// Security driver's `pledge_worth`).
pub fn defence(view: &ObserverView) -> f64 {
    let me = view.observer;
    let allied: f64 = view
        .others
        .iter()
        .filter(|f| {
            view.treaties.iter().any(|t| match t.kind {
                TreatyKind::DefensiveAlliance => t.involves(me) && t.involves(f.id),
                TreatyKind::Guarantee => t.a == f.id && t.b == me,
                _ => false,
            })
        })
        .map(|f| {
            f.military.value * ALLY_WEIGHT * f.credibility_back / 100.0
                * sim_core::domestic::pledge_worth(hostility_toward_us(view, f))
                * reach(view, f)
        })
        .sum();
    view.own.power() + allied + sim_core::domestic::NUCLEAR_WEIGHT * view.own.arsenal as f64
}

/// Is `c` the attacker in a war still being fought (public)?
pub fn is_attacker(view: &ObserverView, c: CountryId) -> bool {
    view.wars.iter().any(|w| w.attacker == c)
}

/// How recently `c` started a war, 0–1, as the observer's ledger records it
/// (public Coercion entries for declared wars), fading over
/// [`crate::war::WAR_MEMORY_TURNS`] like the attacker's own memory of it.
pub fn aggression_record(view: &ObserverView, c: CountryId) -> f64 {
    view.ledger
        .iter()
        .filter(|e| {
            e.actor == c && e.kind == sim_core::EntryKind::Coercion && e.cause_code == sim_core::CauseCode::WarDeclared
        })
        .map(|e| (1.0 - (view.turn as i32 - e.turn) as f64 / crate::war::WAR_MEMORY_TURNS).clamp(0.0, 1.0))
        .fold(0.0, f64::max)
}

/// Estimated arsenal reach into our area (level 2+ reaches everywhere).
pub fn nuclear_reach(view: &ObserverView, f: &ForeignView) -> f64 {
    match (&f.area, &view.own.area) {
        (Some(a), Some(b)) if a != b && f.arsenal < 2 => 0.0,
        _ => 1.0,
    }
}

/// Did `c` use nuclear weapons (in the observer's ledger)?
pub fn used_nuclear(view: &ObserverView, c: CountryId) -> bool {
    view.ledger
        .iter()
        .any(|e| e.actor == c && e.cause_code == sim_core::CauseCode::NuclearStrike)
}

/// Strongest arsenal standing behind `target`: its own, or a guarantor's
/// or ally's discounted by our reading of their Credibility(Back).
pub fn nuclear_cover(view: &ObserverView, target: &ForeignView) -> f64 {
    let own = target.arsenal as f64;
    let protectors = view
        .others
        .iter()
        .filter(|p| {
            p.id != target.id
                && view.treaties.iter().any(|t| match t.kind {
                    TreatyKind::DefensiveAlliance => t.involves(p.id) && t.involves(target.id),
                    TreatyKind::Guarantee => t.a == p.id && t.b == target.id,
                    _ => false,
                })
        })
        .map(|p| p.arsenal as f64 * p.credibility_back / 100.0)
        .fold(0.0, f64::max);
    own.max(protectors)
}

/// Threat from `f` as it would be without our own arsenal (what giving it
/// up would expose us to).
pub fn threat_without_arsenal(view: &ObserverView, f: &ForeignView) -> f64 {
    let own = (defence(view) - sim_core::domestic::NUCLEAR_WEIGHT * view.own.arsenal as f64).max(1.0);
    threat_against(view, f, own)
}

/// How threatening `f` is to the observer, 0–100: estimated power share
/// against our side (own + allies) × hostility × paranoia (DESIGN §14.3).
/// Reach arrives with regions.
pub fn threat(view: &ObserverView, f: &ForeignView) -> f64 {
    (military_threat(view, f) + subversion(view, f)).clamp(0.0, 100.0)
}

/// Threat from `f`'s armed forces (and arsenal) against our defence.
pub fn military_threat(view: &ObserverView, f: &ForeignView) -> f64 {
    threat_against(view, f, defence(view).max(1.0))
}

/// Regime threat independent of armies: a hostile revolutionary state
/// within reach can subvert a non-revolutionary regime, the more so the
/// weaker that regime's legitimacy at home (revolution exported to Iraq's
/// Shia, the Gulf monarchies, Central America). Visible state only.
pub fn subversion(view: &ObserverView, f: &ForeignView) -> f64 {
    use sim_core::Government::Revolutionary;
    if f.government != Revolutionary || view.own.government == Revolutionary {
        return 0.0;
    }
    let hostility = (f.tension / 100.0).max(-f.their_opinion_of_us / 100.0).clamp(0.0, 1.0);
    let fragility = (1.2 - view.own.legitimacy / 100.0).clamp(0.2, 1.2);
    (SUBVERSION_THREAT * hostility * fragility * reach(view, f) * (0.5 + view.own.personality.paranoia)).clamp(0.0, 100.0)
}

/// Scale of the subversion threat (threat points at full hostility, a
/// fragile regime and an average paranoia).
pub const SUBVERSION_THREAT: f64 = 30.0;

/// Threat from `f`'s conventional forces alone against our defence: what
/// a war could remove. A war destroys armies, not an arsenal; the arsenal's
/// weight in a war decision is the catastrophic-risk term (D74).
pub fn conventional_threat(view: &ObserverView, f: &ForeignView) -> f64 {
    threat_parts(view, f, defence(view).max(1.0), false)
}

fn threat_against(view: &ObserverView, f: &ForeignView, own: f64) -> f64 {
    threat_parts(view, f, own, true)
}

fn threat_parts(view: &ObserverView, f: &ForeignView, own: f64, nuclear: bool) -> f64 {
    let arsenal = if nuclear {
        sim_core::domestic::NUCLEAR_WEIGHT * f.arsenal as f64 * nuclear_reach(view, f)
    } else {
        0.0
    };
    let theirs = f.military.value.max(0.0) * reach(view, f) + arsenal;
    let share = theirs / (own + theirs);
    let hostility = (f.tension / 100.0).max(-f.their_opinion_of_us / 100.0).clamp(0.0, 1.0);
    (100.0 * share * hostility * (0.5 + view.own.personality.paranoia)).clamp(0.0, 100.0)
}

/// The most threatening other country and its threat score.
pub fn top_threat(view: &ObserverView) -> Option<(&ForeignView, f64)> {
    view.others
        .iter()
        .map(|f| (f, threat(view, f)))
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.id.cmp(&a.0.id)))
}

/// Same as [`top_threat`] but ignoring one country (e.g. the proposer).
pub fn top_threat_excluding(view: &ObserverView, exclude: CountryId) -> Option<(&ForeignView, f64)> {
    view.others
        .iter()
        .filter(|f| f.id != exclude)
        .map(|f| (f, threat(view, f)))
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.id.cmp(&a.0.id)))
}

pub fn foreign(view: &ObserverView, id: CountryId) -> Option<&ForeignView> {
    view.others.iter().find(|f| f.id == id)
}

/// Public tension between any two countries.
pub fn pair_tension(view: &ObserverView, a: CountryId, b: CountryId) -> f64 {
    if a == b {
        return 0.0;
    }
    if a == view.observer {
        return foreign(view, b).map_or(0.0, |f| f.tension);
    }
    if b == view.observer {
        return foreign(view, a).map_or(0.0, |f| f.tension);
    }
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    view.pair_tensions
        .iter()
        .find(|(x, y, _)| *x == lo && *y == hi)
        .map_or(0.0, |t| t.2)
}

/// Highest tension `c` has with anyone other than `except`.
pub fn max_tension_of(view: &ObserverView, c: CountryId, except: CountryId) -> f64 {
    let ids = std::iter::once(view.observer).chain(view.others.iter().map(|f| f.id));
    ids.filter(|&x| x != c && x != except)
        .map(|x| pair_tension(view, c, x))
        .fold(0.0, f64::max)
}

/// Affinity −1..1 from alignment tags and government type, mirroring the
/// simulation's rule (same bloc +1, rival blocs −1, else same government +0.5).
pub fn affinity(a_align: &Option<String>, a_gov: Government, b_align: &Option<String>, b_gov: Government) -> f64 {
    match (a_align, b_align) {
        (Some(x), Some(y)) if x == y => 1.0,
        (Some(_), Some(_)) => -1.0,
        _ if a_gov == b_gov => 0.5,
        _ => 0.0,
    }
}

pub fn affinity_with(view: &ObserverView, f: &ForeignView) -> f64 {
    affinity(&view.own.alignment, view.own.government, &f.alignment, f.government)
}

/// Ideological distance 0..1.
pub fn ideological_distance(view: &ObserverView, f: &ForeignView) -> f64 {
    (1.0 - affinity_with(view, f)) / 2.0
}

pub fn has_treaty(view: &ObserverView, kind: TreatyKind, a: CountryId, b: CountryId) -> bool {
    view.treaties.iter().any(|t| {
        std::mem::discriminant(&t.kind) == std::mem::discriminant(&kind)
            && ((t.a == a && t.b == b) || (!directed(kind) && t.a == b && t.b == a))
    })
}

fn directed(kind: TreatyKind) -> bool {
    matches!(kind, TreatyKind::Guarantee | TreatyKind::Basing)
}

/// Is the observer protected by an alliance or a guarantee?
pub fn is_protected(view: &ObserverView) -> bool {
    let me = view.observer;
    view.treaties.iter().any(|t| match t.kind {
        TreatyKind::DefensiveAlliance => t.involves(me),
        TreatyKind::Guarantee => t.b == me,
        _ => false,
    })
}

/// Most relevant ledger entries the observer has seen about `actor`, as
/// short text: own experiences first, then the most recent.
pub fn precedents(view: &ObserverView, actor: CountryId, k: usize) -> Vec<String> {
    let mut seen: Vec<_> = view
        .ledger
        .iter()
        .filter(|e| e.actor == actor && e.grade.and_then(|g| g.score()).is_some())
        .collect();
    seen.sort_by(|a, b| {
        let own = |e: &&sim_core::LedgerEntry| e.counterpart == view.observer;
        own(b).cmp(&own(a)).then(b.turn.cmp(&a.turn)).then(a.id.cmp(&b.id))
    });
    seen.into_iter()
        .take(k)
        .map(|e| {
            let whom = if e.counterpart == view.observer {
                "us".to_string()
            } else {
                foreign(view, e.counterpart).map_or_else(|| format!("#{}", e.counterpart.0), |f| f.code.clone())
            };
            format!("{:?} toward {whom}: {} (turn {})", e.grade.unwrap(), e.cause, e.turn)
        })
        .collect()
}
