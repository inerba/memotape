# Concept "mozzo zoomato": una bobina vista da vicino, in basso a destra nel quadrato salvia.
from math import cos, sin, radians, sqrt
SAGE, INK, PAPER = "#576b3c", "#241e1a", "#fcfaf6"
SQUARE = "M52,16H204A36,36 0 0 1 240,52V204A36,36 0 0 1 204,240H52A36,36 0 0 1 16,204V52A36,36 0 0 1 52,16Z"

def f(v): return f"{v:.2f}".rstrip("0").rstrip(".")

def slotted(cx, cy, R, r, w, n=6, rot=-90):
    """Disco di raggio R con n tacche larghe 2w fino al raggio r (il mozzo dell'attuale simbolo)."""
    d = ""
    for k in range(n):
        t = radians(rot + 360 * k / n)
        u, p = (cos(t), sin(t)), (-sin(t), cos(t))
        pt = lambda rad, s: (cx + p[0]*s*w + u[0]*sqrt(rad*rad-w*w), cy + p[1]*s*w + u[1]*sqrt(rad*rad-w*w))
        o1, i1, i2, o2 = pt(R, -1), pt(r, -1), pt(r, 1), pt(R, 1)
        d += ("M" if k == 0 else "A%s,%s 0 0 1 " % (f(R), f(R))) + f"{f(o1[0])},{f(o1[1])}"
        d += f"L{f(i1[0])},{f(i1[1])}A{f(r)},{f(r)} 0 0 1 {f(i2[0])},{f(i2[1])}L{f(o2[0])},{f(o2[1])}"
    t = radians(rot)
    u, p = (cos(t), sin(t)), (-sin(t), cos(t))
    s = sqrt(R*R-w*w)
    return d + f"A{f(R)},{f(R)} 0 0 1 {f(cx - p[0]*w + u[0]*s)},{f(cy - p[1]*w + u[1]*s)}Z"

def circle(cx, cy, R):
    return f"M{f(cx-R)},{f(cy)}A{f(R)},{f(R)} 0 1 1 {f(cx+R)},{f(cy)}A{f(R)},{f(R)} 0 1 1 {f(cx-R)},{f(cy)}Z"

def svg(body):
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">'
            f'<defs><clipPath id="q"><path d="{SQUARE}"/></clipPath></defs>'
            f'<path d="{SQUARE}" fill="{SAGE}"/><g clip-path="url(#q)">{body}</g></svg>')

def hub_with_hole(cx, cy, R, rh, rt, w):
    """Anello carta con il foro centrale a 6 denti verso l'interno: dal foro si vede la salvia."""
    hole = slotted(cx, cy, rh, rt, w)
    return f'<path d="{circle(cx, cy, R)} {hole}" fill="{PAPER}" fill-rule="evenodd"/>', hole

def pack(cx, cy, R, hole=None):
    d = circle(cx, cy, R) + (" " + hole if hole else "")
    return f'<path d="{d}" fill="{INK}" fill-rule="evenodd"/>'

V = {}
# 1. Il mozzo dell'attuale simbolo, ingrandito
c = 172
V["1-classico"] = svg(pack(c, c, 132) + f'<path d="{slotted(c, c, 54, 30, 10)}" fill="{PAPER}"/>')
# 2. Mozzo vero: anello e foro dentato da cui passa la salvia
h, hole = hub_with_hole(c, c, 60, 34, 22, 7)
V["2-foro"] = svg(pack(c, c, 132, hole) + h)
if __name__ == "__main__":
  for k, s in V.items():
      open(f"memotape-mozzo-{k}.svg", "w").write(s)
