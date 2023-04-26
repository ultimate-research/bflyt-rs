use binrw::io::{Cursor, SeekFrom, TakeSeekExt};
use binrw::meta::{EndianKind, ReadEndian};
use binrw::{binread, BinRead, BinResult, NullString, Endian};
use byteorder::{LittleEndian, ReadBytesExt}; // 1.2.7
use nnsdk::ui2d::{
    ResColor, ResPane, ResPicture as ResPictureBase, ResTextBox as ResTextBoxBase, ResVec2, ResVec3
};
use serde::{Serialize, Serializer};
use std::{
    fs::File,
    io::{Read, Seek},
};

#[derive(Debug, BinRead, Default, Clone)]
pub struct SerdeNullString(NullString);

impl Serialize for SerdeNullString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

pub unsafe fn str_from_u8_nul_utf8_unchecked(utf8_src: &[u8]) -> &str {
    let nul_range_end = utf8_src.iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    ::std::str::from_utf8_unchecked(&utf8_src[0..nul_range_end])
}

fn cstr_deserialize<S>(x: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(unsafe {
        str_from_u8_nul_utf8_unchecked(x)
    })
}

impl ReadEndian for SerdeNullString {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}


#[derive(Serialize, Debug)]
#[binread]
#[br(little, magic = b"FLYT")]
pub struct BflytFile {
    header: BflytHeader,
    #[br(count = header.section_count)]
    sections: Vec<BflytSection>,
}

#[binread]
#[derive(Serialize, Debug)]
pub struct BflytHeader {
    byte_order: u16,
    header_size: u16,
    version: u32,
    file_size: u32,
    section_count: u16,
    padding: u16
}

#[repr(C)]
#[derive(Serialize, BinRead, Debug, Copy, Clone)]
pub struct ResColorTest {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
#[derive(Serialize, BinRead, Debug, Copy, Clone)]
pub struct ResVec2Test {
    pub x: f32,
    pub y: f32,
}

impl ReadEndian for ResVec2Test {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[repr(C)]
#[derive(Serialize, BinRead, Debug, Copy, Clone)]
pub struct ResVec3Test {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ReadEndian for ResVec3Test {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[repr(C)]
#[derive(Serialize, BinRead, Debug, Clone)]
pub struct ResPaneTest {
    pub flag: u8,
    pub base_position: u8,
    pub alpha: u8,
    pub flag_ex: u8,
    #[serde(serialize_with = "cstr_deserialize")]
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
#[derive(Serialize, BinRead, Debug)]
pub struct TextureListInner {
    pub tex_count: i32,
    #[br(count = tex_count)]
    pub offsets: Vec<i32>,
    #[br(count = tex_count)]
    pub texture_names: Vec<SerdeNullString>
}

#[repr(C)]
#[derive(Serialize, BinRead, Debug, Clone)]
pub struct ResPictureTest {
    pub pane: ResPaneTest,
    pub vtx_cols: [ResColorTest; 4],
    pub material_idx: u16,
    pub tex_coord_count: u8,
    pub flags: u8,
    #[br(count = tex_coord_count)]
    pub tex_coords: Vec<[ResVec2Test; 4]>,
}

#[derive(Serialize, BinRead, Debug, Default)]
pub struct ResAnimationInfo {
    pub kind: u32,
    pub count: u8,
    pub padding: [u8; 3],
}

impl ReadEndian for ResAnimationInfo {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[derive(Serialize, BinRead, Debug)]
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

#[derive(Serialize, BinRead, Debug)]
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
    #[br(
        if(text_id_offset > 0), 
        // Can't use absolute offsets, so... we know it's after the text string.
        seek_before = SeekFrom::Current((text_id_offset as u64 - (text_str_offset as u64 + text_buf_bytes as u64)) as i64)
    )]
    pub text_id: SerdeNullString,
    // Not sure if any of the following work, so if something breaks, check here.
    #[br(if(line_width_offset_offset > 0))]
    pub line_width_offset_count: u8,
    #[br(if(line_width_offset_offset > 0), count = line_width_offset_count)]
    pub line_offset: Vec<f32>,
    #[br(if(line_width_offset_offset > 0), count = line_width_offset_count)]
    pub line_width: Vec<f32>,
    #[br(if(per_character_transform_offset > 0))]
    pub per_character_transform: Option<ResPerCharacterTransform>,
    #[br(if(per_character_transform_offset > 0))]
    pub per_character_transform_animation_info: Option<ResAnimationInfo>,
}

#[repr(C)]
#[derive(Serialize, BinRead, Debug)]
pub struct ResPartsProperty {
    #[serde(serialize_with = "cstr_deserialize")]
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
#[derive(Serialize, BinRead, Debug)]
pub struct ResPartsTest {
    pub size: u32,
    pub pane: ResPaneTest,
    pub property_count: u32,
    pub magnify: ResVec2Test,
    #[br(count = property_count)]
    pub properties: Vec<ResPartsProperty>,
    #[br(dbg, align_after = 4)]
    pub part_name: SerdeNullString,
    // Not actually
    #[br(count = property_count)]
    pub sections: Vec<BflytSection>
}

#[repr(C)]
#[derive(Serialize, BinRead, Debug)]
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
    assert!(curr_pos == base_offset + size as u64, "Failed to parse ResParts. Expected to read {size} bytes, but read {}", curr_pos - base_offset);

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

#[derive(Serialize, BinRead, Debug)]
pub enum BflytSection {
    #[br(magic = b"pan1")]
    Pane {
        size: u32,
        pane: ResPaneTest
    },

    #[br(magic = b"txl1")]
    TextureList {
        size: u32,
        #[br(parse_with = texture_list_parser, align_after = 4)]
        texture_list: TextureListInner
    },

    #[br(magic = b"pic1")]
    Picture {
        size: u32,
        picture: ResPictureTest
    },

    #[br(magic = b"txt1")]
    TextBox {
        size: u32,
        #[br(align_after = 4)]
        text_box: ResTextBoxTest
    },

    #[br(magic = b"prt1")]
    Part {
        #[br(parse_with = res_parts_parser)]
        part: ResPartsTest,
    },

    #[br(magic = b"mat1")]
    Material {
        size: u32,
        #[serde(skip_serializing)]
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"wnd1")]
    Window {
        size: u32,
        #[serde(skip_serializing)]
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"pas1")]
    PaneStart {
        #[br(assert(size == 8))]
        #[serde(skip_serializing)]
        size: u32,
    },

    #[br(magic = b"pae1")]
    PaneEnd {
        #[br(assert(size == 8))]
        #[serde(skip_serializing)]
        size: u32,
    },

    #[br(magic = b"grp1")]
    Group {
        size: u32,
        #[serde(skip_serializing)]
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"grs1")]
    GroupStart {
        #[br(assert(size == 8))]
        #[serde(skip_serializing)]
        size: u32,
    },

    #[br(magic = b"gre1")]
    GroupEnd {
        #[br(assert(size == 8))]
        #[serde(skip_serializing)]
        size: u32,
    },

    #[br(magic = b"bnd1")]
    Bounding {
        size: u32,
        #[serde(skip_serializing)]
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"lyt1")]
    Layout {
        size: u32,
        #[serde(skip_serializing)]
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"fnl1")]
    FontList {
        size: u32,
        #[serde(skip_serializing)]
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"usd1")]
    UserDataList {
        size: u32,
        #[serde(skip_serializing)]
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
}
