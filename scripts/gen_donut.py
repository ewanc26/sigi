#!/usr/bin/env python3
"""Generate examples/donut.si: a fully rendered polygonal torus in sigi.

The program computes the surface in-language: sin/cos opcodes over a
ring/tube parameterisation, a per-frame spin about X plus a fixed tilt
about Y, rotated-normal Lambert shading (unclamped: the opacity band
absorbs negative light), orthographic projection, and alpha-filled SVG
polygons as a filmstrip. Numeric prints end with newlines, which SVG
treats as whitespace inside point lists and attribute values.
"""

import math
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Geometry
R, TUBE = 3.0, 1.25
RING, TUBE_N = 16, 10
SCALE, OY_BASE = 28.0, 130.0
FRAMES = 4
TILT = 0.72
SPIN_STEP = 0.62
LIGHT = (0.32, 0.58, 0.75)
COL_W = 260

# Numbered variable slots (sigi has 100).
_names = [
    "ct1",
    "st1",
    "ct2",
    "st2",
    "cp1",
    "sp1",
    "cp2",
    "sp2",
    "ca",
    "sa",
    "x",
    "y",
    "z",
    "rad",
    "cy",
    "cz",
    "nx",
    "ny",
    "nz",
    "u",
]
SL = {n: i + 1 for i, n in enumerate(_names)}
for i, n in enumerate(_names, 1):
    SL[n] = i

TH_STEP = 2 * math.pi / RING
PH_STEP = 2 * math.pi / TUBE_N
CB, SB = round(math.cos(TILT), 12), round(math.sin(TILT), 12)

SLOT_BASE = 1
num = lambda v: "!{}".format(repr(round(v, 12)))
ld = lambda n: "<{}>".format(SL[n])
st = lambda n: "!{} :".format(SL[n])  # value already on top; push index; store


def sp(s):  # sigi string literal
    return '"{}"'.format(s)


lines = []


def put(*ops):
    lines.append(" ".join(str(o) for o in ops))


body = []
for frame in range(FRAMES):
    A = round(SPIN_STEP * frame, 12)
    if frame == 0:
        put(
            sp(
                "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='270'>".format(
                    FRAMES * COL_W
                )
            )
        )
    body.append("\\ frame spin {}".format(A))
    put(num(A), "C", st("ca"))
    put(num(A), "S", st("sa"))
    for i in range(RING):
        th, th2 = round(i * TH_STEP, 12), round(i * TH_STEP + TH_STEP, 12)
        put(num(th), "C", st("ct1"))
        put(num(th), "S", st("st1"))
        put(num(th2), "C", st("ct2"))
        put(num(th2), "S", st("st2"))
        for j in range(TUBE_N):
            ph, ph2 = round(j * PH_STEP, 12), round(j * PH_STEP + PH_STEP, 12)
            put(num(ph), "C", st("cp1"))
            put(num(ph), "S", st("sp1"))
            put(num(ph2), "C", st("cp2"))
            put(num(ph2), "S", st("sp2"))

            # Rotated surface normal at the quad's (phi, theta) corner pair:
            # n = (cp1*ct1, sp1, cp1*st1), spin about X, tilt about Y.
            # ny' = sp1*ca - cp1*st1*sa
            put(
                ld("sp1"),
                ld("ca"),
                "*",
                ld("cp1"),
                ld("st1"),
                "*",
                ld("sa"),
                "*",
                "-",
                st("ny"),
            )
            # nz' = sp1*sa + cp1*st1*ca
            put(
                ld("sp1"),
                ld("sa"),
                "*",
                ld("cp1"),
                ld("st1"),
                "*",
                ld("ca"),
                "*",
                "+",
                st("nz"),
            )
            # nx' = cp1*ct1*cb + nz'*sb
            put(
                ld("cp1"),
                ld("ct1"),
                "*",
                num(CB),
                "*",
                ld("nz"),
                num(SB),
                "*",
                "+",
                st("nx"),
            )
            # opacity = 0.18 + 0.30*(L.n + 1)   (L.n in [-1,1])
            put(
                ld("nx"),
                num(LIGHT[0]),
                "*",
                ld("ny"),
                num(LIGHT[1]),
                "*",
                "+",
                ld("nz"),
                num(LIGHT[2]),
                "*",
                "+",
                num(1.0),
                "+",
                num(0.30),
                "*",
                num(0.18),
                "+",
                st("u"),
            )

            put(sp("<polygon points='"))
            for cpn, ctn in (
                ("cp1", "ct1"),
                ("cp2", "ct1"),
                ("cp2", "ct2"),
                ("cp1", "ct2"),
            ):
                cpth, stnth = "sp" + cpn[2], "st" + ctn[2]
                # rad = R + tube*cos(phi); corner = (rad*cosRing, tube*sinPhi, rad*sinRing)
                put(ld(cpn), num(TUBE), "*", num(R), "+", st("rad"))
                put(ld("rad"), ld(ctn), "*", st("x"))
                put(ld(cpth), num(TUBE), "*", st("y"))
                put(ld("rad"), ld(stnth), "*", st("z"))
                # spin about X: cy = y*ca - z*sa ; cz = y*sa + z*ca
                put(ld("y"), ld("ca"), "*", ld("z"), ld("sa"), "*", "-", st("cy"))
                put(ld("y"), ld("sa"), "*", ld("z"), ld("ca"), "*", "+", st("cz"))
                # tilt about Y and project: px = ox + (x*cb + cz*sb)*scale
                ox = OX_BASE if False else 130.0 + frame * COL_W
                put(
                    ld("x"),
                    num(CB),
                    "*",
                    ld("cz"),
                    num(SB),
                    "*",
                    "+",
                    num(SCALE),
                    "*",
                    num(ox),
                    "+",
                    "|",
                )
                put(sp(","))
                put(num(-1.0), ld("cy"), num(SCALE), "*", "*", num(OY_BASE), "+", "|")
                if (cpn, ctn) != ("cp1", "ct2"):
                    put(sp(" "))
            put(sp("' fill='#6b4432' fill-opacity='"))
            put(ld("u"), "|")
            put(sp("'/>"))
    body.append("\\ end frame")
put(sp("</svg>"))

(R2 := ROOT / "examples").mkdir(exist_ok=True)
text = "\\ Generated by scripts/gen_donut.py: a polygonal torus filmstrip in SVG.\n"
text += "\n".join(lines) + "\n"
(ROOT / "examples" / "donut.si").write_text(text, encoding="utf-8")
print("wrote examples/donut.si: {} lines".format(len(text.splitlines())))
