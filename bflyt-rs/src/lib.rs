use binrw::io::SeekFrom;
use binrw::meta::{EndianKind, ReadEndian};
use binrw::{binread, BinRead, BinResult, NullString, Endian, BinWrite, BinWriterExt, binwrite};
use byteorder::{LittleEndian, ReadBytesExt}; // 1.2.7
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};
use std::{
    fs::File,
    io::{Read, Seek},
};

#[derive(Debug, BinRead, BinWrite, Default, Clone)]
pub struct SerdeNullString(NullString);

impl Serialize for SerdeNullString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for SerdeNullString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
    {
        deserializer.deserialize_str(SerdeNullStringVisitor)
    }
}

struct SerdeNullStringVisitor;

impl<'de> Visitor<'de> for SerdeNullStringVisitor {
    type Value = SerdeNullString;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string to be converted to null string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where E: de::Error
    {
        let null_str = NullString::from(s);

        Ok(SerdeNullString(null_str))
    }
}

pub unsafe fn str_from_u8_nul_utf8_unchecked(utf8_src: &[u8]) -> &str {
    let nul_range_end = utf8_src.iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    ::std::str::from_utf8_unchecked(&utf8_src[0..nul_range_end])
}

fn cstr_serialize<S>(x: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(unsafe {
        str_from_u8_nul_utf8_unchecked(x)
    })
}

struct CstrVisitor;

impl<'de> Visitor<'de> for CstrVisitor {
    type Value = [u8; 24];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string to be converted to byte array")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where E: de::Error
    {
        let mut bytes = [0u8; 24];
        let null_str = s.as_bytes();
        assert!(null_str.len() <= 24);
        for (idx, byte) in null_str.bytes().enumerate() {
            bytes[idx] = byte.unwrap();    
        }

        Ok(bytes)
    }
}

fn cstr_deserialize<'de, D>(d: D) -> Result<[u8; 24], D::Error>
where
    D: Deserializer<'de>,
{
    let buf = d.deserialize_str(CstrVisitor)?;
    Ok(buf)
}

impl ReadEndian for SerdeNullString {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}


#[derive(Serialize, Deserialize, Debug)]
#[binread]
#[binwrite]
#[brw(little, magic = b"FLYT")]
pub struct BflytFile {
    header: BflytHeader,
    #[br(count = header.section_count)]
    sections: Vec<BflytSection>,
}

#[binread]
#[binwrite]
#[derive(Serialize, Deserialize, Debug)]
pub struct BflytHeader {
    byte_order: u16,
    header_size: u16,
    #[br(dbg)]
    version: u32,
    file_size: u32,
    section_count: u16,
    padding: u16
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug, Copy, Clone)]
pub struct ResColorTest {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug, Copy, Clone)]
pub struct ResVec2Test {
    pub x: f32,
    pub y: f32,
}

impl ReadEndian for ResVec2Test {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug, Copy, Clone)]
pub struct ResVec3Test {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ReadEndian for ResVec3Test {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug, Clone)]
pub struct ResPaneTest {
    pub flag: u8,
    pub base_position: u8,
    pub alpha: u8,
    pub flag_ex: u8,
    #[serde(serialize_with = "cstr_serialize", deserialize_with = "cstr_deserialize")]
    pub name: [u8; 24],
    pub user_data: [u8; 8],
    pub pos: ResVec3Test,
    pub rot_x: f32,
    pub rot_y: f32,
    pub rot_z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub size_x: f32,
    pub size_y: f32,
}

impl ReadEndian for ResPaneTest {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

fn texture_list_parser<R: Read + Seek>(reader: &mut R, _: Endian, _: ()) -> BinResult<TextureListInner> {
    let mut texture_names: Vec<SerdeNullString> = Vec::new();

    let tex_count = reader.read_i32::<LittleEndian>()?;
    let base_offset = reader.stream_position()?;

    let mut offsets = vec![0i32; tex_count as usize];
    reader.read_i32_into::<LittleEndian>(offsets.as_mut_slice())?;
    for offset in &offsets {
        reader.seek(SeekFrom::Start(base_offset + *offset as u64))?;
        texture_names.push(SerdeNullString::read(reader)?);
    }

    Ok(TextureListInner { tex_count, offsets, texture_names })
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct TextureListInner {
    pub tex_count: i32,
    #[br(count = tex_count)]
    pub offsets: Vec<i32>,
    #[br(count = tex_count)]
    pub texture_names: Vec<SerdeNullString>
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResFont {
    pub offset: u32
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct FontListInner {
    pub font_count: u16,
    padding: u16,
    #[br(count = font_count)]
    pub fonts: Vec<ResFont>,
    #[br(count = font_count)]
    pub font_names: Vec<SerdeNullString>
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug, Clone)]
pub struct ResPictureTest {
    pub pane: ResPaneTest,
    pub vtx_cols: [ResColorTest; 4],
    pub material_idx: u16,
    pub tex_coord_count: u8,
    pub flags: u8,
    #[br(count = tex_coord_count)]
    pub tex_coords: Vec<[ResVec2Test; 4]>,
}

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug, Default)]
pub struct ResAnimationInfo {
    pub kind: u32,
    pub count: u8,
    pub padding: [u8; 3],
}

impl ReadEndian for ResAnimationInfo {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResPerCharacterTransform {
    pub eval_time_offset: f32,
    pub eval_time_width: f32,
    pub loop_type: u8,
    pub origin_v: u8,
    pub has_animation_info: u8,
    pub padding: [u8; 1],
}

impl ReadEndian for ResPerCharacterTransform {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResTextBoxTest {
    pub pane: ResPaneTest,
    pub text_buf_bytes: u16,
    pub text_str_bytes: u16,
    pub material_idx: u16,
    pub font_idx: u16,
    pub text_position: u8,
    pub text_alignment: u8,
    pub text_box_flag: u16,
    pub italic_ratio: f32,
    pub text_str_offset: u32,
    pub text_cols: [ResColorTest; 2],
    pub font_size: ResVec2Test,
    pub char_space: f32,
    pub line_space: f32,
    pub text_id_offset: u32,
    pub shadow_offset: ResVec2Test,
    pub shadow_scale: ResVec2Test,
    pub shadow_cols: [ResColorTest; 2],
    pub shadow_italic_ratio: f32,
    pub line_width_offset_offset: u32,
    pub per_character_transform_offset: u32,
    #[br(count = text_buf_bytes)]
    pub text: Vec<u8>,
    #[bw(
        if(*text_id_offset > 0), 
        // Can't use absolute offsets, so... we know it's after the text string.
        pad_before = (*text_id_offset as u64 - (*text_str_offset as u64 + *text_buf_bytes as u64))
    )]
    #[br(
        if(text_id_offset > 0), 
        // Can't use absolute offsets, so... we know it's after the text string.
        pad_before = (text_id_offset as u64 - (text_str_offset as u64 + text_buf_bytes as u64))
    )]
    pub text_id: SerdeNullString,
    // Not sure if any of the following work, so if something breaks, check here.
    #[br(if(line_width_offset_offset > 0))]
    #[bw(if(*line_width_offset_offset > 0))]
    pub line_width_offset_count: u8,
    #[br(if(line_width_offset_offset > 0), count = line_width_offset_count)]
    #[bw(if(*line_width_offset_offset > 0))]
    pub line_offset: Vec<f32>,
    #[br(if(line_width_offset_offset > 0), count = line_width_offset_count)]
    #[bw(if(*line_width_offset_offset > 0))]
    pub line_width: Vec<f32>,
    #[br(if(per_character_transform_offset > 0))]
    #[bw(if(*per_character_transform_offset > 0))]
    pub per_character_transform: Option<ResPerCharacterTransform>,
    #[br(if(per_character_transform_offset > 0))]
    #[bw(if(*per_character_transform_offset > 0))]
    pub per_character_transform_animation_info: Option<ResAnimationInfo>,
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResPartsProperty {
    #[serde(serialize_with = "cstr_serialize", deserialize_with = "cstr_deserialize")]
    pub name: [u8; 24],
    pub usage_flag: u8,
    pub basic_usage_flag: u8,
    pub material_usage_flag: u8,
    pub system_ext_user_data_override_flag: u8,
    pub property_offset: u32,
    pub ext_user_data_offset: u32,
    pub pane_basic_info_offset: u32,
}

impl ReadEndian for ResPartsProperty {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResPartsTest {
    pub size: u32,
    pub pane: ResPaneTest,
    pub property_count: u32,
    pub magnify: ResVec2Test,
    #[br(count = property_count)]
    pub properties: Vec<ResPartsProperty>,
    #[brw(align_after = 4)]
    pub part_name: SerdeNullString,
    // Not actually
    #[br(count = property_count)]
    pub sections: Vec<BflytSection>
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResPartsPaneBasicInfo {
    pub user_data: [u8; 8],
    pub translate: ResVec3Test,
    pub rotate: ResVec3Test,
    pub scale: ResVec2Test,
    pub size: ResVec2Test,
    pub alpha: u8,
    padding: [u8; 3]
}

fn res_parts_parser<R: Read + Seek>(reader: &mut R, _: Endian, _: ()) -> BinResult<ResPartsTest> {
    let base_offset = reader.stream_position()? - 4;

    let size = reader.read_u32::<LittleEndian>()?;
    let pane = ResPaneTest::read(reader)?;

    let mut properties: Vec<ResPartsProperty> = Vec::new();

    let property_count = reader.read_u32::<LittleEndian>()?;
    let magnify = ResVec2Test::read(reader)?;

    for _ in 0..property_count {
        let property = ResPartsProperty::read(reader)?;
        properties.push(property);
    }

    let part_name = SerdeNullString::read(reader)?;
    let pos = reader.stream_position()?;
    if pos % 4 != 0 {
        reader.seek(SeekFrom::Current((4 - (pos % 4)) as i64))?;
    }

    let mut sections = Vec::new();
    for property in &properties {
        if property.property_offset != 0 {
            reader.seek(SeekFrom::Start(base_offset + property.property_offset as u64))?;
            let section = BflytSection::read(reader)?;
            sections.push(section);
        }

        if property.ext_user_data_offset != 0 {
            reader.seek(SeekFrom::Start(base_offset + property.ext_user_data_offset as u64))?;
            let section = BflytSection::read(reader)?;
            sections.push(section);
        }

        if property.pane_basic_info_offset != 0 {
            reader.seek(SeekFrom::Start(base_offset + property.pane_basic_info_offset as u64))?;
            let section = BflytSection::read(reader)?;
            sections.push(section);
        }
    }

    let curr_pos = reader.stream_position()?;
    assert!(curr_pos == base_offset + size as u64, "Failed to parse ResParts with pane name {} and part name {:#?}. Expected to read {size} bytes, but read {}", 
        unsafe { str_from_u8_nul_utf8_unchecked(&pane.name) }, part_name, curr_pos - base_offset);

    let parts = ResPartsTest {
        size,
        pane,
        property_count,
        magnify,
        properties,
        part_name,
        sections
    };

    Ok(parts)
}

#[binrw::parser(reader: reader, endian: _endian)]
fn material_list_parser() -> BinResult<MaterialListInner> {
    let base_offset = reader.stream_position()?;
    println!("base_offset: {base_offset}");

    let mut materials: Vec<ResMaterial> = Vec::new();

    let material_count: u16 = reader.read_u16::<LittleEndian>()?;
    
    println!("material_count: {material_count}");

    let _ = reader.seek_relative(2);

    let mut offsets = vec![0u32; material_count as usize];

    reader.read_u32_into::<LittleEndian>(&mut offsets.as_mut_slice())?;

    offsets = offsets.iter().map(|x| x - 8).collect();

    for offset in &offsets {
        let _ = reader.seek(SeekFrom::Start(base_offset + *offset as u64))?;

        // let name = SerdeNullString::read(reader)?;

        let mut name_buf = vec![0u8;28];
        reader.read_exact(&mut name_buf)?;

        let name = SerdeNullString(NullString(name_buf));

        let bitflags = reader.read_u32::<LittleEndian>()?;

        let _unkown = reader.read_i32::<LittleEndian>()?;

        let fg_clr = ResColorTest {
            r: reader.read_u8()?,
            g: reader.read_u8()?,
            b: reader.read_u8()?,
            a: reader.read_u8()?
        };
        let bg_clr = ResColorTest {
            r: reader.read_u8()?,
            g: reader.read_u8()?,
            b: reader.read_u8()?,
            a: reader.read_u8()?
        };
            
        println!("Material: {:?}", name);
        println!("  bitflags: {bitflags}, position: {}", reader.stream_position()?);

        let texture_map_count = bitflags & 3;
        let mut texture_maps: Vec<ResTexMap> = Vec::new();

        let texture_transform_count = (bitflags & 0xC) >> 2;
        let mut texture_transforms: Vec<ResTexTransform>= Vec::new();

        let tex_coord_gen_count = (bitflags >> 4) & 3;
        let mut texture_coord_gens: Vec<ResTexCoordGen> = Vec::new();

        let tev_stages_count = (bitflags >> 6) & 7;
        let mut tev_stages: Vec<ResTevStage> = Vec::new();

        let alpha_compare_count = (bitflags >> 9) & 1;
        let mut alpha_compares: Vec<ResAlphaCompare> = Vec::new();

        let blend_mode_count = (bitflags >> 10) & 1;
        let mut blend_modes: Vec<ResBlendMode> = Vec::new();

        let indirect_params_count = (bitflags >> 0) & 1;
        let mut indirect_params: Vec<ResIndirectParameter> = Vec::new();
        
        println!("  textures: {texture_map_count},\n  transforms: {texture_transform_count},\n  tex_coords: {tex_coord_gen_count},\n  tev_stages: {tev_stages_count},\n  alpha_comp: {alpha_compare_count},\n  blend_modes: {blend_mode_count},\n  indirect_params: {indirect_params_count}");

        for _ in 0..texture_map_count {
            texture_maps.push(ResTexMap {
                tex_idx: reader.read_u16::<LittleEndian>()?,
                wrap_s_flt: reader.read_u8()?,
                wrap_t_flt: reader.read_u8()?
            });
        }

        for _ in 0..texture_transform_count {
            texture_transforms.push(ResTexTransform {
                rotate: reader.read_f32::<LittleEndian>()?,
                scale: ResVec2Test {
                    x: reader.read_f32::<LittleEndian>()?,
                    y: reader.read_f32::<LittleEndian>()?
                },
                translate: ResVec2Test {
                    x: reader.read_f32::<LittleEndian>()?,
                    y: reader.read_f32::<LittleEndian>()?
                },
            });
        }

        for _ in 0..tex_coord_gen_count {
            texture_coord_gens.push(ResTexCoordGen {
                matrix_type: TexGenType::Matrix2x4,
                source: TexGenSourceType::from_u8(reader.read_u8()?),
                test: reader.read_u16::<LittleEndian>()?
            });
        }

        for _ in 0..tev_stages_count {
            let combine_rgb: TevMode = TevMode::from_u8(reader.read_u8()?);
            let combine_alpha: TevMode = TevMode::from_u8(reader.read_u8()?);
            tev_stages.push(ResTevStage {
                combine_rgb,
                combine_alpha
            });
        }

        for _ in 0..alpha_compare_count {
            let alpha_test: AlphaTest = AlphaTest::from_u8(reader.read_u8()?);

            alpha_compares.push(ResAlphaCompare {
                alpha_test,
                target: reader.read_f32::<LittleEndian>()?
            })
        }

        for _ in 0..blend_mode_count {
            let src_factor = reader.read_u8()?;
            let dest_factor = reader.read_u8()?;
            let blend_op = reader.read_u8()?;
            let logical_op = reader.read_u8()?;

            blend_modes.push(ResBlendMode {
                src_factor: Factor::from_u8(src_factor),
                dest_factor: Factor::from_u8(dest_factor),
                blend_op: BlendOp::from_u8(blend_op),
                logical_op: LogicalOp::from_u8(logical_op)
            })
        }

        for _ in 0..indirect_params_count {
            indirect_params.push(ResIndirectParameter {
                rotate: reader.read_f32::<LittleEndian>()?,
                scale: ResVec2Test::read(reader)?
            })
        }

        let mat: ResMaterial = ResMaterial {
            // size: material_size,
            name,
            bitflags,
            mat_black_color: fg_clr,
            mat_white_color: bg_clr,
            // resource_count,
            texture_maps,
            texture_transforms,
            texture_coord_gens,
            tev_stages,
            alpha_compares,
            blend_modes,
            indirect_params
        };

        // println!("mat: {:?}", mat);
        materials.push(mat);
    }

    Ok(MaterialListInner { /* size, */ material_count, offsets, materials })
}

// #[repr(C)]
// #[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
// pub enum ResMaterialResource {
//     ResTexMap {
//         tex_idx: u16,
//         wrap_s_flt: u8,
//         wrap_t_flt: u8
//     },
//     ResTexTransform {
//         rotate: f32,
//         scale: ResVec2Test,
//         translate: ResVec2Test
//     },
//     ResTexCoordGen {
//         matrix_type: TexGenType,
//         source: TexGenSourceType,
//         test: u16
//     },
//     ResTevStage {
//         combine_rgb: TevMode,
//         combine_alpha: TevMode
//     },
//     ResAlphaCompare {
//         alpha_test: AlphaTest,
//         target: f32
//     },
//     ResBlendMode {
//         src_factor: Factor,
//         dest_factor: Factor,
//         blend_op: BlendOp,
//         logical_op: LogicalOp
//     },
// }

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
#[brw(repr = u8)]
pub enum TexGenType {
    Matrix2x4
}

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
#[brw(repr = u8)]
pub enum TexGenSourceType {
    Tex0,
    Tex1,
    Tex2,
    OrthoProjection,
    PaneBaseOrthoProjection,
    PerspectiveProjection
}

impl TexGenSourceType {
    pub fn from_u8(value: u8) -> TexGenSourceType {
        let result = match value {
            0 => TexGenSourceType::Tex0,
            1 => TexGenSourceType::Tex1,
            2 => TexGenSourceType::Tex2,
            3 => TexGenSourceType::OrthoProjection,
            4 => TexGenSourceType::PaneBaseOrthoProjection,
            5 => TexGenSourceType::PaneBaseOrthoProjection,
            6 => TexGenSourceType::PerspectiveProjection,
            _ => panic!("")
        };

        result
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResTexMap {
    tex_idx: u16,
    wrap_s_flt: u8,
    wrap_t_flt: u8
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResTexTransform {
    rotate: f32,
    scale: ResVec2Test,
    translate: ResVec2Test
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResTexCoordGen {
    matrix_type: TexGenType,
    source: TexGenSourceType,
    test: u16
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResAlphaCompare {
    alpha_test: AlphaTest,
    target: f32
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResTevStage {
    combine_rgb: TevMode,
    combine_alpha: TevMode
}

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
#[brw(repr = u8)]
pub enum TevMode {
    Replace,
    Modulate,
    Add,
    AddSigned,
    Interpolate,
    Subtract,
    AddMultiply,
    MultiplyAdd,
    Overlay,
    Lighten,
    Darken,
    Indirect,
    BlendIndirect,
    EachIndirect
}

impl TevMode {
    pub fn from_u8(value: u8) -> TevMode {
        match value {
            0 => TevMode::Replace, 
            1 => TevMode::Modulate, 
            2 => TevMode::Add, 
            3 => TevMode::AddSigned,
            4 => TevMode::Interpolate, 
            5 => TevMode::Subtract,
            6 => TevMode::AddMultiply, 
            7 => TevMode::MultiplyAdd, 
            8 => TevMode::Overlay, 
            9 => TevMode::Lighten, 
            10 => TevMode::Darken, 
            11 => TevMode::Indirect, 
            12 => TevMode::BlendIndirect, 
            13 => TevMode::EachIndirect,
            _ => panic!("")
        }
    }
}
        
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
#[brw(repr = u8)]
pub enum AlphaTest {
    Never,
    Less,
    LessEqual,
    Equal,
    NotEqual,
    GreaterEqual,
    Greater,
    Always
}

impl AlphaTest {
    pub fn from_u8(value: u8) -> AlphaTest {
        match value {
            0 => AlphaTest::Never,
            1 => AlphaTest::Less,
            2 => AlphaTest::LessEqual,
            3 => AlphaTest::Equal,
            4 => AlphaTest::NotEqual,
            5 => AlphaTest::GreaterEqual,
            6 => AlphaTest::Greater,
            7 => AlphaTest::Always,
            _ => panic!("")
        }
    }
}

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
#[brw(repr = u8)]
pub enum Factor {
    Zero,
    One,
    DestColor,
    InverseDestColor,
    SrcAlpha,
    InverseSrcAlpha,
    DestAlpha,
    InverseDestAlpha,
    SrcColor,
    InverseSrcColor
}

impl Factor {
    pub fn from_u8(value: u8) -> Factor {
        match value {
            0 => Factor::Zero,
            1 => Factor::One,
            2 => Factor::DestColor,
            3 => Factor::InverseDestColor,
            4 => Factor::SrcAlpha,
            5 => Factor::InverseSrcAlpha,
            6 => Factor::DestAlpha,
            7 => Factor::InverseDestAlpha,
            8 => Factor::SrcColor,
            9 => Factor::InverseSrcColor,
            _ => panic!("")
        }
    }
}

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
#[brw(repr = u8)]
pub enum BlendOp {
    Disable,
    Add,
    Subtract,
    ReverseSubtract,
    SelectMin,
    SelectMax
}

impl BlendOp {
    pub fn from_u8(value: u8) -> BlendOp {
        match value {
            0 => BlendOp::Disable,
            1 => BlendOp::Add,
            2 => BlendOp::Subtract,
            3 => BlendOp::ReverseSubtract,
            4 => BlendOp::SelectMin,
            5 => BlendOp::SelectMax,
            _ => panic!("")
        }
    }
}

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
#[brw(repr = u8)]
pub enum LogicalOp {
    Disable,
    NoOp,
    Clear,
    Set,
    Copy,
    InvCopy,
    Inv,
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Equiv,
    RevAnd,
    InvAnd,
    RevOr,
    InvOr,
}

impl LogicalOp {
    pub fn from_u8(value: u8) -> LogicalOp {
        match value {
            0 => LogicalOp::Disable,
            1 => LogicalOp::NoOp,
            2 => LogicalOp::Clear,
            3 => LogicalOp::Set,
            4 => LogicalOp::Copy,
            5 => LogicalOp::InvCopy,
            6 => LogicalOp::Inv,
            7 => LogicalOp::And,
            8 => LogicalOp::Nand,
            9 => LogicalOp::Or,
            10 => LogicalOp::Nor,
            11 => LogicalOp::Xor,
            12 => LogicalOp::Equiv,
            13 => LogicalOp::RevAnd,
            14 => LogicalOp::InvAnd,
            15 => LogicalOp::RevOr,
            16 => LogicalOp::InvOr,
            _ => panic!("")
        }
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResBlendMode {
    pub src_factor: Factor,
    pub dest_factor: Factor,
    pub blend_op: BlendOp,
    pub logical_op: LogicalOp
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct MaterialListInner {
    // size: u32,
    material_count: u16,
    #[br(count = material_count)]
    offsets: Vec<u32>,
    #[br(count = material_count)]
    materials: Vec<ResMaterial>
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResIndirectParameter {
    rotate: f32,
    scale: ResVec2Test
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResMaterial {
    // pub size: u32,
    pub name: SerdeNullString,
    pub bitflags: u32,
    pub mat_black_color: ResColorTest,
    pub mat_white_color: ResColorTest,
    #[br(count = bitflags & 3)]
    pub texture_maps: Vec<ResTexMap>,
    #[br(count = (bitflags >> 2) & 3)]
    pub texture_transforms: Vec<ResTexTransform>,
    #[br(count = (bitflags >> 4) & 3)]
    pub texture_coord_gens: Vec<ResTexCoordGen>,
    #[br(count = (bitflags >> 6) & 7 )]
    pub tev_stages: Vec<ResTevStage>,
    #[br(count = (bitflags >> 9) & 1)]
    pub alpha_compares: Vec<ResAlphaCompare>,
    #[br(count = (bitflags >> 10) & 1)]
    pub blend_modes: Vec<ResBlendMode>,
    #[br(count = (bitflags >> 0) & 1)]
    pub indirect_params: Vec<ResIndirectParameter>
}

#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub enum BflytSection {
    #[brw(magic = b"pan1")]
    Pane {
        size: u32,
        pane: ResPaneTest
    },

    #[brw(magic = b"txl1")]
    TextureList {
        size: u32,
        #[br(parse_with = texture_list_parser)]
        #[brw(align_after = 4)]
        texture_list: TextureListInner
    },

    #[brw(magic = b"pic1")]
    Picture {
        size: u32,
        picture: ResPictureTest
    },

    #[brw(magic = b"txt1")]
    TextBox {
        size: u32,
        #[brw(align_after = 4)]
        text_box: ResTextBoxTest
    },

    #[brw(magic = b"prt1")]
    Part {
        #[br(parse_with = res_parts_parser)]
        part: ResPartsTest,
    },

    #[brw(magic = b"mat1")]
    MaterialList {
        #[br(dbg)]
        size: u32,
        #[br(parse_with = material_list_parser)]
        material_list: MaterialListInner
        // material_count: i16,
        // #[br(count = material_count, pad_before = 2)]
        // offsets: Vec<u32>,
        // #[br(parse_with = binrw::file_ptr::parse_from_iter(offsets.iter().copied()), seek_before = SeekFrom::Start(4))]
        // materials: Vec<ResMaterial>
    },

    #[brw(magic = b"wnd1")]
    Window {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[brw(magic = b"pas1")]
    PaneStart {
        #[br(assert(size == 8))]
        size: u32,
    },

    #[brw(magic = b"pae1")]
    PaneEnd {
        #[br(assert(size == 8))]
        size: u32,
    },

    #[brw(magic = b"grp1")]
    Group {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[brw(magic = b"grs1")]
    GroupStart {
        #[br(assert(size == 8))]
        size: u32,
    },

    #[brw(magic = b"gre1")]
    GroupEnd {
        #[br(assert(size == 8))]
        size: u32,
    },

    #[brw(magic = b"bnd1")]
    Bounding {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[brw(magic = b"lyt1")]
    Layout {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[brw(magic = b"fnl1")]
    FontList {
        size: u32,
        // #[brw(align_after = 4)]
        #[br(pad_size_to = size as usize - 8)]
        font_list: FontListInner
    },

    #[brw(magic = b"usd1")]
    UserDataList {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    PartsBasicInfo {
        info: ResPartsPaneBasicInfo
    }
}

impl ReadEndian for BflytSection {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

impl BflytFile {
    pub fn new_from_file(filename: &str) -> Result<BflytFile, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        let bflyt = BflytFile::read(&mut file).unwrap();
        println!("Parsed BFLYT!");

        Ok(bflyt)
    }

    pub fn write_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(filename)?;
        file.write_le(self)?;

        Ok(())
    }
}
