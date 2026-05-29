use std::io::{Read, Write, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::error::{Result, W3StringsError};
use crate::language::W3Language;
use crate::vlq::{read_bit6, write_bit6};

#[derive(Debug, Clone)]
pub struct W3StringBlock1 {
    pub str_id: u32,
    pub offset: u32,
    pub strlen: u32,
    pub str: String,
}

#[derive(Debug, Clone)]
pub struct W3StringBlock2 {
    pub str_key_hex: u32,
    pub str_id: u32,
}

#[derive(Debug, Clone)]
pub struct W3StringFile {
    pub version: u32,
    pub key1: u16,
    pub key2: u16,
    pub language: W3Language,
    pub block1: Vec<W3StringBlock1>,
    pub block2: Vec<W3StringBlock2>,
    pub block3count: i32,
}

impl W3StringFile {
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // 1. Read magic
        let mut magic_bytes = [0u8; 4];
        reader.read_exact(&mut magic_bytes)?;
        if &magic_bytes != b"RTSW" {
            return Err(W3StringsError::InvalidMagic);
        }

        // 2. Read version
        let version = reader.read_u32::<LittleEndian>()?;

        // 3. Read key1
        let key1 = reader.read_u16::<LittleEndian>()?;

        // 4. Seek to end to read key2
        reader.seek(SeekFrom::End(-2))?;
        let key2 = reader.read_u16::<LittleEndian>()?;

        // 5. Determine language
        let key = ((key1 as u32) << 16) | (key2 as u32);
        let language = W3Language::from_key(key)?;
        let magic = language.magic;

        // 6. Seek back to offset 10 to read block 1
        reader.seek(SeekFrom::Start(10))?;

        // 7. Read block 1
        let block1count = read_bit6(reader)?;
        let mut block1 = Vec::with_capacity(block1count as usize);
        for _ in 0..block1count {
            let str_id_hashed = reader.read_u32::<LittleEndian>()?;
            let str_id = str_id_hashed ^ magic;
            let offset = reader.read_u32::<LittleEndian>()?;
            let strlen = reader.read_u32::<LittleEndian>()?;
            block1.push(W3StringBlock1 {
                str_id,
                offset,
                strlen,
                str: String::new(),
            });
        }

        // 8. Read block 2
        let block2count = read_bit6(reader)?;
        let mut block2 = Vec::with_capacity(block2count as usize);
        for _ in 0..block2count {
            let str_key_hex = reader.read_u32::<LittleEndian>()?;
            let str_id_hashed = reader.read_u32::<LittleEndian>()?;
            let str_id = str_id_hashed ^ magic;
            block2.push(W3StringBlock2 {
                str_key_hex,
                str_id,
            });
        }

        // 9. Read block 3 count
        let block3count = read_bit6(reader)?;
        let str_start = reader.stream_position()?;

        // 10. Read and decrypt strings
        let string_key_init = ((magic >> 8) & 0xffff) as u16;
        for block in &mut block1 {
            let offset = block.offset as u64 * 2 + str_start;
            reader.seek(SeekFrom::Start(offset))?;

            let mut string_key = string_key_init;
            let mut utf16_chars = Vec::with_capacity(block.strlen as usize);

            for _ in 0..block.strlen {
                let mut buf = [0u8; 2];
                reader.read_exact(&mut buf)?;
                let mut b1 = buf[0];
                let mut b2 = buf[1];

                let char_key = (((block.strlen + 1) as u32 * string_key as u32) & 0xffff) as u16;
                b1 ^= (char_key & 0xff) as u8;
                b2 ^= ((char_key >> 8) & 0xff) as u8;

                string_key = ((string_key << 1) | (string_key >> 15)) as u16;

                let char_val = (b1 as u16) | ((b2 as u16) << 8);
                utf16_chars.push(char_val);
            }

            block.str = String::from_utf16(&utf16_chars)?;
        }

        // Sort block1 and block2 to maintain order consistency
        block1.sort_by_key(|b| b.str_id ^ magic);
        block2.sort_by_key(|b| b.str_key_hex);

        Ok(W3StringFile {
            version,
            key1,
            key2,
            language,
            block1,
            block2,
            block3count,
        })
    }

    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        let magic = self.language.magic;

        // Sort data models first (ensures consistent order in output)
        let mut sorted_block1 = self.block1.clone();
        sorted_block1.sort_by_key(|b| b.str_id ^ magic);

        let mut sorted_block2 = self.block2.clone();
        sorted_block2.sort_by_key(|b| b.str_key_hex);

        // 1. Write magic
        writer.write_all(b"RTSW")?;

        // 2. Write version
        writer.write_u32::<LittleEndian>(self.version)?;

        // 3. Write key1
        writer.write_u16::<LittleEndian>(self.key1)?;

        // Write string buffer to memory first to compute offsets and size
        let mut string_buffer = Vec::new();
        let string_key_init = ((magic >> 8) & 0xffff) as u16;

        for block in &mut sorted_block1 {
            block.offset = (string_buffer.len() / 2) as u32;
            let utf16_chars: Vec<u16> = block.str.encode_utf16().collect();
            block.strlen = utf16_chars.len() as u32;

            let mut string_key = string_key_init;
            for &char_val in &utf16_chars {
                let mut b1 = (char_val & 255) as u8;
                let mut b2 = (char_val >> 8) as u8;

                let char_key = (((block.strlen + 1) as u32 * string_key as u32) & 0xffff) as u16;
                b1 ^= (char_key & 0xff) as u8;
                b2 ^= ((char_key >> 8) & 0xff) as u8;

                string_key = ((string_key << 1) | (string_key >> 15)) as u16;

                string_buffer.write_all(&[b1, b2])?;
            }

            // Write 0x0000 null-terminator for string
            string_buffer.write_all(&[0, 0])?;
        }

        // 4. Write block 1 count and entries
        write_bit6(writer, sorted_block1.len() as i32)?;
        for block in &sorted_block1 {
            let str_id_hashed = block.str_id ^ magic;
            writer.write_u32::<LittleEndian>(str_id_hashed)?;
            writer.write_u32::<LittleEndian>(block.offset)?;
            writer.write_u32::<LittleEndian>(block.strlen)?;
        }

        // 5. Write block 2 count and entries
        write_bit6(writer, sorted_block2.len() as i32)?;
        for block in &sorted_block2 {
            writer.write_u32::<LittleEndian>(block.str_key_hex)?;
            let str_id_hashed = block.str_id ^ magic;
            writer.write_u32::<LittleEndian>(str_id_hashed)?;
        }

        // 6. Write block 3 count (string buffer size in 2-byte units)
        let block3count = (string_buffer.len() / 2) as i32;
        write_bit6(writer, block3count)?;

        // 7. Write string buffer
        writer.write_all(&string_buffer)?;

        // 8. Write key2
        writer.write_u16::<LittleEndian>(self.key2)?;

        Ok(())
    }
}
