"""Fit per-country structural growth (scenario data) to real 1980-2000 GDP.

Loop: simulate, compare each country's median GDP growth with the middle of
its historical band, nudge its structural growth modifier by the log gap
spread over 80 quarters, regenerate the scenario, repeat. Writes
tools/growth_fit.json, which gen1980.py adds to its COMMAND drag.

Usage: python tools/fit_growth.py [rounds=4] [seeds=40]
"""

import json
import math
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import calibrate_military as cal  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIT = os.path.join(ROOT, "tools", "growth_fit.json")
DAMPING = 0.8  # partial steps: GDP feeds back through wars and trade


def main():
    rounds = int(sys.argv[1]) if len(sys.argv) > 1 else 4
    seeds = int(sys.argv[2]) if len(sys.argv) > 2 else 40
    fit = json.load(open(FIT)) if os.path.exists(FIT) else {}
    for r in range(rounds):
        m, n = cal.measure(seeds)
        worst = 0.0
        for code, (lo, hi) in cal.GDP.items():
            if code in cal.DECADE_GDP:
                # Fit the 1980s (command stagnation); the collapse is an event.
                g = m[code]["gdp"][1] / m[code]["gdp"][0]
                target, quarters = cal.DECADE_GDP[code], 40.0
            elif code in cal.EVENT_DRIVEN:
                fit.pop(code, None)  # emerges from wars and sanctions
                continue
            else:
                g = m[code]["gdp"][2] / m[code]["gdp"][0]
                target, quarters = math.sqrt(lo * hi), 80.0
            step = DAMPING * math.log(target / g) / quarters
            fit[code] = round(fit.get(code, 0.0) + step, 5)
            worst = max(worst, abs(math.log(target / g)))
        json.dump(fit, open(FIT, "w"), indent=1, sort_keys=True)
        subprocess.run([sys.executable, os.path.join(ROOT, "tools", "gen1980.py")], check=True,
                       stdout=subprocess.DEVNULL)
        print(f"round {r + 1}: worst log gap {worst:.2f}")
    cal.report(*cal.measure(seeds))


if __name__ == "__main__":
    main()
