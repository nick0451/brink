//! Engine-free session core for the Godot front-end: owns a world, runs the
//! AI and the narrator turn by turn exactly as the headless runner does, and
//! flattens each turn into plain views the presentation layer can draw.
//! Kept free of Godot types so it is unit-testable (determinism vs headless).

use std::path::Path;

use ai::{Controller, Decision, DecisionRecord, ReasonLine, Strategist};
use sim_core::{
    diplomacy, observe, resolve_turn, CountryId, DiplomaticEvent, Government, ObserverView, Order, OrderSet, Tier,
    WorldState,
};

use crate::player;
use voice::narrator::{Narrator, TurnInput};

/// One map-worthy event (DESIGN §1 resolution playback).
#[derive(Clone, Debug, PartialEq)]
pub struct EventView {
    pub kind: &'static str,
    pub a: Option<CountryId>,
    pub b: Option<CountryId>,
    pub value: f64,
    /// Gravity for presentation (behaviour-and-voice §2.2): 5 = nuclear.
    pub gravity: u8,
}

/// One narration: always a plain fact, sometimes a flavoured line.
#[derive(Clone, Debug, PartialEq)]
pub struct NarrationView {
    pub fact: String,
    pub gravity: u8,
    /// (heading, speaker, text) when a line was chosen.
    pub line: Option<(String, String, String)>,
    /// The country speaking, when the speaker is a country.
    pub speaker: Option<CountryId>,
    /// Portrait policy (D56): a face only on G1–G2; an office seal on G3;
    /// nothing on G4–G5.
    pub portrait: Portrait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Portrait {
    Face,
    Seal,
    None,
}

impl Portrait {
    pub fn for_gravity(g: u8) -> Self {
        match g {
            0..=2 => Portrait::Face,
            3 => Portrait::Seal,
            _ => Portrait::None,
        }
    }
}

/// One of the player's orders the simulation refused, with its own reason.
#[derive(Clone, Debug, PartialEq)]
pub struct Rejection {
    pub order: Order,
    /// The order in the player's terms.
    pub text: String,
    /// The simulation's reason, verbatim.
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StepOutput {
    pub turn: u32,
    pub events: Vec<EventView>,
    pub narration: Vec<NarrationView>,
    /// The player's orders the simulation refused this turn (empty without
    /// a player).
    pub rejected: Vec<Rejection>,
    /// Why the game ended ("Collapse", "Coup", "Extinct"), once it has.
    pub game_over: Option<String>,
}

/// One recommended order from the player's cabinet (the player's own
/// Strategist run on the player's own view; D105 #2).
#[derive(Clone, Debug, PartialEq)]
pub struct AdviceItem {
    pub order: Order,
    /// `None` when the order is outside the V-2a subset (text only).
    pub spec: Option<player::OrderSpec>,
    pub text: String,
    pub cost: u8,
    /// What was decided, and the reason lines (largest first).
    pub subject: Option<String>,
    pub score: Option<f64>,
    pub reasons: Vec<ReasonLine>,
}

pub struct Session {
    pub world: WorldState,
    ais: Vec<Strategist>,
    narrator: Option<Narrator>,
    /// Codes a human may choose (scenario data, D105 #7); empty = all
    /// active `Playable` countries.
    selectable: Vec<String>,
    player: Option<CountryId>,
    pending: Vec<Order>,
    delegate: bool,
    /// The decision a Strategist already took this turn: `(turn, country,
    /// decision)`. Each Strategist decides exactly once per turn, whether
    /// for advice or for play, so its counters stay in step with headless.
    decided: Option<(u32, CountryId, Decision)>,
    game_over: Option<String>,
}

impl Session {
    /// Load a scenario (and optionally voice lines) with a seed.
    pub fn load(scenario: &Path, lines: Option<&Path>, seed: u64) -> Result<Self, String> {
        let def = scenario::load(scenario)?;
        let world = scenario::build(&def, Some(seed))?;
        let lines = match lines {
            Some(p) => Some(voice::lines::load_lines(p)?),
            None => None,
        };
        let mut s = Session::from_world(world, lines, seed);
        for code in &def.selectable {
            let ok = s
                .world
                .find(code)
                .is_some_and(|c| s.world.country(c).tier == Tier::Playable);
            if !ok {
                return Err(format!("selectable country {code} is not a Playable country"));
            }
        }
        s.selectable = def.selectable.clone();
        Ok(s)
    }

    /// A session over an already-built world (fixtures). Every active
    /// `Playable` country is selectable.
    pub fn from_world(world: WorldState, lines: Option<Vec<voice::lines::Line>>, seed: u64) -> Self {
        let ais = world.ids().map(|_| Strategist::with_seed(seed)).collect();
        let narrator = lines.map(|l| Narrator::new(l, voice::Intensity::default(), seed));
        Session {
            world,
            ais,
            narrator,
            selectable: Vec::new(),
            player: None,
            pending: Vec::new(),
            delegate: false,
            decided: None,
            game_over: None,
        }
    }

    // ------------------------------------------------------------ player --

    /// May a human choose `c`? Active `Playable` countries named by the
    /// scenario's `selectable` list (all of them if the list is empty).
    pub fn selectable(&self, c: CountryId) -> bool {
        let x = self.world.country(c);
        x.active && x.tier == Tier::Playable && (self.selectable.is_empty() || self.selectable.contains(&x.code))
    }

    /// Every active `Playable` country with whether it may be chosen.
    pub fn playable(&self) -> Vec<(CountryId, bool)> {
        self.world
            .countries
            .iter()
            .filter(|c| c.active && c.tier == Tier::Playable)
            .map(|c| (c.id, self.selectable(c.id)))
            .collect()
    }

    /// Take control of a country (a new seat: the queue is cleared).
    /// Refused once the game is over: a fallen regime stays fallen.
    pub fn set_player(&mut self, code: &str) -> Result<CountryId, String> {
        if self.game_over.is_some() {
            return Err("the game is over".into());
        }
        let c = self.world.find(code).ok_or_else(|| format!("unknown country {code}"))?;
        if !self.selectable(c) {
            return Err(format!("{code} cannot be played"));
        }
        self.player = Some(c);
        self.pending.clear();
        Ok(c)
    }

    pub fn player(&self) -> Option<CountryId> {
        self.player
    }

    /// Let the cabinet (the player's own Strategist) play the player's
    /// orders at resolution instead of the queue.
    pub fn set_delegate(&mut self, on: bool) {
        self.delegate = on;
    }

    pub fn delegate(&self) -> bool {
        self.delegate
    }

    pub fn game_over(&self) -> Option<&str> {
        self.game_over.as_deref()
    }

    /// The player's view: the only source of player-facing data.
    pub fn player_view(&self) -> Option<ObserverView> {
        self.player.map(|p| observe(&self.world, p))
    }

    /// Is the player the reserve-currency holder (may set the monetary
    /// stance)? A public fact.
    pub fn player_holds_reserve(&self) -> bool {
        self.player.is_some() && sim_core::money::holder(&self.world) == self.player
    }

    /// Run `c`'s Strategist for this turn unless it already ran; returns
    /// the decision. Only the player's decision is cached (advice is asked
    /// for before the turn resolves); caching others would evict it.
    fn decide(&mut self, c: CountryId, view: &ObserverView) -> Decision {
        let turn = self.world.turn;
        if let Some((t, who, d)) = &self.decided {
            if *t == turn && *who == c {
                return d.clone();
            }
        }
        let d = self.ais[c.index()].decide(view);
        if Some(c) == self.player {
            self.decided = Some((turn, c, d.clone()));
        }
        d
    }

    /// Start the player's turn: the cabinet decides once (cached as advice).
    pub fn begin_turn(&mut self) {
        if let Some(p) = self.player {
            let view = observe(&self.world, p);
            self.decide(p, &view);
        }
    }

    /// The cabinet's recommended orders for this turn, with their reasons.
    pub fn advice(&mut self) -> Vec<AdviceItem> {
        let Some(p) = self.player else { return Vec::new() };
        let view = observe(&self.world, p);
        let d = self.decide(p, &view);
        let mut used = vec![false; d.records.len()];
        d.orders
            .iter()
            .map(|o| {
                let target = player::order_counterpart(o, &view);
                let linked = d
                    .records
                    .iter()
                    .enumerate()
                    .filter(|(i, r)| r.chosen && !used[*i] && player::explains(o, r.kind))
                    .min_by_key(|(_, r)| u8::from(target.is_some() && r.counterpart != target))
                    .map(|(i, r)| (i, r.clone()));
                if let Some((i, _)) = linked {
                    used[i] = true;
                }
                let record = linked.map(|(_, r)| r);
                AdviceItem {
                    order: o.clone(),
                    spec: player::spec_of(o, &view),
                    text: player::describe_order(o, &view),
                    cost: diplomacy::initiative_cost(&self.world, p, o),
                    subject: record.as_ref().map(|r| r.subject.clone()),
                    score: record.as_ref().map(|r| r.score),
                    reasons: record.map_or_else(Vec::new, |r| {
                        let mut v = r.lines;
                        v.sort_by(|a, b| b.value.abs().total_cmp(&a.value.abs()));
                        v
                    }),
                }
            })
            .collect()
    }

    /// Queue an order for the player. Rehearses the whole queue on a copy
    /// of the world (the player's orders only, in order, with the same
    /// checks as resolution) and refuses the order with the simulation's own
    /// reason if it would fail. Returns its Initiative cost. Other
    /// countries' simultaneous orders can still change the outcome.
    pub fn queue(&mut self, order: Order) -> Result<u8, String> {
        if self.player.is_none() {
            return Err("no player country".into());
        }
        if self.game_over.is_some() {
            return Err("the game is over".into());
        }
        let p = self.player.expect("checked");
        let mut all = self.pending.clone();
        all.push(order.clone());
        let cost = rehearse(&self.world, p, &all).pop().expect("one result per order")?;
        self.pending.push(order);
        Ok(cost)
    }

    /// Queue without rehearsal (replaying a saved order log): the
    /// simulation judges the order at resolution.
    pub fn queue_unchecked(&mut self, order: Order) {
        self.pending.push(order);
    }

    pub fn unqueue(&mut self, index: usize) -> Option<Order> {
        (index < self.pending.len()).then(|| self.pending.remove(index))
    }

    pub fn pending(&self) -> &[Order] {
        &self.pending
    }

    /// Initiative still free after the queued orders (0 without a player).
    pub fn initiative_left(&self) -> u8 {
        let Some(p) = self.player else { return 0 };
        let spent: u8 = rehearse(&self.world, p, &self.pending)
            .into_iter()
            .filter_map(Result::ok)
            .sum();
        self.world.country(p).initiative.available().saturating_sub(spent)
    }

    // -------------------------------------------------------------- turn --

    /// Resolve one turn: simultaneous planning from the same state (the
    /// same loop as `headless::run_campaign_with`), then narration. With a
    /// player, the player's orders are the queue (or the cabinet's advice
    /// when delegating), and the player's decisions are never narrated.
    /// After game over, nothing resolves.
    pub fn step(&mut self) -> StepOutput {
        if let Some(reason) = &self.game_over {
            return StepOutput {
                turn: self.world.turn,
                game_over: Some(reason.clone()),
                ..StepOutput::default()
            };
        }
        let views: Vec<_> = self.world.ids().map(|c| observe(&self.world, c)).collect();
        let mut orders = Vec::with_capacity(views.len());
        let mut decisions = Vec::new();
        for view in &views {
            let c = view.observer;
            let d = self.decide(c, view);
            if Some(c) == self.player {
                let mine = if self.delegate {
                    d.orders
                } else {
                    std::mem::take(&mut self.pending)
                };
                orders.push(OrderSet {
                    country: c,
                    orders: mine,
                });
                continue;
            }
            decisions.extend(d.records.iter().cloned().map(|r| (c, r)));
            orders.push(OrderSet {
                country: c,
                orders: d.orders,
            });
        }
        self.pending.clear();
        let pre = self.player.map(|p| views[p.index()].clone());
        self.resolve(orders, &decisions, pre)
    }

    /// The resolution half of [`Session::step`]: resolve the given orders,
    /// narrate, and filter everything through the player's fog. Public for
    /// fixtures that script other countries' orders; `pre` is the player's
    /// view taken before resolution.
    pub fn resolve(
        &mut self,
        orders: Vec<OrderSet>,
        decisions: &[(CountryId, DecisionRecord)],
        pre: Option<ObserverView>,
    ) -> StepOutput {
        if let Some(n) = self.narrator.as_mut() {
            n.observe_before(&self.world);
        }
        let report = resolve_turn(&mut self.world, orders);
        let fog = match (self.player, pre) {
            (Some(p), Some(pre)) => Some((pre, observe(&self.world, p))),
            _ => None,
        };
        let shown = match &fog {
            Some((pre, post)) => player::visible_report(pre, post, &report),
            None => report.clone(),
        };
        let world = &self.world;
        let narration = match self.narrator.as_mut() {
            Some(n) => n
                .narrate(TurnInput {
                    state: world,
                    report: &shown,
                    decisions,
                })
                .into_iter()
                .filter(|x| {
                    fog.as_ref()
                        .is_none_or(|(pre, post)| player::narration_visible(pre, post, x.trigger, x.actor, x.target))
                })
                .map(|x| {
                    let speaker = x
                        .line
                        .as_ref()
                        .and_then(|l| world.countries.iter().find(|c| c.name == l.1).map(|c| c.id));
                    NarrationView {
                        fact: x.fact,
                        gravity: x.gravity,
                        line: x.line.map(|(h, s, t, _)| (h, s, t)),
                        speaker,
                        portrait: Portrait::for_gravity(x.gravity),
                    }
                })
                .collect(),
            None => Vec::new(),
        };
        let mut out = StepOutput {
            turn: report.turn,
            events: shown.events.iter().filter_map(event_view).collect(),
            narration,
            ..StepOutput::default()
        };
        if let (Some(p), Some((pre, post))) = (self.player, &fog) {
            out.rejected = shown
                .rejected
                .iter()
                .map(|(_, order, reason)| Rejection {
                    order: order.clone(),
                    text: player::describe_order(order, pre),
                    reason: reason.clone(),
                })
                .collect();
            self.game_over = player::game_over(p, &report.events, post.own.active);
            out.game_over = self.game_over.clone();
        }
        out
    }

    /// Portrait set key for a country (D56: by regime state, never date).
    pub fn regime_key(&self, c: CountryId) -> String {
        let x = self.world.country(c);
        let state = match (x.government, x.transition.reformed_turn) {
            (Government::Democracy, Some(_)) => "reformed",
            (Government::Revolutionary, _) => "revolutionary",
            _ => "base",
        };
        format!("{}_{}", x.code.to_lowercase(), state)
    }
}

/// Map-worthy events only; routine diplomacy stays in the ledger panel.
pub fn event_view(e: &DiplomaticEvent) -> Option<EventView> {
    use DiplomaticEvent as E;
    let v = |kind, a, b, value, gravity| {
        Some(EventView {
            kind,
            a: Some(a),
            b,
            value,
            gravity,
        })
    };
    match *e {
        E::WarDeclared { attacker, defender, .. } => v("war", attacker, Some(defender), 0.0, 3),
        E::FrontReport { war: _, progress, .. } => {
            // Fronts are drawn from the war list; the report only animates.
            let _ = progress;
            None
        }
        E::PeaceMade { attacker, defender, .. } => v("peace", attacker, Some(defender), 0.0, 3),
        E::JoinedWar { country, .. } => v("joined_war", country, None, 0.0, 3),
        E::NuclearStrike { by, target, .. } => v("nuclear", by, Some(target), 0.0, 5),
        E::SanctionImposed { by, target } => v("sanction", by, Some(target), 0.0, 2),
        E::TreatySigned { treaty } => v("treaty", treaty.a, Some(treaty.b), 0.0, 1),
        E::TransitionCrisis { country } => v("crisis", country, None, 0.0, 2),
        E::Transition { country, .. } => v("transition", country, None, 0.0, 2),
        E::Secession { parent, successor } => v("secession", parent, Some(successor), 0.0, 2),
        E::ProgrammeExposed { country, by } => v("programme", country, Some(by), 0.0, 2),
        E::Mobilized { country, level } => v("mobilized", country, None, level.level() as f64, 2),
        E::EnergyPolicy { country, .. } => v("energy", country, None, 0.0, 1),
        _ => None,
    }
}

/// Rehearse one country's orders on a copy of the world, in order, with the
/// same checks as the orders phase of `resolve_turn`. Returns each order's
/// Initiative cost or the simulation's reason for refusing it.
pub fn rehearse(world: &WorldState, actor: CountryId, orders: &[Order]) -> Vec<Result<u8, String>> {
    let mut w = world.clone();
    orders
        .iter()
        .map(|order| match order {
            Order::SetBudget(_) | Order::SetDeficit(_) | Order::SetEnergyPolicy(_) => Ok(0),
            Order::SetMonetaryStance(_) => {
                if sim_core::money::holder(&w) == Some(actor) {
                    Ok(0)
                } else {
                    Err("only the reserve-currency holder sets the stance".into())
                }
            }
            action => {
                let cost = diplomacy::initiative_cost(&w, actor, action);
                if w.country(actor).initiative.available() < cost {
                    return Err("not enough Initiative".into());
                }
                diplomacy::apply(&mut w, actor, action.clone())?;
                w.country_mut(actor).initiative.spend(cost);
                Ok(cost)
            }
        })
        .collect()
}
