use byteorder::{LittleEndian, BigEndian, ReadBytesExt}; // 1.2.7
use std::{
    fs::File,
    io::{self, Read},
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

pub struct BflytFile {
    header: BflytHeader
}


impl BflytFile {
    pub fn new_from_file(filename: &str) -> Result<(), std::io::Error> {
        let mut file = File::open(filename)?;

        // Read the magic number
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if magic != "FLYT".as_bytes() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid magic number",
            ));
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
            // match curr_signature {
            //     "TXL1" => ,

            // }

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
