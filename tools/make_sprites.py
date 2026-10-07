"""Placeholder NES-style pixel art for the V-1 visual spike.

Writes client/art/: 32x32 portraits keyed by regime (D56), an office seal,
16x16 event markers and a 64x64 city tile in three damage states (D57).

Every portrait carries a 3-pixel magenta corner: PLACEHOLDER, replace with
commissioned art. Caricature rules (behaviour-and-voice §2.3): exaggerate
public persona and conduct only (hair, brows, medals, beret); never ethnicity,
faith, religious dress, disability or "national character". City art never
shows people.
"""

import os
import random

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "client", "art")

# A small NES-like palette.
P = {
    ".": None,
    "k": (12, 12, 20), "w": (236, 236, 236), "g": (124, 124, 124), "G": (188, 188, 188),
    "s": (252, 188, 148), "S": (216, 140, 104), "b": (100, 60, 24), "B": (60, 32, 8),
    "r": (200, 24, 24), "n": (24, 40, 104), "N": (44, 64, 140), "o": (88, 104, 40),
    "O": (60, 72, 24), "y": (248, 200, 24), "Y": (184, 132, 0), "m": (255, 0, 255),
    "d": (40, 40, 48), "e": (148, 148, 156),
}


def canvas(n, bg=(16, 28, 36)):
    return Image.new("RGBA", (n, n), bg + (255,))


def put(img, x, y, c):
    if c is not None and 0 <= x < img.width and 0 <= y < img.height:
        img.putpixel((x, y), c + (255,) if len(c) == 3 else c)


def rect(img, x0, y0, x1, y1, c):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(img, x, y, P[c] if isinstance(c, str) else c)


def oval(img, cx, cy, rx, ry, c):
    for y in range(cy - ry, cy + ry + 1):
        for x in range(cx - rx, cx + rx + 1):
            if ((x - cx) / (rx + 0.5)) ** 2 + ((y - cy) / (ry + 0.5)) ** 2 <= 1:
                put(img, x, y, P[c])


def face(img, skin="s", shade="S"):
    oval(img, 16, 15, 7, 9, skin)
    for y in range(9, 24):
        put(img, 22, y, P[shade])
    rect(img, 13, 24, 19, 26, skin)  # neck


def eyes(img, y=14):
    for x in (13, 19):
        put(img, x, y, P["k"])
    put(img, 16, y + 3, P["S"])  # nose


def body(img, suit, tie=None):
    oval(img, 16, 31, 13, 5, suit)
    if tie:
        rect(img, 16, 27, 16, 31, tie)
    put(img, 14, 27, P["w"])
    put(img, 18, 27, P["w"])


def placeholder(img):
    for x, y in ((0, 0), (1, 0), (0, 1)):
        put(img, x, y, P["m"])


def portrait_usa_base():
    i = canvas(32)
    body(i, "n", "r")
    face(i)
    oval(i, 16, 7, 8, 3, "b")  # pompadour
    rect(i, 9, 8, 10, 13, "b")
    rect(i, 22, 8, 23, 12, "b")
    eyes(i)
    rect(i, 13, 19, 19, 19, "k")  # broad grin
    rect(i, 14, 20, 18, 20, "w")
    return i


def portrait_sov_base():
    i = canvas(32)
    body(i, "d")
    for x in (7, 9, 11):  # rows of medals
        for y in (28, 30):
            put(i, x, y, P["y"])
            put(i, x + 14, y, P["y"])
    face(i)
    oval(i, 16, 7, 7, 3, "G")  # swept grey hair
    eyes(i, 15)
    rect(i, 11, 12, 15, 13, "k")  # heavy brows
    rect(i, 17, 12, 21, 13, "k")
    rect(i, 14, 20, 18, 20, "S")  # flat mouth
    return i


def portrait_sov_reformed():
    i = canvas(32)
    body(i, "e", "n")
    face(i)
    eyes(i)
    for x in (12, 13, 14, 18, 19, 20):  # spectacles
        put(i, x, 13, P["k"])
        put(i, x, 15, P["k"])
    rect(i, 15, 14, 17, 14, "k")
    rect(i, 14, 20, 18, 20, "S")
    return i


def portrait_irn_base():
    # No headwear or religious dress (caricature rule): beard and frown only.
    i = canvas(32)
    body(i, "k")
    face(i)
    oval(i, 16, 21, 6, 5, "w")  # white beard
    eyes(i)
    rect(i, 11, 11, 14, 11, "k")  # frown brows slant
    rect(i, 18, 11, 21, 11, "k")
    put(i, 14, 12, P["k"])
    put(i, 18, 12, P["k"])
    oval(i, 16, 7, 6, 2, "G")
    return i


def portrait_irq_base():
    i = canvas(32)
    body(i, "o")
    rect(i, 3, 26, 7, 27, "y")  # epaulettes
    rect(i, 25, 26, 29, 27, "y")
    face(i)
    oval(i, 15, 7, 9, 3, "k")  # beret
    put(i, 22, 6, P["r"])
    eyes(i)
    rect(i, 12, 18, 20, 19, "k")  # moustache
    return i


def portrait_generic(kind):
    i = canvas(32)
    face(i, "e", "g")
    if kind == "revolutionary":
        body(i, "O")
        rect(i, 7, 5, 25, 8, "O")  # peaked cap
        rect(i, 5, 9, 27, 9, "k")
    else:
        body(i, "N", "r")
        oval(i, 16, 7, 7, 2, "g")
    eyes(i)
    return i


def seal():
    i = canvas(32, (0, 0, 0))
    i.putalpha(0)
    oval(i, 16, 16, 14, 14, "Y")
    oval(i, 16, 16, 12, 12, "n")
    for dx, dy in ((0, -5), (-1, -3), (0, -3), (1, -3), (-5, -1), (-4, -1), (-3, -1), (-2, -1), (-1, -1), (0, -1),
                   (1, -1), (2, -1), (3, -1), (4, -1), (5, -1), (-3, 1), (-2, 1), (-1, 1), (0, 1), (1, 1), (2, 1),
                   (3, 1), (-2, 3), (-1, 3), (1, 3), (2, 3), (-3, 5), (3, 5)):
        put(i, 16 + dx, 16 + dy, P["w"])
    return i


MARKERS = {
    # kind: (colour, 16x16 pattern of '#' cells drawn in rows of 8 doubled)
    "war": ((255, 60, 40), ["#......#", ".#....#.", "..#..#..", "...##...", "...##...", "..#..#..", ".#....#.", "#......#"]),
    "joined_war": ((255, 130, 60), ["........", ".#....#.", "..#..#..", "...##...", "...##...", "..#..#..", ".#....#.", "........"]),
    "peace": ((200, 255, 220), ["........", "...##...", "..####..", ".######.", "...##...", "...##...", "...##...", "........"]),
    "sanction": ((255, 190, 40), ["..####..", ".#....#.", ".#....#.", "########", "########", "###..###", "########", "########"]),
    "crisis": ((255, 230, 60), ["...##...", "...##...", "...##...", "...##...", "...##...", "........", "...##...", "...##..."]),
    "transition": ((120, 220, 255), [".######.", "#......#", "#..##..#", "#.####.#", "#..##..#", "#......#", ".######.", "........"]),
    "secession": ((200, 140, 255), ["###..###", "##....##", "#......#", "........", "........", "#......#", "##....##", "###..###"]),
    "programme": ((120, 255, 140), ["...##...", ".#.##.#.", "..####..", "########", "########", "..####..", ".#.##.#.", "...##..."]),
    "nuclear": ((240, 240, 240), ["...##...", "..####..", ".######.", "########", "...##...", "..####..", ".##..##.", "##....##"]),
    "treaty": ((120, 200, 255), ["########", "#......#", "#.####.#", "#......#", "#.####.#", "#......#", "########", "........"]),
    "mobilized": ((255, 120, 120), ["#......#", "##....##", ".##..##.", "..####..", "#......#", "##....##", ".##..##.", "..####.."]),
    "energy": ((255, 170, 30), ["...##...", "...##...", "..####..", ".######.", "########", "########", ".######.", "..####.."]),
}


def marker(colour, rows):
    i = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch == "#":
                for dx in (0, 1):
                    for dy in (0, 1):
                        i.putpixel((x * 2 + dx, y * 2 + dy), colour + (255,))
    return i


def city(state):
    """64x64 night city in hologram greens; damage is buildings, fire, smoke only."""
    rnd = random.Random(1980)
    i = Image.new("RGBA", (64, 64), (6, 14, 12, 255))
    for y in range(0, 64, 8):  # street grid
        for x in range(64):
            i.putpixel((x, y), (20, 52, 40, 255))
            i.putpixel((y, x), (20, 52, 40, 255))
    blocks = [(bx, by) for by in range(1, 64, 8) for bx in range(1, 64, 8)]
    for n, (bx, by) in enumerate(blocks):
        r = rnd.random()
        collapsed = (state == "damaged" and r < 0.25) or (state == "burning" and r < 0.5)
        burning = state == "burning" and 0.5 <= r < 0.8 or state == "damaged" and 0.25 <= r < 0.35
        for y in range(by, by + 7):
            for x in range(bx, bx + 7):
                if collapsed:
                    c = (60, 56, 50) if rnd.random() < 0.6 else (24, 24, 22)
                else:
                    c = (30, 90, 70)
                    if (x + y) % 3 == 0 and rnd.random() < 0.5:
                        c = (150, 255, 190) if state == "intact" else (70, 120, 90)
                if burning and rnd.random() < 0.55:
                    c = rnd.choice([(255, 80, 20), (255, 160, 30), (255, 220, 80), (180, 30, 10)])
                i.putpixel((x, y), c + (255,))
    if state != "intact":  # smoke drifting over the grid
        for _ in range(220 if state == "damaged" else 600):
            x, y = rnd.randrange(64), rnd.randrange(64)
            g = rnd.randrange(30, 70)
            i.putpixel((x, y), (g, g, g, 255))
    return i


def save(img, *parts, scale=1):
    path = os.path.join(OUT, *parts)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    img.save(path)


def main():
    portraits = {
        "usa_base": portrait_usa_base(), "sov_base": portrait_sov_base(),
        "sov_reformed": portrait_sov_reformed(), "irn_base": portrait_irn_base(),
        "irq_base": portrait_irq_base(), "generic_base": portrait_generic("base"),
        "generic_revolutionary": portrait_generic("revolutionary"),
    }
    for key, img in portraits.items():
        placeholder(img)
        save(img, "portraits", key + ".png")
    save(seal(), "portraits", "seal.png")
    for kind, (c, rows) in MARKERS.items():
        save(marker(c, rows), "markers", kind + ".png")
    for state in ("intact", "damaged", "burning"):
        save(city(state), "city", state + ".png")
    print(f"{len(portraits)} portraits + seal, {len(MARKERS)} markers, 3 city states -> {OUT}")


if __name__ == "__main__":
    main()
