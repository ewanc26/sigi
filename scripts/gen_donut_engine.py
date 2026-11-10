#!/usr/bin/env python3
"""Generate examples/donut_engine.si: a live orbit-lit torus streaming the
per-frame vertex protocol consumed by host/donut_host.c, which is a faithful
embed of the canonical renderer in doughnut.c (800x600, U_STEPS=50, V_STEPS=30,
R_MAJOR=2.0, R_MINOR=0.8, fov=500, camera_dist=5.0).

Frame n (0-indexed) uses, exactly as doughnut.c does:
  angleX = 0.5, angleY = 0.5 (fixed)
  angleZ = 0.003 * n        (advanced AFTER each rendered frame)
  light_angle = 0.025 * (n + 1)  (advanced BEFORE computing)
  light_pos = (cos(la)*4.5, sin(la)*4.5, sin(la*0.7)*2.5)

Per vertex (i in 0..49, j in 0..29, emitted in i*30+j order) the program
prints px, py, z, intensity, each with sigi's numeric print (`|`, "%g\\n"), on
its own line:

  rot_pos, rot_norm = rotate_vector(pos/norm, angleX, angleY, angleZ)  (X, then Y, then Z)
  light_dir = light_pos - rot_pos, normalized
  dist = |light_dir|; intensity = max(dot(rot_norm, light_dir), 0.08) / (1 + 0.04*dist)
  px = (rot.x / (rot.z + 5)) * 500 + 400 ; py = (rot.y / (rot.z + 5)) * 500 + 300
  z  = rot.z + 5

800x600 rasterization, Lambert shading and the purple-pink palette live in the
host, matching doughnut.c exactly.

sigi's while form peeks the stack top before each iteration, so loops follow
one discipline: the body drops the incoming condition with `$` first and
pushes the next condition as its final act. The frame loop never ends on its
own; run the program through the host (or Ctrl-C) to stop.
"""

import math
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent

U_STEPS, V_STEPS = 50, 30
CAM_DIST = 5.0
FOV = 500.0
ANGLE_XY = 0.5
CAXY = round(math.cos(ANGLE_XY), 12)
SAXY = round(math.sin(ANGLE_XY), 12)
U_STEP = round(2 * math.pi / U_STEPS, 12)
V_STEP = round(2 * math.pi / V_STEPS, 12)
NL = r"!10 ^"

P = []


def put(*ops):
    P.append(" ".join(str(o) for o in ops))


def num(v):
    return "!{}".format(repr(round(v, 12)))


def ld(n):
    return "<{}>".format(n)


def st(n):
    return "!{} :".format(n)


def comment(text):
    P.append("\\ " + text)


def emit_str(s):
    put('"{}"'.format(s), NL)


# Slots:
#  1 az (angleZ), 2 la (light_angle), 3 caz, 4 saz,
#  5 lx, 6 ly, 7 lz (light position),
#  8 cax=cay, 9 sax=say (fixed 0.5 tilts),
# 12 i, 13 cu, 14 su, 15 j, 16 cv, 17 sv,
# 18 rad, 19 bx, 20 by, 21 bz, 22 nx, 23 ny,
# 24 py1 25 pz1 26 px2 27 pz2 (position rotation temps),
# 28 rx, 29 ry (rot.z == slot 27),
# 30 ny1 31 nz1 32 nx2 33 nz2 (normal rotation temps),
# 34 rnx, 35 rny (rot normal z == slot 33),
# 36 dx, 37 dy, 38 dz, 39 dist, 40 w (z+5), 41 intensity.


def vertex():
    # Torus position/normal, doughnut.c generate_torus().
    put("!2.0", ld(16), "!0.8", "*", "+", st(18))  # rad = R + r*cos(v)
    put(ld(18), ld(13), "*", st(19))  # bx = rad*cos(u)
    put(ld(18), ld(14), "*", st(20))  # by = rad*sin(u)
    put("!0.8", ld(17), "*", st(21))  # bz = r*sin(v)
    put(ld(16), ld(13), "*", st(22))  # nx = cos(v)*cos(u)
    put(ld(16), ld(14), "*", st(23))  # ny = cos(v)*sin(u)
    # nz = sin(v) == slot 17

    # rotate_vector(pos, ax, ay, az): X, then Y, then Z.
    put(ld(20), ld(8), "*", ld(21), ld(9), "*", "-", st(24))  # y1
    put(ld(20), ld(9), "*", ld(21), ld(8), "*", "+", st(25))  # z1
    put(ld(19), ld(8), "*", ld(25), ld(9), "*", "+", st(26))  # x2
    put(ld(25), ld(8), "*", ld(19), ld(9), "*", "-", st(27))  # z2 (rot z)
    put(ld(26), ld(3), "*", ld(24), ld(4), "*", "-", st(28))  # rx
    put(ld(26), ld(4), "*", ld(24), ld(3), "*", "+", st(29))  # ry

    # rotate_vector(normal, ...) with the same fixed tilt and spin.
    put(ld(23), ld(8), "*", ld(17), ld(9), "*", "-", st(30))  # y1
    put(ld(23), ld(9), "*", ld(17), ld(8), "*", "+", st(31))  # z1
    put(ld(22), ld(8), "*", ld(31), ld(9), "*", "+", st(32))  # x2
    put(ld(31), ld(8), "*", ld(22), ld(9), "*", "-", st(33))  # z2 (rot nz)
    put(ld(32), ld(3), "*", ld(30), ld(4), "*", "-", st(34))  # rnx
    put(ld(32), ld(4), "*", ld(30), ld(3), "*", "+", st(35))  # rny

    # Orbiting point light, normalized direction, distance.
    put(ld(5), ld(28), "-", st(36))  # dx
    put(ld(6), ld(29), "-", st(37))  # dy
    put(ld(7), ld(27), "-", st(38))  # dz
    put(
        ld(36),
        ld(36),
        "*",
        ld(37),
        ld(37),
        "*",
        "+",
        ld(38),
        ld(38),
        "*",
        "+",
        "R",
        st(39),
    )
    put(ld(36), ld(39), "/", st(36))
    put(ld(37), ld(39), "/", st(37))
    put(ld(38), ld(39), "/", st(38))

    # Lambert + 0.08 ambient floor + 1/(1+0.04*dist) attenuation.
    put(ld(34), ld(36), "*", ld(35), ld(37), "*", "+", ld(33), ld(38), "*", "+", st(41))
    put(
        ld(41),
        "!0.08",
        "<",
        "{",
        "!0.08",
        ";",
        ld(41),
        "}",
        "!1.0",
        ld(39),
        "!0.04",
        "*",
        "+",
        "/",
        st(41),
    )

    # Perspective projection with camera_dist = 5, fov = 500, centered 400/300.
    put(ld(27), num(CAM_DIST), "+", st(40))  # w = rot.z + 5
    put(ld(28), ld(40), "/", num(FOV), "*", "!400.0", "+", "|")
    put(ld(29), ld(40), "/", num(FOV), "*", "!300.0", "+", "|")
    put(ld(40), "|")
    put(ld(41), "|")


comment("A live orbit-lit torus streaming per-frame vertices for donut_host (SDL2),")
comment("a faithful port of doughnut.c. Generated by scripts/gen_donut_engine.py;")
comment("run:")
comment("  ./build/sigic examples/donut_engine.si --run --cc cc | ./host/donut_host")
put("!0.0", st(1))  # az = 0
put("!0.0", st(2))  # la = 0
put(num(CAXY), st(8))  # cos(0.5): both tilt planes share it
put(num(SAXY), st(9))  # sin(0.5)

put("!1")
P.append("[")
put("$")  # drop incoming condition
put(ld(2), "!0.025", "+", st(2))  # light advanced BEFORE computing
put(ld(2), "C", "!4.5", "*", st(5))  # lx = cos(la)*4.5
put(ld(2), "S", "!4.5", "*", st(6))  # ly = sin(la)*4.5
put(ld(2), "!0.7", "*", "S", "!2.5", "*", st(7))  # lz = sin(0.7*la)*2.5
put(ld(1), "C", st(3))  # caz
put(ld(1), "S", st(4))  # saz
emit_str("F")

put("!0.0", st(12))  # i = 0
put("!1")
P.append("[")
put("$")
put(ld(12), num(U_STEP), "*", "C", st(13))  # cu = cos(u)
put(ld(12), num(U_STEP), "*", "S", st(14))  # su = sin(u)
put("!0.0", st(15))  # j = 0
put("!1")
P.append("[")
put("$")
put(ld(15), num(V_STEP), "*", "C", st(16))  # cv = cos(v)
put(ld(15), num(V_STEP), "*", "S", st(17))  # sv = sin(v)
vertex()
put(ld(15), "!1.0", "+", st(15))
put(ld(15), num(V_STEPS), "<")
P.append("]")
put("$")
put(ld(12), "!1.0", "+", st(12))
put(ld(12), num(U_STEPS), "<")
P.append("]")
put("$")
emit_str("S")
put(ld(1), "!0.003", "+", st(1))  # angleZ advanced AFTER the rendered frame
put("!1")
P.append("]")

text = "\n".join(P) + "\n"
(ROOT / "examples" / "donut_engine.si").write_text(text, encoding="utf-8")
print("wrote examples/donut_engine.si: {} lines".format(len(text.splitlines())))
