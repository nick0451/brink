//! Issue 23: balancing aid against a *regional* threat.
//!
//! The only balancing term (§14.9) read global power share (> 0.4), which a
//! regional threat never reaches, and the AI had no way to give money at
//! all: support for a belligerent was band 2, our own army's arms, valued
//! only by how far it moved the friend's odds of winning (Saudi Arabia on
//! Iraq's side: +0.4 against -16 for the arms). So no state ever financed
//! the enemy of its enemy (review 22: Iraq got no Gulf money in 100 runs).
//! Now a state whose main threat is at war pledges money to the threat's
//! enemy, in proportion to how much of the threat the war ties down.

use std::path::PathBuf;

use ai::{Controller, Strategist};
use sim_core::{observe, resolve_turn, CountryId, Order, OrderSet, WarAim, WorldState};

fn world() -> WorldState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let def = scenario::load(root.join("data/scenarios/1980.ron")).unwrap();
    scenario::build(&def, Some(3)).unwrap()
}

/// Iraq declares war on Iran at the start; every AI then plays 16 turns.
/// Returns the money the funder pledged to each country while the war ran.
fn pledges(w: &mut WorldState, funder: &str) -> Vec<(CountryId, f64)> {
    let (irq, irn) = (w.find("IRQ").unwrap(), w.find("IRN").unwrap());
    let me = w.find(funder).unwrap();
    let mut ais: Vec<Strategist> = w.ids().map(|_| Strategist::with_seed(3)).collect();
    let mut out = Vec::new();
    for turn in 0..16 {
        let views: Vec<_> = w.ids().map(|c| observe(w, c)).collect();
        let mut orders: Vec<OrderSet> = views
            .iter()
            .zip(ais.iter_mut())
            .map(|(v, ai)| OrderSet {
                country: v.observer,
                orders: ai.decide(v).orders,
            })
            .collect();
        if turn == 0 {
            orders[irq.index()].orders.push(Order::DeclareWar {
                target: irn,
                aim: WarAim::Major,
            });
        }
        let at_war = !w.wars.active.is_empty();
        for o in orders[me.index()].orders.iter() {
            if let (Order::Aid { to, amount }, true) = (o, at_war) {
                out.push((*to, *amount));
            }
        }
        resolve_turn(w, orders);
    }
    out
}

fn aid_to(pledged: &[(CountryId, f64)], to: CountryId) -> f64 {
    pledged.iter().filter(|p| p.0 == to).map(|p| p.1).sum()
}

/// Saudi Arabia's main threat is revolutionary Iran (its own estimate:
/// reach × power share × hostility, plus subversion). With Iran tied down
/// by Iraq, it pays Iraq. Fails without the fix (no aid order exists in the
/// AI at all).
#[test]
fn a_state_whose_main_threat_is_at_war_funds_the_threats_enemy() {
    let mut w = world();
    let irq = w.find("IRQ").unwrap();
    let orders = pledges(&mut w, "SAU");
    let view = observe(&w, w.find("SAU").unwrap());
    let top = ai::inputs::top_threat(&view).map(|(f, t)| (f.code.clone(), t));
    let amount = aid_to(&orders, irq);
    println!("SAU top threat {top:?}; aid pledged to IRQ {amount:.3}");
    assert_eq!(
        top.as_ref().map(|t| t.0.as_str()),
        Some("IRN"),
        "fixture: Iran is Saudi Arabia's main threat"
    );
    assert!(amount > 0.0, "the enemy of our main threat gets money: {orders:?}");
}

/// Data swap: move Saudi Arabia's quarrel with Iran (tension and both
/// opinions) onto a distant country at peace. The same war is no longer a
/// war against Saudi Arabia's threat, and it pays nobody: hostility to a
/// belligerent is not enough, the war must tie down *our* main threat.
#[test]
fn it_does_not_fund_a_war_against_a_state_that_is_not_its_threat() {
    let mut w = world();
    let (sau, irn, irq, arg) = (
        w.find("SAU").unwrap(),
        w.find("IRN").unwrap(),
        w.find("IRQ").unwrap(),
        w.find("ARG").unwrap(),
    );
    let (t_irn, t_arg) = (w.tension.get(sau, irn), w.tension.get(sau, arg));
    w.tension.set(sau, irn, t_arg);
    w.tension.set(sau, arg, t_irn);
    for (a, b, c, d) in [(sau, irn, sau, arg), (irn, sau, arg, sau)] {
        let x = std::mem::take(w.opinions.modifiers_mut(a, b));
        let y = std::mem::replace(w.opinions.modifiers_mut(c, d), x);
        *w.opinions.modifiers_mut(a, b) = y;
    }
    let orders = pledges(&mut w, "SAU");
    let view = observe(&w, sau);
    let top = ai::inputs::top_threat(&view).map(|(f, t)| (f.code.clone(), t));
    let amount = aid_to(&orders, irq) + aid_to(&orders, irn);
    println!("swapped SAU top threat {top:?}; aid to the belligerents {amount:.3}");
    assert_ne!(
        top.as_ref().map(|t| t.0.as_str()),
        Some("IRN"),
        "fixture: Iran is no longer the main threat"
    );
    assert_eq!(
        amount, 0.0,
        "no money for a war that does not tie down our threat: {orders:?}"
    );
}
