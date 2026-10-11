//! I Tempi nella Nota (ADR-0032): `[m:ss]`, `[mm:ss]` o `[h:mm:ss]` fuori dal codice inline e dai
//! link. La stessa regola, con gli stessi casi (`src/features/notes/tempi.casi.json`), vive in
//! `src/features/notes/tempi.ts`: si guardano solo caratteri ASCII, quindi le posizioni in byte qui
//! e in UTF-16 là coincidono sul testo che si tiene. Senza Tauri.

use std::ops::Range;

/// Un Tempo nella Nota: il tratto `da..a` (byte) con le quadre e il punto dell'audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tempo {
    pub da: usize,
    pub a: usize,
    pub ms: u32,
}

/// Il tempo tra le quadre che cominciano in `i`.
fn tempo_in(b: &[u8], i: usize) -> Option<Tempo> {
    let chiusa = i + b.get(i..)?.iter().position(|&c| c == b']')?;
    if chiusa - i > 9 {
        return None;
    }
    let parti: Vec<&[u8]> = b[i + 1..chiusa].split(|&c| c == b':').collect();
    let lunghe: Vec<usize> = parti.iter().map(|p| p.len()).collect();
    if !parti
        .iter()
        .all(|p| !p.is_empty() && p.iter().all(u8::is_ascii_digit))
    {
        return None;
    }
    let numeri: Vec<u32> = parti
        .iter()
        .map(|p| p.iter().fold(0, |n, &c| n * 10 + u32::from(c - b'0')))
        .collect();
    let valide = match lunghe[..] {
        [m, 2] => m <= 2,
        [h, 2, 2] => h <= 2 && numeri[1] < 60,
        _ => false,
    };
    let (h, m, s) = match numeri[..] {
        [m, s] => (0, m, s),
        [h, m, s] => (h, m, s),
        _ => return None,
    };
    (valide && s < 60).then_some(Tempo {
        da: i,
        a: chiusa + 1,
        ms: ((h * 60 + m) * 60 + s) * 1000,
    })
}

/// La posizione dopo la chiusa che pareggia `apre` in `i`, saltando le annidate.
fn chiusura(b: &[u8], i: usize, apre: u8, chiude: u8) -> Option<usize> {
    let mut livello = 0;
    let mut j = i;
    while j < b.len() {
        if b[j] == b'\\' {
            j += 1;
        } else if b[j] == apre {
            livello += 1;
        } else if b[j] == chiude {
            livello -= 1;
            if livello == 0 {
                return Some(j + 1);
            }
        }
        j += 1;
    }
    None
}

/// Quanti `c` di fila da `i`.
fn fila(b: &[u8], i: usize, c: u8) -> usize {
    b[i..].iter().take_while(|&&x| x == c).count()
}

/// Dove finisce ciò che comincia in `i` e non contiene tempi (codice, link, indirizzi).
fn salta(b: &[u8], i: usize) -> Option<usize> {
    match b[i] {
        b'`' => {
            let n = fila(b, i, b'`');
            let mut j = i + n;
            while j < b.len() {
                if b[j] == b'`' {
                    let m = fila(b, j, b'`');
                    if m == n {
                        return Some(j + m);
                    }
                    j += m;
                } else {
                    j += 1;
                }
            }
            Some(i + n)
        }
        b'<' => {
            let fine = i + b[i..].iter().position(|&c| c == b'>')?;
            let dentro = &b[i + 1..fine];
            let senza_spazi = !dentro.iter().any(|&c| c.is_ascii_whitespace() || c == b'<');
            (senza_spazi && dentro.iter().any(|&c| c == b':' || c == b'@')).then_some(fine + 1)
        }
        _ if b[i..].starts_with(b"http://")
            || b[i..].starts_with(b"https://")
            || b[i..].starts_with(b"www.") =>
        {
            Some(
                b[i..]
                    .iter()
                    .position(u8::is_ascii_whitespace)
                    .map_or(b.len(), |p| i + p),
            )
        }
        b'[' => {
            let quadra = chiusura(b, i, b'[', b']')?;
            if b.get(quadra) == Some(&b'(') {
                chiusura(b, quadra, b'(', b')')
            } else {
                None
            }
        }
        _ => None,
    }
}

/// I Tempi nella Nota di `testo`, in ordine.
pub fn tempi(testo: &str) -> Vec<Tempo> {
    let b = testo.as_bytes();
    let mut tempi = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' {
            i += 2;
            continue;
        }
        if b[i] == b'['
            && let Some(tempo) = tempo_in(b, i)
            && !matches!(b.get(tempo.a), Some(b'(' | b'['))
        {
            tempi.push(tempo);
            i = tempo.a;
            continue;
        }
        i = salta(b, i).filter(|&dopo| dopo > i).unwrap_or(i + 1);
    }
    tempi
}

/// Il punto `ms` sulla linea del tempo dopo aver tolto i tratti `tagli`; `None` se è tolto.
pub fn sposta_ms(ms: u32, tagli: &[Range<u32>]) -> Option<u32> {
    if tagli.iter().any(|r| r.contains(&ms)) {
        return None;
    }
    Some(
        ms - tagli
            .iter()
            .map(|r| ms.clamp(r.start, r.end) - r.start)
            .sum::<u32>(),
    )
}

/// `testo` dopo un Taglio dei tratti `tagli`: i tempi dopo un tratto si spostano indietro, quelli
/// dentro perdono le quadre e tornano testo, il resto resta carattere per carattere.
pub fn sposta(testo: &str, tagli: &[Range<u32>]) -> String {
    let mut fuori = String::with_capacity(testo.len());
    let mut pos = 0;
    for Tempo { da, a, ms } in tempi(testo) {
        let dopo = sposta_ms(ms, tagli);
        if dopo == Some(ms) {
            continue;
        }
        fuori.push_str(&testo[pos..da]);
        match dopo {
            Some(dopo) => {
                fuori.push('[');
                fuori.push_str(&crate::transcript::duration(dopo));
                fuori.push(']');
            }
            None => fuori.push_str(&testo[da + 1..a - 1]),
        }
        pos = a;
    }
    fuori.push_str(&testo[pos..]);
    fuori
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct Casi {
        riconosce: Vec<Riconosce>,
        sposta: Vec<Sposta>,
    }

    #[derive(serde::Deserialize)]
    struct Riconosce {
        caso: String,
        testo: String,
        tempi: Vec<(String, u32)>,
    }

    #[derive(serde::Deserialize)]
    struct Sposta {
        caso: String,
        testo: String,
        tagli: Vec<(u32, u32)>,
        atteso: String,
    }

    /// Gli stessi casi di `tempi.test.ts`: la regola è una sola.
    fn casi() -> Casi {
        serde_json::from_str(include_str!("../../src/features/notes/tempi.casi.json")).unwrap()
    }

    #[test]
    fn riconosce_i_tempi_nella_nota_come_il_frontend() {
        for caso in casi().riconosce {
            let trovati: Vec<_> = tempi(&caso.testo)
                .into_iter()
                .map(|t| (caso.testo[t.da..t.a].to_string(), t.ms))
                .collect();
            assert_eq!(trovati, caso.tempi, "{}", caso.caso);
        }
    }

    #[test]
    fn sposta_i_tempi_nella_nota_dopo_un_taglio_come_il_frontend() {
        for caso in casi().sposta {
            let tagli: Vec<_> = caso.tagli.iter().map(|&(a, b)| a..b).collect();
            assert_eq!(sposta(&caso.testo, &tagli), caso.atteso, "{}", caso.caso);
        }
    }
}
