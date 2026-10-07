//! War decisions (DESIGN §14.9, §11.2): involvement-band choice at a crisis,
//! hold / withdraw / escalate, peace, war declaration and mobilization.
//! Everything reads the observer's view: estimated power, public treaties,
//! visible involvements and its own reputation readings.
//!
//! ```text
//! U(band) = Stake·ΔP(success) + RepDelta(band)
//!         − Cost·(0.5+greed) − DomesticCost − Tension·(1−aggression)
//!         − EscalationRisk·(1−risk) − Entanglement·(1−risk)
//! ```

use sim_core::view::ForeignView;
use sim_core::war::{band_allocation, band_stake, DEFENCE_MULT};
use sim_core::{
    CountryId, DecisionKind, Mobilization, ObserverView, Side, StreamKind, TreatyKind, Visibility, War, WarAim,
};

use crate::inputs::{affinity_with, foreign, threat, top_threat};
use crate::reflexes::{self, power_share, Parties, GREAT_POWER_SHARE};
use crate::Score;

/// Share of own strength sent per turn as proxy arms.
pub const PROXY_ARMS_SHARE: f64 = 0.04;
/// Value of pressing a full-weight claim by war.
pub const CLAIM_WAR_VALUE: f64 = 20.0;
/// Turns over which the memory of our last war on a country fades.
pub const WAR_MEMORY_TURNS: f64 = 40.0;
/// A change of band must beat staying put by this much (no flip-flopping).
pub const BAND_HYSTERESIS: f64 = 5.0;
/// The nuclear catastrophic-risk term never falls below this share of its
/// Major-war value, whatever the aim: a nuclear state's own use trigger is
/// "losing badly", and any front can be pushed that far.
pub const NUCLEAR_RISK_FLOOR: f64 = 0.25;

/// How far a war with `aim` threatens the target's survival, 0–1, as the
/// attacker can judge it from public facts. It is the aim's offensive
/// weight (committed power × intensity, what drives front progress and so
/// the defender's "losing badly" nuclear trigger) relative to a Major war:
/// Major 1.0, Limited about 0.4, Punitive about 0.25, never below
/// [`NUCLEAR_RISK_FLOOR`]. A target visibly in crisis reads any war as
/// existential, so against it the share is 1. This is the
/// stability–instability paradox: parity makes limited conflict thinkable,
/// not total war (Kargil, the Falklands), and the floor keeps it a gamble.
pub fn existential_share(aim: WarAim, target: &ForeignView) -> f64 {
    let cornered = matches!(
        target.stability_band,
        Some(sim_core::view::StabilityBand::Collapse | sim_core::view::StabilityBand::Crisis)
    );
    if cornered {
        return 1.0;
    }
    (aim.offensive_weight() / WarAim::Major.offensive_weight()).clamp(NUCLEAR_RISK_FLOOR, 1.0)
}

/// One involvement option: a band, and for band 2 whether it is covert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BandOption {
    pub band: u8,
    pub covert: bool,
}

impl BandOption {
    pub const ALL: [BandOption; 6] = [
        BandOption { band: 0, covert: false },
        BandOption { band: 1, covert: false },
        BandOption { band: 2, covert: false },
        BandOption { band: 2, covert: true },
        BandOption { band: 3, covert: false },
        BandOption { band: 4, covert: false },
    ];

    pub fn label(self) -> String {
        match (self.band, self.covert) {
            (0, _) => "band 0 (stay out)".into(),
            (1, _) => "band 1 (coerce)".into(),
            (2, true) => "band 2 (covert proxy)".into(),
            (2, false) => "band 2 (proxy)".into(),
            (3, _) => "band 3 (limited intervention)".into(),
            _ => "band 4 (major intervention)".into(),
        }
    }
}

/// Estimated power of a country from the observer's view.
pub fn est_power(view: &ObserverView, c: CountryId) -> f64 {
    if c == view.observer {
        view.own.power()
    } else {
        foreign(view, c).map_or(0.0, |f| f.military.value)
    }
}

/// Estimated power committed by one side of a war.
pub fn side_power(view: &ObserverView, war: &War, side: Side) -> f64 {
    war.side(side).map(|b| est_power(view, b.country) * b.allocation).sum()
}

/// Chance a side ends ahead, from an attack ÷ defence ratio (logistic in
/// ln ratio).
fn win_chance(ratio: f64) -> f64 {
    1.0 / (1.0 + (-1.5 * ratio.max(1e-6).ln()).exp())
}

/// Chance `side` comes out ahead if `extra` power is added to it.
fn side_chance(view: &ObserverView, war: &War, side: Side, extra: f64) -> f64 {
    let mut att = side_power(view, war, Side::Attacker) * war.ground_mult(Side::Attacker);
    let mut def = side_power(view, war, Side::Defender) * war.ground_mult(Side::Defender);
    match side {
        Side::Attacker => att += extra * war.ground_mult(Side::Attacker),
        Side::Defender => def += extra * war.ground_mult(Side::Defender),
    }
    let p = win_chance(att / def.max(1e-6));
    match side {
        Side::Attacker => p,
        Side::Defender => 1.0 - p,
    }
}

fn formal_commitment(view: &ObserverView, to: CountryId) -> bool {
    let me = view.observer;
    view.treaties.iter().any(|t| match t.kind {
        TreatyKind::DefensiveAlliance => t.involves(me) && t.involves(to),
        TreatyKind::Guarantee => t.a == me && t.b == to,
        _ => false,
    })
}

/// Our defensive word is engaged only when the friend is the one attacked:
/// alliances and guarantees are contingency orders against an attack on
/// it (the ledger opens its Back test for the defender's allies only), not
/// a promise to join its own war of choice.
fn commitment_engaged(view: &ObserverView, war: &War, side: Side) -> bool {
    side == Side::Defender && formal_commitment(view, war.leader(side))
}

fn funds(view: &ObserverView, to: CountryId) -> bool {
    view.streams.iter().any(|s| s.from == view.observer && s.to == to)
}

/// How much the observer cares about `side` winning, 0–1, and why.
pub fn stake(view: &ObserverView, war: &War, side: Side) -> f64 {
    let friend = war.leader(side);
    let enemy = war.leader(side.other());
    let mut v: f64 = 0.0;
    if commitment_engaged(view, war, side) {
        v = v.max(1.0);
    }
    if funds(view, friend) {
        v = v.max(0.6);
    }
    if let Some(e) = foreign(view, enemy) {
        v = v.max(threat(view, e) / 100.0);
    }
    if let Some(f) = foreign(view, friend) {
        v = v.max(0.3 * affinity_with(view, f).max(0.0));
    }
    v.min(1.0)
}

/// The side the observer favours and its stake, if any.
pub fn favoured_side(view: &ObserverView, war: &War) -> Option<(Side, f64)> {
    let a = stake(view, war, Side::Attacker);
    let d = stake(view, war, Side::Defender);
    let (side, s) = if d >= a {
        (Side::Defender, d)
    } else {
        (Side::Attacker, a)
    };
    (s >= 0.15).then_some((side, s))
}

/// Can the observer get arms to `to`? Mirrors the simulation's route rule
/// using public treaties and estimated navies.
pub fn arms_route(view: &ObserverView, to: CountryId) -> bool {
    let at_war: Vec<&War> = view.wars.iter().filter(|w| w.side_of(to).is_some()).collect();
    if at_war.is_empty() {
        return true;
    }
    let me = view.observer;
    if at_war
        .iter()
        .any(|w| w.side_of(me).is_some() && w.side_of(me) == w.side_of(to))
    {
        return true;
    }
    let enemies: Vec<CountryId> = at_war
        .iter()
        .flat_map(|w| {
            let s = w.side_of(to).expect("belligerent");
            w.side(s.other()).map(|b| b.country).collect::<Vec<_>>()
        })
        .collect();
    let based = view
        .treaties
        .iter()
        .any(|t| t.kind == TreatyKind::Basing && t.a == me && (t.b == to || !enemies.contains(&t.b)));
    if based {
        return true;
    }
    let own = view.own.forces.naval.value() * view.own.forces.readiness_factor();
    let enemy = enemies
        .iter()
        .filter_map(|&e| foreign(view, e))
        .map(|f| f.forces.naval.value)
        .fold(0.0, f64::max);
    own >= 0.5 * enemy
}

fn my_involvement(view: &ObserverView, war: &War) -> Option<(u8, Visibility)> {
    if let Some(b) = war.belligerent(view.observer) {
        return Some((b.band, Visibility::Public));
    }
    view.involvements
        .iter()
        .find(|i| i.actor == view.observer && i.war == war.id)
        .map(|i| (i.band, i.visibility))
}

/// Current band of the observer in a war it doesn't lead.
pub fn current_band(view: &ObserverView, war: &War, side: Side) -> BandOption {
    if let Some((band, vis)) = my_involvement(view, war) {
        return BandOption {
            band,
            covert: vis == Visibility::Covert,
        };
    }
    let enemy = war.side(side.other()).any(|b| {
        view.sanctions
            .iter()
            .any(|s| s.by == view.observer && s.target == b.country)
    });
    BandOption {
        band: u8::from(enemy),
        covert: false,
    }
}

/// Value of involvement option `o` for `side` in `war` (DESIGN §14.9).
pub fn band(view: &ObserverView, war: &War, side: Side, o: BandOption) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let me = view.observer;
    let friend = war.leader(side);
    let enemy_id = war.leader(side.other());
    let enemy = foreign(view, enemy_id);
    let own_power = view.own.power();
    let value = stake(view, war, side);
    let current = current_band(view, war, side);

    // Stake · ΔP(success)
    let extra = match o.band {
        0 => 0.0,
        // Sanctions bleed the enemy a little.
        1 => -0.05 * side_power(view, war, side.other()),
        // A year of proxy arms.
        2 => 4.0 * proxy_amount(view, war.leader(side)) * 0.6,
        b => band_allocation(b) * own_power,
    };
    let base = side_chance(view, war, side, 0.0);
    let with = if o.band == 1 {
        // Weakening the enemy is the same as strengthening the friend.
        side_chance(view, war, side, -extra)
    } else {
        side_chance(view, war, side, extra)
    };
    s.add("stake in the outcome", 60.0 * value * (with - base));

    // RepDelta: a formal commitment or a claimed norm is graded by the band.
    if commitment_engaged(view, war, side) {
        let enemy_committed = side_power(view, war, side.other());
        let grade = match o.band {
            0 => 0.0,
            1 | 2 => 0.5,
            b if band_allocation(b) * own_power >= 0.5 * enemy_committed => 1.0,
            _ => 0.5,
        };
        // Everyone else who relies on our word is watching (RepDelta summed
        // over the observers we care about).
        let others_relying = view
            .treaties
            .iter()
            .filter(|t| match t.kind {
                TreatyKind::DefensiveAlliance => t.involves(me) && !t.involves(friend),
                TreatyKind::Guarantee => t.a == me && t.b != friend,
                _ => false,
            })
            .count() as f64;
        s.add(
            "our word as an ally",
            30.0 * (grade - 0.5) * (0.5 + pers.loyalty) * (1.0 + 0.5 * others_relying),
        );
    }
    let claims_norm = view.ledger.iter().any(|e| {
        e.actor == me && matches!(e.kind, sim_core::EntryKind::Norm(_)) && e.grade == Some(sim_core::Grade::Honoured)
    });
    if claims_norm && side == Side::Defender {
        let grade = match o.band {
            0 => 0.0,
            1 | 2 => 0.5,
            _ => 1.0,
        };
        s.add("the principle we enforced before", 15.0 * (grade - 0.5));
    }
    // Leaving an involvement writes Abandoned/Partial with its stake.
    if current.band >= 2 && o.band < current.band {
        let seen = if current.covert { 0.3 } else { 1.0 };
        let losing = war.progress_for(side) < 0.0;
        let grade = if losing { 0.0 } else { 0.5 };
        s.add(
            "walking away on the record",
            -30.0 * band_stake(current.band) * seen * (1.0 - grade),
        );
    }

    // Cost · (0.5 + greed)
    let money = 0.5 + pers.greed;
    let enemy_power = enemy.map_or(0.0, |e| e.military.value);
    match o.band {
        1 => {
            let share = enemy.map_or(0.0, |e| e.gdp / (e.gdp + view.own.gdp));
            s.add("lost trade", -20.0 * share * money);
        }
        2 => {
            s.add("arms we give away", -12.0 * money);
        }
        b @ 3..=4 => {
            let exposure = (band_allocation(b) * own_power).min(enemy_power) / own_power.max(1.0);
            s.add("casualties and war spending", -60.0 * exposure * money);
        }
        _ => {}
    }

    // Domestic cost: wars of choice, especially for open societies.
    if o.band >= 3 {
        let defending_self = enemy.is_some_and(|e| threat(view, e) >= 60.0);
        let choice = if defending_self { 0.3 } else { 1.0 };
        s.add(
            "war of choice at home",
            -20.0 * view.own.openness * choice * (o.band - 2) as f64,
        );
        s.add("existing war weariness", -0.5 * view.own.war_weariness);
    }
    if o.band == 2 && o.covert {
        s.add("exposure risk", -6.0 * view.own.openness);
    }

    // Tension · (1 − aggression)
    let visible_band = if o.covert { 1.0 } else { o.band as f64 };
    s.add("added tension", -3.0 * visible_band * (1.0 - pers.aggression));

    // Escalation risk · (1 − risk): taking on a great power.
    if let Some(e) = enemy {
        if power_share(view, e) >= GREAT_POWER_SHARE {
            let step = match o.band {
                0 | 1 => 0.0,
                2 if o.covert => 0.1,
                2 => 0.2,
                3 => 1.0,
                _ => 1.5,
            };
            s.add("escalation against a great power", -30.0 * step * (1.0 - pers.risk));
        }
    }

    // Nuclear escalation risk when fighting an arsenal state directly.
    if let Some(e) = enemy {
        if o.band >= 3 && e.arsenal > 0 {
            s.add("nuclear escalation risk", -20.0 * e.arsenal as f64 * (1.0 - pers.risk));
        }
    }

    // Entanglement · (1 − risk)
    let stake_now = band_stake(o.band) * if o.covert { 0.4 } else { 1.0 };
    s.add("entanglement", -10.0 * stake_now * (1.0 - pers.risk));

    if o.band == 2 && !arms_route(view, friend) {
        s.add("no supply route", -100.0);
    }
    if o.band == 0 {
        s.add("status quo", 5.0);
    }

    reflexes::apply(
        view,
        DecisionKind::Band(o.band),
        Parties {
            counterpart: foreign(view, friend),
            secondary: enemy,
        },
        &mut s,
    );
    s
}

/// Value of continuing a war the observer leads (negative: offer peace).
/// Forward-looking only: losses already taken don't count (DESIGN §14.9).
pub fn continue_war(view: &ObserverView, war: &War) -> Score {
    let mut s = Score::new();
    let me = view.observer;
    let side = war.side_of(me).expect("leader");
    let pers = view.own.personality;
    let att = side_power(view, war, Side::Attacker) * war.ground_mult(Side::Attacker);
    let def = side_power(view, war, Side::Defender) * war.ground_mult(Side::Defender);
    let ratio = if side == Side::Attacker {
        att / def.max(1e-6)
    } else {
        def / att.max(1e-6)
    };
    let progress = war.progress_for(side);

    if war.aim == WarAim::Punitive {
        // Only the target decides: accept the strike or fight a limited war.
        if side == Side::Defender {
            s.add("odds in a wider war", 15.0 * ratio.ln());
            s.add("refusing to be coerced", 10.0 * pers.ideology + 5.0 * pers.aggression);
            s.add("cost of a wider war", -12.0 * (1.0 - pers.risk));
        } else {
            s.add("strike delivered", 20.0);
        }
        return s;
    }

    s.add("battlefield trend", 12.0 * ratio.ln());
    s.add("position held", 0.2 * progress);
    s.add("war weariness", -0.6 * view.own.war_weariness);
    let terms_now = if progress > 25.0 {
        10.0
    } else if progress < -25.0 {
        -15.0
    } else {
        0.0
    };
    s.add("terms available now", -terms_now);
    let aim = match war.aim {
        WarAim::Major => 15.0,
        _ => 10.0,
    } * (0.5 + pers.aggression);
    if side == Side::Defender {
        // A defender that has carried the war onto the aggressor's soil
        // fights to punish it, not for the status quo ante (Iran after
        // Khorramshahr, 1982): the war aim the attacker declared becomes
        // the defender's, by the share of the fighting now on the
        // aggressor's ground (sim-core's rule). DefenderWon pays it.
        s.add("punishing the aggressor", aim * war.home_share(Side::Attacker));
    }
    if side == Side::Attacker {
        s.add("war aim", aim);
        // Quitting right after declaring is a public climbdown that tells
        // everyone our threats are empty.
        let turns = view.turn.saturating_sub(war.started) as f64;
        if progress <= 0.0 {
            s.add("backing down so soon", 20.0 * (1.0 - turns / 4.0).max(0.0));
        }
    }
    // By the share of the fighting on our own soil (sim-core's rule).
    s.add("defending our territory", 8.0 * war.home_share(side));
    s.add("cost of continuing", -8.0 * (0.5 + pers.greed));
    reflexes::apply(
        view,
        DecisionKind::Withdraw,
        Parties {
            counterpart: foreign(view, war.leader(side.other())),
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Force the target's treaty protectors can bring to its area, as the
/// observer estimates it (mirrors the simulation's Security driver).
pub fn protector_force(view: &ObserverView, target: &ForeignView) -> f64 {
    view.others
        .iter()
        .filter(|p| {
            p.id != target.id
                && view.treaties.iter().any(|tr| match tr.kind {
                    TreatyKind::DefensiveAlliance => tr.involves(p.id) && tr.involves(target.id),
                    TreatyKind::Guarantee => tr.a == p.id && tr.b == target.id,
                    _ => false,
                })
        })
        // Only the force a protector can bring to the target's area counts.
        .map(|p| p.military.value * 0.5 * p.credibility_back / 100.0 * crate::inputs::reach_into(p, &target.area))
        .sum()
}

/// The observer's odds of winning a war on `target` with `aim`: committed
/// power against the target and whoever is likely to come to its aid, all
/// as the observer estimates them.
pub fn war_odds(view: &ObserverView, target: &ForeignView, aim: WarAim) -> f64 {
    let own = view.own.power();
    let protectors = protector_force(view, target);
    if aim == WarAim::Punitive {
        // KNOWN GAP (review 14): this strike ratio is not what the simulation
        // uses to resolve a raid. There, the programme setback is certain and
        // the target's ground-force ratio decides whether the raid escalates
        // into a Limited war, which is not costed here.
        let strike = (view.own.forces.naval.value() + view.own.forces.air.value()) * view.own.forces.readiness_factor();
        let air_def = target.forces.air.value + 0.25 * target.forces.land.value;
        win_chance(aim.allocation() * strike / (air_def * DEFENCE_MULT).max(1e-6))
    } else {
        win_chance(aim.allocation() * own / ((target.military.value + protectors) * DEFENCE_MULT).max(1e-6))
    }
}

/// Value of declaring war on `target` with `aim`. War is never the default:
/// status quo, costs and reactions weigh against it (DESIGN principle 4).
pub fn declare_war(view: &ObserverView, target: &ForeignView, aim: WarAim) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let own = view.own.power();
    // A war can remove the target's armies, not its arsenal (D74).
    let t = crate::inputs::conventional_threat(view, target);
    let sub = crate::inputs::subversion(view, target);

    let protectors = protector_force(view, target);
    let odds = war_odds(view, target, aim);
    // The stakes of the outcome itself, an expected value: victory's
    // standing (+20 x odds) against defeat's (-20 x (1 - odds)), as the
    // simulation pays them (legitimacy, opinion, a claim half given up).
    s.add("odds of success", 40.0 * (odds - 0.5));
    // Every gain below is collected only if the war goes our way (a threat
    // removed, a claim settled, a subversive regime silenced, a programme
    // set back; sim-core settles a claim on AttackerWon only), so each is
    // worth its odds, while the costs further down are paid either way.
    let if_won = odds;
    s.add(
        "removing a threat",
        0.3 * t
            * if_won
            * match aim {
                WarAim::Punitive => 0.5,
                WarAim::Limited => 1.0,
                WarAim::Major => 1.5,
            },
    );
    // A standing claim (a province, a border, a debt) is something to win.
    let claim = view
        .claims
        .iter()
        .filter(|c| c.by == view.observer && c.against == target.id)
        .map(|c| c.weight)
        .fold(0.0, f64::max);
    if claim > 0.0 {
        s.add(
            "pressing our claim",
            CLAIM_WAR_VALUE
                * claim
                * if_won
                * match aim {
                    WarAim::Punitive => 0.3,
                    WarAim::Limited | WarAim::Major => 1.0,
                },
        );
    }
    if sub > 0.0 {
        s.add(
            "ending their subversion of our regime",
            0.3 * sub
                * if_won
                * match aim {
                    WarAim::Punitive => 0.5,
                    WarAim::Limited => 1.0,
                    WarAim::Major => 1.5,
                },
        );
    }
    s.add(
        "appetite",
        match aim {
            WarAim::Punitive => 6.0,
            WarAim::Limited => 10.0,
            WarAim::Major => 15.0,
        } * pers.aggression,
    );
    if protectors > 0.0 {
        s.add("their protectors", -20.0 * (protectors / own.max(1.0)).min(2.0));
    }
    let exposure = (aim.allocation() * own).min(target.military.value) / own.max(1.0);
    s.add(
        "casualties and war spending",
        -match aim {
            WarAim::Punitive => 10.0,
            _ => 40.0,
        } * exposure.max(0.1)
            * (1.5 - pers.risk),
    );
    s.add(
        "lost trade with them",
        -15.0 * target.gdp / (target.gdp + view.own.gdp) * (0.5 + pers.greed) * (1.0 - target.tension / 100.0),
    );
    s.add("war of choice at home", -20.0 * view.own.openness);
    // Diversionary logic from the rally effect (DESIGN §11.3): a regime
    // short of legitimacy gains from a named enemy, more so if closed.
    s.add(
        "rally at home",
        0.6 * (55.0 - view.own.legitimacy).max(0.0) * (1.0 - view.own.openness),
    );
    if target.tension < sim_core::war::CASUS_BELLI_TENSION {
        s.add("no casus belli", -30.0);
    }
    s.add("world reaction", -10.0);
    if power_share(view, target) >= GREAT_POWER_SHARE {
        s.add("attacking a great power", -40.0 * (1.0 - pers.risk));
    }
    // Counter-proliferation (P10; P9 absorbed into Punitive): a strike sets
    // a hostile programme back.
    if aim == WarAim::Punitive && target.programme.is_some() && target.arsenal == 0 {
        let hostility = (target.tension / 100.0)
            .max(-target.our_opinion_of_them / 100.0)
            .clamp(0.0, 1.0);
        // Only a programme within reach of each other threatens us (Osirak,
        // not Havana striking Tehran).
        s.add(
            "stop their weapons programme",
            40.0 * hostility * (0.5 + pers.paranoia) * crate::inputs::reach(view, target) * if_won,
        );
    }
    // Diminishing returns: a strike or war of ours on them already made the
    // point, and its memory fades over a decade (the ledger remembers).
    let memory = view
        .ledger
        .iter()
        .filter(|e| {
            e.actor == view.observer
                && e.counterpart == target.id
                && e.kind == sim_core::EntryKind::Coercion
                && matches!(
                    e.cause_code,
                    sim_core::CauseCode::PunitiveStrike | sim_core::CauseCode::WarDeclared
                )
        })
        .map(|e| (1.0 - (view.turn as i32 - e.turn) as f64 / WAR_MEMORY_TURNS).clamp(0.0, 1.0))
        .fold(0.0, f64::max);
    if memory > 0.0 {
        s.add("we fought them not long ago", -25.0 * memory);
    }
    // Deterrence (DESIGN §11.4): an arsenal, or one standing behind the
    // target, makes any attack a catastrophic gamble, in proportion to how
    // far the aim threatens the target's survival (D68 #5): a Major war
    // aimed at the regime carries the full risk, a Limited war over a claim
    // or a border part of it, a punitive strike the least, and none of it
    // ever vanishes.
    //
    // Risk appetite discounts a gamble, not a certainty: the existential
    // share of the risk (all of it for a Major aim, which is itself the
    // defender's use trigger) counts in full; only the rest, the chance that
    // a smaller war is pushed that far, is weighed by appetite (D85). A
    // gamble never weighs more than the certain loss, so a cautious
    // government's appetite factor stops at 1.
    let cover = crate::inputs::nuclear_cover(view, target);
    if cover > 0.0 {
        let share = existential_share(aim, target);
        let gamble = (1.2 - pers.risk).min(1.0);
        s.add(
            "catastrophic risk (nuclear)",
            -(30.0 + 0.5 * view.global_tension) * cover * (share + (1.0 - share) * gamble),
        );
    }
    // Inertia: cautious governments are more anchored to the status quo.
    s.add("status quo", -25.0 * (1.5 - pers.risk));
    reflexes::apply(
        view,
        DecisionKind::Band(aim.band()),
        Parties {
            counterpart: Some(target),
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Mobilization the observer wants, with the score behind it.
pub fn mobilization(view: &ObserverView) -> (Mobilization, Score) {
    let mut s = Score::new();
    let t = top_threat(view).map_or(0.0, |x| x.1);
    let leading = view.wars.iter().any(|w| w.is_leader(view.observer));
    let fighting = view.wars.iter().any(|w| w.side_of(view.observer).is_some());
    s.add("threat", 0.6 * t);
    s.add("paranoia", 15.0 * view.own.personality.paranoia);
    if fighting {
        s.add("at war", if leading { 40.0 } else { 20.0 });
    }
    s.add("economic cost", -30.0 - 0.3 * view.own.war_weariness);
    if !fighting {
        // A standing mobilization in peacetime: years of conscripts off the
        // labour market and a public that stops believing the alarm.
        // Aggressive regimes keep their forces ready anyway.
        s.add(
            "mobilized in peacetime",
            -20.0 * (1.0 - view.own.personality.aggression).clamp(0.0, 1.0),
        );
    }
    let level = match s.total() {
        v if v > 30.0 && leading => Mobilization::Full,
        v if v > 0.0 => Mobilization::Partial,
        _ => Mobilization::Peacetime,
    };
    (level, s)
}

/// Our side's committed share of an enemy's estimated power (helper for
/// tests and logs).
pub fn balance(view: &ObserverView, war: &War) -> f64 {
    let side = war.side_of(view.observer).unwrap_or(Side::Defender);
    let ours = side_power(view, war, side) + 1e-9;
    ours / (ours + side_power(view, war, side.other()))
}

/// Proxy arms per turn for the observer.
/// Wartime arms per turn to `to`: our proxy share, capped by what the
/// client can absorb (calibration: arms sized to the recipient, not to us).
pub fn proxy_amount(view: &ObserverView, to: CountryId) -> f64 {
    (PROXY_ARMS_SHARE * view.own.forces.strength()).min(WAR_CLIENT_ARMS * client_gdp(view, to))
}

/// Peacetime arms per turn to a threatened friend, sized to the client's
/// economy (not its army, which the arms themselves grow: no compounding).
pub fn gift_amount(view: &ObserverView, to: CountryId) -> f64 {
    (0.02 * view.own.forces.strength()).min(PEACE_CLIENT_ARMS * client_gdp(view, to))
}

fn client_gdp(view: &ObserverView, c: CountryId) -> f64 {
    if c == view.observer {
        return view.own.gdp;
    }
    view.others.iter().find(|f| f.id == c).map_or(0.0, |f| f.gdp)
}

/// Estimated total strength of `c` (our own if it is us).
pub fn client_strength(view: &ObserverView, c: CountryId) -> f64 {
    if c == view.observer {
        return view.own.forces.strength();
    }
    view.others
        .iter()
        .find(|f| f.id == c)
        .map_or(0.0, |f| f.forces.land.value + f.forces.naval.value + f.forces.air.value)
}

/// Arms a client receives per turn, in strength per unit of its GDP: about
/// half of what a typical client budget builds in peace; at war a client
/// takes in several times its own production (Afghanistan, Iraq in the 1980s).
pub const PEACE_CLIENT_ARMS: f64 = 0.06;
pub const WAR_CLIENT_ARMS: f64 = 0.4;

/// Is there an arms stream of ours to `to`?
pub fn arming(view: &ObserverView, to: CountryId) -> bool {
    view.streams
        .iter()
        .any(|s| s.from == view.observer && s.to == to && s.kind == StreamKind::Arms)
}

/// Value of a limited nuclear strike on `target` (DESIGN §11.4 AI
/// reasoning). Considered only when existential: our war is being lost
/// badly, or we were struck first. Survival value against the penalty
/// package and the retaliation risk, weighted heavily by risk tolerance.
pub fn nuclear_use(view: &ObserverView, war: &War, target: &ForeignView) -> Option<Score> {
    let me = view.observer;
    let side = war.side_of(me)?;
    if view.own.arsenal == 0 || view.own.strike_capacity < 1.0 || war.side_of(target.id) != Some(side.other()) {
        return None;
    }
    let losing = -war.progress_for(side);
    let struck = crate::inputs::used_nuclear(view, target.id)
        && view
            .ledger
            .iter()
            .any(|e| e.actor == target.id && e.counterpart == me && e.cause_code == sim_core::CauseCode::NuclearStrike);
    let collapsing = view.own.stability < 20.0;
    if losing < 60.0 && !struck && !collapsing {
        return None;
    }
    let pers = view.own.personality;
    let mut s = Score::new();
    s.add(
        "survival of the state",
        0.8 * (losing - 40.0).max(0.0) + if collapsing { 15.0 } else { 0.0 },
    );
    if struck {
        s.add("retaliation for their strike", 60.0);
    }
    s.add("pariah forever", -80.0);
    s.add("their retaliation", -40.0 * target.arsenal as f64);
    s.add(
        "the shock at home",
        if view.own.government == sim_core::Government::Democracy {
            -30.0
        } else {
            -10.0
        },
    );
    s.add("appetite for risk", 50.0 * (pers.risk - 0.5));
    if view.own.government == sim_core::Government::Revolutionary {
        s.add("revolutionary certainty", 15.0);
    }
    reflexes::apply(
        view,
        DecisionKind::Band(4),
        Parties {
            counterpart: Some(target),
            secondary: None,
        },
        &mut s,
    );
    Some(s)
}
