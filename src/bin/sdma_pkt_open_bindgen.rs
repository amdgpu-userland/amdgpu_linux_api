/// Reads an AMD SDMA packet C header file from stdin or a path argument,
/// and emits a macro-DSL packet description to stdout.
///
/// Build:   rustc sdma_pkt_open_bindgen.rs -o sdma_bindgen
/// Run:     ./sdma_bindgen navi10_sdma_pkt_open.h
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, Read};

// ── Step 1: collect declared packet names from section-header comments ────────
//   /* ** Definitions for SDMA_PKT_COPY_LINEAR packet */
fn collect_packet_names(src: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if !line.contains("Definitions for SDMA_") {
            continue;
        }
        if let Some(pos) = line.find("Definitions for SDMA_") {
            let rest = &line[pos + "Definitions for SDMA_".len()..];
            let name_part = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('*')
                .trim_end_matches('/');
            let key = if let Some(r) = name_part.strip_prefix("PKT_") {
                r.to_string()
            } else if let Some(r) = name_part.strip_prefix("AQL_PKT_") {
                format!("AQL_{}", r)
            } else {
                continue;
            };
            if !key.is_empty() && !names.contains(&key) {
                names.push(key);
            }
        }
    }
    names
}

// ── Step 2: parse _offset / _mask / _shift triples ───────────────────────────

fn parse_u32_hex_or_dec(s: &str) -> Option<u32> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(h, 16).ok()
    } else {
        s.parse().ok()
    }
}

#[derive(Debug)]
struct RawField {
    pkt_name: String,
    word_name: String,  // e.g. "HEADER", "DW_3", "DATA0"
    field_name: String, // e.g. "encrypt", "count", "data0"
    dw: u32,
    mask: u32,
    shift: u32,
}

fn parse_raw_fields(src: &str, pkt_names: &[String]) -> Vec<RawField> {
    let mut sorted: Vec<&String> = pkt_names.iter().collect();
    sorted.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut triples: HashMap<String, (Option<u32>, Option<u32>, Option<u32>)> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for line in src.lines() {
        let line = line.trim();
        if !line.starts_with("#define ") {
            continue;
        }
        let rest = &line[8..];
        let mut parts = rest.splitn(2, char::is_whitespace);
        let name = match parts.next() {
            Some(n) => n.trim(),
            None => continue,
        };
        let value_str = match parts.next() {
            Some(v) => v.trim(),
            None => continue,
        };

        let (kind, base) = if let Some(b) = name.strip_suffix("_offset") {
            ("offset", b)
        } else if let Some(b) = name.strip_suffix("_mask") {
            ("mask", b)
        } else if let Some(b) = name.strip_suffix("_shift") {
            ("shift", b)
        } else {
            continue;
        };

        let e = triples
            .entry(base.to_string())
            .or_insert((None, None, None));
        if !order.contains(&base.to_string()) {
            order.push(base.to_string());
        }
        let val = parse_u32_hex_or_dec(value_str);
        match kind {
            "offset" => e.0 = val,
            "mask" => e.1 = val,
            "shift" => e.2 = val,
            _ => {}
        }
    }

    let mut fields: Vec<RawField> = Vec::new();

    'outer: for base in &order {
        let (offset, mask, shift) = match triples[base] {
            (Some(o), Some(m), Some(s)) => (o, m, s),
            _ => continue,
        };

        let body = match base.strip_prefix("SDMA_") {
            Some(b) => b,
            None => continue,
        };

        // Match packet name (longest-prefix).
        let (pkt_key, after_pkt) = if let Some(r) = body.strip_prefix("AQL_PKT_") {
            let mut found = None;
            for n in &sorted {
                if let Some(aql_body) = n.strip_prefix("AQL_") {
                    if r.starts_with(aql_body) {
                        let after = &r[aql_body.len()..];
                        if after.is_empty() || after.starts_with('_') {
                            found = Some((*n, after));
                            break;
                        }
                    }
                }
            }
            match found {
                Some(p) => p,
                None => continue 'outer,
            }
        } else if let Some(r) = body.strip_prefix("PKT_") {
            let mut found = None;
            for n in &sorted {
                if n.starts_with("AQL_") {
                    continue;
                }
                if r.starts_with(n.as_str()) {
                    let after = &r[n.len()..];
                    if after.is_empty() || after.starts_with('_') {
                        found = Some((*n, after));
                        break;
                    }
                }
            }
            match found {
                Some(p) => p,
                None => continue 'outer,
            }
        } else {
            continue;
        };

        // after_pkt is like "_HEADER_encrypt" or "_DW_3_count"
        let rest = after_pkt.trim_start_matches('_');
        if rest.is_empty() {
            continue;
        }

        // Split into tokens.  The field name is the trailing all-lowercase
        // (or digit) tokens; the word name is everything before that.
        // However, word names like "DW_3" or "DATA0" contain digits too.
        // Strategy: scan from right while token is all-lowercase OR
        // is a digit-only run that follows a non-digit token (e.g. "31" in
        // "src_addr_31_0").  In practice: the rightmost contiguous group of
        // tokens that are all lowercase ascii (no uppercase) is the field.
        let tokens: Vec<&str> = rest.split('_').collect();
        let field_start = {
            let mut idx = tokens.len();
            while idx > 1 {
                let t = tokens[idx - 1];
                // A token is "field-like" if it contains no uppercase letters.
                if t.chars().all(|c| c.is_lowercase() || c.is_ascii_digit()) {
                    idx -= 1;
                } else {
                    break;
                }
            }
            idx
        };
        if field_start == tokens.len() {
            continue;
        }

        let field_name = tokens[field_start..].join("_");
        let _word_name = tokens[..field_start].join("_");

        // Skip metadata fields.
        if field_name == "op" || field_name == "sub_op" || field_name == "subop" {
            continue;
        }

        // If the word_name itself ends with digits (e.g. DW_3 → field starts
        // after "DW"), strip the numeric suffix from field_name to avoid
        // producing "3_count".  Example: tokens = ["DW","3","count"] →
        // field_start = 1 (because "3" is all-digits, "count" is lowercase).
        // We want word="DW_3", field="count".
        // Re-run: find the longest leading numeric-only tokens and fold them
        // into the word name.
        let (word_name, field_name) = {
            let mut word_tokens: Vec<&str> = tokens[..field_start].to_vec();
            let mut field_tokens: Vec<&str> = tokens[field_start..].to_vec();
            // If field_tokens starts with tokens that are purely numeric,
            // move them back to word_tokens.
            while !field_tokens.is_empty() && field_tokens[0].chars().all(|c| c.is_ascii_digit()) {
                word_tokens.push(field_tokens.remove(0));
            }
            if field_tokens.is_empty() {
                continue;
            }
            (word_tokens.join("_"), field_tokens.join("_"))
        };

        // Skip metadata again after re-split.
        if field_name == "op" || field_name == "sub_op" || field_name == "subop" {
            continue;
        }

        fields.push(RawField {
            pkt_name: pkt_key.to_string(),
            word_name,
            field_name,
            dw: offset,
            mask,
            shift,
        });
    }
    fields
}

// ── Data model ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct BitField {
    name: String,
    mask: u32,
    shift: u32,
}

#[derive(Debug, Default)]
struct Packet {
    name: String,
    bits: BTreeMap<u32, Vec<BitField>>,
    full: BTreeMap<u32, (String, String)>, // dw → (word_name, field_name)
}

fn to_pascal(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
            }
        })
        .collect()
}

// ── Step 3: group into Packet structs ─────────────────────────────────────────

fn build_packets(raw_fields: Vec<RawField>, pkt_names_ordered: &[String]) -> Vec<Packet> {
    let mut map: HashMap<String, Packet> = HashMap::new();
    for rf in raw_fields {
        let pascal = to_pascal(&rf.pkt_name);
        let pkt = map.entry(pascal.clone()).or_insert_with(|| Packet {
            name: pascal,
            ..Default::default()
        });
        if rf.mask == 0xFFFF_FFFF && rf.shift == 0 {
            pkt.full
                .entry(rf.dw)
                .or_insert((rf.word_name, rf.field_name));
        } else {
            let dw_bits = pkt.bits.entry(rf.dw).or_default();
            if !dw_bits.iter().any(|b| b.name == rf.field_name) {
                dw_bits.push(BitField {
                    name: rf.field_name,
                    mask: rf.mask,
                    shift: rf.shift,
                });
            }
        }
    }
    for pkt in map.values_mut() {
        for fields in pkt.bits.values_mut() {
            fields.sort_by_key(|f| f.shift);
        }
    }
    pkt_names_ordered
        .iter()
        .filter_map(|raw| map.remove(&to_pascal(raw)))
        .collect()
}

// ── Step 4: detect @join (lo/hi 64-bit pairs) ────────────────────────────────

fn detect_joins(pkt: &Packet) -> Vec<(u32, u32, String, &'static str)> {
    let mut joins = Vec::new();
    let entries: Vec<(u32, &String)> = pkt.full.iter().map(|(k, (_, f))| (*k, f)).collect();
    let mut used: HashSet<u32> = HashSet::new();
    for i in 0..entries.len() {
        let (lo_dw, lo_name) = entries[i];
        if used.contains(&lo_dw) {
            continue;
        }
        if let Some(base) = lo_name.strip_suffix("_31_0") {
            let hi_target = format!("{}_63_32", base);
            for j in (i + 1)..entries.len() {
                let (hi_dw, hi_name) = entries[j];
                if !used.contains(&hi_dw) && hi_name == &hi_target {
                    joins.push((lo_dw, hi_dw, base.to_string(), "u64"));
                    used.insert(lo_dw);
                    used.insert(hi_dw);
                    break;
                }
            }
        }
    }
    joins
}

// ── Step 5: detect @dyn variable-length payload ───────────────────────────────
//
// We consider a dw to be the start of a dynamic payload only when its
// WORD name ends with a digit (e.g. "DATA0", "DATA0" → index 0 of the array).
// Fixed-value data dwords have plain word names like "DATA" or "SRC_DATA".

fn detect_dyn(pkt: &Packet, join_dws: &HashSet<u32>) -> Option<(u32, String)> {
    for (dw, (word_name, field_name)) in &pkt.full {
        if join_dws.contains(dw) {
            continue;
        }
        // Word name must end with a digit to indicate an indexed array entry.
        let word_ends_digit = word_name
            .chars()
            .last()
            .map_or(false, |c| c.is_ascii_digit());
        if word_ends_digit {
            return Some((*dw, field_name.clone()));
        }
    }
    None
}

// ── Step 6: type inference ────────────────────────────────────────────────────

fn infer_type(mask: u32) -> &'static str {
    match mask.count_ones() {
        1 => "bool",
        2..=8 => "u8",
        9..=16 => "u16",
        _ => "u32",
    }
}

fn fmt_mask(mask: u32) -> String {
    format!("0x{:x}", mask)
}

// ── Step 7: emit ──────────────────────────────────────────────────────────────

fn emit_packet(pkt: &Packet) -> String {
    let joins = detect_joins(pkt);
    let join_dws: HashSet<u32> = joins.iter().flat_map(|(lo, hi, _, _)| [*lo, *hi]).collect();
    let dyn_field = detect_dyn(pkt, &join_dws);
    let dyn_dw: Option<u32> = dyn_field.as_ref().map(|(dw, _)| *dw);
    let needs_lifetime = dyn_field.is_some();
    let mut out = String::new();

    if needs_lifetime {
        out.push_str(&format!("{}<'a> {{\n", pkt.name));
    } else {
        out.push_str(&format!("{} {{\n", pkt.name));
    }

    // @bits
    let bits_dws: Vec<u32> = pkt
        .bits
        .keys()
        .copied()
        .filter(|dw| !join_dws.contains(dw) && Some(*dw) != dyn_dw)
        .collect();
    if !bits_dws.is_empty() {
        out.push_str("\t@bits\n");
        for dw in &bits_dws {
            out.push_str(&format!("\tdw[{}] = {{\n", dw));
            for bf in &pkt.bits[dw] {
                out.push_str(&format!(
                    "\t\t& {} << {} = {}: {};\n",
                    fmt_mask(bf.mask),
                    bf.shift,
                    bf.name,
                    infer_type(bf.mask)
                ));
            }
            out.push_str("\t}\n");
        }
    }

    // @full (single 32-bit dwords, not part of a join/dyn)
    let full_plain: Vec<(u32, &String)> = pkt
        .full
        .iter()
        .filter(|(dw, _)| !join_dws.contains(dw) && Some(**dw) != dyn_dw)
        .map(|(k, (_, f))| (*k, f))
        .collect();
    if !full_plain.is_empty() {
        out.push_str("\t@full\n");
        for (dw, name) in &full_plain {
            out.push_str(&format!("\tdw[{}] = {}: u32;\n", dw, name));
        }
    }

    // @join
    if !joins.is_empty() {
        out.push_str("\t@join\n");
        for (lo, hi, name, ty) in &joins {
            out.push_str(&format!("\tdw[{}], dw[{}] = {}: {};\n", lo, hi, name, ty));
        }
    }

    // @dyn
    if let Some((dw, name)) = dyn_field {
        out.push_str("\t@dyn\n");
        out.push_str(&format!(
            "\t// TODO: identify the length field index\n\tdw[{}..] = {}: &'a [u32],\n\tdw[???] = len\n",
            dw, name));
    }

    out.push_str("}\n");
    out
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    let src: String = {
        let args: Vec<String> = env::args().collect();
        if args.len() >= 2 {
            fs::read_to_string(&args[1]).unwrap_or_else(|e| {
                eprintln!("Error reading '{}': {}", args[1], e);
                std::process::exit(1);
            })
        } else {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("Error reading stdin: {}", e);
                std::process::exit(1);
            });
            buf
        }
    };

    let pkt_names = collect_packet_names(&src);
    let raw_fields = parse_raw_fields(&src, &pkt_names);
    let packets = build_packets(raw_fields, &pkt_names);

    for pkt in &packets {
        if pkt.bits.is_empty() && pkt.full.is_empty() {
            continue;
        }
        print!("{}", emit_packet(pkt));
        println!();
    }
}
