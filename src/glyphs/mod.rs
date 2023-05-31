use csv::ReaderBuilder;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::process::Command;

pub use self::types::{GlyphRef, UnicodeData};

pub mod types;

pub fn for_ufo(ufodir: String) -> Vec<GlyphRef> {
    let (_, cmd) = mfek_ipc::module::available("metadata", "0.0.4")
        .expect("MFEKmetadata unavailable; cannot continue");
    let mut glyphs_cmd = Command::new(cmd);

    glyphs_cmd.args(&[&ufodir, "glyphs"]);

    let status = glyphs_cmd.status().expect("MFEKmetadata failed to run");

    assert!(status.success(), "MFEKmetadata failed to run");

    let glyphs = glyphs_cmd.output().expect("MFEKmetadata failed to run");
    let glyphs = String::from_utf8(glyphs.stdout).unwrap();

    return parse_tsv(&glyphs).expect("Failed to parse mfekmetadata data!");
}

pub fn parse_tsv(tsv_data: &str) -> Result<Vec<GlyphRef>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(tsv_data.as_bytes());

    let header_map = {
        let headers = reader.headers()?.iter().enumerate();
        let mut header_map = HashMap::new();
        for (i, header) in headers {
            header_map.insert(header.to_string(), i);
        }
        header_map
    };

    let mut data: Vec<GlyphRef> = Vec::new();
    for result in reader.records() {
        let record = result?;
        let glyph = GlyphRef::from((&header_map, &record));
        data.push(glyph);
    }
    Ok(data)
}

pub fn to_unique_codepoints(gvec: &[GlyphRef]) -> HashSet<UnicodeData> {
    let mut unique_encodings: HashSet<UnicodeData> = HashSet::new();
    for gr in gvec.iter() {
        gr.unicode.iter().for_each(|ud| {
            if !unique_encodings.insert(ud.clone()) {
                log::warn!("Two glyphs with identical encoding in font: U+{0:04X}! Try `grep -R {0:04X}` on glyphs dir.", ud.encoding as u32);
            }
        });
    }
    unique_encodings
}
