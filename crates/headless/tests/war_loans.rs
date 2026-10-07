//! Issue 24 (D95): war loans and creditors.
//!
//! One-off aid paid to a state at war is a loan: it buys the same army, but
//! the recipient's debt rises by it and the funder holds a public claim.
//! Once the debtor is at peace and its debt drags its growth, the creditor
//! decides each assessment: forgive (the debt falls; the debtor is
//! grateful) or hold (a public hostile act that renews the debtor's
//! historical grudge).

use std::path::PathBuf;

use ai::{Controller, Strategist};
use sim_core::opinion::HostileAct;
use sim_core::{observe, resolve_turn, CountryId, Order, OrderSet, WarAim, WorldState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn four_actor() -> WorldState {
    let def = scenario::load(root().join("data/fixtures/four_actor.ron")).unwrap();
    scenario::build(&def, Some(1)).unwrap()
}

/// One turn; `orders` go to their countries, everyone else idles.
fn turn(w: &mut WorldState, orders: &[(CountryId, Order)]) -> sim_core::TurnReport {
    let sets = w
        .ids()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|id| OrderSet {
            country: id,
            orders: orders
                .iter()
                .filter(|(c, _)| *c == id)
                .map(|(_, o)| o.clone())
                .collect(),
        })
        .collect();
    resolve_turn(w, sets)
}

/// Aid pledged to a belligerent raises its debt by the amount paid and
/// records the funder's claim; the same aid in peacetime is a gift. Fails
/// without the fix (aid never touched the recipient's debt).
#[test]
fn a_pledge_to_a_belligerent_is_a_loan() {
    let mut w = four_actor();
    let (maj, aly, riv) = (w.find("MAJ").unwrap(), w.find("ALY").unwrap(), w.find("RIV").unwrap());
    turn(
        &mut w,
        &[(
            riv,
            Order::DeclareWar {
                target: maj,
                aim: WarAim::Major,
            },
        )],
    );
    assert!(w.wars.is_belligerent(maj), "fixture: MAJ is at war");
    let amount = 0.5;
    let (mut with, mut without) = (w.clone(), w.clone());
    turn(&mut with, &[(aly, Order::Aid { to: maj, amount })]);
    turn(&mut without, &[]);
    let paid = with.country(maj).aid_in;
    assert!(paid > 0.0, "the aid was paid");
    let extra_debt = with.country(maj).debt - without.country(maj).debt;
    println!(
        "paid {paid:.3}; MAJ debt {:.3} vs {:.3} without",
        with.country(maj).debt,
        without.country(maj).debt
    );
    assert!(
        (extra_debt - paid).abs() < 1e-9,
        "debt rose by the loan: {extra_debt} vs {paid}"
    );
    let loan = with
        .diplomacy
        .loans
        .iter()
        .find(|l| l.creditor == aly && l.debtor == maj);
    assert!(
        loan.is_some_and(|l| (l.amount - paid).abs() < 1e-9),
        "claim recorded: {:?}",
        with.diplomacy.loans
    );
    // The claim is public: every observer sees it.
    assert!(observe(&with, riv)
        .loans
        .iter()
        .any(|l| l.creditor == aly && l.debtor == maj));

    // Peacetime aid stays a gift.
    let mut p = four_actor();
    let (pm, pa) = (p.find("MAJ").unwrap(), p.find("ALY").unwrap());
    let mut q = p.clone();
    turn(&mut p, &[(pa, Order::Aid { to: pm, amount })]);
    turn(&mut q, &[]);
    assert!(p.diplomacy.loans.is_empty());
    assert!((p.country(pm).debt - q.country(pm).debt).abs() < 1e-9);
}

/// A debtor at peace with debt above the drag threshold, owing `creditor`.
fn distressed(w: &mut WorldState, creditor: CountryId, debtor: CountryId, loan: f64) {
    let gdp = w.country(debtor).gdp;
    w.country_mut(debtor).debt = 0.8 * 4.0 * gdp;
    w.diplomacy.lend(creditor, debtor, loan, 0);
}

/// Forgiving writes the claim off: the debtor's debt falls by it, the claim
/// is gone and the debtor likes the creditor better.
#[test]
fn forgiving_lowers_the_debt_and_buys_goodwill() {
    let mut w = four_actor();
    let (maj, aly) = (w.find("MAJ").unwrap(), w.find("ALY").unwrap());
    distressed(&mut w, aly, maj, 2.0);
    let (debt, opinion) = (w.country(maj).debt, w.opinions.opinion(maj, aly));
    let mut control = w.clone();
    let r = turn(&mut w, &[(aly, Order::ForgiveDebt { debtor: maj })]);
    turn(&mut control, &[]);
    assert!(r.rejected.is_empty(), "{:?}", r.rejected);
    println!(
        "debt {debt:.2} -> {:.2} (control {:.2})",
        w.country(maj).debt,
        control.country(maj).debt
    );
    assert!((control.country(maj).debt - w.country(maj).debt - 2.0).abs() < 1e-6);
    assert!(w.diplomacy.loans.is_empty());
    assert!(w.opinions.opinion(maj, aly) > opinion.max(control.opinions.opinion(maj, aly)));
}

/// Holding a debtor in distress to its loan renews the debtor's historical
/// grudge against the creditor (a decision, not a standing condition: an
/// idle turn renews nothing); holding a debtor that is not in distress is
/// refused and renews nothing.
#[test]
fn holding_a_debtor_in_distress_renews_its_grudge() {
    let mut w = four_actor();
    let (maj, riv) = (w.find("MAJ").unwrap(), w.find("RIV").unwrap());
    // RIV lends to MAJ (grudge -40 between them in the fixture).
    w.diplomacy.lend(riv, maj, 1.0, 0);
    let renewed = |w: &WorldState| {
        w.opinions
            .modifiers(maj, riv)
            .iter()
            .filter_map(|m| m.memory)
            .map(|m| m.renewals)
            .sum::<u32>()
    };
    let before = renewed(&w);
    let r = turn(&mut w, &[(riv, Order::HoldDebt { debtor: maj })]);
    assert_eq!(r.rejected.len(), 1, "not in distress: refused");
    assert_eq!(renewed(&w), before);

    distressed(&mut w, riv, maj, 0.0);
    turn(&mut w, &[]);
    assert_eq!(renewed(&w), before, "a standing debt renews nothing by itself");
    let r = turn(&mut w, &[(riv, Order::HoldDebt { debtor: maj })]);
    assert!(r.rejected.is_empty(), "{:?}", r.rejected);
    let last = w
        .opinions
        .modifiers(maj, riv)
        .iter()
        .rev()
        .find_map(|m| m.memory)
        .unwrap();
    println!("MAJ grudge vs RIV after the hold: {last:?}");
    assert_eq!(renewed(&w), before + 1);
    assert!(matches!(last.renewed, Some((_, HostileAct::DebtHeld))));
}

fn world_1980() -> WorldState {
    let def = scenario::load(root().join("data/scenarios/1980.ron")).unwrap();
    scenario::build(&def, Some(3)).unwrap()
}

/// Saudi Arabia's decision on its loan to a distressed Iraq at peace, from
/// its own view on an assessment turn.
fn saudi_decision(w: &mut WorldState) -> (Vec<Order>, Vec<String>) {
    let sau = w.find("SAU").unwrap();
    w.turn = 8 + (4 - sau.0 as u32 % 4) % 4;
    let view = observe(w, sau);
    let d = Strategist::with_seed(3).decide(&view);
    let orders = d
        .orders
        .into_iter()
        .filter(|o| matches!(o, Order::ForgiveDebt { .. } | Order::HoldDebt { .. }))
        .collect();
    let lines = d
        .records
        .iter()
        .filter(|r| r.kind == sim_core::DecisionKind::ForgiveDebt)
        .map(|r| {
            format!(
                "{} {:+.1}: {:?}",
                r.subject,
                r.score,
                r.lines.iter().map(|l| (&l.label, l.value)).collect::<Vec<_>>()
            )
        })
        .collect();
    (orders, lines)
}

/// The creditor decides by its own interests (data swap). Iraq, still
/// facing Saudi Arabia's main threat (Iran), is a buffer worth keeping
/// solvent: forgive. Move Saudi Arabia's quarrel with Iran onto Iraq
/// (tension and both opinions) and the same debtor is now the danger the
/// debt keeps weak: hold.
#[test]
fn a_creditor_forgives_a_buffer_and_holds_a_threat() {
    let mut w = world_1980();
    let (sau, irq, irn) = (w.find("SAU").unwrap(), w.find("IRQ").unwrap(), w.find("IRN").unwrap());
    let loan = 0.2 * w.country(irq).gdp;
    w.tension.set(irq, irn, 80.0);
    distressed(&mut w, sau, irq, loan);
    let mut swapped = w.clone();

    let (orders, lines) = saudi_decision(&mut w);
    println!("{lines:?}");
    assert_eq!(orders, vec![Order::ForgiveDebt { debtor: irq }]);

    let (t_irn, t_irq) = (swapped.tension.get(sau, irn), swapped.tension.get(sau, irq));
    swapped.tension.set(sau, irn, t_irq);
    swapped.tension.set(sau, irq, t_irn);
    for (a, b, c, d) in [(sau, irn, sau, irq), (irn, sau, irq, sau)] {
        let x = std::mem::take(swapped.opinions.modifiers_mut(a, b));
        let y = std::mem::replace(swapped.opinions.modifiers_mut(c, d), x);
        *swapped.opinions.modifiers_mut(a, b) = y;
    }
    let (orders, lines) = saudi_decision(&mut swapped);
    println!("swapped: {lines:?}");
    assert_eq!(orders, vec![Order::HoldDebt { debtor: irq }]);
}
