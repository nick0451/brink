//! Stability and its four drivers (DESIGN §9.1, implementation plan §4).

use serde::{Deserialize, Serialize};

use crate::country::{Country, Government};
use crate::diplomacy::TreatyKind;
use crate::ids::CountryId;
use crate::reputation::{self, RepKind};
use crate::world::WorldState;

const STABILITY_RATE: f64 = 0.25;
const LEGITIMACY_RATE: f64 = 0.05;
/// Security = FLOOR + SPAN × defence ÷ (defence + hostile pressure): 75
/// with no hostile power, 42.5 when evenly matched, 26 when outweighed 3:1.
const SECURITY_FLOOR: f64 = 10.0;
const SECURITY_SPAN: f64 = 65.0;
/// Share of an ally's power counted toward our defence, before scaling by
/// how much we believe they would actually come (Credibility(Back)).
const ALLY_WEIGHT: f64 = 0.5;
/// Prosperity lost per unit of (trade lost ÷ GDP) × trade exposure.
const LOST_TRADE_PROSPERITY: f64 = 200.0;
const AID_PROSPERITY: f64 = 300.0;
const AID_PROSPERITY_CAP: f64 = 15.0;
const ADJUSTMENT_DECAY: f64 = 1.0;
/// Legitimacy lost per turn at Full or Total mobilization while at peace
/// (DESIGN §6.2: "needs justification").
const UNJUSTIFIED_MOBILIZATION: f64 = 1.0;
/// Military budget share justified with no hostile pressure at all, and the
/// extra justified per unit of hostile pressure share H/(D+H) (N8).
pub const BASE_JUSTIFIED_SHARE: f64 = 0.10;
pub const THREAT_JUSTIFIED_SHARE: f64 = 0.5;
/// Legitimacy-target points per unit of unjustified burden share, scaled by
/// (0.5 + openness): open societies notice the bill sooner.
pub const BURDEN_LEGITIMACY: f64 = 60.0;
/// Rally effect (DESIGN §11.3): moderate tension with a named enemy raises
/// Legitimacy by up to this much (full at tension 70)...
pub const RALLY: f64 = 8.0;
/// ...and tension beyond 70 erodes Prosperity by this much per point.
const BRINK_PROSPERITY: f64 = 0.3;

/// Highest tension `me` has with a hostile country (its named enemy).
pub fn enemy_tension(state: &WorldState, me: CountryId) -> f64 {
    state
        .ids()
        .filter(|&h| h != me && hostility(state, h, me) >= 0.2)
        .map(|h| state.tension.get(h, me))
        .fold(0.0, f64::max)
}

/// Legitimacy rally from a named enemy (DESIGN §11.3).
pub fn rally(enemy_tension: f64) -> f64 {
    RALLY * ((enemy_tension - 40.0) / 30.0).clamp(0.0, 1.0)
}

/// Defence-burden tolerance (N8): military spending above what the biggest
/// perceived threats justify costs Legitimacy. Arms given away count toward
/// the burden. Produces peace-dividend pressure when a rival fades, with no
/// event.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DefenceBurden {
    /// Budget share the hostile pressure justifies.
    pub justified: f64,
    /// Military budget share plus this turn's arms outflow ÷ revenue.
    pub actual: f64,
    /// Points off the Legitimacy target.
    pub penalty: f64,
}

pub fn justified_share(pressure_share: f64) -> f64 {
    BASE_JUSTIFIED_SHARE + THREAT_JUSTIFIED_SHARE * pressure_share.clamp(0.0, 1.0)
}


pub fn defence_burden(state: &WorldState, me: CountryId) -> DefenceBurden {
    let b = security_breakdown(state, me);
    let defence = b.own + b.allies.iter().map(|a| a.1).sum::<f64>();
    let pressure: f64 = b.hostile.iter().map(|h| h.1).sum();
    let share = if defence + pressure > 1e-9 {
        pressure / (defence + pressure)
    } else {
        0.0
    };
    let c = state.country(me);
    let revenue = (c.gdp * c.tax_rate).max(1e-9);
    let arms = crate::diplomacy::arms_value(crate::diplomacy::StreamKind::Arms, c.arms_out) / revenue;
    let justified = justified_share(share);
    let actual = c.budget.military + arms;
    DefenceBurden {
        justified,
        actual,
        penalty: BURDEN_LEGITIMACY * (actual - justified).max(0.0) * (0.5 + c.openness),
    }
}

/// The visible terms behind a country's Prosperity driver (DESIGN §9.4:
/// "a four-driver breakdown"). They sum to the unclamped Prosperity value.
/// Prosperity per unit of quarterly growth.
pub const GROWTH_PROSPERITY: f64 = 1500.0;

pub fn prosperity_terms(c: &Country) -> Vec<(&'static str, f64)> {
    vec![
        ("baseline", 50.0),
        ("growth", GROWTH_PROSPERITY * c.last_growth),
        ("welfare spending", 60.0 * (c.budget.welfare - 0.25)),
        ("debt burden", -30.0 * (c.debt_ratio() - 0.8).max(0.0)),
        (
            "lost trade",
            -LOST_TRADE_PROSPERITY * c.trade_exposure * c.trade_lost / c.gdp,
        ),
        ("integration adjustment", -c.adjustment_shock),
        ("inflation", -c.inflation_hit),
        ("debt service", -c.debt_service_hit),
        (
            "foreign aid",
            (AID_PROSPERITY * c.aid_in / c.gdp).min(AID_PROSPERITY_CAP),
        ),
    ]
}

/// The visible inputs behind a country's Security driver (DESIGN §9.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SecurityBreakdown {
    /// Own military power.
    pub own: f64,
    /// Allies and guarantors: power × [`ALLY_WEIGHT`] × their credibility
    /// in our eyes × (1 − their hostility toward us). A bloc state counts
    /// only defenders that back its regime ([`backs`]). A defender that is
    /// hostile to us is paper: it still appears in `hostile` in full.
    pub allies: Vec<(CountryId, f64)>,
    /// Hostile pressure: power × hostility × reach, for each country with
    /// any hostility (treaty defenders included).
    pub hostile: Vec<(CountryId, f64)>,
    pub value: f64,
}

/// How hostile `h` is toward `me`, 0–1: bilateral tension or `h`'s
/// negative opinion of us, whichever is higher.
pub fn hostility(state: &WorldState, h: CountryId, me: CountryId) -> f64 {
    if state.wars.at_war(h, me) {
        return 1.0;
    }
    (state.tension.get(h, me) / 100.0)
        .max(-state.opinions.opinion(h, me) / 100.0)
        .clamp(0.0, 1.0)
}

/// Power-equivalent of each arsenal level in threat and deterrence sums
/// (DESIGN §11.4: "threat calculations" and "the deterrence effect").
pub const NUCLEAR_WEIGHT: f64 = 20.0;

/// Arsenal reach: level 2+ reaches everywhere (missiles and bombers);
/// level 1 only its own area.
pub fn nuclear_reach(state: &WorldState, h: CountryId, me: CountryId) -> f64 {
    let (a, b) = (state.country(h), state.country(me));
    match (&a.area, &b.area) {
        (Some(x), Some(y)) if x != y && a.arsenal < 2 => 0.0,
        _ => 1.0,
    }
}

/// Reach of countries in another area with no projection forces.
pub const REACH_FLOOR: f64 = 0.2;
/// Naval + air combat value at which a country reaches everywhere.
pub const PROJECTION_FULL: f64 = 40.0;

/// How far forces can project beyond their own area, 0.2–1, from naval
/// and air power (DESIGN §14.9: "power counts only within reach").
pub fn projection(f: &crate::country::Forces) -> f64 {
    (REACH_FLOOR + (f.naval.value() + f.air.value()) * f.readiness_factor() / PROJECTION_FULL).clamp(REACH_FLOOR, 1.0)
}

/// Whether `h` can bring force to bear on `me`: full within the same area
/// (data tag), otherwise its projection. Countries without an area tag are
/// in reach of everyone (fixtures). Regions replace this later.
pub fn reach(state: &WorldState, h: CountryId, me: CountryId) -> f64 {
    let (a, b) = (state.country(h), state.country(me));
    match (&a.area, &b.area) {
        (Some(x), Some(y)) if x != y => projection(&a.forces),
        _ => 1.0,
    }
}

/// Countries pledged to defend `me`: alliance partners and guarantors.
pub fn defenders(state: &WorldState, me: CountryId) -> Vec<CountryId> {
    let mut v: Vec<CountryId> = state
        .diplomacy
        .treaties
        .iter()
        .filter_map(|t| match t.kind {
            TreatyKind::DefensiveAlliance => t.other(me),
            TreatyKind::Guarantee if t.b == me => Some(t.a),
            _ => None,
        })
        .collect();
    v.sort();
    v.dedup();
    v
}

/// Whether a defender underwrites a bloc regime. A bloc state's security
/// rests on defenders of its own camp or its own kind of government: a patron
/// that reformed and left the camp (the Sinatra doctrine of 1989) keeps its
/// treaty on paper but no longer props up the old regime. Unaligned states
/// count every defender. Public inputs only (alignment and government), so
/// the AI reads the same rule from its view.
pub fn backs(me_bloc: Option<&String>, me_gov: Government, ally_bloc: Option<&String>, ally_gov: Government) -> bool {
    match me_bloc {
        Some(bloc) => ally_bloc == Some(bloc) || ally_gov == me_gov,
        None => true,
    }
}

/// A patron is a defender stronger than the client: only a patron's backing
/// is conditional on the client's regime ([`backs`]). A weaker ally (a bloc
/// patron's own new partner, say) counts as an ally whatever its politics.
pub fn is_patron(ally_power: f64, my_power: f64) -> bool {
    ally_power > my_power
}

/// Whether `ally` still counts toward `me`'s security: any non-patron does;
/// a patron only while it backs the regime.
pub fn bloc_backing(state: &WorldState, me: CountryId, ally: CountryId) -> bool {
    let (m, a) = (state.country(me), state.country(ally));
    !is_patron(a.power(), m.power()) || backs(m.alignment.as_ref(), m.government, a.alignment.as_ref(), a.government)
}

/// A defender cannot be both our shield and our threat: its pledge is worth
/// only what its disposition toward us allows (a power at odds with us will
/// not come), while its hostility is pressure in full. One country, one
/// side of the ledger.
pub fn pledge_worth(hostility: f64) -> f64 {
    (1.0 - hostility).clamp(0.0, 1.0)
}

pub fn security_breakdown(state: &WorldState, me: CountryId) -> SecurityBreakdown {
    let own = state.country(me).power() + NUCLEAR_WEIGHT * state.country(me).arsenal as f64;
    let allies: Vec<(CountryId, f64)> = defenders(state, me)
        .into_iter()
        .filter(|&a| bloc_backing(state, me, a))
        .map(|a| {
            let belief = reputation::credibility(state, me, a, RepKind::Back) / 100.0;
            let worth = pledge_worth(hostility(state, a, me));
            (a, state.country(a).power() * ALLY_WEIGHT * belief * worth * reach(state, a, me))
        })
        .collect();
    let hostile: Vec<(CountryId, f64)> = state
        .ids()
        .filter(|&h| h != me)
        .map(|h| {
            let c = state.country(h);
            let force =
                c.power() * reach(state, h, me) + NUCLEAR_WEIGHT * c.arsenal as f64 * nuclear_reach(state, h, me);
            (h, force * hostility(state, h, me))
        })
        .filter(|x| x.1 > 0.0)
        .collect();
    let defence = own + allies.iter().map(|a| a.1).sum::<f64>();
    let pressure: f64 = hostile.iter().map(|h| h.1).sum();
    let ratio = if defence + pressure > 1e-9 {
        defence / (defence + pressure)
    } else {
        1.0
    };
    SecurityBreakdown {
        own,
        allies,
        hostile,
        value: (SECURITY_FLOOR + SECURITY_SPAN * ratio).clamp(0.0, 100.0),
    }
}

/// Legitimacy lost per unit of growth-trend gap behind the leading economy
/// (the regimes that stagnated while their rivals grew: the demonstration
/// effect), and the weight of security in legitimacy (protection is part of
/// a state's claim to rule, performance the larger part).
pub const DEPRIVATION: f64 = 1500.0;
pub const SECURITY_LEGITIMACY: f64 = 0.35;

/// Legitimacy pressure from falling behind the rival bloc: a state in a
/// bloc compares its growth with the largest economy of the other bloc
/// (systemic competition; the East watching the West). Unaligned states
/// aren't in the race.
pub fn falling_behind(state: &WorldState, c: &Country) -> f64 {
    let Some(bloc) = c.alignment.as_ref() else { return 0.0 };
    let rival = state
        .countries
        .iter()
        .filter(|o| o.active && o.alignment.as_ref().is_some_and(|a| a != bloc))
        .max_by(|a, b| a.gdp.total_cmp(&b.gdp));
    rival.map_or(0.0, |r| DEPRIVATION * (r.growth_trend - c.growth_trend).max(0.0))
}

/// Stability target from the regime's standing (`base`, the weighted
/// drivers) and the war weariness it carries. Weariness spends the regime's
/// margin above collapse in proportion to how close the war is to
/// exhaustion: at [`war::EXHAUSTION`](crate::war::EXHAUSTION), when the war
/// system forces peace, the margin is gone. So a long war wears a regime
/// down into crisis and to the peace table, not past collapse while it is
/// still fighting (F1; Iran accepting the 1988 ceasefire). Collapse still
/// comes from the drivers themselves: a lost economy, lost security.
///
/// Two properties follow. Weariness alone can never collapse a regime whose
/// base is at or above the collapse line (the margin is clamped at zero, so
/// a failing regime is not steadied by war either). And regime stability is
/// now tied to `EXHAUSTION`: retuning that threshold rescales this cost too.
pub fn stability_target(base: f64, war_weariness: f64) -> f64 {
    let margin = (base - crate::transition::COLLAPSE_STABILITY).max(0.0);
    (base - margin * war_weariness / crate::war::EXHAUSTION).clamp(0.0, 100.0)
}

pub fn run(state: &mut WorldState) {
    let behind: Vec<f64> = state.countries.iter().map(|c| falling_behind(state, c)).collect();
    let security: Vec<f64> = state.ids().map(|id| security_breakdown(state, id).value).collect();
    let burden: Vec<f64> = state.ids().map(|id| defence_burden(state, id).penalty).collect();
    let enemy: Vec<f64> = state.ids().map(|id| enemy_tension(state, id)).collect();
    let at_war: Vec<bool> = state.ids().map(|id| state.wars.is_belligerent(id)).collect();
    for (((((c, sec), war), burden), enemy), behind) in state
        .countries
        .iter_mut()
        .zip(security)
        .zip(at_war)
        .zip(burden)
        .zip(enemy)
        .zip(behind)
    {
        if !war && c.forces.mobilization.level() >= 2 {
            c.legitimacy = (c.legitimacy - UNJUSTIFIED_MOBILIZATION).max(0.0);
        }
        c.prosperity = (prosperity_terms(c).iter().map(|t| t.1).sum::<f64>()
            - BRINK_PROSPERITY * (enemy - 70.0).max(0.0))
        .clamp(0.0, 100.0);
        c.adjustment_shock = (c.adjustment_shock - ADJUSTMENT_DECAY).max(0.0);
        c.security = sec;

        let legitimacy_target = (1.0 - SECURITY_LEGITIMACY) * c.prosperity + SECURITY_LEGITIMACY * c.security
            - burden
            + rally(enemy)
            - behind;
        c.legitimacy = (c.legitimacy + (legitimacy_target - c.legitimacy) * LEGITIMACY_RATE).clamp(0.0, 100.0);

        let (wp, ws, wl) = c.government.driver_weights();
        let target = stability_target(wp * c.prosperity + ws * c.security + wl * c.legitimacy, c.war_weariness);
        c.stability = (c.stability + (target - c.stability) * STABILITY_RATE).clamp(0.0, 100.0);
    }
}
