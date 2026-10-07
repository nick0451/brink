//! D79: the ProtectClients goal weighs the worst-placed client we can reach,
//! not the sum of every client's worst quarrel. The sum saturated the goal
//! for any state with several allies (FRG: 108 against DevelopEconomy 32), so
//! it held a goal slot every turn and crowded out trade (review 17).

use std::path::PathBuf;

use ai::goals::{self, Goal};
use sim_core::{observe, ObserverView};

fn view(code: &str) -> ObserverView {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let def = scenario::load(root.join("data/scenarios/1980.ron")).unwrap();
    let w = scenario::build(&def, Some(1)).unwrap();
    observe(&w, w.find(code).unwrap())
}

fn protect(view: &ObserverView) -> f64 {
    goals::assess(view)
        .into_iter()
        .find(|(g, _)| *g == Goal::ProtectClients)
        .map_or(0.0, |(_, s)| s.total())
}

/// Data swap: strip West Germany's commitments down to its single most
/// endangered client. Under the old sum the goal fell with every client
/// removed; now it is the same, because only the worst-placed one counts.
#[test]
fn an_alliance_of_moderate_quarrels_does_not_saturate_protect_clients() {
    let full = view("FRG");
    let clients = goals::clients(&full);
    assert!(clients.len() >= 3, "fixture: FRG has several clients: {clients:?}");
    let reach = |c| full.others.iter().find(|f| f.id == c).map_or(1.0, |f| ai::inputs::reach_to(&full, f));
    let worst = *clients
        .iter()
        .max_by(|&&a, &&b| (goals::danger(&full, a) * reach(a)).total_cmp(&(goals::danger(&full, b) * reach(b))))
        .unwrap();

    let mut one = full.clone();
    let me = one.observer;
    one.treaties.retain(|t| !t.involves(me) || t.involves(worst));
    one.streams.retain(|s| s.from != me || s.to == worst);
    assert_eq!(goals::clients(&one), vec![worst]);

    let (all, single) = (protect(&full), protect(&one));
    println!("FRG ProtectClients: {} clients {all:.1}, worst client alone {single:.1}", clients.len());
    assert!((all - single).abs() < 1e-9, "more allies with quarrels must not add up: {all:.1} vs {single:.1}");
}
