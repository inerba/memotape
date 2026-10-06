"""Genera il kit del logo di Memotape: le SVG in `svg/`, i PNG in `png/`, `memotape-kit.png` e le
icone dell'app in `src-tauri/icons/` (vedi LINEE-GUIDA.md, "Rigenerare il kit").

    python docs/brand/genera.py --font <Commissioner[FLAR,VOLM,slnt,wght].ttf>

Servono `fonttools` e `uharfbuzz` (pip) per il logotipo e `bunx` per `@resvg/resvg-js-cli` e
`bun tauri icon`. Senza `--icons` le icone dell'app non si toccano.
"""

import argparse
import shutil
import struct
import subprocess
import tempfile
from io import BytesIO
from math import cos, radians, sin, sqrt
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]

SAGE, INK, PAPER, MUSTARD = "#576b3c", "#241e1a", "#fcfaf6", "#f2c14e"
# La bobina sui fondi scuri: un bruno che si stacca dall'inchiostro.
BROWN = "#45392f"
BLACK, WHITE = "#000000", "#ffffff"

# Le misure stanno su una griglia da 8 nel quadrato da 256 (4 dove serve un mezzo modulo).
CUTS = {
    # Da 48 px in su: due strisce, il mozzo con il foro a 6 denti.
    "grande": dict(square=(16, 36), stripes=((40, 16, MUSTARD), (64, 16, PAPER)),
                   center=176, pack=136, hub=64, hole=(36, 24, 8)),
    # Da 24 a 32 px: quadrato a tutta tela, una striscia più spessa, mozzo più grande.
    "piccolo": dict(square=(0, 44), stripes=((32, 32, MUSTARD),),
                    center=176, pack=144, hub=80, hole=(44, 28, 12)),
    # 16 px: niente strisce, foro tondo.
    "minimo": dict(square=(0, 44), stripes=(), center=168, pack=160, hub=96, hole=(40, None, None)),
}


def f(v):
    return f"{v:.2f}".rstrip("0").rstrip(".")


def rounded_square(inset, r):
    a, b = inset, 256 - inset
    return (f"M{a + r},{a}H{b - r}A{r},{r} 0 0 1 {b},{a + r}V{b - r}A{r},{r} 0 0 1 {b - r},{b}"
            f"H{a + r}A{r},{r} 0 0 1 {a},{b - r}V{a + r}A{r},{r} 0 0 1 {a + r},{a}Z")


def circle(cx, cy, r):
    return f"M{f(cx - r)},{f(cy)}A{f(r)},{f(r)} 0 1 1 {f(cx + r)},{f(cy)}A{f(r)},{f(r)} 0 1 1 {f(cx - r)},{f(cy)}Z"


def toothed_hole(cx, cy, r, tip, w, n=6):
    """Il foro del mozzo: un cerchio di raggio r con n denti larghi 2w che arrivano al raggio tip."""
    if tip is None:
        return circle(cx, cy, r)

    def at(t, rad, s):
        u, p = (cos(t), sin(t)), (-sin(t), cos(t))
        h = sqrt(rad * rad - w * w)
        return f"{f(cx + p[0] * s * w + u[0] * h)},{f(cy + p[1] * s * w + u[1] * h)}"

    d = ""
    for k in range(n):
        t = radians(-90 + 360 * k / n)
        d += (f"M{at(t, r, -1)}" if k == 0 else f"A{f(r)},{f(r)} 0 0 1 {at(t, r, -1)}")
        d += f"L{at(t, tip, -1)}A{f(tip)},{f(tip)} 0 0 1 {at(t, tip, 1)}L{at(t, r, 1)}"
    return d + f"A{f(r)},{f(r)} 0 0 1 {at(radians(-90), r, -1)}Z"


def parts(cut):
    c = CUTS[cut]
    inset, r = c["square"]
    cx = c["center"]
    r_hole, tip, w = c["hole"]
    return dict(square=rounded_square(inset, r), stripes=c["stripes"],
                pack=circle(cx, cx, c["pack"]), hub=circle(cx, cx, c["hub"]),
                hole=toothed_hole(cx, cx, r_hole, tip, w))


def stripe_rects(stripes, fill=None):
    return "".join(f'<rect y="{y}" width="256" height="{h}" fill="{fill or color}"/>' for y, h, color in stripes)


def symbol(cut="grande", pack=INK, uid="m"):
    """Il simbolo a colori, come gruppo da mettere in una SVG da 256."""
    p = parts(cut)
    return (f'<defs><clipPath id="{uid}"><path d="{p["square"]}"/></clipPath></defs>'
            f'<path d="{p["square"]}" fill="{SAGE}"/><g clip-path="url(#{uid})">{stripe_rects(p["stripes"])}'
            f'<path d="{p["pack"]}" fill="{pack}"/><path d="{p["hub"]}" fill="{PAPER}"/>'
            f'<path d="{p["hole"]}" fill="{SAGE}"/></g>')


def symbol_mono(color, uid="m"):
    """A un colore: il quadrato pieno con strisce, bobina e foro ritagliati; il mozzo resta pieno."""
    p = parts("grande")
    return (f'<defs><mask id="{uid}" maskUnits="userSpaceOnUse" x="0" y="0" width="256" height="256">'
            f'<path d="{p["square"]}" fill="#fff"/>{stripe_rects(p["stripes"], "#000")}'
            f'<path d="{p["pack"]}" fill="#000"/><path d="{p["hub"]} {p["hole"]}" fill="#fff" fill-rule="evenodd"/>'
            f'</mask></defs><path d="{p["square"]}" fill="{color}" mask="url(#{uid})"/>')


def hub_alone(color):
    """Il solo mozzo, centrato in una SVG da 128."""
    c = CUTS["grande"]
    s = 64 / c["hub"]
    r_hole, tip, w = c["hole"]
    return (f'<path d="{circle(64, 64, 64)} {toothed_hole(64, 64, r_hole * s, tip * s, w * s)}" '
            f'fill="{color}" fill-rule="evenodd"/>')


def svg(view, body):
    return f'<svg viewBox="{view}" xmlns="http://www.w3.org/2000/svg"><title>Memotape</title>{body}</svg>\n'


class Wordmark:
    """"memotape" in Commissioner, come lo scrive l'app: 500, FLAR 100, VOLM 50, −0,01 em."""

    def __init__(self, font_path):
        import uharfbuzz as hb
        from fontTools.pens.svgPathPen import SVGPathPen
        from fontTools.pens.transformPen import TransformPen
        from fontTools.ttLib import TTFont
        from fontTools.varLib.instancer import instantiateVariableFont

        axes = {"wght": 500, "FLAR": 100, "VOLM": 50, "slnt": 0}
        font = instantiateVariableFont(TTFont(font_path), axes)
        buf = BytesIO()
        font.save(buf)
        upem = self.upem = font["head"].unitsPerEm
        self.x_height = font["OS/2"].sxHeight / upem
        hb_font = hb.Font(hb.Face(buf.getvalue()))
        text = hb.Buffer()
        text.add_str("memotape")
        text.guess_segment_properties()
        hb.shape(hb_font, text, {"kern": True, "liga": True})
        glyphs = font.getGlyphSet()
        names = font.getGlyphOrder()
        self.paths, x = [], 0.0
        tracking = -0.01 * upem
        for info, pos in zip(text.glyph_infos, text.glyph_positions):
            pen = SVGPathPen(glyphs, ntos=lambda v: f"{v:.1f}".rstrip("0").rstrip("."))
            # In unità del font, capovolto (y verso il basso); la scala a 1 em = 1 sta in `at`.
            glyphs[names[info.codepoint]].draw(TransformPen(pen, (1, 0, 0, -1, x + pos.x_offset, 0)))
            self.paths.append(pen.getCommands())
            x += pos.x_advance + tracking
        self.width = (x - tracking) / upem
        bounds = [
            glyphs[names[i.codepoint]] for i in text.glyph_infos
        ]
        from fontTools.pens.boundsPen import BoundsPen
        lo, hi = 0, 0
        for g in bounds:
            bp = BoundsPen(glyphs)
            g.draw(bp)
            if bp.bounds:
                lo, hi = min(lo, bp.bounds[1]), max(hi, bp.bounds[3])
        self.descent, self.ascent = -lo / upem, hi / upem

    def at(self, x, baseline, size, color):
        return (f'<g transform="translate({f(x)} {f(baseline)}) scale({size / self.upem:.6g})" fill="{color}">'
                + "".join(f'<path d="{d}"/>' for d in self.paths) + "</g>")


# Il logotipo nei lockup: corpo 128 su un simbolo da 256, la fascia della x centrata sul simbolo.
SIZE = 128
GAP = 32


def horizontal_parts(word, mark, color):
    """Il lockup orizzontale: (riquadro, contenuto)."""
    baseline = 128 + word.x_height * SIZE / 2
    return (0, 0, 240 + GAP + word.width * SIZE + 16, 256), mark + word.at(240 + GAP, baseline, SIZE, color)


def vertical_parts(word, mark, color):
    """Il lockup verticale: il logotipo a corpo 96 centrato sotto il simbolo."""
    size = 96
    baseline = 256 + 24 + word.ascent * size
    width = word.width * size
    left = min(0, (256 - width) / 2)
    box = (left, 0, max(256, width), baseline + word.descent * size + 16)
    return box, mark + word.at((256 - width) / 2, baseline, size, color)


def as_svg(parts):
    box, body = parts
    return svg(" ".join(f(v) for v in box), body)


def horizontal(word, mark, color):
    return as_svg(horizontal_parts(word, mark, color))


def vertical(word, mark, color):
    return as_svg(vertical_parts(word, mark, color))


def logotype(word, color):
    size = 128
    pad = 16
    h = (word.ascent + word.descent) * size
    return svg(f"0 0 {f(word.width * size + 2 * pad)} {f(h + 2 * pad)}",
               word.at(pad, pad + word.ascent * size, size, color))


def write(name, content):
    """Scrive la SVG come la formatterebbe il pre-commit, perché Biome controlla anche `docs/brand/svg`.
    Passa da stdin: su Windows la scrittura di Biome fallisce sui file che un altro processo tiene
    mappati (os error 1224)."""
    path = HERE / "svg" / f"{name}.svg"
    biome = [shutil.which("bunx"), "biome", "check", "--write", f"--stdin-file-path={path.relative_to(ROOT)}"]
    done = subprocess.run(biome, input=content, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", check=True)
    path.write_text(done.stdout, encoding="utf-8", newline="\n")


def build_svgs(font):
    out = HERE / "svg"
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir()
    for cut, suffix in (("grande", ""), ("piccolo", "-piccolo"), ("minimo", "-minimo")):
        write(f"memotape-simbolo{suffix}", svg("0 0 256 256", symbol(cut)))
        write(f"memotape-simbolo{suffix}-negativo", svg("0 0 256 256", symbol(cut, BROWN)))
    write("memotape-simbolo-nero", svg("0 0 256 256", symbol_mono(BLACK)))
    write("memotape-simbolo-bianco", svg("0 0 256 256", symbol_mono(WHITE)))
    for suffix, color in (("", INK), ("-nero", BLACK), ("-bianco", WHITE), ("-salvia", SAGE)):
        write(f"memotape-mozzo{suffix}", svg("0 0 128 128", hub_alone(color)))
    word = Wordmark(font)
    write("memotape-logotipo", logotype(word, INK))
    write("memotape-logotipo-negativo", logotype(word, PAPER))
    write("memotape-orizzontale", horizontal(word, symbol(), INK))
    write("memotape-orizzontale-negativo", horizontal(word, symbol(pack=BROWN), PAPER))
    write("memotape-orizzontale-nero", horizontal(word, symbol_mono(BLACK), BLACK))
    write("memotape-orizzontale-bianco", horizontal(word, symbol_mono(WHITE), WHITE))
    write("memotape-verticale", vertical(word, symbol(), INK))
    write("memotape-verticale-negativo", vertical(word, symbol(pack=BROWN), PAPER))
    return word


def render(src, dst, width, background=None):
    args = [shutil.which("bunx"), "@resvg/resvg-js-cli", "--no-system-font", "--fit-width", str(width)]
    if background:
        args += ["--background", background]
    subprocess.run(args + [str(src), str(dst)], check=True, capture_output=True)


def build_pngs():
    out = HERE / "png"
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir()
    s = HERE / "svg"
    render(s / "memotape-simbolo.svg", out / "memotape-simbolo-1024.png", 1024)
    render(s / "memotape-simbolo-nero.svg", out / "memotape-simbolo-nero.png", 512)
    render(s / "memotape-mozzo.svg", out / "memotape-mozzo.png", 512)
    render(s / "memotape-logotipo.svg", out / "memotape-logotipo.png", 1200)
    render(s / "memotape-orizzontale.svg", out / "memotape-orizzontale.png", 1600)
    render(s / "memotape-orizzontale-negativo.svg", out / "memotape-orizzontale-negativo.png", 1600, INK)
    render(s / "memotape-verticale.svg", out / "memotape-verticale.png", 800)


def place(parts, x, y, scale):
    (left, top, _, _), body = parts
    return f'<g transform="translate({f(x)} {f(y)}) scale({f(scale)}) translate({f(-left)} {f(-top)})">{body}</g>'


def build_kit(word):
    """La tavola riassuntiva: lockup su carta e su inchiostro, versioni e tagli piccoli."""
    vert = vertical_parts(word, symbol(uid="k2"), INK)
    light = (f'<rect width="1400" height="530" fill="{PAPER}"/>'
             + place(horizontal_parts(word, symbol(uid="k1"), INK), 40, 40, 0.84)
             + place(vert, 1400 - 40 - vert[0][2] * 0.66, 32, 0.66))
    row = [symbol(uid="k3"), symbol_mono(BLACK, "k4")]
    for i, body in enumerate(row):
        light += place(((0, 0, 256, 256), body), 40 + i * 160, 330, 0.5)
    for i, color in enumerate((INK, SAGE)):
        light += place(((0, 0, 128, 128), hub_alone(color)), 380 + i * 150, 338, 0.88)
    for i, (cut, px) in enumerate((("minimo", 16), ("piccolo", 24), ("piccolo", 32), ("grande", 48))):
        light += place(((0, 0, 256, 256), symbol(cut, uid=f"k{5 + i}")), 720 + i * 70, 394 - px / 2, px / 256)
    dark = (f'<rect y="530" width="1400" height="330" fill="{INK}"/>'
            + place(horizontal_parts(word, symbol(pack=BROWN, uid="k9"), PAPER), 40, 602, 0.72)
            + place(horizontal_parts(word, symbol_mono(WHITE, "k10"), WHITE), 720, 602, 0.72))
    with tempfile.TemporaryDirectory() as tmp:
        board = Path(tmp) / "kit.svg"
        board.write_text(svg("0 0 1400 860", light + dark), encoding="utf-8")
        render(board, HERE / "memotape-kit.png", 1400)


def png_bytes(svg_path, px):
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "i.png"
        render(svg_path, out, px)
        return out.read_bytes()


def build_icons():
    """Le icone dell'app: `bun tauri icon` dal PNG a 1024, poi `icon.ico`, `32x32.png` e i loghi
    sotto i 48 px rifatti con i tagli piccoli."""
    icons = ROOT / "src-tauri" / "icons"
    subprocess.run([shutil.which("bun"), "tauri", "icon", str(HERE / "png" / "memotape-simbolo-1024.png")], cwd=ROOT, check=True)
    # Solo Windows: i file per Android, iOS e il 64x64 non servono al bundle.
    for extra in ("android", "ios"):
        shutil.rmtree(icons / extra, ignore_errors=True)
    (icons / "64x64.png").unlink(missing_ok=True)
    s = HERE / "svg"

    def cut_for(px):
        if px <= 16:
            return s / "memotape-simbolo-minimo.svg"
        if px < 48:
            return s / "memotape-simbolo-piccolo.svg"
        return s / "memotape-simbolo.svg"

    sizes = (16, 20, 24, 32, 40, 48, 64, 128, 256)
    images = [png_bytes(cut_for(px), px) for px in sizes]
    header = struct.pack("<HHH", 0, 1, len(sizes))
    offset = 6 + 16 * len(sizes)
    entries = b""
    for px, data in zip(sizes, images):
        entries += struct.pack("<BBBBHHII", px % 256, px % 256, 0, 0, 1, 32, len(data), offset)
        offset += len(data)
    (icons / "icon.ico").write_bytes(header + entries + b"".join(images))
    (icons / "32x32.png").write_bytes(png_bytes(cut_for(32), 32))
    for name, px in (("Square30x30Logo.png", 30), ("Square44x44Logo.png", 44)):
        (icons / name).write_bytes(png_bytes(cut_for(px), px))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--font", required=True, help="il TTF variabile di Commissioner")
    parser.add_argument("--icons", action="store_true", help="rigenera anche src-tauri/icons")
    a = parser.parse_args()
    w = build_svgs(a.font)
    build_pngs()
    build_kit(w)
    if a.icons:
        build_icons()
