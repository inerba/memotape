# Prove colore delle varianti 1 (mozzo attuale) e 2 (mozzo con foro).
import re
from gen import slotted, circle, SQUARE, f

def v1(sq, pk, hb, hole=None, under=''):
    c = 172
    return sq, under + f'<path d="{circle(c,c,132)}" fill="{pk}"/><path d="{slotted(c,c,54,30,10)}" fill="{hb}"/>'

def v2(sq, pk, hb, hole=None, under=''):
    c = 172
    h = slotted(c, c, 34, 22, 7)
    body = f'<path d="{circle(c,c,132)} {h}" fill="{pk}" fill-rule="evenodd"/><path d="{circle(c,c,60)} {h}" fill="{hb}" fill-rule="evenodd"/>'
    if hole:
        body = f'<path d="{h}" fill="{hole}"/>' + body
    return sq, under + body

n = 0
def svg(sq, body, size):
    global n; n += 1
    return (f'<svg width="{size}" height="{size}" viewBox="0 0 256 256"><defs><clipPath id="q{n}"><path d="{SQUARE}"/></clipPath></defs>'
            f'<path d="{SQUARE}" fill="{sq}"/><g clip-path="url(#q{n})">{body}</g></svg>')

# nome, quadrato, bobina, mozzo, foro (None = si vede il quadrato)
P = [
    ("Salvia (attuale)",      "#576b3c", "#241e1a", "#fcfaf6", None),
    ("Arancio cassetta",      "#e0582c", "#241e1a", "#f4e7cc", None),
    ("Senape e bruno",        "#d9a227", "#3a2618", "#fcfaf6", None),
    ("Ossido di ferro",       "#8a4b2a", "#241e1a", "#e9c46a", None),
    ("Carta e salvia",        "#efe6d2", "#576b3c", "#241e1a", None),
    ("Terracotta e bosco",    "#c8705a", "#26331f", "#f6e9dc", None),
    ("Ottanio e nastro",      "#1f5c5a", "#e3b85a", "#241e1a", None),
    ("Notte e rame",          "#1f2c44", "#b8643c", "#f4e7cc", None),
    ("Lilla e prugna",        "#b9a6d6", "#3b1f2b", "#fcfaf6", None),
    ("Salvia, foro arancio",  "#576b3c", "#241e1a", "#fcfaf6", "#e0582c"),
    ("Inchiostro, foro salvia","#241e1a", "#3a312a", "#fcfaf6", "#8fa66a"),
]

def stripes(*cols):
    """Bande orizzontali da musicassetta anni '70 nella parte alta del quadrato."""
    return "".join(f'<rect x="0" y="{40 + 22*i}" width="256" height="14" fill="{c}"/>' for i, c in enumerate(cols))

A = [
    ("Arancio cassetta",        "#e0582c", "#241e1a", "#f4e7cc", None, ""),
    ("Più rosso",               "#d6412a", "#241e1a", "#f4e7cc", None, ""),
    ("Mandarino",               "#ef7d2f", "#241e1a", "#fcfaf6", None, ""),
    ("Bruciato",                "#bf4d1f", "#241e1a", "#f4e7cc", None, ""),
    ("Bobina bruna",            "#e0582c", "#4a2a1a", "#f4e7cc", None, ""),
    ("Mozzo senape",            "#e0582c", "#241e1a", "#f2c14e", None, ""),
    ("Mozzo salvia",            "#e0582c", "#241e1a", "#a9bd85", None, ""),
    ("Foro salvia",             "#e0582c", "#241e1a", "#f4e7cc", "#576b3c", ""),
    ("Foro inchiostro",         "#e0582c", "#241e1a", "#f4e7cc", "#241e1a", ""),
    ("Crema, mozzo arancio",    "#f4e7cc", "#241e1a", "#e0582c", None, ""),
    ("Inchiostro, bobina arancio", "#241e1a", "#e0582c", "#f4e7cc", None, ""),
    ("Arancio su arancio",      "#f08a3c", "#c23f1e", "#fff1dc", None, ""),
    ("Strisce anni '70",        "#e0582c", "#241e1a", "#f4e7cc", None, stripes("#f2c14e", "#f4e7cc")),
    ("Strisce rosse",           "#ef7d2f", "#241e1a", "#f4e7cc", None, stripes("#d6412a", "#bf2f22")),
]
import sys
if sys.argv[1:] == ["arancio"]:
    P, out = A, "arancio.html"
else:
    P, out = [p + ("",) for p in P], "colori.html"
cell = lambda bg, s: f'<div style="background:{bg};padding:8px;border-radius:8px;line-height:0">{s}</div>'
rows = ['<div style="font:13px system-ui;color:#555;display:grid;grid-template-columns:150px repeat(5,auto);gap:10px 14px;align-items:center;justify-items:start">',
        '<span></span><b>1 · 128</b><b>2 · 128</b><b>2 · 32</b><b>1 · 32</b><b>2 · 64 scuro</b>']
for name, sq, pk, hb, hole, under in P:
    rows.append(f'<span>{name}<br><small style="color:#999">{sq} · {pk} · {hb}{" · "+hole if hole else ""}</small></span>')
    rows.append(cell("#fcfaf6", svg(*v1(sq,pk,hb,None,under), 128)))
    rows.append(cell("#fcfaf6", svg(*v2(sq,pk,hb,hole,under), 128)))
    rows.append(cell("#fcfaf6", svg(*v2(sq,pk,hb,hole,under), 32)))
    rows.append(cell("#fcfaf6", svg(*v1(sq,pk,hb,None,under), 32)))
    rows.append(cell("#241e1a", svg(*v2(sq,pk,hb,hole,under), 64)))
rows.append('</div>')
open(out, "w", encoding="utf-8").write('<!doctype html><meta charset="utf-8"><title>Prove colore</title><body style="background:#fff;margin:16px">' + "".join(rows))

# Strisce anni '70, solo variante 2, colori acidi: nome, quadrato, strisce, bobina, mozzo, foro
ACIDI = [
    ("Base (arancio)",          "#e0582c", ("#f2c14e", "#f4e7cc"), "#241e1a", "#f4e7cc", None),
    ("Lime e fucsia",           "#c6f432", ("#ff2e93", "#241e1a"), "#241e1a", "#fcfaf6", None),
    ("Fucsia e giallo acido",   "#ff2e93", ("#e6ff2e", "#fcfaf6"), "#241e1a", "#fcfaf6", None),
    ("Arancio fluo e lime",     "#ff5a1f", ("#c6f432", "#ff2e93"), "#241e1a", "#fcfaf6", None),
    ("Giallo acido e viola",    "#e6ff2e", ("#7b2cff", "#ff2e93"), "#241e1a", "#fcfaf6", None),
    ("Ciano e magenta",         "#19d3e6", ("#ff2e93", "#e6ff2e"), "#241e1a", "#fcfaf6", None),
    ("Viola elettrico e lime",  "#6a2cff", ("#c6f432", "#ff5a1f"), "#241e1a", "#fcfaf6", None),
    ("Verde acido e arancio",   "#39e75f", ("#ff5a1f", "#f4e7cc"), "#241e1a", "#fcfaf6", None),
    ("Rosa acido e verde",      "#ff7ac8", ("#2ee86b", "#fff15a"), "#241e1a", "#fcfaf6", None),
    ("Bobina viola",            "#e6ff2e", ("#ff2e93", "#19d3e6"), "#6a2cff", "#fcfaf6", None),
    ("Bobina magenta",          "#ff5a1f", ("#e6ff2e", "#fcfaf6"), "#b0007a", "#fcfaf6", None),
    ("Mozzo lime",              "#ff5a1f", ("#e6ff2e", "#ff2e93"), "#241e1a", "#c6f432", None),
    ("Lime su inchiostro",      "#241e1a", ("#c6f432", "#ff2e93"), "#c6f432", "#fcfaf6", None),
    ("Fucsia, foro lime",       "#ff2e93", ("#e6ff2e", "#fcfaf6"), "#241e1a", "#fcfaf6", "#c6f432"),
]
SALVIA = [
    ("Salvia, strisce senape",   "#576b3c", ("#f2c14e", "#f4e7cc"), "#241e1a", "#f4e7cc", None),
    ("Salvia, strisce arancio",  "#576b3c", ("#e0582c", "#f2c14e"), "#241e1a", "#f4e7cc", None),
    ("Salvia, strisce lime",     "#576b3c", ("#c6f432", "#fcfaf6"), "#241e1a", "#fcfaf6", None),
    ("Salvia, lime e fucsia",    "#576b3c", ("#c6f432", "#ff2e93"), "#241e1a", "#fcfaf6", None),
    ("Salvia, arancio fluo",     "#576b3c", ("#ff5a1f", "#ff2e93"), "#241e1a", "#fcfaf6", None),
    ("Salvia, giallo acido",     "#576b3c", ("#e6ff2e", "#f4e7cc"), "#241e1a", "#f4e7cc", None),
    ("Salvia tono su tono",      "#576b3c", ("#a9bd85", "#d6e2bf"), "#241e1a", "#fcfaf6", None),
    ("Salvia, strisce crema",    "#576b3c", ("#f4e7cc", "#f4e7cc"), "#241e1a", "#f4e7cc", None),
    ("Salvia, foro arancio",     "#576b3c", ("#e0582c", "#f2c14e"), "#241e1a", "#f4e7cc", "#e0582c"),
    ("Salvia, foro lime",        "#576b3c", ("#c6f432", "#fcfaf6"), "#241e1a", "#fcfaf6", "#c6f432"),
    ("Salvia, mozzo lime",       "#576b3c", ("#c6f432", "#f4e7cc"), "#241e1a", "#c6f432", None),
    ("Salvia chiara",            "#8fa66a", ("#e0582c", "#f2c14e"), "#241e1a", "#f4e7cc", None),
    ("Salvia scura",             "#3f4f2a", ("#e0582c", "#f2c14e"), "#241e1a", "#f4e7cc", None),
    ("Salvia, bobina bruna",     "#576b3c", ("#e0582c", "#f2c14e"), "#4a2a1a", "#f4e7cc", None),
    ("Bobina salvia su inchiostro", "#241e1a", ("#e0582c", "#f2c14e"), "#576b3c", "#f4e7cc", None),
]
if sys.argv[1:] in (["acidi"], ["salvia"]):
    ACIDI, nome = (ACIDI, "acidi") if sys.argv[1] == "acidi" else (SALVIA, "salvia")
    rows = ['<div style="font:13px system-ui;color:#555;display:grid;grid-template-columns:170px repeat(5,auto);gap:10px 14px;align-items:center;justify-items:start">',
            '<span></span><b>128</b><b>64</b><b>32</b><b>16</b><b>64 scuro</b>']
    for name, sq, st, pk, hb, hole in ACIDI:
        rows.append(f'<span>{name}<br><small style="color:#999">{sq} · {" · ".join(st)} · {pk} · {hb}{" · "+hole if hole else ""}</small></span>')
        for size in (128, 64, 32, 16):
            rows.append(cell("#fcfaf6", svg(*v2(sq, pk, hb, hole, stripes(*st)), size)))
        rows.append(cell("#241e1a", svg(*v2(sq, pk, hb, hole, stripes(*st)), 64)))
    rows.append('</div>')
    open(f"{nome}.html", "w", encoding="utf-8").write('<!doctype html><meta charset="utf-8"><title>Strisce ' + nome + '</title><body style="background:#fff;margin:16px">' + "".join(rows))
