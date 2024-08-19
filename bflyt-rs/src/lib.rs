use binrw::io::SeekFrom;
use binrw::meta::{EndianKind, ReadEndian};
use binrw::{binread, BinRead, BinResult, NullString, Endian, BinWrite, BinWriterExt, binwrite};
// use binrw::helpers::args_iter;
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

// fn material_list_parser<R: Read + Seek>(reader: &mut R, _: Endian, _: ()) -> BinResult<MaterialListInner> {
//     // Get the material count
//     // Get the start of the block
//     // Create offsets table
//     // Create a container to store materials
//     // Loop through the offsets in table and store them as ResMaterial in container
//     println!("Running material_list_parser");
//     let mut materials: Vec<ResMaterial> = Vec::new();

//     // println!("{:?}", materials);
//     let material_count = reader.read_u16::<LittleEndian>()?;
//     let _ = reader.read_u16::<LittleEndian>()?;
//     let base_offset = reader.stream_position()?;

//     // println!("count: {}, base offset: {}", material_count, base_offset);
//     let mut offsets = vec![0u32; material_count as usize];

//     reader.read_u32_into::<LittleEndian>(offsets.as_mut_slice())?;

//     println!("offsets: {:?}", offsets);

//     for offset in &offsets {
//         println!("offset: {}", offset);
//         reader.seek(SeekFrom::Start(base_offset + *offset as u64))?;

//         let size = reader.read_u32::<LittleEndian>().unwrap();

//         println!("size: {:?}", size);

//         let name = SerdeNullString::read(reader)?;

//         let resource_count = ResMaterialResourceCount {
//             // size: reader.read_u32::<LittleEndian>()?,
//             tex_map_count: reader.read_u8().unwrap(),
//             tex_srt_count: reader.read_u8().unwrap(),
//             tex_coord_gen_count: reader.read_u8().unwrap(),
//             tev_stage_count: reader.read_u8().unwrap()
//         };

//         let mat: ResMaterial = ResMaterial { name, resource_count };

//         println!("mat: {:?}", mat);
//         materials.push(mat);
//     }

//     Ok(MaterialListInner { material_count, offsets, materials })
// }

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub struct ResMaterialResourceCount {
    // pub tex_map_count: u8,
    // pub tex_srt_count: u8,
    // pub tex_coord_gen_count: u8,
    // pub tev_stage_count: u8
    pub bits: u32
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub enum ResMaterialResource {
    ResTexMap {
        tex_idx: u16,
        wrap_s_flt: u8,
        wrap_t_flt: u8
    },
    ResTexTransform {
        rotate: f32,
        scale: ResVec2Test,
        translate: ResVec2Test
    },
    ResTexCoordGen {
        matrix_type: TexGenType,
        source: TexGenSourceType,
        test: u16
    },
    ResTevStage {
        combine_rgb: TevMode,
        combine_alpha: TevMode
    },
    ResAlphaCompare {
        alpha_test: AlphaTest,
        target: f32
    },
    ResBlendMode {
        src_factor: Factor,
        dest_factor: Factor,
        blend_op: BlendOp,
        logical_op: LogicalOp
    },
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub enum TexGenType {
    Matrix2x4(u8)
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub enum TexGenSourceType {
    Tex0(u8),
    Tex1(u8),
    Tex2(u8),
    OrthoProjection(u8),
    PaneBaseOrthoProjection(u8),
    PerspectiveProjection(u8)
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

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub enum TevMode {
    Replace(u8),
    Modulate(u8),
    Add(u8),
    AddSigned(u8),
    Interpolate(u8),
    Subtract(u8),
    AddMultiply(u8),
    MultiplyAdd(u8),
    Overlay(u8),
    Lighten(u8),
    Darken(u8),
    Indirect(u8),
    BlendIndirect(u8),
    EachIndirect(u8)
}
        
#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub enum AlphaTest {
    Never(u8),
    Less(u8),
    LessEqual(u8),
    Equal(u8),
    NotEqual(u8),
    GreaterEqual(u8),
    Greater(u8),
    Always(u8)
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub enum Factor {
    Zero(u8),
    One(u8),
    DestColor(u8),
    InverseDestColor(u8),
    SrcAlpha(u8),
    InverseSrcAlpha(u8),
    DestAlpha(u8),
    InverseDestAlpha(u8),
    SrcColor(u8),
    InverseSrcColor(u8)
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub enum BlendOp {
    Disable(u8),
    Add(u8),
    Subtract(u8),
    ReverseSubtract(u8),
    SelectMin(u8),
    SelectMax(u8),
}

#[repr(C)]
#[derive(Serialize, Deserialize, BinRead, BinWrite, Debug)]
pub enum LogicalOp {
    Disable(u8),
    NoOp(u8),
    Clear(u8),
    Set(u8),
    Copy(u8),
    InvCopy(u8),
    Inv(u8),
    And(u8),
    Nand(u8),
    Or(u8),
    Nor(u8),
    Xor(u8),
    Equiv(u8),
    RevAnd(u8),
    InvAnd(u8),
    RevOr(u8),
    InvOr(u8),
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
pub struct ResMaterial {
    #[br(dbg)]
    pub name: SerdeNullString,
    pub mat_black_color: f32,
    pub mat_white_color: f32,
    pub resource_count: u32,
    #[br(count = resource_count & 3)]
    pub texture_maps: Vec<ResTexMap>,
    #[br(count = (resource_count >> 2) & 3)]
    pub texture_transforms: Vec<ResTexTransform>,
    #[br(count = (resource_count >> 4) & 3)]
    pub texture_coord_gens: Vec<ResTexCoordGen>,
    #[br(count = (resource_count >> 6) & 7 )]
    pub tev_stages: Vec<ResTevStage>,
    #[br(count = (resource_count >> 9) & 1)]
    pub alpha_compare: Vec<ResAlphaCompare>,
    #[br(count = (resource_count >> 10) & 1)]
    pub blend_mode: Vec<ResBlendMode>
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
        size: u32,
        material_count: u16,
        #[br(count = material_count - 4, pad_before = 2)]
        offsets: Vec<u32>,
        #[br(parse_with = binrw::file_ptr::parse_from_iter(offsets.iter().copied()))]
        materials: Vec<ResMaterial>
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
