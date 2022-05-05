use crate::parsing::*;

use std::io::Cursor;
use std::io::Read;

#[derive(Debug, Default)]
pub struct Dna {
    pub names: Vec<String>,
    pub types: Vec<String>,
    pub types_len: Vec<usize>,
    pub structs: Vec<DnaStruct>,
}
#[derive(Debug, Default)]
pub struct DnaStruct {
    pub name: String,
    pub fields: Vec<DnaField>,
}
#[derive(Debug, Default)]
pub struct DnaField {
    pub ttype: String,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct Blend {
    pub blocks: Vec<BlendBlock>,
    pub dna: Dna,
}
#[derive(Debug, Default)]
pub struct BlendBlock {
    pub tag: String,
    pub size: usize,
    pub addr: usize,
    pub sdna_idx: usize,
    pub count: usize,
    pub data: Vec<u8>,
}

pub fn parse_dna(bytes: &[u8]) -> std::io::Result<Dna> {
    let mut reader = Cursor::new(bytes);
    let sdna_tag = read_tag(&mut reader)?;
    assert_eq!(sdna_tag, "SDNA");
    let name_tag = read_tag(&mut reader)?;
    assert_eq!(name_tag, "NAME");
    let names_count = read_u32_le(&mut reader)?;
    let mut names = Vec::with_capacity(names_count as usize);
    for _ in 0..names_count {
        let name = read_str(&mut reader)?;
        names.push(name);
    }
    align_to(&mut reader, 4)?;
    let type_tag = read_tag(&mut reader)?;
    assert_eq!(type_tag, "TYPE");
    let types_count = read_u32_le(&mut reader)?;
    let mut types = Vec::with_capacity(types_count as usize);
    for _ in 0..types_count {
        let ttype = read_str(&mut reader)?;
        types.push(ttype);
    }
    align_to(&mut reader, 4)?;
    let tlen_tag = read_tag(&mut reader)?;
    assert_eq!(tlen_tag, "TLEN");
    let mut types_len = Vec::with_capacity(types_count as usize);
    for _ in 0..types_count {
        let tlen = read_u16_le(&mut reader)?;
        types_len.push(tlen as usize);
    }
    align_to(&mut reader, 4)?;
    let struct_tag = read_tag(&mut reader)?;
    assert_eq!(struct_tag, "STRC");
    let structs_count = read_u32_le(&mut reader)?;
    let mut structs = Vec::with_capacity(structs_count as usize);
    for _ in 0..structs_count {
        let struct_type = read_u16_le(&mut reader)? as usize;
        let field_count = read_u16_le(&mut reader)? as usize;
        let mut fields = vec![];
        for _ in 0..field_count {
            let field_type = read_u16_le(&mut reader)? as usize;
            let field_name = read_u16_le(&mut reader)? as usize;
            fields.push(DnaField {
                ttype: types[field_type].clone(),
                name: names[field_name].clone(),
            });
        }
        structs.push(DnaStruct {
            name: types[struct_type].clone(),
            fields,
        });
    }

    Ok(Dna {
        names,
        types,
        types_len,
        structs,
    })
}
fn align_to(mut r: &mut Cursor<&[u8]>, n: usize) -> std::io::Result<()> {
    assert!(n == 2 || n == 4 || n == 8 || n == 16);
    let n = n as u64;
    while r.position() & (n - 1) != 0 {
        _ = read_u8(&mut r)?;
    }
    Ok(())
}

pub fn parse_blend(bytes: &[u8]) -> std::io::Result<Blend> {
    let mut blend = Blend::default();

    let mut reader = Cursor::new(bytes);
    const BLEND_HEADER_SIZE: usize = 12;
    let mut b = [0; BLEND_HEADER_SIZE];
    reader.read_exact(&mut b)?;
    assert_eq!(&b[..7], b"BLENDER");
    assert_eq!(b[7], '-' as u8); // TODO: We only support 8 byte pointers
    assert_eq!(b[8], 'v' as u8); // TODO: We only support little endian

    let mut dna = None;
    loop {
        let tag = read_tag(&mut reader)?;
        let block_size = read_u32_le(&mut reader)? as usize;
        let addr = read_u64_le(&mut reader)? as usize;
        let sdna_idx = read_u32_le(&mut reader)? as usize;
        let count = read_u32_le(&mut reader)? as usize;

        let mut data = vec![0; block_size];
        reader.read_exact(&mut data)?;

        if tag == "DNA1" {
            dna = Some(parse_dna(&data)?);
        }

        blend.blocks.push(BlendBlock {
            tag: tag.clone(),
            size: block_size,
            addr,
            sdna_idx,
            count,
            data,
        });

        if tag == "ENDB" {
            break;
        }
    }
    blend.dna = dna.expect("Could not find required DNA1 block in .blend file");

    for block in &blend.blocks {
        match block.tag.as_str() {
            "REND" => {}
            "DATA" => {
                if blend.dna.structs[block.sdna_idx].name.as_str() == "MVert" {
                    //    let mut r = Cursor::new(&block.data);
                }
            }
            "DNA1" => {}
            "GLOB" => {}
            "TEST" => {
                // Thumbnail
                let mut r = Cursor::new(&block.data);
                let width = read_u32_le(&mut r)?;
                let height = read_u32_le(&mut r)?;
                let pixels = &block.data[8..];
                println!("Thumbnail: {}x{}, pixels: {}\n", width, height, pixels.len());
                // Write as .ppm
                //let pixels = data[8..].chunks(4).map(|c| &c[..3]).flatten().map(|b| *b).collect::<Vec<_>>();
                //let mut file = fs::File::create("/home/alex/blendthumb.ppm")?;
                //file.write_all(format!("P6\n{} {}\n255\n", width, height).as_bytes())?;
                //file.write_all(&pixels)?;
            }
            "USER" => {}
            "ENDB" => {}
            "AR" => {}
            "BR" => {}
            "CA" => {}
            "GD" => {}
            "GR" => {}
            "IM" => {}
            "LA" => {}
            "LS" => {}
            "MA" => {}
            "ME" => {
                // Mesh
            }
            "OB" => {}
            "PL" => {}
            "SC" => {}
            "SN" => {}
            "TX" => {}
            "WM" => {}
            "WO" => {}
            "WS" => {}
            t => panic!("Unknown block tag: {}", t),
        }
    }

    Ok(blend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn dna() -> std::io::Result<()> {
        let bytes = fs::read("/home/alex/thirdparty/blender-git/blender/dna.bin").unwrap();
        assert!(bytes.len() != 0);
        let dna = parse_dna(&bytes)?;
        println!("#names:  {}", dna.names.len());
        println!("#types:   {}", dna.types.len());
        println!("#structs: {}", dna.structs.len());

        Ok(())
    }

    #[test]
    fn blend() -> std::io::Result<()> {
        let bytes = fs::read("/home/alex/thirdparty/blender-git/blender/release/datafiles/startup.blend").unwrap();
        let bytes = fs::read("/home/alex/tmp/base_model.blend").unwrap();
        assert!(bytes.len() != 0);
        let _ = parse_blend(&bytes)?;
        Ok(())
    }
}
