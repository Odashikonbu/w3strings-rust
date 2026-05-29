use std::io::{Read, Write};
use crate::error::{Result, W3StringsError};

pub fn read_bit6<R: Read>(reader: &mut R) -> Result<i32> {
    let mut result = 0i32;
    let mut shift = 0;
    let mut i = 1;
    
    loop {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let b = buf[0];
        
        let mut s = 6;
        let mut mask = 255u8;
        
        if b > 127 {
            mask = 127;
            s = 7;
        } else if b > 63 {
            if i == 1 {
                mask = 63;
            }
        }
        
        // We need to make sure shift is safe (avoiding overflow)
        if shift < 32 {
            result |= ((b & mask) as i32) << shift;
        }
        shift += s;
        
        if b < 64 || (i >= 3 && b < 128) {
            break;
        }
        
        i += 1;
    }
    
    Ok(result)
}

pub fn write_bit6<W: Write>(writer: &mut W, c: i32) -> Result<u32> {
    if c == 0 {
        writer.write_all(&[128])?;
        return Ok(1);
    }
    
    let mut bytes = Vec::new();
    let mut left = c;
    let mut i = 0;
    while left > 0 {
        if i == 0 {
            bytes.push(left & 63);
            left >>= 6;
        } else {
            bytes.push(left & 255);
            left >>= 7;
        }
        i += 1;
    }
    
    let mut written = 0;
    let len = bytes.len();
    for i in 0..len {
        let last = i == len - 1;
        let cleft = (len - 1) - i;
        
        let mut val = bytes[i];
        if !last {
            if cleft >= 1 && i >= 1 {
                val |= 128;
            } else if val < 64 {
                val |= 64;
            } else {
                val |= 128;
            }
        }
        
        if val == 128 {
            return Err(W3StringsError::InvalidVlq);
        }
        
        writer.write_all(&[val as u8])?;
        written += 1;
    }
    
    Ok(written)
}
