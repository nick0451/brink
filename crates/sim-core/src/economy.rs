//! Economy-lite (implementation plan §4). All rates are per quarter.
//!
//! Order within the phase: seeded shocks → trade → revenue and spending
//! pools → aid transfers (one-off and streams) → arms transfers → growth,
//! military production, readiness and intelligence spending → sanction
//! adaptation.

use crate::country::{Country, PoolKind};
use crate::diplomacy::StreamKind;
use crate::ids::CountryId;
use crate::trade;
use crate::world::WorldState;

pub const MAX_DEFICIT_RATIO: f64 = 0.3;
/// Share of the gap between actual and target budget closed each turn.
pub const BUDGET_INERTIA: f64 = 0.5;
/// Largest share of its spending pool a country can pay out as aid per turn.
pub const MAX_AID_SHARE: f64 = 0.5;
/// Growth-trend smoothing per turn and its starting value.
pub const TREND_RATE: f64 = 0.05;
pub const TREND_START: f64 = 0.005;
/// Baseline productivity trend per quarter (technology at the frontier).
const BASE_GROWTH: f64 = 0.005;
const CATCH_UP: f64 = 0.009;
const DEVELOPMENT_GROWTH: f64 = 0.008;
/// Growth per unit of (trade above or below the gravity baseline) ÷ GDP.
const TRADE_GROWTH: f64 = 0.015;
/// The trade gap counts only up to this share of GDP: agreements add
/// growth, but a dense trade web can't compound without limit.
const TRADE_GAP_CAP: f64 = 0.1;
pub const DEBT_DRAG_THRESHOLD: f64 = 0.6;
const DEBT_DRAG: f64 = 0.004;
const STABILITY_GROWTH_DIVISOR: f64 = 20000.0;
/// War consumption on credit (issue 21): a mobilized belligerent burns
/// munitions, fuel, spares and soldiers' pay that its peacetime budget does
/// not cover, and borrows the bill. Debt per year as a share of annual GDP
/// (per quarter as a share of quarterly GDP): Partial 3% (the US in Vietnam
/// spent ~2-3% of GDP a year on the war), Full 12% (wartime deficits of a
/// state fighting a sustained conventional war: Israel 1973-75, Iran in the
/// 1980s), Total 25% (Britain and the US at the height of the Second World
/// War; Iraq borrowed ~$80bn over 1980-88 on a ~$45bn economy). Peacetime
/// mobilization fights nothing and borrows nothing: its cost is the
/// conscription growth drag alone.
pub fn war_borrowing_rate(m: crate::country::Mobilization) -> f64 {
    use crate::country::Mobilization;
    match m {
        Mobilization::Peacetime => 0.0,
        Mobilization::Partial => 0.03,
        Mobilization::Full => 0.12,
        Mobilization::Total => 0.25,
    }
}

/// Strength points bought per unit of military spending.
pub const MILITARY_CONVERSION: f64 = 0.5;
/// Share of strength lost to wear each turn unless replaced.
pub const MILITARY_UPKEEP: f64 = 0.05;
/// Largest share of its strength a country can give away as arms per turn.
/// A supplier gives at most what it replaces each turn (the upkeep rate).
pub const MAX_ARMS_SHARE: f64 = MILITARY_UPKEEP;
/// A buyer spends at most this share of its pool on arms purchases (D58).
pub const MAX_PURCHASE_SHARE: f64 = 0.15;
/// Arms export order book per turn, as a share of GDP at industry 1 (D58).
pub const EXPORT_CAPACITY: f64 = 0.03;
/// Received arms can be at most this many levels above the recipient's
/// Military tech (DESIGN §21.6): it can't operate what it can't maintain.
pub const ARMS_QUALITY_MARGIN: f64 = 2.0;
/// Share of the gap to target readiness closed each turn.
const READINESS_RATE: f64 = 0.25;
/// Readiness is rebuilt more slowly than it is lost (training, officers).
const READINESS_RECOVERY_RATE: f64 = 0.12;
const INTEL_CONVERSION: f64 = 0.5;
const INTEL_UPKEEP: f64 = 0.05;

/// Intelligence capacity that a steady quarterly intelligence spend sustains.
pub fn intel_equilibrium(intel_spend: f64) -> f64 {
    intel_spend.max(0.0) * INTEL_CONVERSION / INTEL_UPKEEP
}

pub fn run(state: &mut WorldState) {
    let n = state.countries.len();
    let shock_width = state.params.growth_shock;
    let shocks: Vec<f64> = (0..n).map(|_| state.roll_symmetric(shock_width)).collect();
    // The technology frontier: income per head of the largest economy
    // (catch-up growth converges on the leader, not on a small rich
    // commodity exporter).
    let frontier = state
        .countries
        .iter()
        .filter(|c| c.active)
        .max_by(|a, b| a.gdp.total_cmp(&b.gdp))
        .map_or(f64::MIN_POSITIVE, |c| c.gdp_per_capita().max(f64::MIN_POSITIVE));
    let trade = trade::compute(state);
    crate::money::update(state);
    let money_growth = state.money.stance.growth_effect();
    crate::commodity::clear(state);
    let deviation = state.energy.deviation();

    // Revenue and spending pools.
    let mut pools = vec![0.0; n];
    let mut trade_gap = vec![0.0; n];
    let belligerent: Vec<bool> = state.countries.iter().map(|c| state.wars.is_belligerent(c.id)).collect();
    for (i, c) in state.countries.iter_mut().enumerate() {
        c.budget = c.budget.approach(c.budget_target, BUDGET_INERTIA);
        let revenue = c.gdp * c.tax_rate;
        let debt_service = c.debt_service;
        let energy = crate::commodity::revenue_factor(c.energy_net_exports, c.gdp, deviation);
        let energy = crate::commodity::oil_budget(c, energy);
        pools[i] = (revenue * energy * (1.0 + c.deficit_ratio) - debt_service).max(0.0);
        c.debt += revenue * c.deficit_ratio;
        // War consumption on credit: the bill is borrowed, not bought (the
        // mobilized army's readiness is already granted by its level).
        c.war_borrowing = if belligerent[i] && c.active {
            c.gdp * war_borrowing_rate(c.forces.mobilization)
        } else {
            0.0
        };
        c.debt += c.war_borrowing;
        c.trade = trade.volume[i];
        trade_gap[i] = trade.volume[i] - trade.baseline[i];
        c.trade_lost = trade.lost[i];
        c.aid_in = 0.0;
        c.aid_out = 0.0;
    }

    // War loans (issue 24): each creditor receives the interest on its
    // share of the debtor's debt (part of the debt service the debtor
    // already pays). The principal is never repaid; it stands until it is
    // forgiven.
    for l in &state.diplomacy.loans {
        let d = state.country(l.debtor);
        if d.debt > 0.0 && d.active {
            pools[l.creditor.index()] += d.debt_service * (l.amount / d.debt).min(1.0);
        }
    }

    // Aid: one-off pledges first, then standing streams, in a fixed order.
    // One-off aid paid to a state at war is a war loan (issue 24, D95): it
    // buys the same army, but the recipient's debt rises by it and the
    // funder holds the claim. Standing streams and peacetime aid are gifts.
    let mut transfers: Vec<(CountryId, CountryId, f64)> = std::mem::take(&mut state.diplomacy.pending_aid);
    let one_off = transfers.len();
    transfers.extend(
        state
            .diplomacy
            .streams
            .iter()
            .filter(|s| s.kind == StreamKind::Aid)
            .map(|s| (s.from, s.to, s.amount)),
    );
    let mut paid_cap: Vec<f64> = pools.iter().map(|p| p * MAX_AID_SHARE).collect();
    let turn = state.turn;
    for (k, (from, to, amount)) in transfers.into_iter().enumerate() {
        let pay = amount.min(paid_cap[from.index()]).max(0.0);
        paid_cap[from.index()] -= pay;
        pools[from.index()] -= pay;
        pools[to.index()] += pay;
        if k < one_off && belligerent[to.index()] && pay > 0.0 {
            state.country_mut(to).debt += pay;
            state.diplomacy.lend(from, to, pay, turn);
        }
        state.countries[from.index()].aid_out += pay;
        state.countries[to.index()].aid_in += pay;
    }

    // Arms: one-off transfers first, then standing arms streams.
    for c in &mut state.countries {
        c.arms_in = 0.0;
        c.arms_out = 0.0;
        c.arms_sold = 0.0;
        c.arms_income = 0.0;
    }
    let pending = std::mem::take(&mut state.diplomacy.pending_arms);
    let mut arms_cap: Vec<f64> = state
        .countries
        .iter()
        .map(|c| c.forces.strength() * MAX_ARMS_SHARE)
        .collect();
    let mut arms: Vec<(CountryId, CountryId, f64)> = pending;
    arms.extend(
        state
            .diplomacy
            .streams
            .iter()
            .filter(|s| s.kind == StreamKind::Arms && !s.sale)
            .map(|s| (s.from, s.to, s.amount)),
    );
    for (from, to, amount) in arms {
        if !crate::war::arms_route(state, from, to) {
            continue;
        }
        let give = amount.min(arms_cap[from.index()]).max(0.0);
        arms_cap[from.index()] -= give;
        transfer_arms(state, from, to, give);
    }

    // Arms sales (D58): the buyer pays the market price out of its pool, up
    // to a share of it; the seller's industry builds most of what it sells,
    // within an order book sized by its industry and economy.
    let sales: Vec<(CountryId, CountryId, f64)> = state
        .diplomacy
        .streams
        .iter()
        .filter(|s| s.kind == StreamKind::Arms && s.sale)
        .map(|s| (s.from, s.to, s.amount))
        .collect();
    let mut purse: Vec<f64> = pools.iter().map(|p| p * MAX_PURCHASE_SHARE).collect();
    let mut order_book: Vec<f64> = state.countries.iter().map(export_capacity).collect();
    for (from, to, amount) in sales {
        if !crate::war::arms_route(state, from, to) {
            continue;
        }
        let sanctioners = state.diplomacy.sanctions.iter().filter(|s| s.target == to).count();
        let price = crate::diplomacy::arms_price(sanctioners, state.wars.is_belligerent(to));
        let industry = state.country(from).arms_industry;
        let unit = crate::diplomacy::arms_value(StreamKind::Arms, 1.0) * price;
        let affordable = purse[to.index()].min(order_book[from.index()]) / unit;
        let drawn_cap = arms_cap[from.index()] / (1.0 - industry).max(1e-9);
        let give = amount.min(affordable).min(drawn_cap).max(0.0);
        if give <= 1e-12 {
            continue;
        }
        let pay = give * unit;
        purse[to.index()] -= pay;
        order_book[from.index()] -= pay;
        arms_cap[from.index()] -= give * (1.0 - industry);
        pools[to.index()] -= pay;
        pools[from.index()] += pay;
        sell_arms(state, from, to, give, industry);
        let seller = state.country_mut(from);
        seller.arms_income += pay;
    }

    // Growth and capability spending.
    let turn = state.turn;
    for (i, c) in state.countries.iter_mut().enumerate() {
        let catch_up = CATCH_UP * (1.0 - c.gdp_per_capita() / frontier).max(0.0);
        let growth = BASE_GROWTH
            + catch_up
            + DEVELOPMENT_GROWTH * c.budget.development
            + TRADE_GROWTH * (trade_gap[i] / c.gdp).clamp(-TRADE_GAP_CAP, TRADE_GAP_CAP)
            - DEBT_DRAG * (c.debt_ratio() - DEBT_DRAG_THRESHOLD).max(0.0)
            + (c.stability - 50.0) / STABILITY_GROWTH_DIVISOR
            - c.forces.mobilization.growth_drag()
            + crate::commodity::growth_effect(c.energy_net_exports, c.gdp, deviation)
            + c.growth_modifier
            + c.command_drag
            + money_growth
            + shocks[i];
        c.gdp = (c.gdp * (1.0 + growth)).max(0.001);
        c.last_growth = growth;
        c.growth_trend += (growth - c.growth_trend) * TREND_RATE;

        if c.programme.is_some() {
            pools[i] -= pools[i] * crate::programme::PROGRAMME_COST;
        }
        let pool = pools[i];
        let command = crate::transition::command(c, turn);
        produce_forces(c, pool * c.budget.military * MILITARY_CONVERSION / c.military_cost, command);
        c.intel_capacity = (c.intel_capacity + pool * c.budget.intelligence * INTEL_CONVERSION
            - c.intel_capacity * INTEL_UPKEEP)
            .max(0.0);
    }

    trade::adapt(state);
}

/// Upkeep, new production at the country's own Military tech, and readiness.
/// Readiness heads toward the mobilization cap scaled by how much of the
/// upkeep this turn's production (plus arms received, which arrive serviced)
/// covers: a force that is no longer being replaced goes hollow before it
/// shrinks. `command` ([`crate::transition::command`]) scales the ceiling:
/// a regime in crisis or one purging its officer corps cannot hold its
/// army at the readiness it pays for.
fn produce_forces(c: &mut Country, production: f64, command: f64) {
    let upkeep = c.forces.strength() * MILITARY_UPKEEP;
    let funding = if upkeep > 1e-9 {
        ((production + c.arms_in) / upkeep).min(1.0)
    } else {
        1.0
    };
    let quality = c.military_tech as f64;
    let mix = c.force_mix;
    for kind in PoolKind::ALL {
        let pool = c.forces.pool_mut(kind);
        pool.strength *= 1.0 - MILITARY_UPKEEP;
        pool.add(production.max(0.0) * mix.share(kind), quality);
    }
    let target = c.forces.mobilization.readiness_cap() * funding * command;
    let rate = if target > c.forces.readiness { READINESS_RECOVERY_RATE } else { READINESS_RATE };
    c.forces.readiness += (target - c.forces.readiness) * rate;
    for o in &mut c.arms_origin {
        o.1 *= 1.0 - MILITARY_UPKEEP;
    }
    c.arms_origin.retain(|o| o.1 > 1e-3);
}

/// Move `amount` strength points from `from`'s pools (in proportion to their
/// size) into the same pools of `to`. Incoming quality is capped at the
/// recipient's Military tech + [`ARMS_QUALITY_MARGIN`] and averaged in.
/// Nothing ever moves back (DESIGN §21.6).
/// Money a seller can take in arms orders per turn (D58).
pub fn export_capacity(c: &crate::country::Country) -> f64 {
    c.arms_industry * c.gdp * EXPORT_CAPACITY
}

/// Deliver `amount` sold strength: the buyer receives all of it (permanent,
/// recorded by origin for blowback); the seller's forces give up only the
/// part its industry didn't build.
fn sell_arms(state: &mut WorldState, from: CountryId, to: CountryId, amount: f64, industry: f64) {
    let total = state.country(from).forces.strength();
    if amount <= 0.0 || total <= 1e-9 || from == to {
        return;
    }
    let cap = state.country(to).military_tech as f64 + ARMS_QUALITY_MARGIN;
    for kind in PoolKind::ALL {
        let src = *state.country(from).forces.pool(kind);
        let moved = amount * src.strength / total;
        state.country_mut(from).forces.pool_mut(kind).strength -= moved * (1.0 - industry);
        state
            .country_mut(to)
            .forces
            .pool_mut(kind)
            .add(moved, src.quality.min(cap));
    }
    state.country_mut(from).arms_sold += amount;
    let recipient = state.country_mut(to);
    recipient.arms_in += amount;
    match recipient.arms_origin.iter_mut().find(|o| o.0 == from) {
        Some(o) => o.1 += amount,
        None => {
            recipient.arms_origin.push((from, amount));
            recipient.arms_origin.sort_by_key(|o| o.0);
        }
    }
}

pub fn transfer_arms(state: &mut WorldState, from: CountryId, to: CountryId, amount: f64) {
    let total = state.country(from).forces.strength();
    if amount <= 0.0 || total <= 1e-9 || from == to {
        return;
    }
    let amount = amount.min(total);
    let cap = state.country(to).military_tech as f64 + ARMS_QUALITY_MARGIN;
    for kind in PoolKind::ALL {
        let src = *state.country(from).forces.pool(kind);
        let moved = amount * src.strength / total;
        state.country_mut(from).forces.pool_mut(kind).strength -= moved;
        state
            .country_mut(to)
            .forces
            .pool_mut(kind)
            .add(moved, src.quality.min(cap));
    }
    let supplier = state.country_mut(from);
    supplier.arms_out += amount;
    supplier.arms_out_total += amount;
    let recipient = state.country_mut(to);
    recipient.arms_in += amount;
    match recipient.arms_origin.iter_mut().find(|o| o.0 == from) {
        Some(o) => o.1 += amount,
        None => {
            recipient.arms_origin.push((from, amount));
            recipient.arms_origin.sort_by_key(|o| o.0);
        }
    }
}
