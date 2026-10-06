# Confronto: simbolo attuale contro "Salvia, strisce senape" (variante 2).
import re
from colori import v2, stripes, SQUARE

cur = re.sub(r"<title>.*?</title>", "", open("../../docs/brand/svg/memotape-simbolo.svg", encoding="utf-8").read())
sq, body = v2("#576b3c", "#241e1a", "#f4e7cc", None, stripes("#f2c14e", "#f4e7cc"))
new = (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><defs><clipPath id="q"><path d="{SQUARE}"/></clipPath></defs>'
       f'<path d="{SQUARE}" fill="{sq}"/><g clip-path="url(#q)">{body}</g></svg>')
open("memotape-mozzo-strisce-senape.svg", "w", encoding="utf-8").write(new)

n = 0
def at(s, size):
    global n; n += 1
    s = s.replace('id="q"', f'id="c{n}"').replace("url(#q)", f"url(#c{n})")
    return re.sub(r"<svg ", f'<svg width="{size}" height="{size}" ', s, count=1)

def column(title, s):
    sizes = "".join(f'<div style="display:flex;flex-direction:column;align-items:center;gap:4px">{at(s, z)}<small>{z}</small></div>' for z in (256, 128, 64, 32, 16))
    side = lambda bg, fg: (f'<div style="background:{bg};color:{fg};border-radius:10px;padding:14px 16px;display:flex;align-items:center;gap:10px;width:220px">'
                           f'{at(s, 28)}<span style="font:600 19px Commissioner;font-variation-settings:\'FLAR\' 100,\'VOLM\' 50">Memotape</span></div>')
    bar = "".join(f'<div style="background:#202020;padding:8px;border-radius:6px">{at(s, 24)}</div>' for _ in [0])
    return (f'<section style="flex:1;min-width:320px"><h2 style="font:600 16px system-ui;margin:0 0 12px">{title}</h2>'
            f'<div style="display:flex;align-items:flex-end;gap:14px;flex-wrap:wrap;margin-bottom:18px">{sizes}</div>'
            f'<div style="display:flex;flex-direction:column;gap:10px;margin-bottom:18px">{side("#fcfaf6", "#241e1a")}{side("#2b2622", "#f4e7cc")}</div>'
            f'<div style="display:flex;gap:10px;align-items:center"><small>barra di Windows</small>{bar}'
            f'<div style="background:#576b3c;padding:8px;border-radius:6px">{at(s, 48)}</div><small>su fondo salvia</small></div></section>')

html = ('<!doctype html><meta charset="utf-8"><title>Confronto logo</title>'
        '<link href="https://fonts.googleapis.com/css2?family=Commissioner:FLAR,VOLM,wght@0..100,0..100,100..900&display=swap" rel="stylesheet">'
        '<body style="background:#fff;margin:20px;font:12px system-ui;color:#555">'
        f'<div style="display:flex;gap:40px;flex-wrap:wrap">{column("Attuale", cur)}{column("Mozzo con strisce senape", new)}</div>')
open("confronto.html", "w", encoding="utf-8").write(html)
