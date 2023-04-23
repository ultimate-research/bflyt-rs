use byteorder::{LittleEndian, ReadBytesExt}; // 1.2.7
use nnsdk::ui2d::{
    ResColor, ResPane, ResPicture as ResPictureBase, ResTextBox as ResTextBoxBase, ResVec2, ResVec3,
};
use std::{
    fs::File,
    io::{BufRead, Cursor, Read, Seek, SeekFrom},
};

pub struct BflytHeader {
    signature: String,
    byte_order: u16,
    header_size: u16,
    version: u32,
    file_size: u32,
    section_count: u16,
    padding: u16,
}

#[derive(Debug)]
pub struct ResPicture {
    pub picture: ResPictureBase,
    pub tex_coords: Vec<Vec<ResVec2>>,
}

#[derive(Debug)]
pub struct ResAnimationInfo {
    pub count: u8,
    pub padding: [u8; 3],
}

#[derive(Debug)]
pub struct ResPerCharacterTransform {
    pub eval_time_offset: f32,
    pub eval_time_width: f32,
    pub has_animation_info: u8,
    pub loop_type: u8,
    pub origin_v: u8,
    pub padding: [u8; 1],
}

#[derive(Debug)]
pub struct ResTextBox {
    pub text_box: ResTextBoxBase,
    pub text: String,
    // pub text_id: [u8; 2],
    // pub line_width_offset_count: u8,
    // pub line_offset: f32,
    // pub line_width: f32,
    // pub res_per_character_transform: ResPerCharacterTransform,
    // pub per_character_transform_anim_info: ResAnimationInfo,
}

pub struct BflytFile {
    header: BflytHeader,
}

impl BflytFile {
    pub fn new_from_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;

        // Read the magic number
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if magic != "FLYT".as_bytes() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid magic number",
            )));
        }

        let signature = u32::from_le_bytes(magic);
        let byte_order = file.read_u16::<LittleEndian>()?;
        if byte_order == 0xFFE {
            panic!("BigEndian detected in file, not supported");
        }

        let header_size = file.read_u16::<LittleEndian>()?;
        let version = file.read_u32::<LittleEndian>()?;
        let file_size = file.read_u32::<LittleEndian>()?;
        let section_count = file.read_u16::<LittleEndian>()?;
        let padding = file.read_u16::<LittleEndian>()?;

        println!("{signature:x}, {byte_order:x}, {header_size:x}, {version:x}, {file_size:x}, {section_count:x}, {padding:x}");

        for _ in 0..section_count {
            let mut kind_bytes = [0u8; 4];
            file.read_exact(&mut kind_bytes)?;
            let kind = u32::from_le_bytes(kind_bytes);
            let size = file.read_u32::<LittleEndian>()?;
            let mut data = vec![0u8; size as usize - 8];
            file.read_exact(&mut data)?;
            let mut data = Cursor::new(data);

            let kind_str = std::str::from_utf8(&kind_bytes)?;

            match kind_str {
                "txl1" => {
                    println!("Current Section Signature: {kind_str}");
                    println!("Length: {size}");
                    let tex_count = data.read_i32::<LittleEndian>()?;
                    let base_offset = data.stream_position()?;
                    println!("Num Textures: {tex_count}, Base Offset: {base_offset}");

                    let mut offsets = vec![0i32; tex_count as usize];
                    // data.read_exact(&mut offsets);
                    data.read_i32_into::<LittleEndian>(offsets.as_mut_slice())?;
                    for offset in offsets {
                        data.seek(SeekFrom::Start(base_offset + offset as u64))?;
                        let mut bytes = Vec::new();
                        data.read_until(b'\0', &mut bytes)?;

                        // Convert the byte vector to a string
                        let texture_name = String::from_utf8(bytes).unwrap();
                        println!("Texture: {texture_name}");
                    }
                }
                "pan1" => {
                    println!("Kind: {kind_str}");
                    println!(
                        "Length: {size}; Expecting {}",
                        std::mem::size_of::<ResPane>()
                    );
                    let flag = data.read_u8()?;
                    let base_position = data.read_u8()?;
                    let alpha = data.read_u8()?;
                    let flag_ex = data.read_u8()?;
                    let mut name = [0u8; 24];
                    data.read_exact(&mut name)?;
                    let mut user_data = [0u8; 8];
                    data.read_exact(&mut user_data)?;

                    println!(
                        "Pane Name: {:#?}",
                        String::from_utf8(name.to_vec()).unwrap()
                    );

                    let pos_x = data.read_f32::<LittleEndian>()?;
                    let pos_y = data.read_f32::<LittleEndian>()?;
                    let pos_z = data.read_f32::<LittleEndian>()?;
                    let rot_x = data.read_f32::<LittleEndian>()?;
                    let rot_y = data.read_f32::<LittleEndian>()?;
                    let rot_z = data.read_f32::<LittleEndian>()?;
                    let scale_x = data.read_f32::<LittleEndian>()?;
                    let scale_y = data.read_f32::<LittleEndian>()?;
                    let size_x = data.read_f32::<LittleEndian>()?;
                    let size_y = data.read_f32::<LittleEndian>()?;

                    let res_pane = ResPane {
                        block_header_kind: kind,
                        block_header_size: size,
                        flag,
                        base_position,
                        alpha,
                        flag_ex,
                        name,
                        user_data,
                        pos: ResVec3 {
                            x: pos_x,
                            y: pos_y,
                            z: pos_z,
                        },
                        rot_x,
                        rot_y,
                        rot_z,
                        scale_x,
                        scale_y,
                        size_x,
                        size_y,
                    };

                    println!("{res_pane:#?}");
                }
                "pic1" => {
                    println!("Kind: {kind_str}");
                    println!(
                        "Length: {size}; Expecting {}",
                        std::mem::size_of::<ResPicture>()
                    );
                    let flag = data.read_u8()?;
                    let base_position = data.read_u8()?;
                    let alpha = data.read_u8()?;
                    let flag_ex = data.read_u8()?;
                    let mut name = [0u8; 24];
                    data.read_exact(&mut name)?;
                    let mut user_data = [0u8; 8];
                    data.read_exact(&mut user_data)?;

                    println!(
                        "Pane Name: {:#?}",
                        String::from_utf8(name.to_vec()).unwrap()
                    );

                    let pos_x = data.read_f32::<LittleEndian>()?;
                    let pos_y = data.read_f32::<LittleEndian>()?;
                    let pos_z = data.read_f32::<LittleEndian>()?;
                    let rot_x = data.read_f32::<LittleEndian>()?;
                    let rot_y = data.read_f32::<LittleEndian>()?;
                    let rot_z = data.read_f32::<LittleEndian>()?;
                    let scale_x = data.read_f32::<LittleEndian>()?;
                    let scale_y = data.read_f32::<LittleEndian>()?;
                    let size_x = data.read_f32::<LittleEndian>()?;
                    let size_y = data.read_f32::<LittleEndian>()?;
                    let mut vtx_cols = [[0u8; 4]; 4];

                    for i in 0..vtx_cols.len() {
                        data.read_exact(&mut vtx_cols[i]);
                    }

                    let material_idx = data.read_u16::<LittleEndian>()?;
                    let tex_coord_count = data.read_u8()?;
                    let flags = data.read_u8()?;

                    let mut texture_coords = Vec::new();
                    for i in 0..4 {
                        texture_coords.push(Vec::new());

                        for _j in 0..tex_coord_count {
                            let mut tex_coord = [0f32; 2];
                            data.read_f32_into::<LittleEndian>(&mut tex_coord)?;

                            texture_coords[i].push(ResVec2::new(tex_coord[0], tex_coord[1]));
                        }
                    }

                    let res_picture = ResPicture {
                        picture: ResPictureBase {
                            pane: ResPane {
                                block_header_kind: kind,
                                block_header_size: size,
                                flag,
                                base_position,
                                alpha,
                                flag_ex,
                                name,
                                user_data,
                                pos: ResVec3 {
                                    x: pos_x,
                                    y: pos_y,
                                    z: pos_z,
                                },
                                rot_x,
                                rot_y,
                                rot_z,
                                scale_x,
                                scale_y,
                                size_x,
                                size_y,
                            },
                            vtx_cols: vtx_cols.map(|color| ResColor {
                                r: color[0],
                                g: color[1],
                                b: color[2],
                                a: color[3],
                            }),
                            material_idx,
                            tex_coord_count,
                            flags,
                        },
                        tex_coords: texture_coords,
                    };

                    println!("{res_picture:#?}");
                }
                "txt1" => {
                    println!("Kind: {kind_str}");
                    println!(
                        "Length: {size}; Expecting {}",
                        std::mem::size_of::<ResTextBox>()
                    );

                    let flag = data.read_u8()?;
                    let base_position = data.read_u8()?;
                    let alpha = data.read_u8()?;
                    let flag_ex = data.read_u8()?;
                    let mut name = [0u8; 24];
                    data.read_exact(&mut name)?;
                    let mut user_data = [0u8; 8];
                    data.read_exact(&mut user_data)?;

                    println!(
                        "Pane Name: {:#?}",
                        String::from_utf8(name.to_vec()).unwrap()
                    );

                    let pos_x = data.read_f32::<LittleEndian>()?;
                    let pos_y = data.read_f32::<LittleEndian>()?;
                    let pos_z = data.read_f32::<LittleEndian>()?;
                    let rot_x = data.read_f32::<LittleEndian>()?;
                    let rot_y = data.read_f32::<LittleEndian>()?;
                    let rot_z = data.read_f32::<LittleEndian>()?;
                    let scale_x = data.read_f32::<LittleEndian>()?;
                    let scale_y = data.read_f32::<LittleEndian>()?;
                    let size_x = data.read_f32::<LittleEndian>()?;
                    let size_y = data.read_f32::<LittleEndian>()?;

                    // Beginning of TextBox specific fields
                    let text_length = data.read_u16::<LittleEndian>()?; // text length
                    let restricted_text_length = data.read_u16::<LittleEndian>()?; // restricted text legnth, whatever that means
                    let material_index = data.read_u16::<LittleEndian>()?;
                    let font_index = data.read_u16::<LittleEndian>()?;
                    let text_position = data.read_u8()?;
                    let text_alignment = data.read_u8()?;
                    let text_box_flag = data.read_u16::<LittleEndian>()?;
                    let italic_ratio = data.read_f32::<LittleEndian>()?;
                    let text_string_offset = data.read_u32::<LittleEndian>()?;

                    let mut text_colors = [[0u8; 4]; 2];
                    for i in 0..text_colors.len() {
                        data.read_exact(&mut text_colors[i]);
                    }

                    let mut font_size = [0f32; 2];
                    data.read_f32_into::<LittleEndian>(&mut font_size)?;

                    let char_space = data.read_f32::<LittleEndian>()?;
                    let line_space = data.read_f32::<LittleEndian>()?;
                    let text_id_offset = data.read_u32::<LittleEndian>()?;

                    let mut shadow_offset = [0f32; 2];
                    data.read_f32_into::<LittleEndian>(&mut shadow_offset)?;

                    let mut shadow_scale = [0f32; 2];
                    data.read_f32_into::<LittleEndian>(&mut shadow_scale)?;

                    let mut shadow_colors = [[0u8; 4]; 2];
                    for i in 0..shadow_colors.len() {
                        data.read_exact(&mut shadow_colors[i]);
                    }

                    let shadow_italic_ratio = data.read_f32::<LittleEndian>()?;
                    let line_width_offset_offset = data.read_u32::<LittleEndian>()?;
                    let per_character_transform_offset = data.read_u32::<LittleEndian>()?;

                    data.seek(SeekFrom::Start(text_string_offset as u64 - 8));
                    let mut text = Vec::<u8>::new();

                    for i in 0..text_length {
                        text.push(data.read_u8()?);
                    }

                    let res_text_box = ResTextBox {
                        text_box: ResTextBoxBase {
                            pane: ResPane {
                                block_header_kind: kind,
                                block_header_size: size,
                                flag,
                                base_position,
                                alpha,
                                flag_ex,
                                name,
                                user_data,
                                pos: ResVec3 {
                                    x: pos_x,
                                    y: pos_y,
                                    z: pos_z,
                                },
                                rot_x,
                                rot_y,
                                rot_z,
                                scale_x,
                                scale_y,
                                size_x,
                                size_y,
                            },
                            text_buf_bytes: text_length,
                            text_str_bytes: restricted_text_length,
                            material_idx: material_index,
                            font_idx: font_index,
                            text_position,
                            text_alignment,
                            text_box_flag,
                            italic_ratio,
                            text_str_offset: text_string_offset,
                            text_cols: text_colors.map(|[r, g, b, a]| ResColor { r, g, b, a }),
                            font_size: ResVec2::new(font_size[0], font_size[1]),
                            char_space,
                            line_space,
                            text_id_offset,
                            shadow_offset: ResVec2::new(shadow_offset[0], shadow_offset[1]),
                            shadow_scale: ResVec2::new(shadow_scale[0], shadow_scale[1]),
                            shadow_cols: shadow_colors.map(|[r, g, b, a]| ResColor { r, g, b, a }),
                            shadow_italic_ratio,
                            line_width_offset_offset,
                            per_character_transform_offset,
                        },
                        text: String::from_utf8(text)?,
                    };

                    println!("{res_text_box:#?}");
                    // panic!(
                    //     "text: {}\n{res_text_box:#?}",
                    //     String::from_utf8(res_text_box.text_box.pane.name.to_vec()).unwrap()
                    // );
                }
                _ => (),
            }
        }

        Ok(())
    }
}
// public BflytFile(Stream file) {
//     BinaryDataReader bin = new BinaryDataReader(file);
//     FileByteOrder = ByteOrder.LittleEndian;
//     bin.ByteOrder = FileByteOrder;
//     if (bin.ReadString(4) != "FLYT") throw new Exception("Wrong signature");
//     var bOrder = bin.ReadUInt16(); //BOM
//     if (bOrder == 0xFFFE)
//     {
//         FileByteOrder = ByteOrder.BigEndian;
//         bin.ByteOrder = FileByteOrder;
//     }
//     bin.ReadUInt16(); //HeaderSize
//     Version = bin.ReadUInt32();
//     bin.ReadUInt32(); //File size
//     var sectionCount = bin.ReadUInt16();
//     bin.ReadUInt16(); //padding

//     BasePane lastPane = null;
//     Stack<BasePane> currentRoot = new Stack<BasePane>();
//     void PushPane(BasePane p)
//     {
//         if (p.name == "pas1" || p.name == "grs1")
//             currentRoot.Push(lastPane);
//         else if (p.name == "pae1" || p.name == "gre1")
//             currentRoot.Pop();
//         else if (currentRoot.Count == 0)
//             RootPanes.Add(p);
//         else
//         {
//             p.Parent = currentRoot.Peek();
//             currentRoot.Peek().Children.Add(p);
//         }

//         lastPane = p;
//     }

//     for (int i = 0; i < sectionCount; i++)
//     {
//         string name = bin.ReadString(4);
//         switch (name)
//         {
//             case "txl1":
//                 PushPane(new TextureSection(bin));
//                 break;
//             case "mat1":
//                 PushPane(new MaterialsSection(bin, Version));
//                 break;
//             case "usd1":
//                 lastPane.UserData = new Usd1Pane(bin);
//                 break;
//             case "pic1":
//                 PushPane(new Pic1Pane(bin));
//                 break;
//             case "txt1":
//                 PushPane(new Txt1Pane(bin));
//                 break;
//             case "grp1":
//                 PushPane(new Grp1Pane(bin, Version));
//                 break;
//             case "prt1":
//                 PushPane(new Prt1Pane(bin, Version));
//                 break;
//             case "pan1":
//             case "wnd1":	case "bnd1":
//                 PushPane(new Pan1Pane(bin, name));
//                 break;
//             default:
//                 PushPane(new BasePane(name, bin));
//                 break;
//         }
//     }
// }