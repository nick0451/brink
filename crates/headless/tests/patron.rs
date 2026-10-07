//! Crisis pass 2, issue 1: a bloc regime's security rests on its patron.
//! When the patron reforms and leaves the camp, the client's security (and
//! so its stability and legitimacy) must fall; a weaker ally counts whatever
//! its politics. Before this, a reformed patron kept every
//! client's security exactly where it was, so the 1989 chain couldn't start.

use std::path::PathBuf;

use sim_core::diplomacy::{sign, TreatyKind};
use sim_core::domestic::{backs, security_breakdown};
use sim_core::{CountryId, Government, Order, WorldState};

fn world() -> WorldState {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    scenario::build(&scenario::load(root.join("data/scenarios/1980.ron")).unwrap(), Some(1)).unwrap()
}

fn id(w: &WorldState, code: &str) -> CountryId {
    w.find(code).unwrap()
}

fn force_reform(w: &mut WorldState, c: CountryId) {
    let turn = w.turn;
    w.country_mut(c).transition.pending_since = Some(turn);
    sim_core::transition::apply(w, c, Order::Reform).unwrap();
}

#[test]
fn a_patron_that_reforms_out_of_the_bloc_stops_propping_up_its_clients() {
    let mut w = world();
    let (sov, ddr) = (id(&w, "SOV"), id(&w, "DDR"));
    // Give the client a hostile neighbour so its patron matters.
    let usa = id(&w, "USA");
    w.tension.add(usa, ddr, 60.0);
    let before = security_breakdown(&w, ddr);
    assert!(before.allies.iter().any(|a| a.0 == sov && a.1 > 0.0), "{before:?}");

    force_reform(&mut w, sov);
    assert_eq!(w.country(sov).government, Government::Democracy);
    assert!(w.country(sov).alignment.is_none(), "the reform vector drops the bloc");
    let after = security_breakdown(&w, ddr);
    assert!(!after.allies.iter().any(|a| a.0 == sov), "{after:?}");
    assert!(
        after.value < before.value - 10.0,
        "client security {:.1} -> {:.1}",
        before.value,
        after.value
    );
}

#[test]
fn backing_needs_a_shared_camp_or_a_shared_kind_of_government() {
    let east = Some("east".to_string());
    let west = Some("west".to_string());
    use Government::*;
    // Same camp, any government (Western-aligned dictatorships keep the US).
    assert!(backs(west.as_ref(), Authoritarian, west.as_ref(), Democracy));
    // Unaligned like-minded patron (a dictatorship's unaligned ally).
    assert!(backs(east.as_ref(), Authoritarian, None, Authoritarian));
    // A reformed patron: out of the camp and a different government.
    assert!(!backs(east.as_ref(), Authoritarian, None, Democracy));
    // Unaligned states count every defender.
    assert!(backs(None, Authoritarian, None, Democracy));
}

/// Review regression: a bloc patron that allies with a weaker, unaligned
/// democracy keeps counting that ally in its security; only a stronger
/// defender (a patron) needs to back the regime.
#[test]
fn a_weaker_ally_is_not_a_patron() {
    let mut w = world();
    let (sov, ind) = (id(&w, "SOV"), id(&w, "IND"));
    sign(&mut w, TreatyKind::DefensiveAlliance, sov, ind);
    let sec = security_breakdown(&w, sov);
    assert!(sec.allies.iter().any(|a| a.0 == ind), "{sec:?}");
}

/// A client keeps a second patron that still backs it: North Korea keeps
/// China in its security when the USSR reforms out of the bloc.
#[test]
fn a_remaining_patron_still_counts() {
    let mut w = world();
    let (sov, prk, chn) = (id(&w, "SOV"), id(&w, "PRK"), id(&w, "CHN"));
    // Mid-campaign North Korea often also allies with the USSR.
    sign(&mut w, TreatyKind::DefensiveAlliance, sov, prk);
    force_reform(&mut w, sov);
    let sec = security_breakdown(&w, prk);
    assert!(sec.allies.iter().any(|a| a.0 == chn), "{sec:?}");
    assert!(!sec.allies.iter().any(|a| a.0 == sov), "{sec:?}");
}
