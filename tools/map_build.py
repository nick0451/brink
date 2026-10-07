"""Build client/data/map.json for the Godot hologram map (V-1).

Sources: Natural Earth 110m coastline + admin-0 land boundary lines (public
domain), cached in tools/cache/ (downloaded on first run), and the hand-entered
city table below (capitals + 1-2 key cities per scenario country).

Coordinates are written as plain lon/lat degrees; the client projects them
(equirectangular), so the projection lives in one place.

Known V-1 limitation: Natural Earth borders are present-day (no inner-German
border, Soviet republics drawn). The ~100-region map replaces this later.
"""

import json
import os
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "tools", "cache")
OUT = os.path.join(ROOT, "client", "data", "map.json")
NE = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/"

# code: [(city, lon, lat, size 1-3, capital?)]
CITIES = {
    "USA": [("Washington", -77.04, 38.90, 3, True), ("New York", -74.0, 40.71, 3, False), ("Los Angeles", -118.24, 34.05, 2, False)],
    "SOV": [("Moscow", 37.62, 55.75, 3, True), ("Leningrad", 30.32, 59.93, 2, False), ("Vladivostok", 131.9, 43.12, 1, False)],
    "BAL": [("Riga", 24.11, 56.95, 1, True)],
    "UKR": [("Kiev", 30.52, 50.45, 2, True)],
    "CAU": [("Tbilisi", 44.79, 41.72, 1, True), ("Baku", 49.87, 40.41, 1, False)],
    "CAS": [("Tashkent", 69.24, 41.30, 2, True), ("Alma-Ata", 76.95, 43.24, 1, False)],
    "CHN": [("Beijing", 116.40, 39.90, 3, True), ("Shanghai", 121.47, 31.23, 3, False)],
    "JPN": [("Tokyo", 139.69, 35.69, 3, True), ("Osaka", 135.50, 34.69, 2, False)],
    "FRG": [("Bonn", 7.10, 50.73, 2, True), ("Hamburg", 9.99, 53.55, 2, False)],
    "DDR": [("East Berlin", 13.40, 52.52, 2, True)],
    "POL": [("Warsaw", 21.01, 52.23, 2, True)],
    "YUG": [("Belgrade", 20.46, 44.79, 2, True)],
    "SLO": [("Ljubljana", 14.51, 46.06, 1, True)],
    "CRO": [("Zagreb", 15.98, 45.81, 1, True)],
    "BIH": [("Sarajevo", 18.41, 43.86, 1, True)],
    "IRN": [("Tehran", 51.39, 35.69, 3, True), ("Abadan", 48.30, 30.34, 1, False)],
    "IRQ": [("Baghdad", 44.36, 33.31, 3, True), ("Basra", 47.78, 30.51, 2, False)],
    "SAU": [("Riyadh", 46.72, 24.69, 2, True), ("Dhahran", 50.10, 26.29, 1, False)],
    "ISR": [("Tel Aviv", 34.78, 32.09, 2, True)],
    "IND": [("New Delhi", 77.21, 28.61, 3, True), ("Bombay", 72.88, 19.08, 3, False)],
    "PAK": [("Islamabad", 73.05, 33.68, 2, True), ("Karachi", 67.01, 24.86, 2, False)],
    "PRK": [("Pyongyang", 125.76, 39.04, 2, True)],
    "KOR": [("Seoul", 126.98, 37.57, 3, True)],
    "ZAF": [("Pretoria", 28.19, -25.75, 2, True), ("Johannesburg", 28.05, -26.20, 2, False)],
    "CUB": [("Havana", -82.37, 23.11, 2, True)],
    "GBR": [("London", -0.13, 51.51, 3, True)],
    "FRA": [("Paris", 2.35, 48.86, 3, True), ("Marseille", 5.37, 43.30, 1, False)],
    "TUR": [("Ankara", 32.86, 39.93, 2, True), ("Istanbul", 28.98, 41.01, 3, False)],
    "EGY": [("Cairo", 31.24, 30.04, 3, True)],
    "SYR": [("Damascus", 36.29, 33.51, 2, True)],
    "VNM": [("Hanoi", 105.83, 21.03, 2, True), ("Ho Chi Minh City", 106.63, 10.82, 2, False)],
    "ARG": [("Buenos Aires", -58.38, -34.60, 3, True)],
    "AFG": [("Kabul", 69.17, 34.53, 2, True)],
    "KWT": [("Kuwait City", 47.98, 29.38, 1, True)],
    "TWN": [("Taipei", 121.56, 25.03, 2, True)],
}


def fetch(name):
    os.makedirs(CACHE, exist_ok=True)
    path = os.path.join(CACHE, name + ".geojson")
    if not os.path.exists(path):
        urllib.request.urlretrieve(NE + name + ".geojson", path)
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def lines(gj):
    out = []
    for feat in gj["features"]:
        g = feat["geometry"]
        parts = [g["coordinates"]] if g["type"] == "LineString" else g["coordinates"]
        for p in parts:
            if len(p) >= 2:
                out.append([[round(x, 2), round(y, 2)] for x, y in p])
    return out


def main():
    coast = lines(fetch("ne_110m_coastline"))
    borders = lines(fetch("ne_110m_admin_0_boundary_lines_land"))
    cities = [
        {"country": code, "name": n, "lon": lon, "lat": lat, "size": size, "capital": cap}
        for code, rows in CITIES.items()
        for (n, lon, lat, size, cap) in rows
    ]
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as f:
        json.dump({"coastline": coast, "borders": borders, "cities": cities}, f, separators=(",", ":"))
    pts = sum(len(l) for l in coast + borders)
    print(f"map.json: {len(coast)} coast + {len(borders)} border lines, {pts} points, {len(cities)} cities")


if __name__ == "__main__":
    main()
