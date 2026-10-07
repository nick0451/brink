"""Military calibration for the 1980 scenario.

Runs `brink military` over N seeds and compares each country's median
military power trajectory (2000 / 1980) with bands drawn from real
1980-2000 defence trajectories (SIPRI-era constant-dollar spending, widened
because the sim's power index is capability: strength x quality x readiness).
Also checks the 2000 balance against the United States.

Usage: python tools/calibrate_military.py [seeds=40]
"""

import json
import os
import statistics
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXE = os.path.join(ROOT, "target", "release", "brink.exe")

# code: (low, high) for median power 2000 / 1980, with the real-world anchor.
GROWTH = {
    "USA": (0.9, 1.4),   # ~1.1x constant $; Reagan build-up then peace dividend
    "SOV": (0.3, 0.9),   # stagnation then (in some runs) breakup
    "CHN": (1.0, 2.0),   # smaller PLA, modernised
    "JPN": (1.4, 2.3),   # ~1% of a growing GDP
    "FRG": (0.6, 1.1),   # peace dividend
    "GBR": (0.7, 1.1),   # Options for Change cuts
    "FRA": (0.8, 1.2),
    "ISR": (1.0, 1.7),
    "IND": (1.5, 2.6),
    "PAK": (1.3, 2.3),
    "KOR": (1.8, 3.0),
    "TWN": (1.2, 2.0),
    "SAU": (1.0, 2.2),   # flat spending, large imports
    "IRN": (0.6, 1.4),   # war losses, then rebuilding
    "IRQ": (0.4, 1.6),   # wide: depends on its wars
    "EGY": (1.0, 1.6),
    "SYR": (1.0, 1.7),
    "TUR": (1.4, 2.3),
    "VNM": (0.6, 1.1),
    "PRK": (0.9, 1.6),
    "CUB": (0.3, 0.8),   # loses Soviet support
    "ZAF": (0.5, 0.9),   # end of the border wars and apartheid
    "ARG": (0.4, 0.9),   # post-Falklands, post-junta cuts
    "POL": (0.6, 1.0),
    "KWT": (1.0, 3.0),
}
# Real GDP 2000 / 1980 (constant prices, World Bank / Maddison-era figures,
# widened). Military follows GDP, so growth is calibrated alongside it.
GDP = {
    "USA": (1.7, 2.2), "SOV": (0.6, 1.2), "CHN": (4.0, 7.5), "JPN": (1.6, 2.1), "FRG": (1.4, 1.8),
    "GBR": (1.5, 1.9), "FRA": (1.4, 1.8), "ISR": (1.9, 2.6), "IND": (2.2, 3.0), "PAK": (2.0, 2.8),
    "KOR": (2.8, 4.0), "TWN": (2.6, 3.6), "SAU": (1.1, 1.7), "IRN": (1.1, 1.8), "IRQ": (0.5, 1.3),
    "EGY": (1.9, 2.6), "SYR": (1.6, 2.4), "TUR": (1.9, 2.6), "VNM": (2.0, 3.0), "PRK": (0.6, 1.1),
    "CUB": (0.8, 1.3), "ZAF": (1.2, 1.6), "ARG": (1.2, 1.7), "POL": (1.1, 1.6), "KWT": (1.0, 1.8),
}
# Outcomes that history reached through the Soviet collapse: judged on the
# runs where the USSR breaks up (any successor active by 2000).
CONDITIONAL = {"SOV", "CUB", "VNM"}
SUCCESSORS = {"BAL", "UKR", "CAU", "CAS"}

# GDP paths history reached through events (the Iran-Iraq and Gulf wars,
# sanctions, the Soviet collapse): not fitted as structure; reported only.
EVENT_DRIVEN = {"IRQ", "IRN", "SOV", "CUB", "VNM", "PRK"}
# Command economies whose 1980s stagnation is structural (command drag, which
# a reform removes) but whose 1990s collapse is an event: fitted on 1980-1990.
# Soviet-bloc 1980s growth follows Khanin's recalculation (~0.6%/yr 1981-85,
# stagnation after), not the official/CIA 1.5-2%: the stagnation that
# underlay the crisis.
DECADE_GDP = {"SOV": 1.06, "CUB": 1.2, "VNM": 1.4, "PRK": 1.0}

# 2000 power as a share of the US (median), where history is clear.
VS_USA = {"JPN": (0.08, 0.3), "GBR": (0.08, 0.3), "FRA": (0.08, 0.3), "FRG": (0.08, 0.3),
          "CHN": (0.1, 0.45), "ISR": (0.03, 0.15), "IND": (0.05, 0.25)}


def run(seeds):
    out = subprocess.run([EXE, "military", os.path.join(ROOT, "data", "scenarios", "1980.ron"),
                          "--seeds", str(seeds), "--turns", "80"],
                         capture_output=True, text=True, check=True).stdout
    return [json.loads(l) for l in out.splitlines() if l.startswith("{")]


def measure(seeds):
    """Median 1980/1990/2000 military and GDP per country, and 2000 shares."""
    runs = run(seeds)
    mil, gdp, extra = {}, {}, {}
    cmil = {}
    breakups = 0
    for r in runs:
        last = r["samples"][-1]["countries"]
        broke = any(c["active"] for c in last if c["code"] in SUCCESSORS)
        breakups += broke
        if broke:
            first = {c["code"]: c["military"] for c in r["samples"][0]["countries"]}
            for c in last:
                if c["code"] in CONDITIONAL:
                    cmil.setdefault(c["code"], []).append(c["military"] / first[c["code"]])
        for s in r["samples"]:
            for c in s["countries"]:
                mil.setdefault(c["code"], {}).setdefault(s["turn"], []).append(c["military"])
                gdp.setdefault(c["code"], {}).setdefault(s["turn"], []).append(c["gdp"])
                if s["turn"] == 80:
                    extra.setdefault(c["code"], []).append((c["military_share"], c["military_tech"]))
    med = statistics.median
    out = {}
    for code in mil:
        out[code] = {
            "mil": [med(mil[code][t]) for t in (0, 40, 80)],
            "gdp": [med(gdp[code][t]) for t in (0, 40, 80)],
            "share": med(x[0] for x in extra[code]),
            "tech": med(x[1] for x in extra[code]),
            "breakup_ratio": med(cmil[code]) if code in cmil else None,
        }
    out["_breakups"] = breakups
    return out, len(runs)


def report(m, n):
    usa = m["USA"]["mil"][2]
    mbad = gbad = 0
    print(f"{'':4} {'1980':>6} {'1990':>6} {'2000':>6} {'x':>5}  band      vsUSA  mil%  tech  gdp x  band")
    for code, (lo, hi) in GROWTH.items():
        c = m[code]
        ratio = c["mil"][2] / c["mil"][0]
        note = ""
        if code in CONDITIONAL and c.get("breakup_ratio") is not None:
            note = f"  (all runs {ratio:.2f})"
            ratio = c["breakup_ratio"]
        vs = c["mil"][2] / usa
        g = c["gdp"][2] / c["gdp"][0]
        glo, ghi = GDP[code]
        flag = "" if lo <= ratio <= hi else (" LOW" if ratio < lo else " HIGH")
        if code in VS_USA and not (VS_USA[code][0] <= vs <= VS_USA[code][1]):
            flag += " vsUSA"
        mbad += bool(flag)
        if code in EVENT_DRIVEN:
            flag += " (gdp: event-driven)" if not glo <= g <= ghi else ""
        elif not glo <= g <= ghi:
            flag += " gdpLOW" if g < glo else " gdpHIGH"
            gbad += 1
        print(f"{code:4} {c['mil'][0]:6.1f} {c['mil'][1]:6.1f} {c['mil'][2]:6.1f} {ratio:5.2f}  {lo:.1f}-{hi:.1f}  {vs:6.2f} {100*c['share']:5.1f} {c['tech']:5.0f}  {g:5.2f}  {glo:.1f}-{ghi:.1f}{flag}{note}")
    print(f"\nmilitary: {len(GROWTH) - mbad}/{len(GROWTH)} in band; GDP: {len(GDP) - len(EVENT_DRIVEN) - gbad}/{len(GDP) - len(EVENT_DRIVEN)} structural in band; {n} seeds; USSR breakup in {m['_breakups']} (conditional rows use those runs)")


def main():
    seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 40
    report(*measure(seeds))


if __name__ == "__main__":
    main()
