//! Utility evaluations for AI decisions (DESIGN §14.9). Each returns a
//! [`Score`]; a positive total means "do it". Weights come from the
//! observer's personality; inputs come only from its view.
//!
//! Safeguards built in:
//! - **Anti-ideology-lock:** the ideology term is capped at 30 × ideology
//!   and discounted under threat, while security and need terms reach ~60.
//! - **No identity rules:** proposer/target are just view entries.

use sim_core::view::ForeignView;
use sim_core::{DecisionKind, ObserverView, Proposal, ProposalFamily, Sanction, Treaty, TreatyKind};

use crate::inputs::{
    foreign, has_treaty, ideological_distance, max_tension_of, pair_tension, threat, top_threat, top_threat_excluding,
};
use crate::reflexes::{self, Parties};
use crate::Score;

/// Status-quo inertia for taking on a major commitment.
pub const COMMITMENT_INERTIA: f64 = -10.0;
/// Status-quo inertia for minor acts.
pub const MINOR_INERTIA: f64 = -5.0;
/// Exit threshold for leaving an alliance (also needs two assessments).
pub const EXIT_THRESHOLD: f64 = -10.0;

/// Ideology term, bounded and discounted under threat. `weight` scopes it
/// by decision: 1 for alliances and bases, 0.5 for arms, 0.25 for trade.
fn ideology_term(view: &ObserverView, f: &ForeignView, threat: f64, weight: f64) -> f64 {
    -30.0 * weight * view.own.personality.ideology * ideological_distance(view, f) * (1.0 - 0.8 * threat / 100.0)
}

/// The client's hottest quarrel: the other country with the highest public
/// tension toward it, and that tension.
fn top_rival(view: &ObserverView, client: sim_core::CountryId) -> (Option<&ForeignView>, f64) {
    view.others
        .iter()
        .filter(|f| f.id != client)
        .map(|f| (f, pair_tension(view, client, f.id)))
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.id.cmp(&a.0.id)))
        .map_or((None, 0.0), |(f, t)| (Some(f), t))
}

/// Weights of the moral-hazard terms in underwriting a client (F3).
pub const UNDERWRITING_ATTACKER: f64 = 40.0;
pub const UNDERWRITING_RECORD: f64 = 25.0;
pub const UNDERWRITING_CLAIM: f64 = 20.0;
pub const UNDERWRITING_HOSTILITY: f64 = 20.0;
pub const UNDERWRITING_SANCTION: f64 = 15.0;

/// A defensive pact with an aggressor: the counterpart is the attacker in a
/// war being fought now, or started one recently (the observer's own ledger,
/// fading over [`crate::war::WAR_MEMORY_TURNS`]). The USSR embargoed Iraq in
/// 1980–82 rather than back its invasion; nobody allied with Argentina in
/// 1982.
fn aggressor_terms(view: &ObserverView, client: &ForeignView, s: &mut Score) {
    if crate::inputs::is_attacker(view, client.id) {
        s.add("they are the attacker in a war", -UNDERWRITING_ATTACKER);
    } else {
        s.add(
            "their record of starting wars",
            -UNDERWRITING_RECORD * crate::inputs::aggression_record(view, client.id),
        );
    }
}

/// Moral hazard in underwriting `client` with a guarantee (F3). A guarantee
/// is a contingency order against an attack on the client, so a client that
/// is itself the aggressor, presses a standing claim against the very rival
/// the pledge would face, or is at odds with us, is a liability rather than a
/// protégé. Public inputs only: wars, the observer's ledger, claims, tension
/// and opinion, our own sanctions. Great powers armed and financed such
/// states (band 2) but did not formally guarantee them: the 1980s tilt to
/// Iraq was arms and intelligence, not a defence pledge.
fn underwriting(view: &ObserverView, client: &ForeignView, rival: Option<&ForeignView>, s: &mut Score) {
    aggressor_terms(view, client, s);
    if let Some(r) = rival {
        let claim = view
            .claims
            .iter()
            .filter(|c| c.by == client.id && c.against == r.id)
            .map(|c| c.weight)
            .fold(0.0, f64::max);
        s.add("underwriting their claim", -UNDERWRITING_CLAIM * claim);
    }
    s.add(
        "they are hostile to us",
        -UNDERWRITING_HOSTILITY * crate::inputs::hostility_toward_us(view, client),
    );
    if view
        .sanctions
        .iter()
        .any(|x| x.by == view.observer && x.target == client.id)
    {
        s.add("we are sanctioning them", -UNDERWRITING_SANCTION);
    }
}

/// Evaluate a proposal addressed to the observer.
pub fn proposal(view: &ObserverView, p: &Proposal) -> Score {
    let mut s = Score::new();
    let Some(from) = foreign(view, p.from) else {
        s.add("unknown proposer", -100.0);
        return s;
    };
    let pers = view.own.personality;
    let mut secondary: Option<&ForeignView> = None;
    s.add("relations", 0.2 * from.our_opinion_of_them);
    match p.kind {
        TreatyKind::Trade { deep } => {
            // Gains scale with the partner's size relative to ours.
            let size = (from.gdp / view.own.gdp).sqrt().min(1.5);
            s.add(
                "trade gains",
                if deep { 18.0 } else { 10.0 } * (0.5 + pers.greed) * size,
            );
            if deep {
                s.add("adjustment cost", -6.0 * view.own.trade_exposure);
            }
            s.add("ideology", ideology_term(view, from, 0.0, 0.25));
            s.add("status quo", MINOR_INERTIA);
        }
        TreatyKind::NonAggression => {
            s.add("lowers tension", 0.3 * from.tension);
            s.add("status quo", MINOR_INERTIA);
        }
        TreatyKind::DefensiveAlliance => {
            let (threat_f, t) = top_threat_excluding(view, p.from).map_or((None, 0.0), |(f, t)| (Some(f), t));
            secondary = threat_f;
            s.add("security need", 0.6 * t);
            s.add("their reliability", 0.6 * (from.credibility_back - 50.0));
            s.add("trust", 0.3 * (from.trust - 50.0));
            s.add(
                "entanglement in their quarrels",
                -25.0 * max_tension_of(view, p.from, view.observer) / 100.0 * (1.0 - pers.aggression),
            );
            if threat_f.is_some() {
                s.add("provokes our main threat", -0.2 * t);
            }
            aggressor_terms(view, from, &mut s);
            s.add("ideology", ideology_term(view, from, t, 1.0));
            s.add("status quo", COMMITMENT_INERTIA);
        }
        TreatyKind::Guarantee => {
            // They ask us to guarantee them.
            let (rival, rival_tension) = top_rival(view, p.from);
            s.add("affinity", 10.0 * crate::inputs::affinity_with(view, from));
            secondary = rival;
            if let Some(r) = rival {
                s.add("shared rival", 0.3 * r.tension.min(rival_tension));
                let burden = (r.military.value / view.own.power().max(1.0)).min(2.0);
                s.add("risk of being drawn into war", -0.25 * rival_tension * burden);
            }
            underwriting(view, from, rival, &mut s);
            let commitments = view
                .treaties
                .iter()
                .filter(|t| t.a == view.observer && t.kind == TreatyKind::Guarantee)
                .count() as f64;
            s.add("existing commitments", -5.0 * commitments);
            s.add("status quo", COMMITMENT_INERTIA);
        }
        TreatyKind::Basing => {
            let t = top_threat(view).map_or(0.0, |x| x.1);
            s.add("deterrence", 0.4 * t * (1.0 - from.tension / 100.0));
            s.add(
                "foreign troops on our soil",
                -10.0 - 30.0 * pers.ideology * ideological_distance(view, from),
            );
            s.add("status quo", MINOR_INERTIA);
        }
    }
    reflexes::apply(
        view,
        DecisionKind::AcceptProposal(ProposalFamily::of(p.kind)),
        Parties {
            counterpart: Some(from),
            secondary,
        },
        &mut s,
    );
    s
}

/// Value of asking `patron` to guarantee us against our top threat.
pub fn request_guarantee(view: &ObserverView, patron: &ForeignView) -> Score {
    let mut s = Score::new();
    let Some((threat_f, t)) = top_threat(view) else {
        return s;
    };
    if threat_f.id == patron.id {
        s.add("patron is the threat", -100.0);
        return s;
    }
    let power = patron.military.value / (patron.military.value + threat_f.military.value).max(1.0);
    s.add("security need", 0.6 * t);
    s.add("their reliability", 0.6 * (patron.credibility_back - 50.0));
    s.add("their strength against the threat", 20.0 * power - 10.0);
    s.add("relations", 0.2 * patron.our_opinion_of_them);
    let rivalry = pair_tension(view, patron.id, threat_f.id);
    s.add("odds they agree", 0.2 * rivalry - 8.0);
    s.add("provokes our main threat", -0.2 * t);
    s.add("ideology", ideology_term(view, patron, t, 1.0));
    s.add("status quo", -15.0);
    reflexes::apply(
        view,
        DecisionKind::RequestGuarantee,
        Parties {
            counterpart: Some(patron),
            secondary: Some(threat_f),
        },
        &mut s,
    );
    s
}

/// Weight of the retaliation a sanctioner risks when the target could
/// bring everything it has against its whole defence.
pub const RETALIATION: f64 = 20.0;

/// What a sanctioner risks: the target's force within reach of us against
/// our whole defence (own power, the allies and guarantors we believe
/// would come, our arsenal). A minnow beside a giant with no protector
/// risks everything; a protected or distant sanctioner little (Kuwait did
/// not embargo Iraq; Western Europe embargoed the USSR under NATO).
pub fn retaliation_risk(view: &ObserverView, target: &ForeignView) -> f64 {
    let theirs = target.military.value * crate::inputs::reach(view, target);
    -RETALIATION * theirs / (crate::inputs::defence(view) + theirs).max(1.0)
}

/// Value of joining `sanction` (imposed by someone else on its target).
pub fn join_sanction(view: &ObserverView, sanction: &Sanction) -> Score {
    let mut s = Score::new();
    let (Some(by), Some(target)) = (foreign(view, sanction.by), foreign(view, sanction.target)) else {
        return s;
    };
    let pers = view.own.personality;
    // Friends join when they share the grievance, not out of loyalty alone
    // (D68): Western Europe joined the 1980 measures after Afghanistan but
    // stayed out of the Cuba embargo and the 1982 pipeline sanctions. Both
    // pulls toward the sanctioner (alliance and goodwill) scale with our own
    // quarrel with the target; disliking the sanctioner counts in full.
    let shared = shared_grievance(view, target);
    if has_treaty(view, TreatyKind::DefensiveAlliance, view.observer, by.id) {
        s.add("solidarity with ally", 15.0 * (0.5 + pers.loyalty) * shared);
    }
    let goodwill = by.our_opinion_of_them;
    s.add(
        "standing with sanctioner",
        0.1 * if goodwill > 0.0 { goodwill * shared } else { goodwill },
    );
    s.add(
        "grievance against target",
        0.3 * (-target.our_opinion_of_them).max(0.0) + 0.2 * target.tension,
    );
    let trade_weight = if has_treaty(view, TreatyKind::Trade { deep: false }, view.observer, target.id) {
        1.6
    } else {
        1.0
    };
    // Expected marginal loss as a share of our GDP: nothing if the target
    // already cuts that trade.
    let world: f64 = view.own.gdp + view.others.iter().map(|o| o.gdp).sum::<f64>();
    let already = view
        .sanctions
        .iter()
        .any(|x| x.by == target.id && x.target == view.observer);
    let loss = if already {
        0.0
    } else {
        sim_core::trade::SANCTION_CUT * sim_core::trade::gravity(view.own.gdp, target.gdp, world) * trade_weight
            / view.own.gdp
    };
    s.add("our lost trade", -100.0 * loss * (0.5 + pers.greed));
    s.add("retaliation risk", retaliation_risk(view, target));
    s.add(
        "affinity with target",
        -15.0 * pers.ideology * crate::inputs::affinity_with(view, target).max(0.0),
    );
    if crate::inputs::used_nuclear(view, target.id) {
        s.add("they used nuclear weapons", 40.0);
    }
    if proliferating_stranger(view, target) {
        s.add("they are building the bomb", 20.0);
    }
    s.add("status quo", -10.0);
    reflexes::apply(
        view,
        DecisionKind::JoinSanction,
        Parties {
            counterpart: Some(by),
            secondary: Some(target),
        },
        &mut s,
    );
    s
}

/// Our own quarrel with `target` (tension, hostility or threat, 0-100) at
/// which an ally's sanction is fully our cause too. Reuses the 30-point level
/// `stay_in_alliance` treats as a serious quarrel, but measures the
/// observer's own grievance on a sliding scale, not a partner's as a yes/no.
pub const SHARED_GRIEVANCE_FULL: f64 = 30.0;

/// How far the observer shares a grievance against `target`, 0–1: its own
/// bilateral tension, its hostility (negative opinion) or the threat it
/// estimates from the target, whichever is strongest. View data only.
pub fn shared_grievance(view: &ObserverView, target: &ForeignView) -> f64 {
    let quarrel = target
        .tension
        .max(-target.our_opinion_of_them)
        .max(threat(view, target));
    (quarrel / SHARED_GRIEVANCE_FULL).clamp(0.0, 1.0)
}

/// Value of staying in an alliance (negative = inclined to leave).
pub fn stay_in_alliance(view: &ObserverView, treaty: &Treaty) -> Score {
    let mut s = Score::new();
    let Some(partner) = treaty.other(view.observer).and_then(|p| foreign(view, p)) else {
        return s;
    };
    let pers = view.own.personality;
    let threat_f = top_threat_excluding(view, partner.id);
    let (shared, t) = match threat_f {
        Some((x, t)) => (pair_tension(view, partner.id, x.id) > 30.0, t),
        None => (false, 0.0),
    };
    s.add(
        if shared { "shared threat" } else { "general security" },
        if shared { 0.5 } else { 0.2 } * t,
    );
    s.add("their reliability", 0.5 * (partner.credibility_back - 50.0));
    // Relations without the alliance's own goodwill: a treaty can't be its
    // own reason to exist.
    let own_bonus = 20.0;
    s.add("relations", 0.2 * (partner.our_opinion_of_them - own_bonus));
    s.add(
        "entanglement in their quarrels",
        -20.0 * max_tension_of(view, partner.id, view.observer) / 100.0 * (1.0 - pers.aggression),
    );
    s.add("keeping our word", 20.0 * pers.loyalty);
    if crate::inputs::used_nuclear(view, partner.id) {
        s.add("our ally used nuclear weapons", -60.0);
    }
    if !shared && t < 30.0 {
        // The alliance's purpose has faded (treaty re-evaluation, N8), and
        // whoever spends more on defence notices it is carrying the other.
        s.add("lost purpose", -20.0 * (1.0 - t / 30.0));
        if let Some(b) = partner.budget {
            s.add(
                "carrying their defence",
                -60.0 * (view.own.budget.military - b.military).max(0.0),
            );
        }
    }
    s.add("ideology", ideology_term(view, partner, t, 1.0));
    reflexes::apply(
        view,
        DecisionKind::StayInAlliance,
        Parties {
            counterpart: Some(partner),
            secondary: threat_f.map(|(f, _)| f),
        },
        &mut s,
    );
    s
}

/// The observer's main threat score (exposed for logging and tests).
pub fn main_threat(view: &ObserverView) -> f64 {
    top_threat(view).map_or(0.0, |(f, _)| threat(view, f))
}

/// Value of imposing our own sanctions on `target` (DESIGN §14.9, band 1).
/// `goal` is true when containing or balancing `target` is an active goal.
pub fn impose_sanction(view: &ObserverView, target: &ForeignView, goal: bool) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    s.add(
        "grievance",
        0.3 * (-target.our_opinion_of_them).max(0.0) + 0.2 * target.tension,
    );
    if goal {
        s.add("goal: pressure them", 12.0);
    }
    if proliferating_stranger(view, target) {
        s.add("they are building the bomb", 20.0);
    }
    if crate::inputs::used_nuclear(view, target.id) {
        s.add("they used nuclear weapons", 40.0);
    }
    s.add("their record of coercion", 15.0 * crate::goals::intent(view, target.id));
    let trade_weight = if has_treaty(view, TreatyKind::Trade { deep: false }, view.observer, target.id) {
        1.6
    } else {
        1.0
    };
    // Expected marginal loss as a share of our GDP: nothing if the target
    // already cuts that trade.
    let world: f64 = view.own.gdp + view.others.iter().map(|o| o.gdp).sum::<f64>();
    let already = view
        .sanctions
        .iter()
        .any(|x| x.by == target.id && x.target == view.observer);
    let loss = if already {
        0.0
    } else {
        sim_core::trade::SANCTION_CUT * sim_core::trade::gravity(view.own.gdp, target.gdp, world) * trade_weight
            / view.own.gdp
    };
    s.add("our lost trade", -100.0 * loss * (0.5 + pers.greed));
    s.add("retaliation risk", retaliation_risk(view, target));
    s.add("status quo", -20.0);
    reflexes::apply(
        view,
        DecisionKind::ImposeSanction,
        Parties {
            counterpart: Some(target),
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Value of keeping a guarantee we issued (treaty re-evaluation, N8).
pub fn keep_guarantee(view: &ObserverView, client: &ForeignView, develop: bool) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let danger = crate::goals::danger(view, client.id);
    s.add("they still need it", 30.0 * danger);
    s.add("affinity", 10.0 * crate::inputs::affinity_with(view, client));
    s.add("keeping our word", 15.0 * pers.loyalty);
    s.add("relations", 0.1 * client.our_opinion_of_them);
    s.add("cost of the commitment", -15.0);
    if develop {
        s.add("goal: fewer entanglements", -10.0);
    }
    // The pledge is re-read against what the client has since done with it.
    let (rival, _) = top_rival(view, client.id);
    underwriting(view, client, rival, &mut s);
    reflexes::apply(
        view,
        DecisionKind::StayInAlliance,
        Parties {
            counterpart: Some(client),
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Value of proposing a trade agreement (shallow, or deepening one).
pub fn propose_trade(view: &ObserverView, partner: &ForeignView, deep: bool) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let size = (partner.gdp / view.own.gdp).sqrt().min(1.5);
    s.add("trade gains", if deep { 14.0 } else { 8.0 } * (0.5 + pers.greed) * size);
    if deep {
        s.add("adjustment cost", -6.0 * view.own.trade_exposure);
        s.add("integration as an alternative to blocs", 4.0);
    }
    s.add("relations", 0.15 * partner.our_opinion_of_them);
    s.add("ideology", ideology_term(view, partner, 0.0, 0.25));
    let odds = (partner.their_opinion_of_us / 50.0).clamp(-1.0, 1.0);
    s.add("odds they agree", 6.0 * odds);
    s.add("status quo", -8.0);
    s
}

/// Value of proposing an alliance against `against`.
pub fn propose_alliance(view: &ObserverView, partner: &ForeignView, against: &ForeignView) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let shared = pair_tension(view, partner.id, against.id).min(against.tension);
    s.add("shared threat", 0.4 * shared);
    s.add(
        "their strength",
        15.0 * (partner.military.value / view.own.power().max(1.0)).min(1.0),
    );
    s.add("their reliability", 0.3 * (partner.credibility_back - 50.0));
    s.add(
        "entanglement in their quarrels",
        -20.0 * max_tension_of(view, partner.id, against.id) / 100.0 * (1.0 - pers.aggression),
    );
    s.add("ideology", ideology_term(view, partner, threat(view, against), 1.0));
    s.add("odds they agree", 0.1 * partner.their_opinion_of_us);
    s.add("status quo", COMMITMENT_INERTIA - 5.0);
    reflexes::apply(
        view,
        DecisionKind::AcceptProposal(ProposalFamily::DefensiveAlliance),
        Parties {
            counterpart: Some(partner),
            secondary: Some(against),
        },
        &mut s,
    );
    s
}

/// Value of guaranteeing a threatened friend unasked (coalition leadership).
pub fn offer_guarantee(view: &ObserverView, client: &ForeignView) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let danger = crate::goals::danger(view, client.id);
    let (rival, rival_tension) = top_rival(view, client.id);
    s.add("they are in danger", 25.0 * danger);
    s.add("affinity", 12.0 * crate::inputs::affinity_with(view, client));
    s.add("relations", 0.15 * client.our_opinion_of_them);
    if let Some(r) = rival {
        s.add("their rival is ours", 0.3 * r.tension.min(rival_tension));
        let burden = (r.military.value / view.own.power().max(1.0)).min(2.0);
        s.add(
            "risk of being drawn into war",
            -0.2 * rival_tension * burden * (1.0 - pers.risk),
        );
    }
    underwriting(view, client, rival, &mut s);
    let commitments = view
        .treaties
        .iter()
        .filter(|t| t.a == view.observer && t.kind == TreatyKind::Guarantee)
        .count() as f64;
    s.add("existing commitments", -5.0 * commitments);
    s.add("status quo", COMMITMENT_INERTIA - 5.0);
    reflexes::apply(
        view,
        DecisionKind::AcceptProposal(ProposalFamily::Guarantee),
        Parties {
            counterpart: Some(client),
            secondary: rival,
        },
        &mut s,
    );
    s
}

/// Everything we give away each turn (aid and gifted arms, not sales), as a
/// share of our revenue: each programme looks cheap alone.
pub fn support_burden(view: &ObserverView) -> f64 {
    let revenue = (view.own.gdp * view.own.tax_rate).max(1e-9);
    view.streams
        .iter()
        .filter(|s| s.from == view.observer && !s.sale)
        .map(|s| s.value())
        .sum::<f64>()
        / revenue
}

/// Value of keeping a stream we fund to `client` (patron re-evaluation).
pub fn keep_stream(
    view: &ObserverView,
    client: &ForeignView,
    stream: &sim_core::Stream,
    contain: Option<sim_core::CountryId>,
    budget_pressed: bool,
    develop: bool,
) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let danger = crate::goals::danger(view, client.id);
    s.add("they still need it", 30.0 * danger);
    s.add("affinity", 10.0 * crate::inputs::affinity_with(view, client));
    s.add("keeping our word", 10.0 * pers.loyalty);
    if let Some(x) = contain {
        if pair_tension(view, client.id, x) >= 40.0 {
            s.add("they hold our rival down", 12.0);
        }
    }
    let revenue = (view.own.gdp * view.own.tax_rate).max(1e-9);
    s.add("what it costs", -300.0 * stream.value() / revenue * (0.5 + pers.greed));
    s.add("what all our programmes cost together", -150.0 * support_burden(view));
    // A client building the bomb behind our back (patron leverage, P10).
    if view.own.arsenal > 0 && client.programme.is_some() {
        s.add("they are building the bomb behind our back", -25.0);
    }
    if budget_pressed {
        s.add("budget crisis at home", -15.0);
    }
    if develop {
        s.add("goal: fewer entanglements", -10.0);
    }
    // Patron leverage (D58): a client arming our rivals.
    let armed_rival = view.streams.iter().any(|st| {
        st.from == client.id
            && st.kind == sim_core::StreamKind::Arms
            && st.to != view.observer
            && (view
                .wars
                .iter()
                .any(|w| w.side_of(st.to).is_some() && w.side_of(view.observer).is_some() && w.side_of(st.to) != w.side_of(view.observer))
                || pair_tension(view, view.observer, st.to) >= sim_core::diplomacy::SALE_OBJECTION_TENSION)
    });
    if armed_rival {
        s.add("they are arming our rival", -25.0);
    }
    s.add("status quo", 5.0);
    reflexes::apply(
        view,
        DecisionKind::StreamStop,
        Parties {
            counterpart: Some(client),
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Value of arming a threatened friend in peacetime. Arms are permanent
/// (DESIGN §21.6), so a client we don't trust is a future risk.
pub fn arm_client(view: &ObserverView, client: &ForeignView, contain: Option<sim_core::CountryId>) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let danger = crate::goals::danger(view, client.id);
    s.add("they are in danger", 25.0 * danger * (0.5 + pers.loyalty));
    s.add("affinity", 10.0 * crate::inputs::affinity_with(view, client));
    if let Some(x) = contain {
        if pair_tension(view, client.id, x) >= 40.0 {
            s.add("they face our rival", 15.0);
        }
    }
    s.add("arms we give away", -12.0 * (0.5 + pers.greed));
    s.add("what all our programmes cost together", -150.0 * support_burden(view));
    s.add("we could sell these instead", -20.0 * view.own.arms_industry);
    // Grants go where the client cannot pay. A rich ally buys its arms (Japan,
    // the Federal Republic and Britain paid cash for American weapons; the
    // 1980s burden-sharing quarrel was about exactly this); the poor get them
    // given (Israel, Egypt, Pakistan, Turkey). Scaled by the client's economy
    // against ours; at parity it cancels the whole danger motive.
    s.add(
        "they can pay for their own",
        -25.0 * (client.gdp / view.own.gdp.max(1e-9)).min(1.0),
    );
    s.add("permanent: if they turn on us", -10.0 * (1.0 - client.trust / 100.0));
    s.add("status quo", -8.0);
    reflexes::apply(
        view,
        DecisionKind::StreamStart,
        Parties {
            counterpart: Some(client),
            secondary: contain.and_then(|x| foreign(view, x)),
        },
        &mut s,
    );
    s
}

/// Money per turn, as a share of our revenue, that one state pledges to
/// finance another's war against our main threat (issue 23). Grounding: the
/// Saudi and Kuwaiti loans and grants to Iraq, 1980-88, ran at roughly 2-3%
/// of Saudi GDP a year (about 5% of revenue at a 45% take).
pub const WAR_FINANCE_SHARE: f64 = 0.05;

/// Value of financing `friend`'s war against `enemy`, our main threat
/// (issue 23: balancing against a *regional* threat). The enemy of our
/// enemy is worth paying in proportion to how much of the threat's strength
/// the war ties down: that is containment we don't have to buy ourselves.
/// The motive is our own threat estimate of the enemy (reach × power share
/// against our defence × hostility, so a state strong enough to face the
/// threat alone values it little), less the threat the friend itself poses
/// to us. Money, not arms: it costs Budget, not our own army.
pub fn fund_war(
    view: &ObserverView,
    war: &sim_core::War,
    friend: &ForeignView,
    enemy: &ForeignView,
    amount: f64,
) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let tied = war.belligerent(enemy.id).map_or(0.0, |b| b.allocation.clamp(0.0, 1.0));
    s.add(
        "the war ties down our main threat",
        threat(view, enemy) * (0.5 + pers.paranoia) * tied,
    );
    s.add(
        "they are a danger to us too",
        -threat(view, friend) * (0.5 + pers.paranoia),
    );
    let revenue = (view.own.gdp * view.own.tax_rate).max(1e-9);
    s.add("what it costs", -300.0 * amount / revenue * (0.5 + pers.greed));
    s.add("what all our programmes cost together", -150.0 * support_burden(view));
    s.add("added tension", -3.0 * (1.0 - pers.aggression));
    s.add("status quo", -8.0);
    reflexes::apply(
        view,
        DecisionKind::StreamStart,
        Parties {
            counterpart: Some(friend),
            secondary: Some(enemy),
        },
        &mut s,
    );
    s
}

/// Market price of arms for `buyer`, as this observer sees it (D58).
pub fn market_price(view: &ObserverView, buyer: sim_core::CountryId) -> f64 {
    let sanctioners = view.sanctions.iter().filter(|s| s.target == buyer).count();
    let at_war = view.wars.iter().any(|w| w.side_of(buyer).is_some());
    sim_core::diplomacy::arms_price(sanctioners, at_war)
}

/// Value of selling `amount` strength per turn to `buyer` (D58). Revenue is
/// the price minus what the sale costs our own forces (an arms industry
/// builds most of it); against it: the arms are permanent, and our allies
/// and patrons object to us arming their rivals. Selling covertly mutes the
/// objection at the risk of exposure.
pub fn sell_arms(view: &ObserverView, buyer: &ForeignView, amount: f64, covert: bool, existing: bool) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let me = view.observer;
    let revenue = (view.own.gdp * view.own.tax_rate).max(1e-9);
    let price = market_price(view, buyer.id);
    let value = sim_core::diplomacy::arms_value(sim_core::StreamKind::Arms, amount);
    let margin = value * (price - (1.0 - view.own.arms_industry));
    s.add("export revenue", 300.0 * margin / revenue * (0.5 + pers.greed));
    s.add(
        "they could turn these on us",
        -60.0 * ((buyer.tension - 25.0) / 75.0).clamp(0.0, 1.0),
    );
    s.add("permanent: if they turn on us", -3.0 * (1.0 - buyer.trust / 100.0));
    // Blocs sell inside the camp and embargo the other (export controls); a
    // seller who needs the money may cross the line anyway.
    match (&view.own.alignment, &buyer.alignment) {
        (Some(a), Some(b)) if a == b => {
            s.add("same camp", 8.0);
        }
        (Some(_), Some(_)) => {
            s.add("the other camp (export controls)", -12.0);
        }
        _ => {}
    }
    // Allies and patrons at odds with the buyer object; the more we depend
    // on them, the more it costs (patron leverage).
    let mut friends: Vec<sim_core::CountryId> = view
        .treaties
        .iter()
        .filter(|t| t.kind == TreatyKind::DefensiveAlliance && t.involves(me))
        .filter_map(|t| t.other(me))
        .collect();
    friends.extend(view.streams.iter().filter(|st| st.to == me && !st.sale).map(|st| st.from));
    friends.sort();
    friends.dedup();
    let mut objection = 0.0;
    for x in friends.into_iter().filter(|&x| x != buyer.id) {
        let at_war = view
            .wars
            .iter()
            .any(|w| w.side_of(x).is_some() && w.side_of(buyer.id).is_some() && w.side_of(x) != w.side_of(buyer.id));
        if !at_war && pair_tension(view, x, buyer.id) < sim_core::diplomacy::SALE_OBJECTION_TENSION {
            continue;
        }
        let support: f64 = view
            .streams
            .iter()
            .filter(|st| st.from == x && st.to == me && !st.sale)
            .map(|st| st.value())
            .sum();
        objection += 12.0 + 300.0 * support / revenue;
    }
    if objection > 0.0 {
        let muted = if covert { 0.35 } else { 1.0 };
        s.add("our allies and patrons object", -objection * muted);
    }
    if covert {
        s.add("if it leaks", -4.0);
    }
    // Selling to both sides of a war is recorded in the ledger.
    for w in view.wars.iter() {
        if let Some(side) = w.side_of(buyer.id) {
            let other_side = view.streams.iter().any(|st| {
                st.from == me && st.kind == sim_core::StreamKind::Arms && w.side_of(st.to) == Some(side.other())
            });
            if other_side {
                s.add("we are already arming the other side", -6.0);
            }
        }
    }
    // A supply relationship buys standing in their capital (the stream's
    // opinion modifier); a new one costs some attention.
    s.add("goodwill in their capital", 4.0);
    if existing {
        s.add("status quo", 6.0);
    } else {
        s.add("new customer", -4.0);
    }
    reflexes::apply(
        view,
        DecisionKind::SellArms,
        Parties {
            counterpart: Some(buyer),
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Value of continuing to buy arms from `seller` (D58).
pub fn keep_purchase(view: &ObserverView, seller: &ForeignView, stream: &sim_core::Stream) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let me = view.observer;
    let revenue = (view.own.gdp * view.own.tax_rate).max(1e-9);
    let at_war = view.wars.iter().any(|w| w.side_of(me).is_some());
    s.add("we need them", 35.0 * crate::goals::danger(view, me) + if at_war { 20.0 } else { 0.0 });
    let cost = stream.value() * market_price(view, me);
    s.add("what it costs", -300.0 * cost / revenue * (0.5 + pers.greed));
    s.add("we make our own", -20.0 * view.own.arms_industry);
    s.add(
        "the supplier is hostile",
        -30.0 * ((seller.tension - 25.0) / 75.0).clamp(0.0, 1.0),
    );
    s.add("status quo", 5.0);
    s
}

/// Value of a world monetary stance for the reserve-currency holder (P7):
/// our own inflation and interest bill, our friends' debt pain, and the
/// squeeze on indebted rivals. Others' debts come from our own estimates.
pub fn monetary_stance(view: &ObserverView, stance: sim_core::MonetaryStance) -> Score {
    use sim_core::money::{self, MonetaryStance};
    let mut s = Score::new();
    let pers = view.own.personality;
    let me = view.observer;
    let base = view.money_base_rate();
    // Compare where each stance settles inflation and the rate.
    let pressure = |st: MonetaryStance| money::inflation_target(view.energy.deviation(), st, view.money.inflation);
    let rate = |st: MonetaryStance| money::target_rate(base, st, view.global_tension);
    // Our own Prosperity under this stance vs the current one.
    let own = |st: MonetaryStance| {
        let (inf, debt) = money::prosperity_effects(&view.own, pressure(st), money::debt_service(&view.own, rate(st)));
        // The growth the stance costs or adds (Prosperity's growth term).
        sim_core::domestic::GROWTH_PROSPERITY * st.growth_effect() - (inf + debt)
    };
    // Governments weigh their own economy well above their allies' debts.
    s.add(
        "our prosperity (inflation and interest)",
        2.0 * (own(stance) - own(view.money.stance)) * (0.5 + pers.greed),
    );
    // Debtors' pain, estimated: debt ~ ratio x annual GDP, revenue ~ 30% of GDP.
    let pain = |f: &ForeignView, st: MonetaryStance| {
        let ratio = f.debt_ratio.map_or(0.3, |e| e.value);
        let service = ratio * 4.0 * f.gdp * rate(st);
        money::DEBT_SERVICE_PROSPERITY * (service / (0.3 * f.gdp).max(1e-9) - money::ROUTINE_DEBT_SERVICE).max(0.0)
    };
    let friend = |f: &ForeignView| {
        view.treaties.iter().any(|t| {
            (t.kind == TreatyKind::DefensiveAlliance && t.involves(me) && t.involves(f.id))
                || (t.kind == TreatyKind::Guarantee && t.a == me && t.b == f.id)
        }) || view.streams.iter().any(|st| st.from == me && st.to == f.id && !st.sale)
    };
    let hostile = |f: &ForeignView| f.tension >= 50.0 || f.our_opinion_of_them <= -30.0;
    let (mut friends, mut rivals) = (0.0, 0.0);
    for f in &view.others {
        let delta = pain(f, stance) - pain(f, view.money.stance);
        if friend(f) {
            friends += delta;
        } else if hostile(f) {
            rivals += delta;
        }
    }
    s.add("our friends' debts", -0.15 * friends * (0.5 + pers.loyalty));
    s.add("squeezing indebted rivals", 0.15 * rivals * (0.5 + pers.aggression));
    if stance == view.money.stance {
        s.add("status quo", 3.0);
    }
    reflexes::apply(
        view,
        DecisionKind::MonetaryStance,
        Parties {
            counterpart: None,
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Value of a production policy for an energy exporter (scenario P5).
/// Revenue: own output × price, with the price's response to our own
/// market share; strategy: flooding squeezes hostile exporters, restraint
/// squeezes hostile importers.
pub fn energy_policy(view: &ObserverView, policy: sim_core::ProductionPolicy) -> Score {
    use sim_core::commodity::PRICE_EXPONENT;
    let mut s = Score::new();
    let pers = view.own.personality;
    let m = &view.energy;
    if m.supply <= 0.0 || view.own.energy_capacity <= 0.0 {
        return s;
    }
    let current = view.own.energy_policy.factor();
    let ours = view.own.energy_capacity * current;
    let change = view.own.energy_capacity * policy.factor() - ours;
    // Price response: ΔP/P ≈ −k × Δsupply/supply.
    let dp = (-PRICE_EXPONENT * change / m.supply).max(-0.9);
    // Revenue is earned on what is sold abroad, not on what is pumped (the
    // market's `revenue_factor` is on net exports): a producer that burns
    // most of its own output loses half its exports to a 15% cut and gains
    // a tenth on the price, so restraint is ruinous for it and nearly free
    // for a rentier. The USSR of the 1980s was a volume-maximising
    // price-taker for exactly this reason; Saudi Arabia could swing.
    let exports = view.own.energy_net_exports.max(1e-9);
    let revenue_ratio = ((exports + change) / exports).max(0.0) * (1.0 + dp) - 1.0;
    let dependence = (view.own.energy_net_exports / view.own.gdp).clamp(0.0, 3.0);
    s.add("export revenue", 40.0 * revenue_ratio * dependence * (0.5 + pers.greed));
    // Squeezing others: price moves hit hostile exporters and importers.
    let mut squeeze = 0.0;
    for f in &view.others {
        let hostility = (-f.our_opinion_of_them / 100.0).max(f.tension / 100.0).clamp(0.0, 1.0);
        if hostility < 0.2 {
            continue;
        }
        let exposure = (f.energy_net_exports / f.gdp.max(1e-9)).clamp(-1.0, 3.0);
        squeeze += -dp * exposure * hostility;
    }
    s.add(
        "squeezing hostile producers and consumers",
        20.0 * squeeze * (0.5 + pers.aggression),
    );
    if policy == view.own.energy_policy {
        s.add("status quo", 4.0);
    }
    reflexes::apply(
        view,
        DecisionKind::Flood,
        Parties {
            counterpart: None,
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Nuclear cover over the observer: the best arsenal among its protectors,
/// discounted by its Credibility(Back) reading of them.
fn own_cover(view: &ObserverView) -> f64 {
    let me = view.observer;
    view.others
        .iter()
        .filter(|p| {
            view.treaties.iter().any(|t| match t.kind {
                TreatyKind::DefensiveAlliance => t.involves(me) && t.involves(p.id),
                TreatyKind::Guarantee => t.a == p.id && t.b == me,
                _ => false,
            })
        })
        .map(|p| p.arsenal as f64 * p.credibility_back / 100.0)
        .fold(0.0, f64::max)
}

/// Value of starting a covert arsenal programme (P10).
pub fn start_programme(view: &ObserverView) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let (threat_f, t) = top_threat(view).map_or((None, 0.0), |(f, t)| (Some(f), t));
    let cover = own_cover(view);
    s.add("security against our main threat", 0.6 * t * (1.0 - cover / 3.0));
    if threat_f.is_some_and(|f| f.arsenal > 0 || f.programme.is_some()) {
        s.add("they have, or are building, the bomb", 15.0 * (0.5 + pers.paranoia));
    }
    s.add("programme cost", -10.0 * (0.5 + pers.greed));
    let capability = sim_core::programme::capability(&view.own);
    s.add("a programme we can barely sustain", -25.0 * (1.0 - capability).max(0.0));
    s.add("exposure: sanctions and isolation", -15.0 * (1.0 - pers.risk));
    s.add("non-proliferation pressure", -15.0);
    // A client going nuclear angers the patron that arms or funds it.
    let patron_has_bomb = view
        .streams
        .iter()
        .filter(|st| st.to == view.observer)
        .any(|st| crate::inputs::foreign(view, st.from).is_some_and(|p| p.arsenal > 0));
    if patron_has_bomb {
        s.add("our patron's displeasure", -20.0);
    }
    s.add("status quo", -15.0);
    reflexes::apply(
        view,
        DecisionKind::Programme,
        Parties {
            counterpart: threat_f,
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Value of declaring an undeclared arsenal: deterrence that works on
/// everyone, at an opinion cost.
pub fn declare_arsenal(view: &ObserverView) -> Score {
    let mut s = Score::new();
    let t = top_threat(view).map_or(0.0, |x| x.1);
    let rival_declared = top_threat(view).is_some_and(|(f, _)| f.arsenal > 0);
    s.add("deterrence that everyone can see", 0.4 * t);
    if rival_declared {
        s.add("our rival's arsenal is public", 10.0);
    }
    s.add("the world's reaction", -15.0);
    s.add("status quo (ambiguity)", -8.0);
    reflexes::apply(
        view,
        DecisionKind::Test,
        Parties {
            counterpart: top_threat(view).map(|x| x.0),
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Value of disclosing and dismantling (the South Africa 1991 pattern):
/// trust and relief from isolation when the threat has faded.
pub fn dismantle(view: &ObserverView, develop: bool) -> Score {
    let mut s = Score::new();
    // The threat we'd face without the arsenal, not with it.
    let t = view
        .others
        .iter()
        .map(|f| crate::inputs::threat_without_arsenal(view, f))
        .fold(0.0, f64::max);
    // Isolation that dismantling can end: sanctions from arsenal states
    // (the non-proliferation pressure), not those of plain enemies.
    let sanctions = view
        .sanctions
        .iter()
        .filter(|x| x.target == view.observer)
        .filter(|x| crate::inputs::foreign(view, x.by).is_some_and(|f| f.arsenal > 0 && f.tension < 50.0))
        .count() as f64;
    s.add("trust and an end to isolation", 8.0 * sanctions.min(3.0) + 4.0);
    if develop {
        s.add("goal: development", 8.0);
    }
    s.add("deterrence we give up", -0.8 * t - 5.0 * view.own.arsenal as f64);
    s.add("status quo", -10.0);
    reflexes::apply(
        view,
        DecisionKind::Dismantle,
        Parties {
            counterpart: None,
            secondary: None,
        },
        &mut s,
    );
    s
}

/// Non-proliferation pressure: a nuclear power that can see a programme (or
/// a fresh undeclared arsenal) in a country that isn't its ally or client.
pub fn proliferating_stranger(view: &ObserverView, target: &ForeignView) -> bool {
    if view.own.arsenal == 0 || target.programme.is_none() {
        return false;
    }
    let me = view.observer;
    let protected = view.treaties.iter().any(|t| match t.kind {
        TreatyKind::DefensiveAlliance => t.involves(me) && t.involves(target.id),
        TreatyKind::Guarantee => t.a == me && t.b == target.id,
        _ => false,
    }) || view.streams.iter().any(|s| s.from == me && s.to == target.id);
    !protected
}

/// Value of continuing a running programme (forward-looking: what is left
/// to pay against what it buys; pressure from nuclear powers counts).
pub fn keep_programme(view: &ObserverView) -> Score {
    let mut s = start_programme(view);
    // Running is now the status quo: undo the starting inertia and add the
    // inertia of continuing.
    s.add("status quo (it is already running)", 15.0 + 10.0);
    let progress = view.own.programme.unwrap_or(0.0);
    s.add("already part-built", 0.3 * progress);
    let pressure = view
        .sanctions
        .iter()
        .filter(|x| x.target == view.observer)
        .filter(|x| crate::inputs::foreign(view, x.by).is_some_and(|f| f.arsenal > 0))
        .count() as f64;
    // Pressure saturates: two sanctioning nuclear powers say all there is to say.
    s.add("sanctions from nuclear powers", -12.0 * pressure.min(2.0));
    s
}

/// Reform weight per failed crackdown in the crisis era, for a regime with
/// no mission ideology.
pub const REPRESSION_LESSON: f64 = 15.0;

/// Value of reforming (positive) rather than cracking down (negative) in a
/// transition crisis (scenario P3). Ideology and paranoia hold on; sanctions
/// relief, lost legitimacy, coup risk and failed repression push toward
/// reform.
pub fn reform(view: &ObserverView) -> Score {
    let mut s = Score::new();
    let pers = view.own.personality;
    let democratic_sanctions = view
        .sanctions
        .iter()
        .filter(|x| x.target == view.observer)
        .filter(|x| crate::inputs::foreign(view, x.by).is_some_and(|f| f.government == sim_core::Government::Democracy))
        .count() as f64;
    s.add("an end to sanctions and isolation", 6.0 * democratic_sanctions.min(4.0));
    s.add(
        "legitimacy we no longer have",
        0.5 * (40.0 - view.own.legitimacy).max(0.0),
    );
    s.add(
        "coup risk if we crack down again",
        10.0 * view.own.transition.crackdowns as f64,
    );
    // Repression already failed in this crisis era (the record survives a
    // coup). Rulers without a mission ideology read that as the signal to
    // negotiate an exit; true believers read it as a call for more.
    s.add(
        "repression has already failed",
        REPRESSION_LESSON * view.own.transition.repressions as f64 * (1.0 - pers.ideology),
    );
    s.add("the ideology we would give up", -30.0 * pers.ideology);
    s.add("fear of what reform unleashes", -20.0 * pers.paranoia);
    s.add("status quo (crackdown)", -5.0);
    reflexes::apply(
        view,
        DecisionKind::Reform,
        Parties {
            counterpart: None,
            secondary: None,
        },
        &mut s,
    );
    s
}
