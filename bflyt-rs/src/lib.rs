use byteorder::{LittleEndian, ReadBytesExt}; // 1.2.7
use std::{
    fs::File,
    io::{Read, Seek, BufRead}
};
use nnsdk::ui2d::ResPane;

pub struct BflytHeader {
    signature: String,
    byte_order: u16,
    header_size: u16,
    version: u32,
    file_size: u32,
    section_count: u16,
    padding: u16,
}

pub struct BflytFile {
    header: BflytHeader
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
            let mut curr_signature = [0u8; 4];
            file.read_exact(&mut curr_signature)?;
            let curr_signature = std::str::from_utf8(&curr_signature)?;
            println!("Current Section Signature: {curr_signature}");
            let length = file.read_u32::<LittleEndian>()? as usize;
            println!("Length: {length}");
            let mut data = vec![0u8; length - 8];
            file.read_exact(&mut data)?;
            let mut data = std::io::Cursor::new(data);
            match curr_signature {
                "txl1" => {
                    let tex_count = data.read_i32::<LittleEndian>()?;
                    let base_offset = data.stream_position()?;
                    println!("Num Textures: {tex_count}, Base Offset: {base_offset}");

                    let mut offsets = vec![0i32; tex_count as usize];
                    // data.read_exact(&mut offsets);
                    data.read_i32_into::<LittleEndian>(offsets.as_mut_slice())?;
                    for offset in offsets {
                        data.seek(std::io::SeekFrom::Start(base_offset + offset as u64))?;
                        let mut bytes = Vec::new();
                        data.read_until(b'\0', &mut bytes)?;

                        // Convert the byte vector to a string
                        let texture_name = String::from_utf8(bytes).unwrap();
                        println!("Texture: {texture_name}");
                    }
                },
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
