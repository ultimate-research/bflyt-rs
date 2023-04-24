use binrw::io::{Cursor, SeekFrom, TakeSeekExt};
use binrw::meta::{EndianKind, ReadEndian};
use binrw::{binread, BinRead, BinReaderExt, BinResult, NullString, Endian};
use byteorder::{LittleEndian, ReadBytesExt}; // 1.2.7
use nnsdk::ui2d::{
    ResColor, ResPane, ResPicture as ResPictureBase, ResTextBox as ResTextBoxBase, ResVec2, ResVec3,
};
use std::{
    fs::File,
    io::{BufRead, Read, Seek},
};

#[derive(Debug)]
#[binread]
#[br(little, magic = b"FLYT")]
pub struct BflytFile {
    header: BflytHeader,
    #[br(count = header.section_count)]
    sections: Vec<BflytSection>,
}

#[binread]
#[derive(Debug)]
pub struct BflytHeader {
    byte_order: u16,
    header_size: u16,
    version: u32,
    file_size: u32,
    section_count: u16,
    padding: u16
}

#[repr(C)]
#[derive(BinRead, Debug, Copy, Clone)]
pub struct ResColorTest {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
#[derive(BinRead, Debug, Copy, Clone)]
pub struct ResVec2Test {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(BinRead, Debug, Copy, Clone)]
pub struct ResVec3Test {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(BinRead, Debug, Copy, Clone)]
pub struct ResPaneTest {
    pub flag: u8,
    pub base_position: u8,
    pub alpha: u8,
    pub flag_ex: u8,
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

fn texture_list_parser<R: Read + Seek>(reader: &mut R, _: Endian, _: ()) -> BinResult<TextureListInner> {
    let mut texture_names: Vec<NullString> = Vec::new();

    let tex_count = reader.read_i32::<LittleEndian>()?;
    let base_offset = reader.stream_position()?;

    let mut offsets = vec![0i32; tex_count as usize];
    reader.read_i32_into::<LittleEndian>(offsets.as_mut_slice())?;
    for offset in &offsets {
        reader.seek(SeekFrom::Start(base_offset + *offset as u64))?;
        texture_names.push(NullString::read(reader)?);
    }

    Ok(TextureListInner { tex_count, offsets, texture_names })
}

#[repr(C)]
#[derive(BinRead, Debug)]
pub struct TextureListInner {
    pub tex_count: i32,
    #[br(count = tex_count)]
    pub offsets: Vec<i32>,
    #[br(count = tex_count)]
    pub texture_names: Vec<NullString>
}

#[repr(C)]
#[derive(BinRead, Debug, Clone)]
pub struct ResPictureTest {
    pub pane: ResPaneTest,
    pub vtx_cols: [ResColorTest; 4],
    pub material_idx: u16,
    pub tex_coord_count: u8,
    pub flags: u8,
    #[br(count = tex_coord_count)]
    pub tex_coords: Vec<[ResVec2Test; 4]>,
}

#[derive(BinRead, Debug, Default)]
pub struct ResAnimationInfo {
    pub kind: u32,
    pub count: u8,
    pub padding: [u8; 3],
}

impl ReadEndian for ResAnimationInfo {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

#[derive(BinRead, Debug)]
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

#[derive(BinRead, Debug)]
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
    pub text_id: NullString,
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

#[derive(BinRead, Debug)]
enum BflytSection {
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
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"mat1")]
    Material {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"wnd1")]
    Window {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"pas1")]
    PaneStart {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"pae1")]
    PaneEnd {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"grp1")]
    Group {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"grs1")]
    GroupStart {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"gre1")]
    GroupEnd {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"bnd1")]
    Bounding {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"lyt1")]
    Layout {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"fnl1")]
    FontList {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    #[br(magic = b"usd1")]
    UserDataList {
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },

    Unknown {
        pane_type: [u8; 4],
        size: u32,
        #[br(count = size as usize - 8)]
        data: Vec<u8>,
    },
}

impl BflytFile {
    pub fn new_from_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        let bflyt = BflytFile::read(&mut file).unwrap();

        println!(
            "byte_order: {:x}, header_size: {:x}, version: {:x}, file_size: {:x}, section_count: {:x}",
            bflyt.header.byte_order,
            bflyt.header.header_size,
            bflyt.header.version,
            bflyt.header.file_size,
            bflyt.header.section_count
        );

        for section in bflyt.sections {
            match section {
                // BflytSection::TextureList { size, .. } => println!("{:#?}", size),
                // BflytSection::Pane { size, .. } => println!("{:#?}", size),
                // BflytSection::Picture { size, .. } => println!("{:#?}", size),
                // BflytSection::TextBox { .. } => println!("{section:#?}"),
                // BflytSection::Material { size, data } => println!("{:#?}", size),
                // BflytSection::Window { size, data } => println!("{:#?}", size),
                // BflytSection::Part { size, data } => println!("{:#?}", size),
                // BflytSection::PaneStart { size, data } => println!("{:#?}", size),
                // BflytSection::PaneEnd { size, data } => println!("{:#?}", size),
                // BflytSection::Group { size, data } => println!("{:#?}", size),
                // BflytSection::GroupStart { size, data } => println!("{:#?}", size),
                // BflytSection::GroupEnd { size, data } => println!("{:#?}", size),
                // BflytSection::Bounding { size, data } => println!("{:#?}", size),
                // BflytSection::Layout { size, data } => println!("{:#?}", size),
                // BflytSection::FontList { size, data } => println!("{:#?}", size),
                // BflytSection::UserDataList { size, data } => println!("{:#?}", size),
                BflytSection::Unknown {
                    pane_type,
                    size,
                    data,
                } => println!("{:#?}", String::from_utf8(pane_type.to_vec()).unwrap()),
                _ => (),
            }
        }

        Ok(())
    }
}
