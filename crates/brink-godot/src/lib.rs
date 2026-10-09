//! BRINK ↔ Godot bridge (D54). A thin GDExtension class over the
//! engine-free [`session::Session`]: Godot calls `load`, optionally
//! `set_player`, then `step` once per turn, and draws what comes back.
//! Presentation only reads; it never changes simulation rules.
//!
//! **Fog (principle 7, D105 #5).** Once a player is set, every
//! player-facing answer comes from the player's own `ObserverView` through
//! [`player`]: foreign values are estimates `[value, low, high]`, hidden
//! values are `null`, and narration drops facts the player could not know.
//! Without a player the bridge is an omniscient spectator, as before.
//!
//! The dictionary shapes below are the contract with the client
//! (`v2a_out/API.md` repeats them). Missing values are `null`.

// gdext's #[class(init)] expansion trips this lint on `base: Base<Node>`.
#![allow(clippy::redundant_field_names)]

pub mod player;
pub mod session;

use std::path::PathBuf;

use godot::prelude::*;
use sim_core::{CountryId, Order};

use player::OrderSpec;
use session::{Portrait, Session};

struct BrinkExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BrinkExtension {}

#[derive(GodotClass)]
#[class(base = Node, init)]
pub struct BrinkSim {
    session: Option<Session>,
    base: Base<Node>,
}

fn code(s: &Session, c: Option<CountryId>) -> GString {
    match c {
        Some(c) => GString::from(s.world.country(c).code.as_str()),
        None => GString::new(),
    }
}

fn arr<T: ToGodot>(xs: impl IntoIterator<Item = T>) -> VarArray {
    let mut a = VarArray::new();
    for x in xs {
        a.push(&x.to_variant());
    }
    a
}

fn opt<T: ToGodot>(x: Option<T>) -> Variant {
    x.map_or_else(Variant::nil, |v| v.to_variant())
}

fn strs<'a>(xs: impl IntoIterator<Item = &'a String>) -> VarArray {
    arr(xs.into_iter().map(|s| GString::from(s.as_str())))
}

fn get_str(d: &VarDictionary, k: &str) -> Option<String> {
    let v = d.get(k)?;
    if v.is_nil() {
        return None;
    }
    Some(v.try_to::<GString>().map_or_else(|_| v.to_string(), |s| s.to_string()))
}

fn get_num(d: &VarDictionary, k: &str) -> Option<f64> {
    let v = d.get(k)?;
    v.try_to::<f64>()
        .ok()
        .or_else(|| v.try_to::<i64>().ok().map(|i| i as f64))
}

fn get_bool(d: &VarDictionary, k: &str) -> Option<bool> {
    d.get(k)?.try_to::<bool>().ok()
}

/// Godot dictionary → [`OrderSpec`] (same keys; see `queue_order`).
fn order_spec(d: &VarDictionary) -> OrderSpec {
    let shares = match (
        get_num(d, "military"),
        get_num(d, "development"),
        get_num(d, "welfare"),
        get_num(d, "intelligence"),
    ) {
        (Some(m), Some(dv), Some(w), Some(i)) => Some([m, dv, w, i]),
        _ => None,
    };
    OrderSpec {
        kind: get_str(d, "kind").unwrap_or_default(),
        target: get_str(d, "target"),
        aim: get_str(d, "aim"),
        amount: get_num(d, "amount"),
        flag: get_bool(d, "flag"),
        id: get_num(d, "id").map(|x| x as u32),
        level: get_str(d, "level"),
        treaty: get_str(d, "treaty"),
        side: get_str(d, "side"),
        band: get_num(d, "band").map(|x| x as u8),
        shares,
    }
}

/// [`OrderSpec`] → Godot dictionary (only the keys the order uses).
fn spec_dict(s: &OrderSpec) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("kind", s.kind.as_str());
    if let Some(x) = &s.target {
        d.set("target", x.as_str());
    }
    if let Some(x) = &s.aim {
        d.set("aim", x.as_str());
    }
    if let Some(x) = s.amount {
        d.set("amount", x);
    }
    if let Some(x) = s.flag {
        d.set("flag", x);
    }
    if let Some(x) = s.id {
        d.set("id", x as i64);
    }
    if let Some(x) = &s.level {
        d.set("level", x.as_str());
    }
    if let Some(x) = &s.treaty {
        d.set("treaty", x.as_str());
    }
    if let Some(x) = &s.side {
        d.set("side", x.as_str());
    }
    if let Some(x) = s.band {
        d.set("band", x as i64);
    }
    if let Some([m, dv, w, i]) = s.shares {
        d.set("military", m);
        d.set("development", dv);
        d.set("welfare", w);
        d.set("intelligence", i);
    }
    d
}

fn order_dict(s: &Session, o: &Order) -> VarDictionary {
    let view = s.player_view().expect("player set");
    let mut d = VarDictionary::new();
    d.set("text", player::describe_order(o, &view).as_str());
    d.set("spec", &opt(player::spec_of(o, &view).as_ref().map(spec_dict)));
    d
}

#[godot_api]
impl BrinkSim {
    /// Load a scenario file and voice lines (absolute or relative paths).
    #[func]
    fn load(&mut self, scenario: GString, lines: GString, seed: i64) -> bool {
        let lines_path = PathBuf::from(lines.to_string());
        let lines = (!lines.is_empty()).then_some(lines_path.as_path());
        match Session::load(&PathBuf::from(scenario.to_string()), lines, seed as u64) {
            Ok(s) => {
                self.session = Some(s);
                true
            }
            Err(e) => {
                godot_error!("BRINK: {e}");
                false
            }
        }
    }

    /// Resolve one turn. Returns
    /// `{turn, events: [{kind, a, b, value, gravity}], narration: [{fact,
    /// gravity, heading, speaker, text, speaker_code, portrait}], rejected:
    /// [{text, reason, spec}], game_over: bool, game_over_reason: String}`.
    /// With a player: the player's orders are the queue (cleared after), or
    /// the cabinet's when delegating; events and narration are filtered to
    /// what the player could know; `rejected` lists only the player's
    /// refused orders with the simulation's own reason; `game_over` is true
    /// once the player's regime collapsed or fell to a coup
    /// (`game_over_reason` "Collapse" | "Coup" | "Extinct"). After game over,
    /// `step` resolves nothing and keeps returning `game_over: true`.
    #[func]
    fn step(&mut self) -> VarDictionary {
        let mut out = VarDictionary::new();
        let Some(s) = self.session.as_mut() else { return out };
        let step = s.step();
        let s = self.session.as_ref().expect("session");
        let mut events = VarArray::new();
        for e in &step.events {
            let mut d = VarDictionary::new();
            d.set("kind", e.kind);
            d.set("a", &code(s, e.a));
            d.set("b", &code(s, e.b));
            d.set("value", e.value);
            d.set("gravity", e.gravity as i64);
            events.push(&d.to_variant());
        }
        let mut narration = VarArray::new();
        for n in &step.narration {
            let mut d = VarDictionary::new();
            d.set("fact", n.fact.as_str());
            d.set("gravity", n.gravity as i64);
            let (heading, speaker, text) = n.line.clone().unwrap_or_default();
            d.set("heading", heading.as_str());
            d.set("speaker", speaker.as_str());
            d.set("text", text.as_str());
            d.set("speaker_code", &code(s, n.speaker));
            let portrait = match (n.portrait, n.speaker) {
                (Portrait::Face, Some(c)) => s.regime_key(c),
                (Portrait::Seal, _) => "seal".to_string(),
                _ => String::new(),
            };
            d.set("portrait", portrait.as_str());
            narration.push(&d.to_variant());
        }
        let mut rejected = VarArray::new();
        for r in &step.rejected {
            let mut d = VarDictionary::new();
            d.set("text", r.text.as_str());
            d.set("reason", r.reason.as_str());
            let spec = s.player_view().and_then(|v| player::spec_of(&r.order, &v));
            d.set("spec", &opt(spec.as_ref().map(spec_dict)));
            rejected.push(&d.to_variant());
        }
        out.set("turn", step.turn as i64);
        out.set("events", &events);
        out.set("narration", &narration);
        out.set("rejected", &rejected);
        out.set("game_over", step.game_over.is_some());
        out.set("game_over_reason", step.game_over.unwrap_or_default().as_str());
        out
    }

    /// Every active country. Spectator (no player): `{code, name, area,
    /// government, alignment, gdp, stability, power, arsenal, at_war,
    /// regime}` from the true state. Player mode, from the player's view:
    /// the same keys plus `own: bool`, `stability_band` (String or null) and
    /// `power_band: [low, high]`; for foreign countries `stability` is null,
    /// `power` is the player's estimate and `arsenal` the visible arsenal.
    #[func]
    fn countries(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.session.as_ref() else { return out };
        if let Some(view) = s.player_view() {
            for r in player::map_rows(&view) {
                let mut d = VarDictionary::new();
                d.set("code", r.code.as_str());
                d.set("name", r.name.as_str());
                d.set("area", r.area.unwrap_or_default().as_str());
                d.set("government", r.government.as_str());
                d.set("alignment", r.alignment.unwrap_or_default().as_str());
                d.set("gdp", r.gdp);
                d.set("stability", &opt(r.stability));
                d.set("stability_band", &opt(r.stability_band.map(GString::from)));
                d.set("power", r.power[0]);
                d.set("power_band", &arr([r.power[1], r.power[2]]));
                d.set("arsenal", r.arsenal as i64);
                d.set("at_war", r.at_war);
                d.set("regime", s.regime_key(r.id).as_str());
                d.set("own", r.own);
                out.push(&d.to_variant());
            }
            return out;
        }
        for c in s.world.countries.iter().filter(|c| c.active) {
            let mut d = VarDictionary::new();
            d.set("code", c.code.as_str());
            d.set("name", c.name.as_str());
            d.set("area", c.area.clone().unwrap_or_default().as_str());
            d.set("government", format!("{:?}", c.government).as_str());
            d.set("alignment", c.alignment.clone().unwrap_or_default().as_str());
            d.set("gdp", c.gdp);
            d.set("stability", c.stability);
            d.set("power", c.power());
            d.set("arsenal", c.arsenal as i64);
            d.set("at_war", s.world.wars.is_belligerent(c.id));
            d.set("regime", s.regime_key(c.id).as_str());
            out.push(&d.to_variant());
        }
        out
    }

    /// Active wars (public): `{id, attacker, defender, aim, progress}`
    /// (progress attacker-positive; `id` is what `join_war`, `offer_peace`
    /// and `leave_war` take).
    #[func]
    fn wars(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.session.as_ref() else { return out };
        for w in &s.world.wars.active {
            let mut d = VarDictionary::new();
            d.set("id", w.id.0 as i64);
            d.set("attacker", &code(s, Some(w.attacker)));
            d.set("defender", &code(s, Some(w.defender)));
            d.set("aim", format!("{:?}", w.aim).as_str());
            d.set("progress", w.progress());
            out.push(&d.to_variant());
        }
        out
    }

    /// Bilateral tension between two country codes (0–100; public). In
    /// player mode it is read from the player's view.
    #[func]
    fn tension(&self, a: GString, b: GString) -> f64 {
        let Some(s) = self.session.as_ref() else { return 0.0 };
        if let Some(view) = s.player_view() {
            return player::tension(&view, &a.to_string(), &b.to_string());
        }
        match (s.world.find(&a.to_string()), s.world.find(&b.to_string())) {
            (Some(x), Some(y)) => s.world.tension.get(x, y),
            _ => 0.0,
        }
    }

    #[func]
    fn global_tension(&self) -> f64 {
        self.session.as_ref().map_or(0.0, |s| s.world.global_tension)
    }

    #[func]
    fn year(&self) -> f64 {
        self.session.as_ref().map_or(0.0, |s| s.world.year())
    }

    /// World interest rate, annualised % (P7).
    #[func]
    fn interest_rate(&self) -> f64 {
        self.session.as_ref().map_or(0.0, |s| 400.0 * s.world.money.rate)
    }

    #[func]
    fn energy_price(&self) -> f64 {
        self.session.as_ref().map_or(1.0, |s| s.world.energy.price)
    }

    // ---------------------------------------------------------- player --

    /// Countries offered at the start screen: every active `Playable`
    /// country as `{code, name, selectable: bool}`. Only `selectable` ones
    /// (scenario data, D105 #7: USA and SOV first) can be chosen.
    #[func]
    fn playable(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.session.as_ref() else { return out };
        for (c, ok) in s.playable() {
            let mut d = VarDictionary::new();
            let x = s.world.country(c);
            d.set("code", x.code.as_str());
            d.set("name", x.name.as_str());
            d.set("selectable", ok);
            out.push(&d.to_variant());
        }
        out
    }

    /// Take control of a country. Returns `{ok: bool, reason: String}`.
    #[func]
    fn set_player(&mut self, code: GString) -> VarDictionary {
        let mut out = VarDictionary::new();
        let Some(s) = self.session.as_mut() else {
            out.set("ok", false);
            out.set("reason", "no scenario loaded");
            return out;
        };
        match s.set_player(&code.to_string()) {
            Ok(_) => {
                out.set("ok", true);
                out.set("reason", "");
            }
            Err(e) => {
                out.set("ok", false);
                out.set("reason", e.as_str());
            }
        }
        out
    }

    /// The player's own country, exact. Empty without a player. Keys:
    /// `code, name, government, turn, year, gdp, growth, debt_ratio, deficit,
    /// reserves, stability, prosperity, security, legitimacy, war_weariness,
    /// initiative {allowance, banked, available, left}, budget and
    /// budget_target {military, development, welfare, intelligence},
    /// mobilization, energy_policy, energy_capacity, energy_net_exports,
    /// military, arsenal, at_war, crisis_pending, crisis_since (int or
    /// null), reserve_holder, delegate, game_over, treaties [{id, kind,
    /// with, ours}], sanctioning [codes], sanctioned_by [codes]`.
    #[func]
    fn player_state(&self) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(s) = self.session.as_ref() else { return d };
        let Some(view) = s.player_view() else { return d };
        let p = player::player_state(&view);
        let shares = |b: [f64; 4]| {
            let mut x = VarDictionary::new();
            x.set("military", b[0]);
            x.set("development", b[1]);
            x.set("welfare", b[2]);
            x.set("intelligence", b[3]);
            x
        };
        d.set("code", p.code.as_str());
        d.set("name", p.name.as_str());
        d.set("government", p.government.as_str());
        d.set("turn", p.turn as i64);
        d.set("year", p.year);
        d.set("gdp", p.gdp);
        d.set("growth", p.growth);
        d.set("debt_ratio", p.debt_ratio);
        d.set("deficit", p.deficit);
        d.set("reserves", p.reserves);
        d.set("stability", p.stability);
        d.set("prosperity", p.prosperity);
        d.set("security", p.security);
        d.set("legitimacy", p.legitimacy);
        d.set("war_weariness", p.war_weariness);
        let mut ini = VarDictionary::new();
        ini.set("allowance", p.initiative_allowance as i64);
        ini.set("banked", p.initiative_banked as i64);
        ini.set("available", p.initiative_available as i64);
        ini.set("left", s.initiative_left() as i64);
        d.set("initiative", &ini);
        d.set("budget", &shares(p.budget));
        d.set("budget_target", &shares(p.budget_target));
        d.set("mobilization", p.mobilization.as_str());
        d.set("energy_policy", p.energy_policy.as_str());
        d.set("energy_capacity", p.energy_capacity);
        d.set("energy_net_exports", p.energy_net_exports);
        d.set("military", p.military);
        d.set("arsenal", p.arsenal as i64);
        d.set("at_war", p.at_war);
        d.set("crisis_pending", p.crisis_pending);
        d.set("crisis_since", &opt(p.crisis_since.map(|t| t as i64)));
        d.set("reserve_holder", s.player_holds_reserve());
        d.set("delegate", s.delegate());
        d.set("game_over", s.game_over().is_some());
        let mut treaties = VarArray::new();
        for t in &p.treaties {
            let mut x = VarDictionary::new();
            x.set("id", t.id as i64);
            x.set("kind", t.kind);
            x.set("with", t.with.as_str());
            x.set("ours", t.ours);
            treaties.push(&x.to_variant());
        }
        d.set("treaties", &treaties);
        d.set("sanctioning", &strs(&p.sanctioning));
        d.set("sanctioned_by", &strs(&p.sanctioned_by));
        d
    }

    /// What the player knows about another country (empty if unknown or no
    /// player). Estimates are `[value, low, high]`; values the player's
    /// coverage cannot see are null. Keys: `code, name, government,
    /// alignment, area, coverage, gdp, military, land, naval, air,
    /// debt_ratio ([v,l,h] or null), stability_band (String or null), budget
    /// ({military, development, welfare, intelligence} or null), arsenal,
    /// programme (float or null), energy_policy, energy_capacity,
    /// energy_net_exports, arms_industry, military_tech,
    /// their_opinion_of_us, our_opinion_of_them, tension, credibility_back,
    /// credibility_threat, credibility_norm, trust, at_war, at_war_with_us,
    /// treaties_with_us [String], we_sanction, sanctions_us`.
    #[func]
    fn foreign(&self, code: GString) -> VarDictionary {
        let mut d = VarDictionary::new();
        let Some(s) = self.session.as_ref() else { return d };
        let Some(view) = s.player_view() else { return d };
        let Some(f) = player::foreign_info(&view, &code.to_string()) else {
            return d;
        };
        d.set("code", f.code.as_str());
        d.set("name", f.name.as_str());
        d.set("government", f.government.as_str());
        d.set("alignment", &opt(f.alignment.as_deref().map(GString::from)));
        d.set("area", &opt(f.area.as_deref().map(GString::from)));
        d.set("coverage", f.coverage);
        d.set("gdp", f.gdp);
        d.set("military", &arr(f.military));
        d.set("land", &arr(f.land));
        d.set("naval", &arr(f.naval));
        d.set("air", &arr(f.air));
        d.set("debt_ratio", &opt(f.debt_ratio.map(arr)));
        d.set("stability_band", &opt(f.stability_band.map(GString::from)));
        let budget = f.budget.map(|b| {
            let mut x = VarDictionary::new();
            x.set("military", b[0]);
            x.set("development", b[1]);
            x.set("welfare", b[2]);
            x.set("intelligence", b[3]);
            x
        });
        d.set("budget", &opt(budget));
        d.set("arsenal", f.arsenal as i64);
        d.set("programme", &opt(f.programme));
        d.set("energy_policy", f.energy_policy.as_str());
        d.set("energy_capacity", f.energy_capacity);
        d.set("energy_net_exports", f.energy_net_exports);
        d.set("arms_industry", f.arms_industry);
        d.set("military_tech", f.military_tech as i64);
        d.set("their_opinion_of_us", f.their_opinion_of_us);
        d.set("our_opinion_of_them", f.our_opinion_of_them);
        d.set("tension", f.tension);
        d.set("credibility_back", f.credibility_back);
        d.set("credibility_threat", f.credibility_threat);
        d.set("credibility_norm", f.credibility_norm);
        d.set("trust", f.trust);
        d.set("at_war", f.at_war);
        d.set("at_war_with_us", f.at_war_with_us);
        d.set(
            "treaties_with_us",
            &arr(f.treaties_with_us.into_iter().map(GString::from)),
        );
        d.set("we_sanction", f.we_sanction);
        d.set("sanctions_us", f.sanctions_us);
        d
    }

    /// Proposals the player may answer this turn (they lapse otherwise):
    /// `[{id, from, from_name, kind, turn, accept_cost, text}]`. Answer with
    /// `queue_order({kind: "respond", id, flag: accept})`.
    #[func]
    fn proposals(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(view) = self.session.as_ref().and_then(Session::player_view) else {
            return out;
        };
        for p in player::proposals(&view) {
            let mut d = VarDictionary::new();
            d.set("id", p.id as i64);
            d.set("from", p.from.as_str());
            d.set("from_name", p.from_name.as_str());
            d.set("kind", p.kind);
            d.set("turn", p.turn as i64);
            d.set("accept_cost", p.accept_cost as i64);
            d.set("text", p.text.as_str());
            out.push(&d.to_variant());
        }
        out
    }

    /// Queue an order for the player. `spec` keys (only those the kind
    /// uses): `kind` (one of budget, deficit, mobilization, energy_policy,
    /// monetary_stance, respond, reform, crackdown, propose_treaty,
    /// guarantee, sanction, lift_sanction, denounce, aid, cancel_treaty,
    /// declare_war, join_war, offer_peace, leave_war), `target` (country
    /// code), `aim` (Punitive|Limited|Major), `amount` (float: deficit ratio
    /// or aid), `flag` (bool: accept), `id` (proposal, treaty or war id),
    /// `level` (Peacetime|Partial|Full|Total, Restrain|Normal|Flood,
    /// Tight|Neutral|Loose), `treaty` (Trade|NonAggression|DefensiveAlliance),
    /// `side` (Attacker|Defender), `band` (3|4), and for budget `military,
    /// development, welfare, intelligence`. Returns `{ok, cost, reason,
    /// left}`: refused orders carry the simulation's own reason (a dry run
    /// of the queue), `left` is the Initiative still free.
    #[func]
    fn queue_order(&mut self, spec: VarDictionary) -> VarDictionary {
        let mut out = VarDictionary::new();
        let result = match self.session.as_mut() {
            None => Err("no scenario loaded".to_string()),
            Some(s) => match s.player_view() {
                None => Err("no player country".to_string()),
                Some(view) => player::parse_order(&order_spec(&spec), &view).and_then(|o| s.queue(o)),
            },
        };
        let left = self.session.as_ref().map_or(0, Session::initiative_left);
        match result {
            Ok(cost) => {
                out.set("ok", true);
                out.set("cost", cost as i64);
                out.set("reason", "");
            }
            Err(e) => {
                out.set("ok", false);
                out.set("cost", 0i64);
                out.set("reason", e.as_str());
            }
        }
        out.set("left", left as i64);
        out
    }

    /// Remove the queued order at `index`; true if one was removed.
    #[func]
    fn unqueue(&mut self, index: i64) -> bool {
        self.session
            .as_mut()
            .is_some_and(|s| index >= 0 && s.unqueue(index as usize).is_some())
    }

    /// The queue: `[{text, spec}]` in order.
    #[func]
    fn pending(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.session.as_ref() else { return out };
        if s.player().is_none() {
            return out;
        }
        for o in s.pending() {
            out.push(&order_dict(s, o).to_variant());
        }
        out
    }

    /// Let the cabinet play the player's turns (the queue is then ignored).
    #[func]
    fn set_delegate(&mut self, on: bool) {
        if let Some(s) = self.session.as_mut() {
            s.set_delegate(on);
        }
    }

    /// The cabinet's recommended orders for this turn (the player's own
    /// government AI on the player's own view): `[{text, spec (dict or null
    /// when outside the V-2a order set), cost, subject (String or null),
    /// score (float or null), reasons: [{label, term, value}]}]`, reasons
    /// largest first. Showing it is a client-side toggle (D105 #2).
    #[func]
    fn advice(&mut self) -> VarArray {
        let mut out = VarArray::new();
        let Some(s) = self.session.as_mut() else { return out };
        for a in s.advice() {
            let mut d = VarDictionary::new();
            d.set("text", a.text.as_str());
            d.set("spec", &opt(a.spec.as_ref().map(spec_dict)));
            d.set("cost", a.cost as i64);
            d.set("subject", &opt(a.subject.as_deref().map(GString::from)));
            d.set("score", &opt(a.score));
            let mut reasons = VarArray::new();
            for r in &a.reasons {
                let mut x = VarDictionary::new();
                x.set("label", r.label.as_str());
                x.set("term", r.term.as_str());
                x.set("value", r.value);
                reasons.push(&x.to_variant());
            }
            d.set("reasons", &reasons);
            out.push(&d.to_variant());
        }
        out
    }
}
