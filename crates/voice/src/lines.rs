//! Satirical line data (design/behaviour-and-voice.md §2, approval items
//! 3–5, 10–12). Every line names the trigger that makes it true; its
//! placeholders must be a subset of what that trigger supplies, so a line
//! can never claim something the simulation did not compute.

use std::path::Path;

use serde::Deserialize;

use crate::Intensity;

/// Simulation situations the narrator can detect. Each supplies a fixed
/// set of placeholders (see [`Trigger::placeholders`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum Trigger {
    /// A norm claimant ignored a violation by a friend.
    NormIgnoredByFriend,
    /// A norm claimant answered a violation with sanctions only.
    NormPartialResponse,
    /// A withdrawn commitment's former beneficiary was attacked.
    ShadowThenAttack,
    /// A country asked for protection from a power it recently denounced.
    ProtectionFromDenounced,
    /// The sanctioner loses more (share of GDP) than the target.
    SanctionsHurtSender,
    /// The target rerouted much of the trade lost to sanctions.
    SanctionsSubstituted,
    SanctionsLiftedAfterAdaptation,
    SanctionsReimposed,
    /// Sanctions between deep-trade partners.
    DeepTradeThenSanction,
    /// Joined an ally's sanctions mainly on solidarity/standing terms.
    CoalitionByDependence,
    /// Support cut while the client depended heavily on it.
    ClientCutWhileDependent,
    /// An alliance or guarantee cancelled.
    CommitmentCancelled,
    GuaranteeRefused,
    RepeatedRequestRefused,
    /// Refused a treaty from the patron that funds it.
    RefusedPatron,
    /// A refusal decided mainly by inertia.
    StatusQuoRefusal,
    /// Support kept through a client's crisis (graded Honoured).
    CrisisSupportKept,
    /// Deficit spending stopped by the debt brake.
    DebtBrake,
    /// An action rejected for lack of Initiative.
    OutOfInitiative,
    /// Global tension crossed into a tenser Readiness Condition.
    ReadinessRising,

    // ── V2: war registers (G3 unless noted) ──
    /// A war of limited or major aims began.
    WarBegan,
    /// War declared from a calm baseline (no casus belli).
    WarWithoutCasusBelli,
    PunitiveStrike,
    /// The target refused a punitive strike; it became a limited war.
    StrikeRefusedBecameWar,
    /// A guarantor/ally ignored an attack on the country it defends.
    GuarantorStayedOut,
    /// A guarantor answered an attack with sanctions or arms only (Partial).
    GuarantorSentSanctions,
    /// Joined a war for an ally with too few forces (fake honour → Partial).
    TokenForces,
    /// Armed both sides of the same war.
    ArmsToBothSides,
    /// A client fights its former supplier with the supplier's own arms.
    TransferredArmsUsedAgainstSupplier,
    /// Covert arms exposed (G2).
    CovertArmsExposed,
    /// A war ended where it started.
    WhitePeaceAfterWar,
    /// Full or total mobilization in peacetime (G2).
    MobilizedAtPeace,
    /// Attacked while its forces were hollow from budget cuts.
    CutMilitarySpendingThenThreatened,

    // ── V3 ──
    /// A scenario event template fired; lines pick their event by `event`.
    ScenarioEvent,
    /// Started arming a client in peacetime.
    ArmedClient,
    /// Dropped containment of a rival for other goals.
    StrategyReversed,
    /// The campaign epitaph (Official History column).
    Epitaph,
    /// Nuclear weapons used: G5, always plain. No line may exist for it.
    NuclearUse,
    /// A secret weapons programme was exposed (P10).
    ProgrammeExposed,
    /// An undeclared arsenal was declared.
    ArsenalDeclared,
    /// An arsenal or programme was disclosed and dismantled.
    ArsenalDismantled,
    /// A periphery seceded (P2). The joke is the centre, never the region.
    Secession,
    /// A patron objects to its client selling arms to the patron's rival (D58).
    ArmsSoldToPatronsRival,
    /// The reserve holder tightened money (P7).
    RatesRaised,
    /// The reserve holder eased money (P7).
    RatesCut,
}

impl Trigger {
    /// Placeholders this trigger supplies (besides `actor` and `target`).
    pub fn placeholders(self) -> &'static [&'static str] {
        use Trigger::*;
        match self {
            NormIgnoredByFriend | NormPartialResponse => &["norm"],
            ShadowThenAttack => &[],
            ProtectionFromDenounced => &["turns_ago"],
            SanctionsHurtSender => &["own_loss", "target_loss"],
            SanctionsSubstituted | SanctionsLiftedAfterAdaptation | SanctionsReimposed => &["rerouted"],
            DeepTradeThenSanction => &[],
            CoalitionByDependence => &["term"],
            ClientCutWhileDependent => &["dependence", "amount"],
            CommitmentCancelled => &["treaty"],
            GuaranteeRefused => &["term"],
            RepeatedRequestRefused => &["n"],
            RefusedPatron | StatusQuoRefusal => &["treaty"],
            CrisisSupportKept => &[],
            DebtBrake => &["debt"],
            OutOfInitiative => &["action"],
            ReadinessRising => &["condition", "tension"],
            WarBegan => &["aim"],
            WarWithoutCasusBelli => &["tension"],
            PunitiveStrike | StrikeRefusedBecameWar | GuarantorStayedOut => &[],
            GuarantorSentSanctions => &["response"],
            TokenForces => &["committed", "needed"],
            ArmsToBothSides => &["other"],
            TransferredArmsUsedAgainstSupplier => &["arms"],
            CovertArmsExposed => &[],
            WhitePeaceAfterWar => &["turns"],
            MobilizedAtPeace => &["level"],
            CutMilitarySpendingThenThreatened => &["readiness"],
            ScenarioEvent => &[],
            ArmedClient => &["amount"],
            StrategyReversed => &["old", "new"],
            Epitaph => &["honoured", "partial", "abandoned", "wars", "growth", "years"],
            NuclearUse => &[],
            ProgrammeExposed | ArsenalDeclared | ArsenalDismantled => &[],
            Secession => &[],
            ArmsSoldToPatronsRival => &["buyer", "funds"],
            RatesRaised | RatesCut => &["rate"],
        }
    }

    /// Human cost on screen (design §2.2). Diplomacy and economics: G1;
    /// coercion: G2; war: G3 (Black only on command surfaces; casualty
    /// figures stay in the plain FACT). Nothing here reaches G4+.
    pub fn gravity(self) -> u8 {
        use Trigger::*;
        match self {
            SanctionsHurtSender
            | SanctionsSubstituted
            | SanctionsLiftedAfterAdaptation
            | SanctionsReimposed
            | DeepTradeThenSanction
            | CoalitionByDependence
            | NormIgnoredByFriend
            | NormPartialResponse
            | ShadowThenAttack
            | CovertArmsExposed
            | MobilizedAtPeace
            | ScenarioEvent
            | ProgrammeExposed
            | ArsenalDeclared
            | ArsenalDismantled
            | Secession => 2,
            WarBegan
            | WarWithoutCasusBelli
            | PunitiveStrike
            | StrikeRefusedBecameWar
            | GuarantorStayedOut
            | GuarantorSentSanctions
            | TokenForces
            | ArmsToBothSides
            | TransferredArmsUsedAgainstSupplier
            | WhitePeaceAfterWar
            | CutMilitarySpendingThenThreatened => 3,
            NuclearUse => 5,
            _ => 1,
        }
    }
}

/// Where a line appears and who is speaking (approval item 5: public
/// statements are careful doublespeak; internal material is candid).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Surface {
    /// Public: euphemism, passive voice, legalistic qualification.
    OfficialStatement,
    /// Private/intercepted: openly cynical, real numbers.
    InternalMemo,
    TreasuryMemo,
    Ticker,
}

impl Surface {
    pub fn heading(self) -> &'static str {
        match self {
            Surface::OfficialStatement => "OFFICIAL STATEMENT",
            Surface::InternalMemo => "INTERNAL MEMO",
            Surface::TreasuryMemo => "TREASURY MEMO",
            Surface::Ticker => "WIRE",
        }
    }

    pub fn is_private(self) -> bool {
        matches!(self, Surface::InternalMemo | Surface::TreasuryMemo)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Line {
    pub id: String,
    pub trigger: Trigger,
    pub surface: Surface,
    /// 1 = ledger contradiction … 6 = general flavour (approval item 4).
    pub priority: u8,
    /// Lowest setting that shows the line.
    pub intensity: Intensity,
    /// Highest gravity the line may appear at.
    #[serde(default = "default_max_gravity")]
    pub max_gravity: u8,
    #[serde(default = "default_cooldown")]
    pub cooldown: u32,
    #[serde(default = "default_cap")]
    pub max_per_campaign: u32,
    pub text: String,
    /// For `ScenarioEvent` lines: the event template id the line belongs to.
    #[serde(default)]
    pub event: Option<String>,
}

fn default_max_gravity() -> u8 {
    2
}
fn default_cooldown() -> u32 {
    8
}
fn default_cap() -> u32 {
    3
}

/// Atrocity-specific language never allowed in comic lines (design §2.3,
/// approval item 10). Ordinary bureaucratic doublespeak is welcome.
pub const BANNED_TERMS: &[&str] = &[
    "cleansing",
    "resettlement",
    "final solution",
    "reclassification",
    "homeland",
    "separate development",
    "extermination",
    "liquidat",
];

/// Placeholder names used in a template, e.g. `{actor}`.
pub fn placeholders_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        match after.find('}') {
            Some(end) => {
                out.push(after[..end].to_string());
                rest = &after[end + 1..];
            }
            None => break,
        }
    }
    out
}

impl Line {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=6).contains(&self.priority) {
            return Err(format!("{}: priority must be 1–6", self.id));
        }
        if self.intensity == Intensity::Off {
            return Err(format!("{}: a flavour line cannot require intensity Off", self.id));
        }
        if self.trigger.gravity() > self.max_gravity {
            return Err(format!("{}: trigger gravity exceeds the line's max_gravity", self.id));
        }
        // G4–G5 (mass death, nuclear use) are plain in every mode: no line
        // may be written for them at all.
        if self.trigger.gravity() >= 4 {
            return Err(format!("{}: G4–G5 moments are always plain", self.id));
        }
        // G3: only command and decision-maker surfaces, never the wire
        // (behaviour-and-voice.md approval item 6).
        if self.trigger.gravity() >= 3 && self.surface == Surface::Ticker {
            return Err(format!(
                "{}: war lines belong on command surfaces, not the wire",
                self.id
            ));
        }
        if (self.trigger == Trigger::ScenarioEvent) != self.event.is_some() {
            return Err(format!(
                "{}: scenario-event lines name their event, and only they do",
                self.id
            ));
        }
        let lower = self.text.to_lowercase();
        if let Some(term) = BANNED_TERMS.iter().find(|t| lower.contains(*t)) {
            return Err(format!("{}: banned term \"{term}\"", self.id));
        }
        let allowed = self.trigger.placeholders();
        for p in placeholders_in(&self.text) {
            if p != "actor" && p != "target" && !allowed.contains(&p.as_str()) {
                return Err(format!(
                    "{}: placeholder {{{p}}} not supplied by {:?}",
                    self.id, self.trigger
                ));
            }
        }
        Ok(())
    }
}

pub fn parse_lines(text: &str) -> Result<Vec<Line>, String> {
    let lines: Vec<Line> = ron::from_str(text).map_err(|e| format!("voice lines parse error: {e}"))?;
    let mut ids = std::collections::BTreeSet::new();
    for l in &lines {
        l.validate()?;
        if !ids.insert(l.id.clone()) {
            return Err(format!("duplicate line id {}", l.id));
        }
    }
    Ok(lines)
}

pub fn load_lines(path: impl AsRef<Path>) -> Result<Vec<Line>, String> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    parse_lines(&text)
}
