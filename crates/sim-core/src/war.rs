//! War-lite (DESIGN §6.3–6.6, §11.2; implementation plan step 7).
//!
//! Data model: `War → Vec<Front>`. v0.1 instantiates one front per war,
//! between the two leaders; every belligerent commits a share of its power
//! to it. Multiple fronts per war are an extension, not a redesign.
//!
//! - **Aims:** `Punitive` (a short strike with air and naval power; ends
//!   only if the target accepts peace, otherwise becomes a limited war),
//!   `Limited`, `Major` (regime change).
//! - **Involvement Bands** (DESIGN §11.2): 0 none, 1 coerce, 2 proxy, 3
//!   strike/limited, 4 major. Acting at band 2+ for a side creates an
//!   [`Involvement`]: a Back commitment with the band's stake, which takes
//!   the visibility of the act that created it. Covert exits are cheap;
//!   public high-band exits are not.
//! - **Proxy supply needs a route:** a base in the recipient, a consenting
//!   conduit (a base in a third country), or enough naval power.
//! - **No scripted outcomes:** wars end by decisive progress, mutual peace
//!   offers or exhaustion; consequences flow through ordinary state.

use serde::{Deserialize, Serialize};

use crate::country::{Government, Mobilization, PoolKind};
use crate::diplomacy::{DiplomaticEvent, StreamKind, TreatyKind};
use crate::ids::CountryId;
use crate::intel;
use crate::ledger::{self, CauseCode, EntryKind, Grade, LedgerLog, NormTag, Visibility};
use crate::opinion::OpinionModifier;
use crate::orders::Order;
use crate::world::WorldState;

/// Front progress per unit of ln(attack ÷ defence), per turn.
const PROGRESS_RATE: f64 = 12.0;
const MAX_PROGRESS_STEP: f64 = 25.0;
/// Multiplier for home ground and fortification (DESIGN §6.5): the
/// defender's, fading to the attacker's as the front is pushed back into
/// the attacker's territory ([`War::home_share`]).
pub const DEFENCE_MULT: f64 = 1.5;
/// Share of committed strength lost per turn at an even ratio.
const BASE_LOSS: f64 = 0.03;
/// Punitive strikes are stand-off: the striker loses far less.
const PUNITIVE_STRIKER_LOSS: f64 = 0.01;
/// Turns a punitive strike runs before it must end in peace or escalate.
pub const PUNITIVE_TURNS: u32 = 2;
/// Peace offers stay open this many turns.
const OFFER_TURNS: u32 = 2;
/// A leader at this war weariness can't continue: its side sues for peace.
pub const EXHAUSTION: f64 = 60.0;
/// |progress| needed for a negotiated peace to count as a win.
const WIN_MARGIN: f64 = 25.0;
/// Turns allies have to answer an attack on a country they defend.
pub const ALLY_RESPONSE_TURNS: u32 = 2;
/// Per-turn chance that a covert arms stream is exposed (DESIGN §8.4).
pub const EXPOSURE_CHANCE: f64 = 0.04;
/// Sea route: own naval value must reach this share of the enemy's.
const SEA_ROUTE_SHARE: f64 = 0.5;
/// Tension below which a declaration has no casus belli (DESIGN §11.3).
pub const CASUS_BELLI_TENSION: f64 = 40.0;
const NO_CASUS_BELLI_LEGITIMACY: f64 = 15.0;
const WAR_TENSION: f64 = 30.0;
const JOIN_TENSION: f64 = 20.0;
const MOBILIZE_TENSION: f64 = 5.0;
const WEARINESS_PER_TURN: f64 = 0.5;
const WEARINESS_DECAY: f64 = 0.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WarId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarAim {
    Punitive,
    Limited,
    Major,
}

impl WarAim {
    /// Involvement band of the declarer.
    pub fn band(self) -> u8 {
        match self {
            WarAim::Major => 4,
            _ => 3,
        }
    }

    /// Share of the declarer's power committed.
    pub fn allocation(self) -> f64 {
        match self {
            WarAim::Punitive => 0.3,
            WarAim::Limited => 0.5,
            WarAim::Major => 0.8,
        }
    }

    /// Progress and casualty multiplier (offensive intensity).
    fn intensity(self) -> f64 {
        match self {
            WarAim::Major => 1.5,
            _ => 1.0,
        }
    }

    /// Offensive weight the declarer puts behind the aim: committed share ×
    /// intensity, the quantity that drives front progress (Punitive 0.3,
    /// Limited 0.5, Major 1.2). Public so observers can judge how far an
    /// aim threatens the target without reading hidden state.
    pub fn offensive_weight(self) -> f64 {
        self.allocation() * self.intensity()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Attacker,
    Defender,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Attacker => Side::Defender,
            Side::Defender => Side::Attacker,
        }
    }
}

/// Share of power committed by a joiner at a band.
pub fn band_allocation(band: u8) -> f64 {
    match band {
        0..=2 => 0.0,
        3 => 0.3,
        _ => 0.7,
    }
}

/// Involvement stake per band (DESIGN §11.2).
pub fn band_stake(band: u8) -> f64 {
    [0.0, 0.3, 0.5, 1.0, 1.5][band.min(4) as usize]
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Belligerent {
    pub country: CountryId,
    pub side: Side,
    pub band: u8,
    /// Share of power committed to this war.
    pub allocation: f64,
    pub joined: u32,
    /// Strength lost in this war.
    pub losses: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Front {
    pub id: u32,
    pub attacker: CountryId,
    pub defender: CountryId,
    /// −100..100, positive favours the attacker. ±100 is decisive.
    pub progress: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct War {
    pub id: WarId,
    pub aim: WarAim,
    pub attacker: CountryId,
    pub defender: CountryId,
    pub started: u32,
    pub belligerents: Vec<Belligerent>,
    pub fronts: Vec<Front>,
    /// Leaders' peace offers: (leader, turn offered).
    pub peace_offers: Vec<(CountryId, u32)>,
}

impl War {
    pub fn side_of(&self, c: CountryId) -> Option<Side> {
        self.belligerents.iter().find(|b| b.country == c).map(|b| b.side)
    }

    pub fn belligerent(&self, c: CountryId) -> Option<&Belligerent> {
        self.belligerents.iter().find(|b| b.country == c)
    }

    pub fn leader(&self, side: Side) -> CountryId {
        match side {
            Side::Attacker => self.attacker,
            Side::Defender => self.defender,
        }
    }

    pub fn is_leader(&self, c: CountryId) -> bool {
        c == self.attacker || c == self.defender
    }

    pub fn side(&self, side: Side) -> impl Iterator<Item = &Belligerent> {
        self.belligerents.iter().filter(move |b| b.side == side)
    }

    /// Mean front progress (attacker-positive).
    pub fn progress(&self) -> f64 {
        if self.fronts.is_empty() {
            0.0
        } else {
            self.fronts.iter().map(|f| f.progress).sum::<f64>() / self.fronts.len() as f64
        }
    }

    /// Progress from `side`'s point of view (positive = winning).
    pub fn progress_for(&self, side: Side) -> f64 {
        match side {
            Side::Attacker => self.progress(),
            Side::Defender => -self.progress(),
        }
    }

    /// The side on whose soil the front lies: the defender's until the
    /// attacker is thrown back over its own border (progress below zero).
    /// Home ground (fortification, short supply lines) and the home-front
    /// rally belong to that side, whoever declared the war (DESIGN §6.5
    /// "home-core bonus"): Iraq's failed invasion became a defence of Basra
    /// in 1982, and Iran's counter-offensives then broke on prepared lines.
    /// This is the discrete reading; combat uses [`War::home_share`].
    pub fn home_side(&self) -> Side {
        if self.progress() < 0.0 {
            Side::Attacker
        } else {
            Side::Defender
        }
    }

    /// Share (0–1) of the home-ground advantage `side` enjoys. Defensive
    /// advantage builds with depth (prepared lines, short supply, familiar
    /// terrain); it does not appear at the border line. The declared
    /// defender holds it in full from the start (it was defending from the
    /// first day); a thrown-back invader gains it, and the counter-attacker
    /// loses it, over the first [`WIN_MARGIN`] of progress into the
    /// invader's territory, where the counter-attacker's supply lines
    /// lengthen and the invader falls back on its own prepared positions.
    /// Continuous in progress, so a front held near the border is a line,
    /// not a flip-flop (D65 follow-up).
    pub fn home_share(&self, side: Side) -> f64 {
        let s = (-self.progress() / WIN_MARGIN).clamp(0.0, 1.0);
        match side {
            Side::Defender => 1.0 - s,
            Side::Attacker => s,
        }
    }

    /// Combat multiplier of `side`: 1 to [`DEFENCE_MULT`] by its
    /// [`War::home_share`].
    pub fn ground_mult(&self, side: Side) -> f64 {
        1.0 + (DEFENCE_MULT - 1.0) * self.home_share(side)
    }

    fn offered(&self, c: CountryId, turn: u32) -> bool {
        self.peace_offers.iter().any(|&(x, t)| x == c && t + OFFER_TURNS > turn)
    }
}

/// A Back commitment created by acting at band 2+ for a side (DESIGN §11.2).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Involvement {
    pub actor: CountryId,
    pub beneficiary: CountryId,
    pub war: WarId,
    pub band: u8,
    pub visibility: Visibility,
    pub since: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeaceResult {
    WhitePeace,
    AttackerWon,
    DefenderWon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeaceReason {
    Decisive,
    Negotiated,
    Exhaustion,
    /// A punitive strike the target accepted.
    StrikeAccepted,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Wars {
    pub active: Vec<War>,
    pub involvements: Vec<Involvement>,
    /// (supplier, war) pairs already recorded under the both-sides rule.
    pub both_sides: Vec<(CountryId, WarId)>,
    /// Wars fought so far (ended), for statistics.
    pub ended: u32,
    next_id: u32,
}

impl Wars {
    pub fn find(&self, id: WarId) -> Option<&War> {
        self.active.iter().find(|w| w.id == id)
    }

    fn find_mut(&mut self, id: WarId) -> Option<&mut War> {
        self.active.iter_mut().find(|w| w.id == id)
    }

    /// Are `a` and `b` on opposite sides of any war?
    pub fn at_war(&self, a: CountryId, b: CountryId) -> bool {
        self.active.iter().any(|w| match (w.side_of(a), w.side_of(b)) {
            (Some(x), Some(y)) => x != y,
            _ => false,
        })
    }

    pub fn is_belligerent(&self, c: CountryId) -> bool {
        self.active.iter().any(|w| w.side_of(c).is_some())
    }

    pub fn wars_of(&self, c: CountryId) -> impl Iterator<Item = &War> {
        self.active.iter().filter(move |w| w.side_of(c).is_some())
    }

    /// Countries on the other side from `c` in any of its wars.
    pub fn enemies_of(&self, c: CountryId) -> Vec<CountryId> {
        let mut v: Vec<CountryId> = self
            .active
            .iter()
            .filter_map(|w| w.side_of(c).map(|s| (w, s)))
            .flat_map(|(w, s)| w.side(s.other()).map(|b| b.country).collect::<Vec<_>>())
            .collect();
        v.sort();
        v.dedup();
        v
    }

    /// Total share of power `c` has committed across its wars.
    pub fn total_allocation(&self, c: CountryId) -> f64 {
        self.active
            .iter()
            .filter_map(|w| w.belligerent(c))
            .map(|b| b.allocation)
            .sum()
    }
}

fn modifier(source: &str, value: f64, decay: f64) -> OpinionModifier {
    OpinionModifier {
        source: source.into(),
        value,
        decay,
        memory: None,
    }
}

impl Government {
    /// War weariness per unit of casualties (DESIGN §9.1).
    pub fn weariness_sensitivity(self) -> f64 {
        match self {
            Government::Democracy => 1.5,
            Government::Authoritarian => 0.8,
            Government::Revolutionary => 0.6,
        }
    }

    /// `c` in war-of-choice weariness × (1 + c·(1 − threat from the enemy))
    /// (N8; replaces a flat "Vietnam syndrome" multiplier).
    pub fn war_of_choice(self) -> f64 {
        match self {
            Government::Democracy => 0.5,
            Government::Authoritarian => 0.2,
            Government::Revolutionary => 0.1,
        }
    }
}

impl Mobilization {
    /// Growth lost per quarter while mobilized (DESIGN §6.2: −5% / −15% /
    /// −30% GDP, spread over two years).
    pub fn growth_drag(self) -> f64 {
        match self {
            Mobilization::Peacetime => 0.0,
            Mobilization::Partial => 0.05 / 8.0,
            Mobilization::Full => 0.15 / 8.0,
            Mobilization::Total => 0.30 / 8.0,
        }
    }

    pub fn level(self) -> u8 {
        match self {
            Mobilization::Peacetime => 0,
            Mobilization::Partial => 1,
            Mobilization::Full => 2,
            Mobilization::Total => 3,
        }
    }
}

/// Can `from` deliver arms to `to`? Only matters when `to` is at war:
/// needs a base in `to`, a consenting conduit (a base in a third country
/// not fighting `to`), co-belligerence, or naval power of at least half the
/// strongest enemy navy (N4, F10).
pub fn arms_route(state: &WorldState, from: CountryId, to: CountryId) -> bool {
    let wars = &state.wars;
    if !wars.is_belligerent(to) {
        return true;
    }
    let enemies = wars.enemies_of(to);
    if wars
        .wars_of(to)
        .any(|w| w.side_of(from).is_some() && w.side_of(from) == w.side_of(to))
    {
        return true;
    }
    let d = &state.diplomacy;
    if d.has_treaty(TreatyKind::Basing, from, to) {
        return true;
    }
    // Neighbours (same area) deliver overland.
    if matches!((&state.country(from).area, &state.country(to).area), (Some(a), Some(b)) if a == b) {
        return true;
    }
    let conduit = d.treaties.iter().any(|t| {
        t.kind == TreatyKind::Basing && t.a == from && t.b != to && !enemies.contains(&t.b) && !wars.at_war(t.b, to)
    });
    if conduit {
        return true;
    }
    let naval = |c: CountryId| {
        let f = &state.country(c).forces;
        f.naval.value() * f.readiness_factor()
    };
    let enemy_navy = enemies.iter().map(|&e| naval(e)).fold(0.0, f64::max);
    naval(from) >= SEA_ROUTE_SHARE * enemy_navy
}

/// Initiative cost of a war order.
pub fn initiative_cost(state: &WorldState, actor: CountryId, order: &Order) -> u8 {
    match order {
        Order::DeclareWar { .. } => 2,
        Order::JoinWar { .. } => 1,
        Order::SetMobilization(level) => u8::from(level.level() > state.country(actor).forces.mobilization.level()),
        _ => 0,
    }
}

/// Resolve a war order (orders phase).
pub fn apply(state: &mut WorldState, actor: CountryId, order: Order) -> Result<Option<DiplomaticEvent>, String> {
    let n = state.countries.len();
    match order {
        Order::DeclareWar { target, aim } => {
            if target.index() >= n || target == actor {
                return Err("invalid war target".into());
            }
            if state.wars.at_war(actor, target) {
                return Err("already at war".into());
            }
            if !state.country(target).active {
                return Err("no such state".into());
            }
            if state.diplomacy.has_treaty(TreatyKind::DefensiveAlliance, actor, target) {
                return Err("cancel the alliance first".into());
            }
            let casus_belli = state.tension.get(actor, target) >= CASUS_BELLI_TENSION;
            let broke_pact = state.diplomacy.has_treaty(TreatyKind::NonAggression, actor, target);
            state
                .diplomacy
                .treaties
                .retain(|t| !(t.kind == TreatyKind::NonAggression && t.involves(actor) && t.involves(target)));
            let id = WarId(state.wars.next_id);
            state.wars.next_id += 1;
            let turn = state.turn;
            state.wars.active.push(War {
                id,
                aim,
                attacker: actor,
                defender: target,
                started: turn,
                belligerents: vec![
                    Belligerent {
                        country: actor,
                        side: Side::Attacker,
                        band: aim.band(),
                        allocation: aim.allocation(),
                        joined: turn,
                        losses: 0.0,
                    },
                    Belligerent {
                        country: target,
                        side: Side::Defender,
                        band: 4,
                        allocation: 1.0,
                        joined: turn,
                        losses: 0.0,
                    },
                ],
                fronts: vec![Front {
                    id: 0,
                    attacker: actor,
                    defender: target,
                    progress: 0.0,
                }],
                // A punitive strike comes with its own offer to stop.
                peace_offers: if aim == WarAim::Punitive {
                    vec![(actor, turn + PUNITIVE_TURNS)]
                } else {
                    Vec::new()
                },
            });
            state.tension.add(actor, target, WAR_TENSION);
            state.opinions.add(target, actor, modifier("attacked us", -50.0, 0.5));
            if !casus_belli {
                let c = state.country_mut(actor);
                c.legitimacy = (c.legitimacy - NO_CASUS_BELLI_LEGITIMACY).max(0.0);
            }
            Ok(Some(DiplomaticEvent::WarDeclared {
                war: id,
                attacker: actor,
                defender: target,
                aim,
                casus_belli,
                broke_pact,
            }))
        }

        Order::JoinWar { war, side, band } => {
            if !(3..=4).contains(&band) {
                return Err("join at band 3 or 4".into());
            }
            let turn = state.turn;
            let w = state.wars.find_mut(war).ok_or("no such war")?;
            if w.side_of(actor).is_some() {
                return Err("already fighting".into());
            }
            w.belligerents.push(Belligerent {
                country: actor,
                side,
                band,
                allocation: band_allocation(band),
                joined: turn,
                losses: 0.0,
            });
            let (friend, enemy) = (w.leader(side), w.leader(side.other()));
            state.tension.add(actor, enemy, JOIN_TENSION);
            state
                .opinions
                .add(friend, actor, modifier("fought beside us", 25.0, 0.5));
            state
                .opinions
                .add(enemy, actor, modifier("joined the war against us", -40.0, 0.5));
            Ok(Some(DiplomaticEvent::JoinedWar {
                war,
                country: actor,
                side,
                band,
            }))
        }

        Order::LeaveWar { war } => {
            let w = state.wars.find_mut(war).ok_or("no such war")?;
            if w.is_leader(actor) {
                return Err("leaders end wars by peace".into());
            }
            let pos = w
                .belligerents
                .iter()
                .position(|b| b.country == actor)
                .ok_or("not fighting")?;
            w.belligerents.remove(pos);
            Ok(Some(DiplomaticEvent::LeftWar { war, country: actor }))
        }

        Order::OfferPeace { war } => {
            let turn = state.turn;
            let w = state.wars.find_mut(war).ok_or("no such war")?;
            if !w.is_leader(actor) {
                return Err("only leaders offer peace".into());
            }
            w.peace_offers.retain(|o| o.0 != actor);
            w.peace_offers.push((actor, turn));
            Ok(Some(DiplomaticEvent::PeaceOffered { war, by: actor }))
        }

        Order::SetMobilization(level) => {
            let before = state.country(actor).forces.mobilization;
            if before == level {
                return Ok(None);
            }
            state.country_mut(actor).forces.mobilization = level;
            let steps = level.level() as f64 - before.level() as f64;
            if steps > 0.0 {
                let ids: Vec<CountryId> = state.ids().filter(|&c| c != actor).collect();
                for c in ids {
                    if crate::domestic::hostility(state, c, actor) >= 0.2 {
                        state.tension.add(actor, c, MOBILIZE_TENSION * steps);
                    }
                }
            }
            Ok(Some(DiplomaticEvent::Mobilized { country: actor, level }))
        }

        _ => Err("not a war order".into()),
    }
}

/// Remove `loss` strength from `c`'s pools (pro rata over `pools`), wearing
/// transferred-arms provenance proportionally.
fn apply_losses(state: &mut WorldState, c: CountryId, loss: f64, pools: &[PoolKind]) -> f64 {
    let country = state.country_mut(c);
    let total_before = country.forces.strength();
    let in_pools: f64 = pools.iter().map(|&k| country.forces.pool(k).strength).sum();
    if in_pools <= 1e-9 || loss <= 0.0 {
        return 0.0;
    }
    let loss = loss.min(in_pools);
    for &k in pools {
        let p = country.forces.pool_mut(k);
        p.strength -= loss * p.strength / in_pools;
    }
    let kept = 1.0 - loss / total_before.max(1e-9);
    for o in &mut country.arms_origin {
        o.1 *= kept;
    }
    loss
}

const ALL_POOLS: [PoolKind; 3] = PoolKind::ALL;

/// Destroy `loss` strength across all of `c`'s pools (used by nuclear
/// strikes); provenance of transferred arms wears proportionally.
pub fn destroy(state: &mut WorldState, c: CountryId, loss: f64) -> f64 {
    apply_losses(state, c, loss, &ALL_POOLS)
}
const STRIKE_POOLS: [PoolKind; 2] = [PoolKind::Naval, PoolKind::Air];

/// Combat phase (DESIGN §4.3 step 4): covert exposure, combat on every
/// front, then wars that end.
pub fn resolve(state: &mut WorldState, events: &mut Vec<DiplomaticEvent>) {
    expose_covert(state, events);
    let ids: Vec<WarId> = state.wars.active.iter().map(|w| w.id).collect();
    for id in ids {
        fight(state, id, events);
    }
    end_wars(state, events);
    for i in 0..state.countries.len() {
        let c = CountryId(i as u16);
        if !state.wars.is_belligerent(c) {
            let country = state.country_mut(c);
            country.war_weariness *= 1.0 - WEARINESS_DECAY;
        }
    }
}

fn fight(state: &mut WorldState, id: WarId, events: &mut Vec<DiplomaticEvent>) {
    let Some(war) = state.wars.find(id).cloned() else {
        return;
    };
    let punitive = war.aim == WarAim::Punitive;
    // Each belligerent's committed share, normalised across its wars.
    let share = |state: &WorldState, b: &Belligerent| b.allocation / state.wars.total_allocation(b.country).max(1.0);
    let value = |state: &WorldState, b: &Belligerent, strike: bool| {
        let f = &state.country(b.country).forces;
        let v = match (strike, b.side) {
            (true, Side::Attacker) => f.naval.value() + f.air.value(),
            // Air defence plus ground-based air defence.
            (true, Side::Defender) => f.air.value() + 0.25 * f.land.value(),
            (false, _) => f.land.value() + f.naval.value() + f.air.value(),
        };
        v * f.readiness_factor() * share(state, b)
    };
    let attack: f64 =
        war.side(Side::Attacker).map(|b| value(state, b, punitive)).sum::<f64>() * war.ground_mult(Side::Attacker);
    let defence: f64 =
        war.side(Side::Defender).map(|b| value(state, b, punitive)).sum::<f64>() * war.ground_mult(Side::Defender);
    let variance = 1.0 + state.roll_symmetric(0.25);
    let ratio = (attack * variance / defence.max(1e-6)).clamp(0.05, 20.0);
    let intensity = war.aim.intensity();

    let step = if punitive {
        0.0
    } else {
        (PROGRESS_RATE * intensity * ratio.ln()).clamp(-MAX_PROGRESS_STEP, MAX_PROGRESS_STEP)
    };
    let att_frac = if punitive {
        PUNITIVE_STRIKER_LOSS / ratio.sqrt()
    } else {
        BASE_LOSS * intensity / ratio.sqrt()
    }
    .clamp(0.002, 0.15);
    let def_frac = (BASE_LOSS * intensity * ratio.sqrt()).clamp(0.002, 0.15);

    let (mut att_losses, mut def_losses) = (0.0, 0.0);
    let mut losses: Vec<(CountryId, f64)> = Vec::new();
    // How much each side's enemies actually threaten it: a war against a
    // much weaker enemy is a war of choice.
    let side_power =
        |state: &WorldState, side: Side| -> f64 { war.side(side).map(|b| state.country(b.country).power()).sum() };
    let (att_total, def_total) = (side_power(state, Side::Attacker), side_power(state, Side::Defender));
    for b in &war.belligerents {
        let s = share(state, b);
        let pools: &[PoolKind] = if punitive && b.side == Side::Attacker {
            &STRIKE_POOLS
        } else {
            &ALL_POOLS
        };
        let committed: f64 = pools
            .iter()
            .map(|&k| state.country(b.country).forces.pool(k).strength)
            .sum::<f64>()
            * s;
        let frac = match b.side {
            Side::Attacker => att_frac,
            Side::Defender => def_frac,
        };
        let strength_before = state.country(b.country).forces.strength().max(1e-9);
        let lost = apply_losses(state, b.country, frac * committed, pools);
        match b.side {
            Side::Attacker => att_losses += lost,
            Side::Defender => def_losses += lost,
        }
        losses.push((b.country, lost));
        let (own_side, enemy_side) = match b.side {
            Side::Attacker => (att_total, def_total),
            Side::Defender => (def_total, att_total),
        };
        let threat_from_enemy = (enemy_side / (enemy_side + own_side).max(1e-9)).clamp(0.0, 1.0);
        let country = state.country_mut(b.country);
        // The public rallies to the defence of home soil, not to a war
        // fought on someone else's (by the leader's share of it).
        let home_rally = if b.country == war.leader(b.side) {
            1.0 - 0.5 * war.home_share(b.side)
        } else {
            1.0
        };
        let choice = 1.0 + country.government.war_of_choice() * (1.0 - threat_from_enemy);
        country.war_weariness += (100.0 * lost / strength_before * country.government.weariness_sensitivity()
            + WEARINESS_PER_TURN)
            * home_rally
            * choice;
    }
    let w = state.wars.find_mut(id).expect("war exists");
    for (c, l) in losses {
        if let Some(b) = w.belligerents.iter_mut().find(|b| b.country == c) {
            b.losses += l;
        }
    }
    let mut reports = Vec::new();
    for f in &mut w.fronts {
        f.progress = (f.progress + step).clamp(-100.0, 100.0);
        reports.push(DiplomaticEvent::FrontReport {
            war: id,
            front: f.id,
            progress: f.progress,
            ratio,
            attacker_losses: att_losses,
            defender_losses: def_losses,
        });
    }
    events.extend(reports);
}

fn end_wars(state: &mut WorldState, events: &mut Vec<DiplomaticEvent>) {
    let turn = state.turn;
    let wars = state.wars.active.clone();
    for w in wars {
        let p = w.progress();
        let exhausted = |c: CountryId| state.country(c).war_weariness >= EXHAUSTION;
        let outcome = if p >= 100.0 {
            Some((PeaceResult::AttackerWon, PeaceReason::Decisive))
        } else if p <= -100.0 {
            Some((PeaceResult::DefenderWon, PeaceReason::Decisive))
        } else if w.aim == WarAim::Punitive && w.offered(w.defender, turn) {
            Some((PeaceResult::AttackerWon, PeaceReason::StrikeAccepted))
        } else if w.aim != WarAim::Punitive && w.offered(w.attacker, turn) && w.offered(w.defender, turn) {
            Some((negotiated(p), PeaceReason::Negotiated))
        } else if exhausted(w.attacker) {
            let r = if p > WIN_MARGIN {
                PeaceResult::WhitePeace
            } else {
                PeaceResult::DefenderWon
            };
            Some((r, PeaceReason::Exhaustion))
        } else if exhausted(w.defender) {
            let r = if p < -WIN_MARGIN {
                PeaceResult::WhitePeace
            } else {
                PeaceResult::AttackerWon
            };
            Some((r, PeaceReason::Exhaustion))
        } else {
            None
        };
        match outcome {
            Some((result, reason)) => make_peace(state, &w, result, reason, events),
            None if w.aim == WarAim::Punitive && turn >= w.started + PUNITIVE_TURNS => {
                // The target refused to accept the strike: a limited war.
                let war = state.wars.find_mut(w.id).expect("war");
                war.aim = WarAim::Limited;
                war.peace_offers.clear();
                if let Some(b) = war.belligerents.iter_mut().find(|b| b.country == w.attacker) {
                    b.allocation = b.allocation.max(WarAim::Limited.allocation());
                }
                events.push(DiplomaticEvent::WarEscalated {
                    war: w.id,
                    aim: WarAim::Limited,
                });
            }
            None => {}
        }
    }
}

fn negotiated(p: f64) -> PeaceResult {
    if p > WIN_MARGIN {
        PeaceResult::AttackerWon
    } else if p < -WIN_MARGIN {
        PeaceResult::DefenderWon
    } else {
        PeaceResult::WhitePeace
    }
}

fn make_peace(
    state: &mut WorldState,
    w: &War,
    result: PeaceResult,
    reason: PeaceReason,
    events: &mut Vec<DiplomaticEvent>,
) {
    state.wars.active.retain(|x| x.id != w.id);
    state.wars.ended += 1;
    let (winner, loser) = match result {
        PeaceResult::AttackerWon => (Some(w.attacker), Some(w.defender)),
        PeaceResult::DefenderWon => (Some(w.defender), Some(w.attacker)),
        PeaceResult::WhitePeace => (None, None),
    };
    // A claim pressed and won is settled (the province or the debt is
    // taken); pressed and lost, it is half given up.
    if result == PeaceResult::AttackerWon && w.aim != WarAim::Punitive {
        state.claims.retain(|c| !(c.by == w.attacker && c.against == w.defender));
    } else if result == PeaceResult::DefenderWon {
        for c in state.claims.iter_mut().filter(|c| c.by == w.attacker && c.against == w.defender) {
            c.weight *= 0.5;
        }
    }
    if let (Some(win), Some(lose)) = (winner, loser) {
        let regime_change = w.aim == WarAim::Major && win == w.attacker && reason != PeaceReason::Exhaustion;
        let alignment = state.country(win).alignment.clone();
        {
            let c = state.country_mut(win);
            c.legitimacy = (c.legitimacy + 5.0).min(100.0);
        }
        let punitive = w.aim == WarAim::Punitive;
        let turn = state.turn;
        let c = state.country_mut(lose);
        if regime_change {
            // The defeated government is replaced by one aligned with the
            // victor. For the player this is regime collapse (game over).
            c.alignment = alignment;
            c.legitimacy = 30.0;
            c.stability = (c.stability - 20.0).max(0.0);
            c.transition.took_power = Some(turn as i32);
        } else if !punitive {
            c.legitimacy = (c.legitimacy - 10.0).max(0.0);
        }
        if result == PeaceResult::AttackerWon && w.aim == WarAim::Limited {
            // Reparations: half a turn of the loser's revenue.
            let amount = 0.5 * c.gdp * c.tax_rate;
            state.diplomacy.pending_aid.push((lose, win, amount));
        }
        state.opinions.add(lose, win, modifier("defeated us", -20.0, 0.5));
    }
    events.push(DiplomaticEvent::PeaceMade {
        war: w.id,
        attacker: w.attacker,
        defender: w.defender,
        aim: w.aim,
        result,
        reason,
        progress: w.progress(),
    });
}

fn expose_covert(state: &mut WorldState, events: &mut Vec<DiplomaticEvent>) {
    let covert: Vec<usize> = state
        .diplomacy
        .streams
        .iter()
        .enumerate()
        .filter(|(_, s)| s.covert)
        .map(|(i, _)| i)
        .collect();
    for i in covert {
        if state.roll_unit() >= EXPOSURE_CHANCE {
            continue;
        }
        let s = state.diplomacy.streams[i];
        state.diplomacy.streams[i].covert = false;
        let c = state.country_mut(s.from);
        c.legitimacy = (c.legitimacy - 5.0 * c.openness).max(0.0);
        events.push(DiplomaticEvent::CovertExposed { from: s.from, to: s.to });
    }
}

/// Can `observer` see a covert act by `actor`?
pub fn sees_covert(state: &WorldState, observer: CountryId, actor: CountryId) -> bool {
    intel::coverage(state.country(observer), state.country(actor)) >= state.params.covert_visibility
}

/// Ledger effects of war (DESIGN §21.2, §21.4): Coercion entries, attacks on
/// defended states, implied norms, involvement commitments and their exits,
/// the both-sides rule, and grading of allies' responses. Runs after
/// [`ledger::process`].
pub fn process_ledger(state: &mut WorldState, events: &[DiplomaticEvent], log: &mut Vec<LedgerLog>) {
    for event in events {
        match *event {
            DiplomaticEvent::WarDeclared {
                war,
                attacker,
                defender,
                aim,
                broke_pact,
                ..
            } => {
                let (code, cause) = if aim == WarAim::Punitive {
                    (CauseCode::PunitiveStrike, "punitive strike")
                } else {
                    (CauseCode::WarDeclared, "declared war")
                };
                ledger::record(
                    state,
                    attacker,
                    defender,
                    EntryKind::Coercion,
                    None,
                    0.0,
                    if aim == WarAim::Major { 1.5 } else { 1.0 },
                    false,
                    Visibility::Public,
                    code,
                    cause,
                    log,
                );
                if broke_pact {
                    ledger::record(
                        state,
                        attacker,
                        defender,
                        EntryKind::Back,
                        Some(Grade::Abandoned),
                        0.0,
                        1.5,
                        false,
                        Visibility::Public,
                        CauseCode::BrokePact,
                        "attacked a non-aggression partner",
                        log,
                    );
                }
                ledger::report_attack(state, defender, log);
                ledger::report_violation(state, attacker, NormTag::Aggression, log);
                for ally in crate::domestic::defenders(state, defender) {
                    if ally == attacker {
                        continue;
                    }
                    ledger::open_test(
                        state,
                        ally,
                        defender,
                        EntryKind::Back,
                        ALLY_RESPONSE_TURNS,
                        CauseCode::AllyAttacked,
                        format!("{} attacked", state.country(defender).name),
                        log,
                    );
                }
                let _ = war;
            }
            DiplomaticEvent::JoinedWar {
                war,
                country,
                side,
                band,
            } => {
                let Some(w) = state.wars.find(war) else { continue };
                let (beneficiary, enemy, attacker) = (w.leader(side), w.leader(side.other()), w.attacker);
                state
                    .wars
                    .involvements
                    .retain(|i| !(i.actor == country && i.war == war));
                state.wars.involvements.push(Involvement {
                    actor: country,
                    beneficiary,
                    war,
                    band,
                    visibility: Visibility::Public,
                    since: state.turn,
                });
                if side == Side::Defender && enemy == attacker {
                    ledger::create_norm(state, country, NormTag::Aggression, enemy, "repelled aggression", log);
                }
            }
            DiplomaticEvent::LeftWar { war, country } => {
                exit_involvement(state, country, war, log);
            }
            DiplomaticEvent::StreamStopped { stream } if stream.kind == StreamKind::Arms => {
                let wars: Vec<WarId> = state
                    .wars
                    .involvements
                    .iter()
                    .filter(|i| i.actor == stream.from && i.beneficiary == stream.to && i.band == 2)
                    .map(|i| i.war)
                    .collect();
                for war in wars {
                    exit_involvement(state, stream.from, war, log);
                }
            }
            DiplomaticEvent::PeaceMade { war, .. } => {
                let kept: Vec<Involvement> = state
                    .wars
                    .involvements
                    .iter()
                    .filter(|i| i.war == war)
                    .copied()
                    .collect();
                state.wars.involvements.retain(|i| i.war != war);
                for i in kept {
                    ledger::record(
                        state,
                        i.actor,
                        i.beneficiary,
                        EntryKind::Back,
                        Some(Grade::Honoured),
                        0.25 * i.band as f64,
                        band_stake(i.band),
                        false,
                        i.visibility,
                        CauseCode::WarInvolvementKept,
                        format!("stood by its side to the end (band {})", i.band),
                        log,
                    );
                }
                state.wars.both_sides.retain(|b| b.1 != war);
            }
            DiplomaticEvent::CovertExposed { from, to } => {
                let ids: Vec<ledger::EntryId> = state
                    .ledger
                    .entries
                    .iter()
                    .filter(|e| e.actor == from && e.counterpart == to && e.visibility == Visibility::Covert)
                    .map(|e| e.id)
                    .collect();
                for id in ids {
                    ledger::expose(state, id);
                }
                for i in state.wars.involvements.iter_mut() {
                    if i.actor == from && i.beneficiary == to {
                        i.visibility = Visibility::Public;
                    }
                }
                let enemies = state.wars.enemies_of(to);
                for e in enemies {
                    state
                        .opinions
                        .add(e, from, modifier("secretly armed our enemy", -25.0, 0.5));
                    ledger::record(
                        state,
                        from,
                        e,
                        EntryKind::Coercion,
                        None,
                        0.0,
                        1.0,
                        false,
                        Visibility::Exposed,
                        CauseCode::CovertExposed,
                        "covert arms exposed",
                        log,
                    );
                }
            }
            _ => {}
        }
    }
    sync_proxy_involvements(state);
    both_sides_rule(state, log);
    grade_responses(state);
}

/// Arms streams to a belligerent are band-2 involvement for its side.
fn sync_proxy_involvements(state: &mut WorldState) {
    let streams: Vec<_> = state
        .diplomacy
        .streams
        .iter()
        .filter(|s| s.kind == StreamKind::Arms)
        .copied()
        .collect();
    for s in streams {
        let wars: Vec<WarId> = state.wars.wars_of(s.to).map(|w| w.id).collect();
        for war in wars {
            let exists = state
                .wars
                .involvements
                .iter()
                .any(|i| i.actor == s.from && i.war == war);
            if !exists {
                let visibility = if s.covert {
                    Visibility::Covert
                } else {
                    Visibility::Public
                };
                let turn = state.turn;
                state.wars.involvements.push(Involvement {
                    actor: s.from,
                    beneficiary: s.to,
                    war,
                    band: 2,
                    visibility,
                    since: turn,
                });
            }
        }
    }
}

/// Withdrawal from a war (DESIGN §14.9): Abandoned if the side was losing,
/// Partial otherwise; weighted by the band's stake, with the involvement's
/// visibility, so covert exits are seen by few.
fn exit_involvement(state: &mut WorldState, actor: CountryId, war: WarId, log: &mut Vec<LedgerLog>) {
    let Some(pos) = state
        .wars
        .involvements
        .iter()
        .position(|i| i.actor == actor && i.war == war)
    else {
        return;
    };
    let i = state.wars.involvements.remove(pos);
    let losing = state
        .wars
        .find(war)
        .and_then(|w| w.side_of(i.beneficiary).map(|s| w.progress_for(s) < 0.0))
        .unwrap_or(false);
    let grade = if losing { Grade::Abandoned } else { Grade::Partial };
    ledger::record(
        state,
        actor,
        i.beneficiary,
        EntryKind::Back,
        Some(grade),
        0.0,
        band_stake(i.band),
        false,
        i.visibility,
        CauseCode::WarWithdrawal,
        format!("withdrew from the war (band {})", i.band),
        log,
    );
}

/// Arming both sides of a war writes Partial Back entries toward both
/// (DESIGN §21.2), once per supplier and war.
fn both_sides_rule(state: &mut WorldState, log: &mut Vec<LedgerLog>) {
    let wars = state.wars.active.clone();
    for w in &wars {
        let suppliers: Vec<CountryId> = state.ids().collect();
        for s in suppliers {
            if state.wars.both_sides.contains(&(s, w.id)) {
                continue;
            }
            let arms_to = |side: Side| {
                state
                    .diplomacy
                    .streams
                    .iter()
                    .any(|st| st.kind == StreamKind::Arms && st.from == s && w.side_of(st.to) == Some(side))
            };
            if arms_to(Side::Attacker) && arms_to(Side::Defender) {
                state.wars.both_sides.push((s, w.id));
                for side in [Side::Attacker, Side::Defender] {
                    ledger::record(
                        state,
                        s,
                        w.leader(side),
                        EntryKind::Back,
                        Some(Grade::Partial),
                        0.0,
                        1.0,
                        false,
                        Visibility::Public,
                        CauseCode::ArmedBothSides,
                        "armed both sides",
                        log,
                    );
                }
            }
        }
    }
}

/// Grade responses to open tests from what each actor is doing in the war:
/// fighting with forces at least half the attacker's commitment honours a
/// Back test; token forces, arms or sanctions are Partial (fake honour is
/// caught). Norm tests are answered by fighting the violator.
fn grade_responses(state: &mut WorldState) {
    let tests: Vec<(CountryId, CountryId, EntryKind)> = state
        .ledger
        .tests
        .iter()
        .map(|t| (t.actor, t.counterpart, t.kind))
        .collect();
    for (actor, counterpart, kind) in tests {
        let grade = match kind {
            EntryKind::Back => state
                .wars
                .wars_of(counterpart)
                .filter_map(|w| back_response(state, w, actor, counterpart))
                .max(),
            EntryKind::Norm(_) => {
                if state.wars.at_war(actor, counterpart) {
                    Some(Grade::Honoured)
                } else if state.wars.enemies_of(counterpart).iter().any(|&e| {
                    state
                        .diplomacy
                        .streams
                        .iter()
                        .any(|s| s.kind == StreamKind::Arms && s.from == actor && s.to == e)
                }) {
                    Some(Grade::Partial)
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(g) = grade {
            ledger::respond_to_tests(state, actor, counterpart, g);
        }
    }
}

fn back_response(state: &WorldState, w: &War, actor: CountryId, beneficiary: CountryId) -> Option<Grade> {
    let side = w.side_of(beneficiary)?;
    let committed = |c: CountryId, alloc: f64| state.country(c).power() * alloc;
    if let Some(b) = w.belligerent(actor).filter(|b| b.side == side) {
        let enemy: f64 = w.side(side.other()).map(|e| committed(e.country, e.allocation)).sum();
        return Some(if committed(actor, b.allocation) >= 0.5 * enemy {
            Grade::Honoured
        } else {
            Grade::Partial
        });
    }
    let arms = state
        .diplomacy
        .streams
        .iter()
        .any(|s| s.kind == StreamKind::Arms && s.from == actor && s.to == beneficiary);
    let sanctions = w
        .side(side.other())
        .any(|e| state.diplomacy.is_sanctioned_by(e.country, actor));
    (arms || sanctions).then_some(Grade::Partial)
}
