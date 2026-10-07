"""Generate data/scenarios/1980.ron from design/scenario-1980.md B.17/B.18/E.1.

Starting military strength is set to the level each budget sustains
(equilibrium = 10 x pool x military share / military_cost), so the 1980 world
doesn't decay or balloon on turn one. military_cost (data) reflects price
levels and conscription; it is chosen so equilibrium strength lands near the
B.17 table (land+naval+air) x 0.25.
"""
import json
import os

os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))

# code, name, gov, tier, area, align, pop, gdp, tax, debt_ratio(annual), fin_weight,
# mil_share, land, naval, air, techM, techI, stab, legit, cost, personality(7), reflex set
C = [
    ("USA", "United States", "Democracy", "Playable", "americas", "west", 227, 100, 0.20, 0.30, 0.55, 0.26, 55, 100, 100, 8, 8, 58, 55, 0.85, (.45, .50, .55, .65, .60, .65, .60), "USA"),
    ("SOV", "Soviet Union", "Authoritarian", "Playable", "europe", "east", 265, 45, 0.40, 0.15, 0.10, 0.40, 100, 60, 75, 7, 7, 55, 50, 1.25, (.45, .35, .85, .65, .30, .80, .45), "SOV"),
    ("CHN", "China", "Authoritarian", "Playable", "east_asia", None, 981, 7, 0.30, 0.05, 0.0, 0.28, 85, 15, 25, 3, 4, 60, 60, 0.25, (.35, .30, .80, .30, .75, .35, .45), "CHN"),
    ("JPN", "Japan", "Democracy", "Playable", "east_asia", "west", 117, 38, 0.22, 0.40, 0.30, 0.06, 12, 25, 20, 6, 4, 78, 75, 0.35, (.10, .15, .45, .85, .90, .20, .25), "JPN"),
    ("FRG", "West Germany", "Democracy", "Playable", "europe", "west", 61, 33, 0.30, 0.30, 0.35, 0.14, 30, 6, 22, 7, 6, 65, 65, 1.0, (.10, .25, .45, .80, .80, .30, .40), "FRG"),
    ("POL", "Poland", "Authoritarian", "Playable", "europe", "east", 36, 5, 0.40, 0.90, 0.0, 0.20, 18, 3, 7, 5, 5, 38, 35, 0.5, (.10, .25, .75, .35, .60, .30, .60), "POL"),
    ("YUG", "Yugoslavia", "Authoritarian", "Playable", "europe", None, 22, 3, 0.35, 0.70, 0.0, 0.20, 15, 4, 5, 5, 5, 55, 55, 0.4, (.20, .30, .75, .30, .60, .25, .45), "YUG"),
    ("IRN", "Iran", "Revolutionary", "Playable", "middle_east", None, 39, 3, 0.35, 0.10, 0.0, 0.30, 25, 4, 8, 3, 3, 45, 60, 0.35, (.60, .70, .90, .70, .30, .90, .40), "IRN"),
    ("IRQ", "Iraq", "Authoritarian", "Playable", "middle_east", None, 13, 2, 0.45, 0.10, 0.0, 0.45, 22, 1, 8, 4, 3, 55, 50, 0.5, (.80, .85, .90, .15, .60, .40, .90), "IRQ"),
    ("SAU", "Saudi Arabia", "Authoritarian", "Playable", "middle_east", None, 10, 6, 0.45, 0.05, 0.20, 0.30, 5, 2, 10, 4, 2, 65, 65, 1.5, (.10, .20, .85, .60, .75, .55, .30), "SAU"),
    ("ISR", "Israel", "Democracy", "Playable", "middle_east", "west", 4, 1, 0.45, 1.00, 0.0, 0.45, 15, 2, 15, 6, 7, 60, 65, 0.25, (.50, .55, .90, .45, .40, .40, .55), "ISR"),
    ("IND", "India", "Democracy", "Playable", "south_asia", None, 697, 6.5, 0.15, 0.40, 0.0, 0.22, 40, 6, 12, 4, 4, 55, 60, 0.15, (.30, .30, .70, .50, .40, .45, .35), "IND"),
    ("PAK", "Pakistan", "Authoritarian", "Playable", "south_asia", None, 80, 1, 0.18, 0.40, 0.0, 0.40, 18, 2, 6, 3, 5, 45, 40, 0.12, (.50, .70, .90, .35, .50, .55, .80), "PAK"),
    ("PRK", "North Korea", "Authoritarian", "Playable", "east_asia", "east", 18, 0.5, 0.55, 0.50, 0.0, 0.55, 30, 2, 6, 3, 4, 65, 60, 0.16, (.50, .70, .95, .20, .50, .85, .75), "PRK"),
    ("ZAF", "South Africa", "Authoritarian", "Playable", "africa", None, 29, 3, 0.25, 0.30, 0.05, 0.22, 8, 2, 6, 5, 5, 55, 45, 0.5, (.60, .45, .90, .25, .55, .60, .50), "ZAF"),
    ("CUB", "Cuba", "Revolutionary", "Playable", "americas", "east", 10, 1, 0.45, 0.60, 0.0, 0.30, 6, 1, 3, 3, 6, 60, 65, 0.55, (.60, .65, .85, .90, .20, .95, .45), "CUB"),
    # Tier 2: major AIs (same engine and AI).
    ("GBR", "United Kingdom", "Democracy", "Major", "europe", "west", 56, 22, 0.32, 0.40, 0.30, 0.15, 10, 25, 15, 7, 7, 60, 60, 1.0, (.35, .45, .55, .70, .55, .50, .45), None),
    ("FRA", "France", "Democracy", "Major", "europe", "west", 54, 27, 0.35, 0.25, 0.25, 0.13, 18, 12, 15, 7, 6, 60, 60, 1.0, (.40, .50, .50, .45, .60, .40, .55), None),
    ("DDR", "East Germany", "Authoritarian", "Major", "europe", "east", 17, 4, 0.45, 0.50, 0.0, 0.20, 10, 2, 5, 6, 8, 55, 45, 0.85, (.20, .25, .85, .80, .40, .80, .30), None),
    ("TUR", "Turkey", "Authoritarian", "Major", "europe", "west", 45, 3, 0.20, 0.50, 0.0, 0.25, 20, 4, 6, 4, 4, 40, 40, 0.25, (.40, .45, .70, .60, .45, .45, .45), None),
    ("EGY", "Egypt", "Authoritarian", "Major", "middle_east", "west", 42, 1.5, 0.25, 0.80, 0.0, 0.25, 15, 2, 6, 4, 4, 50, 50, 0.2, (.30, .35, .65, .50, .50, .40, .45), None),
    ("SYR", "Syria", "Authoritarian", "Major", "middle_east", "east", 9, 0.6, 0.35, 0.40, 0.0, 0.40, 12, 1, 5, 4, 4, 50, 45, 0.15, (.55, .55, .85, .50, .40, .60, .60), None),
    ("VNM", "Vietnam", "Authoritarian", "Major", "east_asia", "east", 53, 0.8, 0.35, 0.40, 0.0, 0.45, 25, 2, 4, 4, 4, 50, 55, 0.16, (.55, .55, .80, .70, .35, .80, .50), None),
    ("KOR", "South Korea", "Authoritarian", "Major", "east_asia", "west", 38, 2.5, 0.20, 0.40, 0.05, 0.30, 15, 2, 5, 5, 5, 50, 45, 0.3, (.25, .35, .85, .70, .75, .45, .35), None),
    ("ARG", "Argentina", "Authoritarian", "Major", "americas", None, 28, 3, 0.20, 0.60, 0.0, 0.20, 6, 3, 4, 4, 4, 50, 40, 0.5, (.60, .70, .60, .40, .45, .55, .75), None),
    # Tier 3: minors (same AI, restricted actions).
    ("AFG", "Afghanistan", "Revolutionary", "Minor", "south_asia", "east", 14, 0.2, 0.20, 0.30, 0.0, 0.40, 4, 0, 1, 2, 2, 25, 20, 0.1, (.40, .50, .80, .50, .30, .70, .40), None),
    ("KWT", "Kuwait", "Authoritarian", "Minor", "middle_east", None, 1.4, 1.2, 0.40, 0.05, 0.10, 0.08, 1, 0.5, 1, 3, 2, 65, 65, 1.5, (.10, .20, .70, .50, .80, .30, .30), None),
    ("TWN", "Taiwan", "Authoritarian", "Minor", "east_asia", "west", 18, 1.8, 0.20, 0.20, 0.05, 0.35, 8, 2, 4, 5, 5, 60, 55, 0.4, (.20, .30, .90, .60, .75, .50, .30), None),
]

# Energy capacity (DESIGN §2.4). Oil producers from real 1980 shares of world
# oil output (USA 16%, USSR 19%, Saudi 16%, Iraq 4%, Kuwait 2.7%, Iran 2.4%,
# China 3.3%, UK 2.5%, Egypt 1%, Argentina 0.8%) x world demand (sum of GDP);
# others: domestic energy as a share of their own demand (B.17 balances).
OIL_SHARE = {"USA": .162, "SOV": .19, "SAU": .157, "IRQ": .041, "IRN": .024, "KWT": .027, "CHN": .033,
             "GBR": .025, "EGY": .0095, "SYR": .0025, "ARG": .008}
OWN_ENERGY = {"POL": 1.4, "YUG": 0.6, "DDR": 0.7, "ZAF": 0.6, "FRG": 0.3, "JPN": 0.1, "KOR": 0.1, "TWN": 0.1,
              "TUR": 0.4, "FRA": 0.3, "ISR": 0.05, "PAK": 0.4, "PRK": 0.6, "CUB": 0.1, "VNM": 1.0, "AFG": 1.0,
              "IND": 0.7}
# Strategic arsenals (B.17; DESIGN §11.4). Israel's is undeclared (P10).
ARSENAL = {"USA": 3, "SOV": 3, "CHN": 2, "GBR": 2, "FRA": 2, "ISR": 1}
UNDECLARED = {"ISR"}
# B.18 transition vectors (P3): personality after a reform.
# Peripheries (P2): (id, name, pop share, gdp share, loyalty offset, successor).
# Loyalty is political (distance from the centre, history), never ethnic.
REGIONS = {
    # Baltics: annexed 1940, never reconciled; Caucasus: distant, restive.
    "SOV": [("baltics", "Baltic republics", .03, .04, -35, "BAL"), ("ukraine", "Ukraine", .19, .17, -15, "UKR"),
            ("caucasus", "Caucasus republics", .05, .04, -25, "CAU"), ("central_asia", "Central Asian republics", .15, .08, -5, "CAS")],
    # The richer republics resented federal transfers.
    "YUG": [("slovenia", "Slovenia", .08, .17, -25, "SLO"), ("croatia", "Croatia", .20, .25, -25, "CRO"),
            ("bosnia", "Bosnia", .18, .12, -20, "BIH")],
    "IRQ": [("north", "Northern provinces", .15, .08, -25, None)],
    "IRN": [("west", "Western provinces", .07, .04, -20, None)],
}
# Dormant successor states: (code, name, government, area).
SUCCESSORS = [("BAL", "Baltic States", "Democracy", "europe"), ("UKR", "Ukraine", "Democracy", "europe"),
              ("CAU", "Caucasus States", "Authoritarian", "europe"), ("CAS", "Central Asian States", "Authoritarian", "south_asia"),
              ("SLO", "Slovenia", "Democracy", "europe"), ("CRO", "Croatia", "Authoritarian", "europe"),
              ("BIH", "Bosnia", "Democracy", "europe")]

# Command economies stagnate (the "Command Economy" trait, as data): growth
# per quarter below their peers until a reform (P3) removes it.
# Export-oriented defence industry, 0-1 (D58): how much of what a country
# sells comes off its production lines and how big its order book is relative
# to its economy. Abstract index from 1980s export profiles (SIPRI-era
# literature): the USSR, US, France and UK by volume; Israel, North Korea and
# South Africa (Armscor) far above their size; China selling to both sides.
ARMS_INDUSTRY = {"ISR": 0.8, "FRA": 0.7, "SOV": 0.6, "PRK": 0.6, "GBR": 0.5, "CHN": 0.5,
                 "USA": 0.4, "ZAF": 0.4, "FRG": 0.3, "EGY": 0.2}
# Structural growth fitted to real 1980-2000 GDP by tools/fit_growth.py
# (demographics, export-led catch-up, reform era, sanctions, oil dependence):
# data, not engine branching. Separate from the command-economy drag below,
# which a reform removes.
_FIT_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "growth_fit.json")
GROWTH_FIT = json.load(open(_FIT_PATH)) if os.path.exists(_FIT_PATH) else {}
COMMAND = {"YUG": -0.003, "SOV": -0.003, "POL": -0.003, "DDR": -0.002, "CUB": -0.003, "VNM": -0.003, "PRK": -0.004}  # CHN: reforming since 1978
REFORM = {"SOV": (.25, .35, .55, .40, .55, .45, .45), "ZAF": (.15, .25, .40, .55, .65, .30, .35)}
# Covert programmes under way in 1980 (P10), with rough progress to a first
# device: South Africa close, India's "basement bomb", Pakistan racing,
# Iraq's Osirak project, North Korea and Taiwan early, Argentina's.
PROGRAMME = {"ZAF": 70, "IND": 60, "PAK": 35, "IRQ": 25, "PRK": 15, "TWN": 15, "ARG": 10}

# Post-1979 shock: start high, with a balanced market underneath (the glut).
ENERGY_PRICE = 1.5

# E.1: Iran's army purged after the revolution.
READINESS = {"IRN": 0.15}
# Rulers brought by revolution or installed by a conqueror, turn relative to
# the start (quarters): Iran's revolution, February 1979; Afghanistan's
# Soviet-installed regime, December 1979. Their purge recovery (sim-core
# `transition::command`) sets the readiness ceiling; Iran's 0.15 above is
# where the purge has left it on the first turn. Coups are not listed (the
# army was the coup: Korea 1979).
TOOK_POWER = {"IRN": -4, "AFG": -1}

WORLD_DEMAND = sum(row[7] for row in C)
CAPACITY = []
lines = []
out = lines.append
out("// 1980–2000 scenario (design/scenario-1980.md). Generated from B.17/B.18/E.1")
out("// by a first-pass script; numbers are to be tuned by headless runs (B.17).")
out("// Starting strength = what each budget sustains: 10 x revenue x military share")
out("// / military_cost. military_cost is data (price levels, conscription).")
out("Scenario(")
out('    name: "1980",')
out("    seed: 1980,")
out("    start_year: 1980,")
out("    turns: 80,")
out("    base_interest_rate: 0.014,")
out("    personality_jitter: 0.1,")
out(f"    energy_price: {ENERGY_PRICE},")
# P7: the Federal Reserve tightened in October 1979 (Volcker); 1980 opens Tight.
out("    monetary_stance: Tight,")
# US inflation was ~13% in 1980 after a decade of wage-price spiral.
out("    inflation: 1.0,")
out("    energy_outside_supply: OUTSIDE_SUPPLY,")
out('    include_reflexes: ["../reflexes/1980.ron"],')
out('    include_events: ["../events/1980.ron"],')
out("    countries: [")
report = []
for (code, name, gov, tier, area, align, pop, gdp, tax, debt_r, fw, share, land, naval, air, tm, ti, stab, legit, cost, pers, rset) in C:
    pool = gdp * tax
    strength = 10 * pool * share / cost
    table = (land + naval + air) * 0.25
    total = land + naval + air
    mix = (land / total, naval / total, air / total) if total else (0.6, 0.2, 0.2)
    welfare = 0.45 if gov == "Democracy" else 0.35
    intel = 0.07 if gov == "Democracy" else 0.10
    dev = max(0.05, 1.0 - share - welfare - intel)
    debt = debt_r * 4 * gdp
    a, r, p, l, g, i, o = pers
    al = f'Some("{align}")' if align else "None"
    rs = f'reflex_sets: ["{rset}"], ' if rset else ""
    out("        (")
    out(f'            id: "{code}", name: "{name}", government: {gov}, tier: {tier}, area: Some("{area}"), alignment: {al},')
    out(f"            population: {pop}, gdp: {gdp}, tax_rate: {tax}, debt: {debt:.2f}, financial_weight: {fw},")
    out(f"            budget: (military: {share}, development: {dev:.2f}, welfare: {welfare}, intelligence: {intel}),")
    out(f"            military: {strength:.2f}, military_cost: {cost}, force_mix: (land: {mix[0]:.3f}, naval: {mix[1]:.3f}, air: {mix[2]:.3f}),")
    out(f"            military_tech: {tm}, intel_tech: {ti}, stability: {stab}, legitimacy: {legit},")
    out(f"            personality: (aggression: {a}, risk: {r}, paranoia: {p}, loyalty: {l}, greed: {g}, ideology: {i}, opportunism: {o}),")
    if rs:
        out(f"            {rs.strip().rstrip(',')},")
    if code in READINESS:
        out(f"            readiness: Some({READINESS[code]}),")
    if code in TOOK_POWER:
        out(f"            took_power: Some({TOOK_POWER[code]}),")
    cap = OIL_SHARE.get(code, 0) * WORLD_DEMAND + OWN_ENERGY.get(code, 0) * gdp
    out(f"            energy_capacity: {cap:.2f},")
    if code in ARSENAL:
        out(f"            arsenal: {ARSENAL[code]},")
    if code in UNDECLARED:
        out("            arsenal_declared: false,")
    # A command economy's fitted stagnation is command drag (a reform ends
    # it); everyone else's fit is persistent structure.
    if code in COMMAND:
        out(f"            command_drag: {COMMAND[code] + GROWTH_FIT.get(code, 0.0):.5f},")
    elif GROWTH_FIT.get(code, 0.0):
        out(f"            growth_modifier: {GROWTH_FIT[code]:.5f},")
    if code in ARMS_INDUSTRY:
        out(f"            arms_industry: {ARMS_INDUSTRY[code]},")
    if code in REFORM:
        ra, rr, rp, rl, rg, ri, ro = REFORM[code]
        out(f"            reform_personality: Some((aggression: {ra}, risk: {rr}, paranoia: {rp}, loyalty: {rl}, greed: {rg}, ideology: {ri}, opportunism: {ro})),")
    if code in REGIONS:
        welfare_ = 0.45 if gov == "Democracy" else 0.35
        base_t = 30 + 0.4 * stab + 0.2 * legit + 40 * (welfare_ - 0.3)
        regs = []
        for rid, rname, ps, gs, off, succ in REGIONS[code]:
            loy = max(0.0, min(100.0, base_t + off))
            sc = f'Some("{succ}")' if succ else "None"
            regs.append(f'(id: "{rid}", name: "{rname}", pop_share: {ps}, gdp_share: {gs}, loyalty: {loy:.1f}, base: {off}.0, successor: {sc})')
        out("            regions: [" + ", ".join(regs) + "],")
    if code in PROGRAMME:
        out(f"            programme: Some({PROGRAMME[code]}.0),")
    CAPACITY.append(cap)
    out("        ),")
    report.append((code, round(strength, 1), round(table, 1)))
for scode, sname, sgov, sarea in SUCCESSORS:
    out("        (")
    out(f'            id: "{scode}", name: "{sname}", government: {sgov}, tier: Minor, area: Some("{sarea}"), alignment: None, dormant: true,')
    out("            population: 0.001, gdp: 0.001, tax_rate: 0.25, debt: 0.0,")
    out("            budget: (military: 0.15, development: 0.35, welfare: 0.40, intelligence: 0.10),")
    out("            military: 0.0, military_tech: 4, intel_tech: 3, stability: 50.0, legitimacy: 60.0,")
    out("        ),")
out("    ],")

op = []
def rel(a, b, ab, ba=None, source=None):
    src = f', source: "{source}"' if source else ""
    op.append(f'        (from: "{a}", to: "{b}", value: {ab}{src}),')
    op.append(f'        (from: "{b}", to: "{a}", value: {ba if ba is not None else ab}{src}),')

rel("USA", "SOV", -45)
# Cold War confrontation beyond the superpowers (D69/D72): the USSR and the
# Western alliance states on its perimeter. Scaled to the US-USSR pair (-45):
# the superpowers carried the quarrel; their allies shared it with less heat
# and more trade. Each pair is a documented 1980 confrontation. This is
# deliberately NOT a generic "rival bloc tag -> hostility" rule: that would
# also set Western Europe against Cuba (which it never embargoed) and Turkey
# against Vietnam, and the AI reads threat as power x hostility x reach, so
# only the pairs that confronted each other should carry it.
# West Germany: NATO's front line and its largest European army; the INF
# "double-track" decision (Dec 1979) answered the SS-20s. Softer toward
# Moscow than Moscow toward Bonn: Ostpolitik, the 1970 Moscow Treaty, the
# gas-pipeline trade.
rel("SOV", "FRG", -35, -25, "the inner-German border")
# Britain: Thatcher's line (the Soviet press coined "Iron Lady" in 1976);
# backed the US measures after Afghanistan.
rel("SOV", "GBR", -35, -35, "the second Cold War")
# France: Gaullist independence and detente; Giscard met Brezhnev in Warsaw
# (May 1980) while the allies boycotted Moscow. A quarrel at half heat.
rel("SOV", "FRA", -20, -20, "detente")
# Turkey: NATO's south-eastern flank, the Bosporus (the Black Sea Fleet's
# exit), centuries of Russo-Turkish war; the 1980 junta was fiercely
# anti-communist. Tempered by 1970s Soviet industrial credits.
rel("SOV", "TUR", -35, -30, "the Straits")
# Japan: no peace treaty since 1945 (the Northern Territories), the Soviet
# build-up on the Kurils from 1978, and Japan joined the Afghanistan measures
# and the Olympic boycott.
rel("SOV", "JPN", -30, -35, "Northern Territories")
# South Korea carries no Soviet quarrel of its own: Seoul's enemy was
# Pyongyang (the North's claim, tension 60), Moscow's quarrel was with
# Washington, and Seoul courted Moscow from 1988 (relations 1990). In this
# engine a -30 Soviet opinion makes the USSR Seoul's top threat (power share
# x hostility outranks the small, claim-holding neighbour), which stops KOR
# sanctioning the North and removes the North's isolation-driven reform
# exit (issue 10 measured 17 reforms to 77 coups). Deferred to the threat
# model (claims and neighbours), not represented as data.
rel("USA", "IRN", -70, -80, "hostage crisis")
rel("USA", "CUB", -50, -55)
rel("USA", "PRK", -60)
rel("SOV", "CHN", -50)
rel("CHN", "VNM", -60, -60, "border war")
rel("IRN", "IRQ", -55, -50, "border dispute")
rel("ISR", "SYR", -60)
rel("ISR", "IRQ", -50)
rel("IND", "PAK", -50, -55, "Kashmir")
rel("KOR", "PRK", -70)
rel("ARG", "GBR", -35, -20, "Falklands claim")
rel("TWN", "CHN", -60)
rel("ZAF", "CUB", -50)
rel("SAU", "IRN", -30, -35)
rel("IRQ", "KWT", -20, -10, "war debts")
rel("SYR", "IRQ", -30)
rel("PAK", "AFG", -40)
rel("SOV", "PAK", -40)
rel("CHN", "IND", -30)
rel("EGY", "SYR", -30)
rel("USA", "GBR", 45); rel("USA", "FRG", 40); rel("USA", "JPN", 40); rel("USA", "FRA", 25)
rel("USA", "KOR", 35); rel("USA", "TUR", 25); rel("USA", "ISR", 45, 50); rel("USA", "EGY", 30)
rel("USA", "SAU", 25, 20); rel("USA", "PAK", 15, 20); rel("USA", "TWN", 25, 40)
rel("GBR", "FRG", 30); rel("FRA", "FRG", 30); rel("GBR", "FRA", 20)
rel("SOV", "DDR", 40); rel("SOV", "POL", 30, 5); rel("SOV", "CUB", 30, 50); rel("SOV", "VNM", 30, 40)
rel("SOV", "SYR", 25, 30); rel("SOV", "IRQ", 15, 10); rel("SOV", "IND", 30); rel("SOV", "AFG", 30, -10)
rel("CHN", "PAK", 30); rel("CHN", "PRK", 30, 25); rel("SOV", "PRK", 15, 15)
rel("SAU", "KWT", 30); rel("SAU", "PAK", 25)
out("    opinions: [")
lines.extend(op)
out("    ],")

tr = []
def t(kind, a, b):
    tr.append(f'        (kind: {kind}, a: "{a}", b: "{b}"),')
nato = ["USA", "GBR", "FRA", "FRG", "TUR"]
for i in range(len(nato)):
    for j in range(i + 1, len(nato)):
        t("DefensiveAlliance", nato[i], nato[j])
for a, b in [("SOV", "POL"), ("SOV", "DDR"), ("POL", "DDR")]:
    t("DefensiveAlliance", a, b)
for a, b in [("USA", "JPN"), ("USA", "KOR"), ("CHN", "PRK"), ("SOV", "VNM")]:
    t("DefensiveAlliance", a, b)
for a, b in [("SOV", "IND"), ("SOV", "IRQ"), ("EGY", "ISR")]:
    t("NonAggression", a, b)
for g, c in [("USA", "SAU"), ("USA", "ISR")]:
    t("Guarantee", g, c)
for a, b in [("FRG", "FRA"), ("FRG", "GBR"), ("FRA", "GBR"), ("SOV", "DDR"), ("SOV", "POL")]:
    t("Trade(deep: true)", a, b)
for a, b in [("USA", "JPN"), ("USA", "FRG"), ("USA", "GBR"), ("USA", "KOR"), ("USA", "SAU"), ("JPN", "SAU"),
             ("FRG", "SAU"), ("USA", "TWN"), ("JPN", "KOR"), ("SOV", "CUB"), ("SOV", "VNM"), ("SOV", "IND"),
             ("FRA", "IRQ"), ("JPN", "CHN"), ("USA", "ISR"), ("USA", "EGY"), ("FRG", "TUR"), ("YUG", "FRG"),
             ("YUG", "SOV"), ("KWT", "JPN"), ("ARG", "SOV"), ("ZAF", "GBR"), ("IND", "GBR"), ("PAK", "CHN")]:
    t("Trade(deep: false)", a, b)
for host, owner in [("FRG", "USA"), ("JPN", "USA"), ("KOR", "USA"), ("TUR", "USA"), ("GBR", "USA"),
                    ("DDR", "SOV"), ("POL", "SOV"), ("CUB", "SOV"), ("VNM", "SOV"), ("AFG", "SOV")]:
    t("Basing", owner, host)
out("    treaties: [")
lines.extend(tr)
out("    ],")
out("    sanctions: [")
for by, tg in [("USA", "SOV"), ("USA", "IRN"), ("USA", "CUB")]:
    out(f'        (by: "{by}", target: "{tg}"),')
out("    ],")
# Standing claims (claimant, target, weight 0-1): the quarrels wars were
# fought over. Iraq: Shatt al-Arab and Khuzestan (1980), Kuwait as a "lost
# province" plus war debts (1990). Argentina: the Falklands. Kashmir both
# ways. One China. Korea both ways. Syria: the Golan. China-Vietnam border.
CLAIMS = [("IRQ", "IRN", 1.0), ("IRQ", "KWT", 0.7), ("ARG", "GBR", 1.0), ("PAK", "IND", 0.8),
          ("IND", "PAK", 0.4), ("CHN", "TWN", 1.0), ("PRK", "KOR", 0.8), ("KOR", "PRK", 0.3),
          ("SYR", "ISR", 0.8), ("CHN", "VNM", 0.3)]
out("    claims: [")
for by, against, w in CLAIMS:
    out(f'        (by: "{by}", against: "{against}", weight: {w}),')
out("    ],")
out("    streams: [")
# Aid sized to the recipient's government revenue (calibration 2026-10-04):
# US aid ~20% of Israel's and Egypt's revenue; Soviet support ~40% of Cuba's
# (sugar and oil subsidies), ~50% of Vietnam's; Afghanistan is a client at war.
for a, b, amt in [("USA", "ISR", 0.09), ("USA", "EGY", 0.08), ("SOV", "CUB", 0.18), ("SOV", "VNM", 0.14), ("SOV", "AFG", 0.04)]:
    out(f'        (from: "{a}", to: "{b}", amount: {amt}),')
out("    ],")
out("    tensions: [")
# The other Cold War pairs start at the baseline their opinions imply (~20);
# the two acute 1980 disputes start above it: the Euromissile crisis on the
# central front, and the Soviet build-up on the Kurils.
for a, b, v in [("IRN", "IRQ", 70), ("USA", "SOV", 45), ("CHN", "VNM", 75), ("USA", "IRN", 75), ("IND", "PAK", 50),
                ("KOR", "PRK", 60), ("ISR", "SYR", 60), ("SOV", "PAK", 45), ("PAK", "AFG", 50),
                ("SOV", "FRG", 30), ("SOV", "JPN", 30)]:
    out(f'        (a: "{a}", b: "{b}", value: {v}),')
out("    ],")
out("    history: [")
for actor, cp, kind, grade, ago, cause in [
    ("USA", "VNM", "Back", "Some(Abandoned)", 20, "withdrew support from South Vietnam (1975)"),
    ("USA", "ISR", "Back", "Some(Honoured)", 26, "airlift during the 1973 war"),
    ("SOV", "AFG", "Coercion", "None", 1, "invasion of Afghanistan (1979)"),
    ("SOV", "DDR", "Coercion", "None", 47, "Prague-style intervention precedent (1968)"),
    ("CHN", "VNM", "Coercion", "None", 4, "punitive war on Vietnam (1979)"),
    ("SOV", "CUB", "Back", "Some(Honoured)", 40, "kept Cuba supplied for two decades"),
]:
    out(f'        (actor: "{actor}", counterpart: "{cp}", kind: {kind}, grade: {grade}, turns_ago: {ago}, cause: "{cause}"),')
out("    ],")
out(")")
os.makedirs("data/scenarios", exist_ok=True)
outside = max(0.0, WORLD_DEMAND - sum(CAPACITY))
text = "\n".join(lines) + "\n"
text = text.replace("OUTSIDE_SUPPLY", f"{outside:.2f}")
open("data/scenarios/1980.ron", "w", encoding="utf-8").write(text)
print("world demand", round(WORLD_DEMAND, 1), "roster capacity", round(sum(CAPACITY), 1), "outside", round(outside, 1))
for r in report:
    print(r)
