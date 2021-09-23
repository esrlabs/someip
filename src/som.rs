// wip: someip types

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::any::Any;
use ux::{i24, u24};

pub trait SOMType {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError>;
    fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError>;
    fn size(&self) -> usize;
}

#[derive(Debug)]
pub enum SOMError {
    BufferExhausted(String),
    InvalidPayload(String),
    InvalidType(String),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SOMEndian {
    Big,
    Little,
}

pub trait SOMTypeWithEndian {
    fn endian(&self) -> SOMEndian;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SOMLengthField {
    None,
    U8,
    U16,
    U32,
}

impl SOMLengthField {
    fn size(&self) -> usize {
        match self {
            SOMLengthField::None => 0usize,
            SOMLengthField::U8 => std::mem::size_of::<u8>(),
            SOMLengthField::U16 => std::mem::size_of::<u16>(),
            SOMLengthField::U32 => std::mem::size_of::<u32>(),
        }
    }
}

pub trait SOMTypeWithLengthField {
    fn lengthfield(&self) -> SOMLengthField;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SOMTypeField {
    U8,
    U16,
    U32,
}

impl SOMTypeField {
    fn size(&self) -> usize {
        match self {
            SOMTypeField::U8 => std::mem::size_of::<u8>(),
            SOMTypeField::U16 => std::mem::size_of::<u16>(),
            SOMTypeField::U32 => std::mem::size_of::<u32>(),
        }
    }
}

pub trait SOMTypeWithTypeField {
    fn typefield(&self) -> SOMTypeField;
}

pub struct SOMSerializer<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}

struct SOMSerializerPromise {
    offset: usize,
    size: usize,
}

impl<'a> SOMSerializer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> SOMSerializer<'a> {
        SOMSerializer { buffer, offset: 0 }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn promise(&mut self, size: usize) -> Result<SOMSerializerPromise, SOMError> {
        self.check_size(size)?;
        let result = SOMSerializerPromise {
            offset: self.offset,
            size,
        };
        self.offset += size;
        Ok(result)
    }

    fn write_lengthfield(
        &mut self,
        promise: SOMSerializerPromise,
        lengthfield: SOMLengthField,
        value: usize,
    ) -> Result<(), SOMError> {
        if promise.size != lengthfield.size() {
            return Err(SOMError::InvalidType(format!(
                "Invalid Length-Field size: {} at offset: {}",
                lengthfield.size(),
                promise.offset
            )));
        }

        match lengthfield {
            SOMLengthField::None => {}
            SOMLengthField::U8 => self.buffer[promise.offset] = value as u8,
            SOMLengthField::U16 => {
                BigEndian::write_u16(&mut self.buffer[promise.offset..], value as u16)
            }
            SOMLengthField::U32 => {
                BigEndian::write_u32(&mut self.buffer[promise.offset..], value as u32)
            }
        };

        Ok(())
    }

    fn write_typefield(&mut self, typefield: SOMTypeField, value: usize) -> Result<(), SOMError> {
        match typefield {
            SOMTypeField::U8 => self.write_u8(value as u8)?,
            SOMTypeField::U16 => self.write_u16(value as u16, SOMEndian::Big)?,
            SOMTypeField::U32 => self.write_u32(value as u32, SOMEndian::Big)?,
        };

        Ok(())
    }

    fn write_bool(&mut self, value: bool) -> Result<(), SOMError> {
        let size = std::mem::size_of::<bool>();
        self.check_size(size)?;

        self.buffer[self.offset] = match value {
            true => 1,
            false => 0,
        };

        self.offset += size;
        Ok(())
    }

    fn write_u8(&mut self, value: u8) -> Result<(), SOMError> {
        let size = std::mem::size_of::<u8>();
        self.check_size(size)?;

        self.buffer[self.offset] = value;

        self.offset += size;
        Ok(())
    }

    fn write_i8(&mut self, value: i8) -> Result<(), SOMError> {
        let size = std::mem::size_of::<i8>();
        self.check_size(size)?;

        self.buffer[self.offset] = value as u8;

        self.offset += size;
        Ok(())
    }

    fn write_u16(&mut self, value: u16, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<u16>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => BigEndian::write_u16(&mut self.buffer[self.offset..], value),
            SOMEndian::Little => LittleEndian::write_u16(&mut self.buffer[self.offset..], value),
        }

        self.offset += size;
        Ok(())
    }

    fn write_i16(&mut self, value: i16, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<i16>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => BigEndian::write_i16(&mut self.buffer[self.offset..], value),
            SOMEndian::Little => LittleEndian::write_i16(&mut self.buffer[self.offset..], value),
        }

        self.offset += size;
        Ok(())
    }

    fn write_u24(&mut self, value: u24, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<u16>() + std::mem::size_of::<u8>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => {
                BigEndian::write_uint(&mut self.buffer[self.offset..], u64::from(value), size)
            }
            SOMEndian::Little => {
                LittleEndian::write_uint(&mut self.buffer[self.offset..], u64::from(value), size)
            }
        }

        self.offset += size;
        Ok(())
    }

    fn write_i24(&mut self, value: i24, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<i16>() + std::mem::size_of::<i8>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => {
                BigEndian::write_int(&mut self.buffer[self.offset..], i64::from(value), size)
            }
            SOMEndian::Little => {
                LittleEndian::write_int(&mut self.buffer[self.offset..], i64::from(value), size)
            }
        }

        self.offset += size;
        Ok(())
    }

    fn write_u32(&mut self, value: u32, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<u32>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => BigEndian::write_u32(&mut self.buffer[self.offset..], value),
            SOMEndian::Little => LittleEndian::write_u32(&mut self.buffer[self.offset..], value),
        }

        self.offset += size;
        Ok(())
    }

    fn write_i32(&mut self, value: i32, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<i32>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => BigEndian::write_i32(&mut self.buffer[self.offset..], value),
            SOMEndian::Little => LittleEndian::write_i32(&mut self.buffer[self.offset..], value),
        }

        self.offset += size;
        Ok(())
    }

    fn write_u64(&mut self, value: u64, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<u64>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => BigEndian::write_u64(&mut self.buffer[self.offset..], value),
            SOMEndian::Little => LittleEndian::write_u64(&mut self.buffer[self.offset..], value),
        }

        self.offset += size;
        Ok(())
    }

    fn write_i64(&mut self, value: i64, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<i64>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => BigEndian::write_i64(&mut self.buffer[self.offset..], value),
            SOMEndian::Little => LittleEndian::write_i64(&mut self.buffer[self.offset..], value),
        }

        self.offset += size;
        Ok(())
    }

    fn write_f32(&mut self, value: f32, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<f32>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => BigEndian::write_f32(&mut self.buffer[self.offset..], value),
            SOMEndian::Little => LittleEndian::write_f32(&mut self.buffer[self.offset..], value),
        }

        self.offset += size;
        Ok(())
    }

    fn write_f64(&mut self, value: f64, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<f64>();
        self.check_size(size)?;

        match endian {
            SOMEndian::Big => BigEndian::write_f64(&mut self.buffer[self.offset..], value),
            SOMEndian::Little => LittleEndian::write_f64(&mut self.buffer[self.offset..], value),
        }

        self.offset += size;
        Ok(())
    }

    fn check_size(&self, size: usize) -> Result<(), SOMError> {
        if self.buffer.len() < (self.offset + size) {
            return Err(SOMError::BufferExhausted(format!(
                "Serializer exausted at offset: {} for Object size: {}",
                self.offset, size
            )));
        }

        Ok(())
    }
}

pub struct SOMParser<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> SOMParser<'a> {
    pub fn new(buffer: &'a [u8]) -> SOMParser<'a> {
        SOMParser { buffer, offset: 0 }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn read_lengthfield(&mut self, lengthfield: SOMLengthField) -> Result<usize, SOMError> {
        let size = lengthfield.size();
        self.check_size(size)?;

        let result = match lengthfield {
            SOMLengthField::None => 0usize,
            SOMLengthField::U8 => self.read_u8()? as usize,
            SOMLengthField::U16 => self.read_u16(SOMEndian::Big)? as usize,
            SOMLengthField::U32 => self.read_u32(SOMEndian::Big)? as usize,
        };

        Ok(result)
    }

    fn read_typefield(&mut self, typefield: &mut SOMTypeField) -> Result<usize, SOMError> {
        let size = typefield.size();
        self.check_size(size)?;

        let result = match typefield {
            SOMTypeField::U8 => self.read_u8()? as usize,
            SOMTypeField::U16 => self.read_u16(SOMEndian::Big)? as usize,
            SOMTypeField::U32 => self.read_u32(SOMEndian::Big)? as usize,
        };

        Ok(result)
    }

    fn read_bool(&mut self) -> Result<bool, SOMError> {
        let size = std::mem::size_of::<bool>();
        self.check_size(size)?;

        let value = self.buffer[self.offset];
        let result = match value {
            1 => true,
            0 => false,
            _ => {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Bool value: {} at offset: {}",
                    value, self.offset
                )))
            }
        };

        self.offset += size;
        Ok(result)
    }

    fn read_u8(&mut self) -> Result<u8, SOMError> {
        let size = std::mem::size_of::<u8>();
        self.check_size(size)?;

        let result = self.buffer[self.offset];

        self.offset += size;
        Ok(result)
    }

    fn read_i8(&mut self) -> Result<i8, SOMError> {
        let size = std::mem::size_of::<i8>();
        self.check_size(size)?;

        let result = self.buffer[self.offset] as i8;

        self.offset += size;
        Ok(result)
    }

    fn read_u16(&mut self, endian: SOMEndian) -> Result<u16, SOMError> {
        let size = std::mem::size_of::<u16>();
        self.check_size(size)?;

        let result = match endian {
            SOMEndian::Big => BigEndian::read_u16(&self.buffer[self.offset..]),
            SOMEndian::Little => LittleEndian::read_u16(&self.buffer[self.offset..]),
        };

        self.offset += size;
        Ok(result)
    }

    fn read_i16(&mut self, endian: SOMEndian) -> Result<i16, SOMError> {
        let size = std::mem::size_of::<i16>();
        self.check_size(size)?;

        let result = match endian {
            SOMEndian::Big => BigEndian::read_i16(&self.buffer[self.offset..]),
            SOMEndian::Little => LittleEndian::read_i16(&self.buffer[self.offset..]),
        };

        self.offset += size;
        Ok(result)
    }

    fn read_u24(&mut self, endian: SOMEndian) -> Result<u24, SOMError> {
        let size = std::mem::size_of::<u16>() + std::mem::size_of::<u8>();
        self.check_size(size)?;

        let result = u24::new(match endian {
            SOMEndian::Big => BigEndian::read_uint(&self.buffer[self.offset..], size),
            SOMEndian::Little => LittleEndian::read_uint(&self.buffer[self.offset..], size),
        } as u32);

        self.offset += size;
        Ok(result)
    }

    fn read_i24(&mut self, endian: SOMEndian) -> Result<i24, SOMError> {
        let size = std::mem::size_of::<i16>() + std::mem::size_of::<i8>();
        self.check_size(size)?;

        let result = i24::new(match endian {
            SOMEndian::Big => BigEndian::read_int(&self.buffer[self.offset..], size),
            SOMEndian::Little => LittleEndian::read_int(&self.buffer[self.offset..], size),
        } as i32);

        self.offset += size;
        Ok(result)
    }

    fn read_u32(&mut self, endian: SOMEndian) -> Result<u32, SOMError> {
        let size = std::mem::size_of::<u32>();
        self.check_size(size)?;

        let result = match endian {
            SOMEndian::Big => BigEndian::read_u32(&self.buffer[self.offset..]),
            SOMEndian::Little => LittleEndian::read_u32(&self.buffer[self.offset..]),
        };

        self.offset += size;
        Ok(result)
    }

    fn read_i32(&mut self, endian: SOMEndian) -> Result<i32, SOMError> {
        let size = std::mem::size_of::<i32>();
        self.check_size(size)?;

        let result = match endian {
            SOMEndian::Big => BigEndian::read_i32(&self.buffer[self.offset..]),
            SOMEndian::Little => LittleEndian::read_i32(&self.buffer[self.offset..]),
        };

        self.offset += size;
        Ok(result)
    }

    fn read_u64(&mut self, endian: SOMEndian) -> Result<u64, SOMError> {
        let size = std::mem::size_of::<u64>();
        self.check_size(size)?;

        let result = match endian {
            SOMEndian::Big => BigEndian::read_u64(&self.buffer[self.offset..]),
            SOMEndian::Little => LittleEndian::read_u64(&self.buffer[self.offset..]),
        };

        self.offset += size;
        Ok(result)
    }

    fn read_i64(&mut self, endian: SOMEndian) -> Result<i64, SOMError> {
        let size = std::mem::size_of::<i64>();
        self.check_size(size)?;

        let result = match endian {
            SOMEndian::Big => BigEndian::read_i64(&self.buffer[self.offset..]),
            SOMEndian::Little => LittleEndian::read_i64(&self.buffer[self.offset..]),
        };

        self.offset += size;
        Ok(result)
    }

    fn read_f32(&mut self, endian: SOMEndian) -> Result<f32, SOMError> {
        let size = std::mem::size_of::<f32>();
        self.check_size(size)?;

        let result = match endian {
            SOMEndian::Big => BigEndian::read_f32(&self.buffer[self.offset..]),
            SOMEndian::Little => LittleEndian::read_f32(&self.buffer[self.offset..]),
        };

        self.offset += size;
        Ok(result)
    }

    fn read_f64(&mut self, endian: SOMEndian) -> Result<f64, SOMError> {
        let size = std::mem::size_of::<f64>();
        self.check_size(size)?;

        let result = match endian {
            SOMEndian::Big => BigEndian::read_f64(&self.buffer[self.offset..]),
            SOMEndian::Little => LittleEndian::read_f64(&self.buffer[self.offset..]),
        };

        self.offset += size;
        Ok(result)
    }

    fn check_size(&self, size: usize) -> Result<(), SOMError> {
        if self.buffer.len() < (self.offset + size) {
            return Err(SOMError::BufferExhausted(format!(
                "Parser exausted at offset: {} for Object size: {}",
                self.offset, size
            )));
        }

        Ok(())
    }
}

mod primitives {
    use super::*;

    #[derive(Debug)]
    pub struct SOMPrimitiveType<T> {
        pub endian: SOMEndian,
        pub value: Option<T>,
    }

    impl<T: Copy + Clone + PartialEq> SOMPrimitiveType<T> {
        pub fn empty(endian: SOMEndian) -> SOMPrimitiveType<T> {
            SOMPrimitiveType {
                endian,
                value: None,
            }
        }

        pub fn new(endian: SOMEndian, value: T) -> SOMPrimitiveType<T> {
            SOMPrimitiveType {
                endian,
                value: Some(value),
            }
        }

        pub fn set(&mut self, value: T) {
            self.value = Some(value);
        }

        pub fn get(&self) -> Option<T> {
            self.value
        }
    }

    impl<T> SOMTypeWithEndian for SOMPrimitiveType<T> {
        fn endian(&self) -> SOMEndian {
            self.endian
        }
    }

    impl SOMType for SOMPrimitiveType<bool> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_bool(value)?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_bool()?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<bool>()
        }
    }

    impl SOMType for SOMPrimitiveType<u8> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_u8(value)?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_u8()?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u8>()
        }
    }

    impl SOMType for SOMPrimitiveType<i8> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_i8(value)?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_i8()?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i8>()
        }
    }

    impl SOMType for SOMPrimitiveType<u16> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_u16(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_u16(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u16>()
        }
    }

    impl SOMType for SOMPrimitiveType<i16> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_i16(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_i16(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i16>()
        }
    }

    impl SOMType for SOMPrimitiveType<u24> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_u24(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_u24(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u16>() + std::mem::size_of::<u8>()
        }
    }

    impl SOMType for SOMPrimitiveType<i24> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_i24(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_i24(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i16>() + std::mem::size_of::<i8>()
        }
    }

    impl SOMType for SOMPrimitiveType<u32> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_u32(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_u32(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u32>()
        }
    }

    impl SOMType for SOMPrimitiveType<i32> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_i32(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_i32(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i32>()
        }
    }

    impl SOMType for SOMPrimitiveType<u64> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_u64(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_u64(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u64>()
        }
    }

    impl SOMType for SOMPrimitiveType<i64> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_i64(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_i64(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i64>()
        }
    }

    impl SOMType for SOMPrimitiveType<f32> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_f32(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_f32(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<f32>()
        }
    }

    impl SOMType for SOMPrimitiveType<f64> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.value {
                Some(value) => serializer.write_f64(value, self.endian())?,
                None => {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized Type at offset: {}",
                        offset
                    )))
                }
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            self.value = Some(parser.read_f64(self.endian())?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<f64>()
        }
    }
}

pub type SOMBool = primitives::SOMPrimitiveType<bool>;
pub type SOMu8 = primitives::SOMPrimitiveType<u8>;
pub type SOMi8 = primitives::SOMPrimitiveType<i8>;
pub type SOMu16 = primitives::SOMPrimitiveType<u16>;
pub type SOMi16 = primitives::SOMPrimitiveType<i16>;
pub type SOMu24 = primitives::SOMPrimitiveType<u24>;
pub type SOMi24 = primitives::SOMPrimitiveType<i24>;
pub type SOMu32 = primitives::SOMPrimitiveType<u32>;
pub type SOMi32 = primitives::SOMPrimitiveType<i32>;
pub type SOMu64 = primitives::SOMPrimitiveType<u64>;
pub type SOMi64 = primitives::SOMPrimitiveType<i64>;
pub type SOMf32 = primitives::SOMPrimitiveType<f32>;
pub type SOMf64 = primitives::SOMPrimitiveType<f64>;

mod arrays {
    use super::*;

    #[derive(Debug)]
    pub struct SOMArrayType<T: SOMType + Any> {
        lengthfield: SOMLengthField,
        elements: Vec<T>,
        min: usize,
        max: usize,
    }

    impl<T: SOMType + Any> SOMArrayType<T> {
        pub fn fixed(size: usize) -> SOMArrayType<T> {
            SOMArrayType {
                lengthfield: SOMLengthField::None,
                elements: Vec::<T>::new(),
                min: size,
                max: size,
            }
        }

        pub fn dynamic(lengthfield: SOMLengthField, min: usize, max: usize) -> SOMArrayType<T> {
            SOMArrayType {
                lengthfield,
                elements: Vec::<T>::new(),
                min,
                max,
            }
        }

        pub fn is_dynamic(&self) -> bool {
            self.min != self.max
        }

        pub fn len(&self) -> usize {
            self.elements.len()
        }

        pub fn add(&mut self, obj: T) {
            let _obj = &obj as &dyn Any;
            self.elements
                .push(match _obj.downcast_ref::<SOMArrayMember>() {
                    Some(_) => {
                        if self.elements.is_empty()
                            || (std::mem::discriminant(self.elements.get(0).unwrap())
                                == std::mem::discriminant(&obj))
                        {
                            obj
                        } else {
                            return;
                        }
                    }
                    None => obj,
                });
        }

        pub fn remove(&mut self, index: usize) -> T {
            self.elements.remove(index)
        }

        pub fn get(&self, index: usize) -> Option<&T> {
            self.elements.get(index)
        }

        fn check_length(&self, offset: usize) -> Result<(), SOMError> {
            let length = self.elements.len();
            if (length < self.min) || (length > self.max) {
                return Err(SOMError::InvalidType(format!(
                    "Invalid Array length: {} at offset: {}",
                    length, offset
                )));
            }

            Ok(())
        }
    }

    impl<T: SOMType + Any> SOMTypeWithLengthField for SOMArrayType<T> {
        fn lengthfield(&self) -> SOMLengthField {
            self.lengthfield
        }
    }

    impl<T: SOMType + Any> SOMType for SOMArrayType<T> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();
            self.check_length(offset)?;

            let lengthfield_promise = serializer.promise(self.lengthfield.size())?;
            for element in &self.elements {
                element.serialize(serializer)?;
            }

            let size = serializer.offset() - offset;
            if self.is_dynamic() {
                serializer.write_lengthfield(
                    lengthfield_promise,
                    self.lengthfield,
                    size - self.lengthfield.size(),
                )?;
            }

            Ok(size)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();
            self.check_length(offset)?;

            let lengthfield_size = parser.read_lengthfield(self.lengthfield)?;
            for element in &mut self.elements {
                element.parse(parser)?;
            }

            let size = parser.offset() - offset;
            if self.is_dynamic() && (lengthfield_size != (size - self.lengthfield.size())) {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Length-Field size: {} at offset: {}",
                    lengthfield_size, offset
                )));
            }

            Ok(size)
        }

        fn size(&self) -> usize {
            let mut size: usize = 0;

            size += self.lengthfield.size();
            for element in &self.elements {
                size += element.size();
            }

            size
        }
    }
}

pub type SOMArrayMember = wrapper::SOMTypeWrapper;
pub type SOMArray = arrays::SOMArrayType<SOMArrayMember>;

pub type SOMBoolArray = arrays::SOMArrayType<SOMBool>;
pub type SOMu8Array = arrays::SOMArrayType<SOMu8>;
pub type SOMi8Array = arrays::SOMArrayType<SOMi8>;
pub type SOMu16Array = arrays::SOMArrayType<SOMu16>;
pub type SOMi16Array = arrays::SOMArrayType<SOMi16>;
pub type SOMu24Array = arrays::SOMArrayType<SOMu24>;
pub type SOMi24Array = arrays::SOMArrayType<SOMi24>;
pub type SOMu32Array = arrays::SOMArrayType<SOMu32>;
pub type SOMi32Array = arrays::SOMArrayType<SOMi32>;
pub type SOMu64Array = arrays::SOMArrayType<SOMu64>;
pub type SOMi64Array = arrays::SOMArrayType<SOMi64>;
pub type SOMf32Array = arrays::SOMArrayType<SOMf32>;
pub type SOMf64Array = arrays::SOMArrayType<SOMf64>;

mod structs {
    use super::*;

    #[derive(Debug)]
    pub struct SOMStructType<T: SOMType> {
        members: Vec<T>,
    }

    impl<T: SOMType> SOMStructType<T> {
        pub fn new() -> SOMStructType<T> {
            SOMStructType {
                members: Vec::<T>::new(),
            }
        }

        pub fn len(&self) -> usize {
            self.members.len()
        }

        pub fn add(&mut self, obj: T) {
            self.members.push(obj);
        }

        pub fn get(&self, index: usize) -> Option<&T> {
            self.members.get(index)
        }

        pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
            self.members.get_mut(index)
        }
    }

    impl<T: SOMType> SOMType for SOMStructType<T> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            for member in &self.members {
                member.serialize(serializer)?;
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            for member in &mut self.members {
                member.parse(parser)?;
            }

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            let mut size: usize = 0;

            for member in &self.members {
                size += member.size();
            }

            size
        }
    }
}

pub type SOMStructMember = wrapper::SOMTypeWrapper;
pub type SOMStruct = structs::SOMStructType<SOMStructMember>;

mod unions {
    use super::*;

    #[derive(Debug)]
    pub struct SOMUnionType<T: SOMType + Any> {
        typefield: SOMTypeField,
        variants: Vec<T>,
        index: usize,
    }

    impl<T: SOMType + Any> SOMUnionType<T> {
        pub fn new(typefield: SOMTypeField) -> SOMUnionType<T> {
            SOMUnionType {
                typefield,
                variants: Vec::<T>::new(),
                index: 0,
            }
        }

        pub fn add(&mut self, obj: T) {
            self.variants.push(obj);
        }

        pub fn len(&self) -> usize {
            self.variants.len()
        }

        pub fn has_value(&self) -> bool {
            self.index != 0
        }

        pub fn get(&self) -> Option<&T> {
            if self.has_value() {
                self.variants.get(self.index - 1)
            } else {
                None
            }
        }

        pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
            if index > 0 && index <= self.len() {
                self.index = index;
                self.variants.get_mut(index - 1)
            } else {
                None
            }
        }

        pub fn clear(&mut self) {
            self.index = 0;
        }
    }

    impl<T: SOMType + Any> SOMTypeWithTypeField for SOMUnionType<T> {
        fn typefield(&self) -> SOMTypeField {
            self.typefield
        }
    }

    impl<T: SOMType + Any> SOMType for SOMUnionType<T> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            serializer.write_typefield(self.typefield, self.index)?;

            if self.has_value() {
                self.get().unwrap().serialize(serializer)?;
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            let index = parser.read_typefield(&mut self.typefield)?;

            if index <= self.len() {
                self.index = index;
            } else {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Union index: {} at offset: {}",
                    index, offset
                )));
            }

            if self.has_value() {
                self.get_mut(index).unwrap().parse(parser)?;
            }

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            let mut size: usize = 0;

            size += self.typefield.size();

            if self.has_value() {
                size += self.get().unwrap().size();
            }

            size
        }
    }
}

pub type SOMUnionMember = wrapper::SOMTypeWrapper;
pub type SOMUnion = unions::SOMUnionType<SOMUnionMember>;

mod enums {
    use super::*;

    #[derive(Debug)]
    pub struct SOMEnumType<T> {
        endian: SOMEndian,
        variants: Vec<(String, T)>,
        index: usize,
    }

    impl<T: Copy + Clone + PartialEq> SOMEnumType<T> {
        pub fn new(endian: SOMEndian) -> SOMEnumType<T> {
            SOMEnumType {
                endian,
                variants: Vec::<(String, T)>::new(),
                index: 0,
            }
        }

        pub fn add(&mut self, key: String, value: T) {
            for variant in &self.variants {
                if (variant.0 == key) || (variant.1 == value) {
                    return;
                }
            }

            self.variants.push((key, value));
        }

        pub fn len(&self) -> usize {
            self.variants.len()
        }

        pub fn has_value(&self) -> bool {
            self.index != 0
        }

        pub fn get(&self) -> Option<T> {
            if self.has_value() {
                let (_, value) = self.variants.get(self.index - 1).unwrap();
                Some(*value)
            } else {
                None
            }
        }

        pub fn set(&mut self, key: String) -> bool {
            let mut index: usize = 0;
            for variant in &self.variants {
                index += 1;
                if variant.0 == key {
                    self.index = index;
                    return true;
                }
            }

            false
        }

        fn set_value(&mut self, value: T) -> bool {
            let mut index: usize = 0;
            for variant in &self.variants {
                index += 1;
                if variant.1 == value {
                    self.index = index;
                    return true;
                }
            }

            false
        }

        pub fn clear(&mut self) {
            self.index = 0;
        }
    }

    impl<T> SOMTypeWithEndian for SOMEnumType<T> {
        fn endian(&self) -> SOMEndian {
            self.endian
        }
    }

    impl SOMType for SOMEnumType<u8> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            if self.has_value() {
                let mut temp = SOMu8::empty(self.endian);
                temp.set(self.get().unwrap());
                temp.serialize(serializer)?;
            } else {
                return Err(SOMError::InvalidType(format!(
                    "Uninitialized Type at offset: {}",
                    offset
                )));
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            let mut temp = SOMu8::empty(self.endian);
            temp.parse(parser)?;

            let value: u8 = temp.get().unwrap();
            if !self.set_value(value) {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Enum value: {} at offset: {}",
                    value, offset
                )));
            }

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u8>()
        }
    }

    impl SOMType for SOMEnumType<u16> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            if self.has_value() {
                let mut temp = SOMu16::empty(self.endian);
                temp.set(self.get().unwrap());
                temp.serialize(serializer)?;
            } else {
                return Err(SOMError::InvalidType(format!(
                    "Uninitialized Type at offset: {}",
                    offset
                )));
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            let mut temp = SOMu16::empty(self.endian);
            temp.parse(parser)?;

            let value: u16 = temp.get().unwrap();
            if !self.set_value(value) {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Enum value: {} at offset: {}",
                    value, offset
                )));
            }

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u16>()
        }
    }

    impl SOMType for SOMEnumType<u32> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            if self.has_value() {
                let mut temp = SOMu32::empty(self.endian);
                temp.set(self.get().unwrap());
                temp.serialize(serializer)?;
            } else {
                return Err(SOMError::InvalidType(format!(
                    "Uninitialized Type at offset: {}",
                    offset
                )));
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            let mut temp = SOMu32::empty(self.endian);
            temp.parse(parser)?;

            let value: u32 = temp.get().unwrap();
            if !self.set_value(value) {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Enum value: {} at offset: {}",
                    value, offset
                )));
            }

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u32>()
        }
    }

    impl SOMType for SOMEnumType<u64> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            if self.has_value() {
                let mut temp = SOMu64::empty(self.endian);
                temp.set(self.get().unwrap());
                temp.serialize(serializer)?;
            } else {
                return Err(SOMError::InvalidType(format!(
                    "Uninitialized Type at offset: {}",
                    offset
                )));
            }

            Ok(serializer.offset() - offset)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            let mut temp = SOMu64::empty(self.endian);
            temp.parse(parser)?;

            let value: u64 = temp.get().unwrap();
            if !self.set_value(value) {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Enum value: {} at offset: {}",
                    value, offset
                )));
            }

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u64>()
        }
    }
}

pub type SOMu8Enum = enums::SOMEnumType<u8>;
pub type SOMu16Enum = enums::SOMEnumType<u16>;
pub type SOMu32Enum = enums::SOMEnumType<u32>;
pub type SOMu64Enum = enums::SOMEnumType<u64>;

mod wrapper {
    use super::*;

    macro_rules! som_type_wrapper {
        ([$($value:tt($type:tt),)*]) => {
            #[derive(Debug)]
            pub enum SOMTypeWrapper {$($value($type),)*}

            impl SOMType for SOMTypeWrapper {
                fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
                    match self {
                        $(SOMTypeWrapper::$value(obj) => obj.serialize(serializer),)*
                    }
                }

                fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
                    match self {
                        $(SOMTypeWrapper::$value(obj) => obj.parse(parser),)*
                    }
                }

                fn size(&self) -> usize {
                    match self {
                        $(SOMTypeWrapper::$value(obj) => obj.size(),)*
                    }
                }
            }
        };
    }

    som_type_wrapper!([
        Bool(SOMBool),
        U8(SOMu8),
        I8(SOMi8),
        U16(SOMu16),
        I16(SOMi16),
        U24(SOMu24),
        I24(SOMi24),
        U32(SOMu32),
        I32(SOMi32),
        U64(SOMu64),
        I64(SOMi64),
        F32(SOMf32),
        F64(SOMf64),
        Struct(SOMStruct),
        Union(SOMUnion),
        Array(SOMArray),
        ArrayBool(SOMBoolArray),
        ArrayU8(SOMu8Array),
        ArrayI8(SOMi8Array),
        ArrayU16(SOMu16Array),
        ArrayI16(SOMi16Array),
        ArrayU24(SOMu24Array),
        ArrayI24(SOMi24Array),
        ArrayU32(SOMu32Array),
        ArrayI32(SOMi32Array),
        ArrayU64(SOMu64Array),
        ArrayI64(SOMi64Array),
        ArrayF23(SOMf32Array),
        ArrayF64(SOMf64Array),
        EnumU8(SOMu8Enum),
        EnumU16(SOMu16Enum),
        EnumU32(SOMu32Enum),
        EnumU64(SOMu64Enum),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serialize_parse<T: SOMType>(obj1: &T, obj2: &mut T, data: &[u8]) {
        let size = data.len();
        assert_eq!(size, obj1.size());

        let mut buffer = vec![0u8; size];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(size, obj1.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, data);

        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(size, obj2.parse(&mut parser).unwrap());
        assert_eq!(size, obj2.size());
    }

    fn serialize_fail<T: SOMType>(obj: &T, buffer: &mut [u8], error: &str) {
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        match obj.serialize(&mut serializer) {
            Err(err) => match err {
                SOMError::BufferExhausted(msg) => assert_eq!(msg, error),
                SOMError::InvalidPayload(msg) => assert_eq!(msg, error),
                SOMError::InvalidType(msg) => assert_eq!(msg, error),
            },
            _ => panic!(),
        }
    }

    fn parse_fail<T: SOMType>(obj: &mut T, buffer: &[u8], error: &str) {
        let mut parser = SOMParser::new(&buffer[..]);
        match obj.parse(&mut parser) {
            Err(err) => match err {
                SOMError::BufferExhausted(msg) => assert_eq!(msg, error),
                SOMError::InvalidPayload(msg) => assert_eq!(msg, error),
                SOMError::InvalidType(msg) => assert_eq!(msg, error),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_som_primitive() {
        // generic
        {
            let obj = SOMu8::new(SOMEndian::Big, 1u8);
            assert_eq!(SOMEndian::Big, obj.endian());
            assert_eq!(1u8, obj.get().unwrap());

            let mut obj = SOMu8::empty(SOMEndian::Little);
            assert_eq!(SOMEndian::Little, obj.endian());
            assert_eq!(None, obj.get());
            obj.set(1u8);
            assert_eq!(1u8, obj.get().unwrap());
        }

        // bool
        {
            let obj1 = SOMBool::new(SOMEndian::Big, true);
            let mut obj2 = SOMBool::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0x01]);
            assert_eq!(true, obj2.get().unwrap());

            let obj1 = SOMBool::new(SOMEndian::Big, false);
            let mut obj2 = SOMBool::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0x00]);
            assert_eq!(false, obj2.get().unwrap());

            let obj1 = SOMBool::new(SOMEndian::Little, true);
            let mut obj2 = SOMBool::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x01]);
            assert_eq!(true, obj2.get().unwrap());

            let mut obj = SOMBool::new(SOMEndian::Big, true);
            serialize_fail(
                &obj,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 1",
            );
            parse_fail(
                &mut obj,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 1",
            );

            let mut obj = SOMBool::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 1], "Uninitialized Type at offset: 0");
            parse_fail(&mut obj, &[0x2], "Invalid Bool value: 2 at offset: 0");
        }

        // u8
        {
            let obj1 = SOMu8::new(SOMEndian::Big, 195u8);
            let mut obj2 = SOMu8::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xC3]);
            assert_eq!(195u8, obj2.get().unwrap());

            let obj1 = SOMu8::new(SOMEndian::Little, 195u8);
            let mut obj2 = SOMu8::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0xC3]);
            assert_eq!(195u8, obj2.get().unwrap());

            let mut obj = SOMu8::new(SOMEndian::Big, 195u8);
            serialize_fail(
                &obj,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 1",
            );
            parse_fail(
                &mut obj,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 1",
            );

            let obj = SOMu8::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 1], "Uninitialized Type at offset: 0");
        }

        // i8
        {
            let obj1 = SOMi8::new(SOMEndian::Big, -95i8);
            let mut obj2 = SOMi8::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xA1]);
            assert_eq!(-95i8, obj2.get().unwrap());

            let obj1 = SOMi8::new(SOMEndian::Little, -95i8);
            let mut obj2 = SOMi8::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0xA1]);
            assert_eq!(-95i8, obj2.get().unwrap());

            let mut obj = SOMi8::new(SOMEndian::Big, -95i8);
            serialize_fail(
                &obj,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 1",
            );
            parse_fail(
                &mut obj,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 1",
            );

            let obj = SOMi8::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 1], "Uninitialized Type at offset: 0");
        }

        // u16
        {
            let obj1 = SOMu16::new(SOMEndian::Big, 49200u16);
            let mut obj2 = SOMu16::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xC0, 0x30]);
            assert_eq!(49200u16, obj2.get().unwrap());

            let obj1 = SOMu16::new(SOMEndian::Little, 49200u16);
            let mut obj2 = SOMu16::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x30, 0xC0]);
            assert_eq!(49200u16, obj2.get().unwrap());

            let mut obj = SOMu16::new(SOMEndian::Big, 49200u16);
            serialize_fail(
                &obj,
                &mut [0u8; 1],
                "Serializer exausted at offset: 0 for Object size: 2",
            );
            parse_fail(
                &mut obj,
                &[0u8; 1],
                "Parser exausted at offset: 0 for Object size: 2",
            );

            let obj = SOMu16::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // i16
        {
            let obj1 = SOMi16::new(SOMEndian::Big, -9200i16);
            let mut obj2 = SOMi16::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xDC, 0x10]);
            assert_eq!(-9200i16, obj2.get().unwrap());

            let obj1 = SOMi16::new(SOMEndian::Little, -9200i16);
            let mut obj2 = SOMi16::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x10, 0xDC]);
            assert_eq!(-9200i16, obj2.get().unwrap());

            let mut obj = SOMi16::new(SOMEndian::Big, -9200i16);
            serialize_fail(
                &obj,
                &mut [0u8; 1],
                "Serializer exausted at offset: 0 for Object size: 2",
            );
            parse_fail(
                &mut obj,
                &[0u8; 1],
                "Parser exausted at offset: 0 for Object size: 2",
            );

            let obj = SOMi16::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // u24
        {
            let obj1 = SOMu24::new(SOMEndian::Big, u24::new(12513060u32));
            let mut obj2 = SOMu24::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xBE, 0xEF, 0x24]);
            assert_eq!(u24::new(12513060u32), obj2.get().unwrap());

            let obj1 = SOMu24::new(SOMEndian::Little, u24::new(12513060u32));
            let mut obj2 = SOMu24::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x24, 0xEF, 0xBE]);
            assert_eq!(u24::new(12513060u32), obj2.get().unwrap());

            let mut obj = SOMu24::new(SOMEndian::Big, u24::new(12513060u32));
            serialize_fail(
                &obj,
                &mut [0u8; 2],
                "Serializer exausted at offset: 0 for Object size: 3",
            );
            parse_fail(
                &mut obj,
                &[0u8; 2],
                "Parser exausted at offset: 0 for Object size: 3",
            );

            let obj = SOMu24::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // i24
        {
            let obj1 = SOMi24::new(SOMEndian::Big, i24::new(-2513060i32));
            let mut obj2 = SOMi24::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xD9, 0xA7, 0x5C]);
            assert_eq!(i24::new(-2513060i32), obj2.get().unwrap());

            let obj1 = SOMi24::new(SOMEndian::Little, i24::new(-2513060i32));
            let mut obj2 = SOMi24::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x5C, 0xA7, 0xD9]);
            assert_eq!(i24::new(-2513060i32), obj2.get().unwrap());

            let mut obj = SOMi24::new(SOMEndian::Big, i24::new(-2513060i32));
            serialize_fail(
                &obj,
                &mut [0u8; 2],
                "Serializer exausted at offset: 0 for Object size: 3",
            );
            parse_fail(
                &mut obj,
                &[0u8; 2],
                "Parser exausted at offset: 0 for Object size: 3",
            );

            let obj = SOMi24::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // u32
        {
            let obj1 = SOMu32::new(SOMEndian::Big, 3405691582u32);
            let mut obj2 = SOMu32::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xCA, 0xFE, 0xBA, 0xBE]);
            assert_eq!(3405691582u32, obj2.get().unwrap());

            let obj1 = SOMu32::new(SOMEndian::Little, 3405691582u32);
            let mut obj2 = SOMu32::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0xBE, 0xBA, 0xFE, 0xCA]);
            assert_eq!(3405691582u32, obj2.get().unwrap());

            let mut obj = SOMu32::new(SOMEndian::Big, 3405691582u32);
            serialize_fail(
                &obj,
                &mut [0u8; 3],
                "Serializer exausted at offset: 0 for Object size: 4",
            );
            parse_fail(
                &mut obj,
                &[0u8; 3],
                "Parser exausted at offset: 0 for Object size: 4",
            );

            let obj = SOMu32::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // i32
        {
            let obj1 = SOMi32::new(SOMEndian::Big, -405691582i32);
            let mut obj2 = SOMi32::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xE7, 0xD1, 0xA3, 0x42]);
            assert_eq!(-405691582i32, obj2.get().unwrap());

            let obj1 = SOMi32::new(SOMEndian::Little, -405691582i32);
            let mut obj2 = SOMi32::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x42, 0xA3, 0xD1, 0xE7]);
            assert_eq!(-405691582i32, obj2.get().unwrap());

            let mut obj = SOMi32::new(SOMEndian::Big, -405691582i32);
            serialize_fail(
                &obj,
                &mut [0u8; 3],
                "Serializer exausted at offset: 0 for Object size: 4",
            );
            parse_fail(
                &mut obj,
                &[0u8; 3],
                "Parser exausted at offset: 0 for Object size: 4",
            );

            let obj = SOMi32::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // u64
        {
            let obj1 = SOMu64::new(SOMEndian::Big, 16045704242864831166u64);
            let mut obj2 = SOMu64::empty(SOMEndian::Big);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0xDE, 0xAD, 0xCA, 0xFE, 0xBE, 0xEF, 0xBA, 0xBE],
            );
            assert_eq!(16045704242864831166u64, obj2.get().unwrap());

            let obj1 = SOMu64::new(SOMEndian::Little, 16045704242864831166u64);
            let mut obj2 = SOMu64::empty(SOMEndian::Little);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0xBE, 0xBA, 0xEF, 0xBE, 0xFE, 0xCA, 0xAD, 0xDE],
            );
            assert_eq!(16045704242864831166u64, obj2.get().unwrap());

            let mut obj = SOMu64::new(SOMEndian::Big, 16045704242864831166u64);
            serialize_fail(
                &obj,
                &mut [0u8; 7],
                "Serializer exausted at offset: 0 for Object size: 8",
            );
            parse_fail(
                &mut obj,
                &[0u8; 7],
                "Parser exausted at offset: 0 for Object size: 8",
            );

            let obj = SOMu64::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // i64
        {
            let obj1 = SOMi64::new(SOMEndian::Big, -6045704242864831166i64);
            let mut obj2 = SOMi64::empty(SOMEndian::Big);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0xAC, 0x19, 0x58, 0x05, 0xCA, 0xF8, 0x45, 0x42],
            );
            assert_eq!(-6045704242864831166i64, obj2.get().unwrap());

            let obj1 = SOMi64::new(SOMEndian::Little, -6045704242864831166i64);
            let mut obj2 = SOMi64::empty(SOMEndian::Little);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0x42, 0x45, 0xF8, 0xCA, 0x05, 0x58, 0x19, 0xAC],
            );
            assert_eq!(-6045704242864831166i64, obj2.get().unwrap());

            let mut obj = SOMi64::new(SOMEndian::Big, -6045704242864831166i64);
            serialize_fail(
                &obj,
                &mut [0u8; 7],
                "Serializer exausted at offset: 0 for Object size: 8",
            );
            parse_fail(
                &mut obj,
                &[0u8; 7],
                "Parser exausted at offset: 0 for Object size: 8",
            );

            let obj = SOMi64::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // f32
        {
            let obj1 = SOMf32::new(SOMEndian::Big, 1.0f32);
            let mut obj2 = SOMf32::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0x3F, 0x80, 0x00, 0x00]);
            assert_eq!(1.0f32, obj2.get().unwrap());

            let obj1 = SOMf32::new(SOMEndian::Little, 1.0f32);
            let mut obj2 = SOMf32::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x00, 0x00, 0x80, 0x3F]);
            assert_eq!(1.0f32, obj2.get().unwrap());

            let mut obj = SOMf32::new(SOMEndian::Big, 1.0f32);
            serialize_fail(
                &obj,
                &mut [0u8; 3],
                "Serializer exausted at offset: 0 for Object size: 4",
            );
            parse_fail(
                &mut obj,
                &[0u8; 3],
                "Parser exausted at offset: 0 for Object size: 4",
            );

            let obj = SOMf32::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // f64
        {
            let obj1 = SOMf64::new(SOMEndian::Big, 1.0f64);
            let mut obj2 = SOMf64::empty(SOMEndian::Big);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            );
            assert_eq!(1.0f64, obj2.get().unwrap());

            let obj1 = SOMf64::new(SOMEndian::Little, 1.0f64);
            let mut obj2 = SOMf64::empty(SOMEndian::Little);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F],
            );
            assert_eq!(1.0f64, obj2.get().unwrap());

            let mut obj = SOMf64::new(SOMEndian::Big, 1.0f64);
            serialize_fail(
                &obj,
                &mut [0u8; 7],
                "Serializer exausted at offset: 0 for Object size: 8",
            );
            parse_fail(
                &mut obj,
                &[0u8; 7],
                "Parser exausted at offset: 0 for Object size: 8",
            );

            let obj = SOMf64::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }
    }

    #[test]
    fn test_som_struct() {
        // empty struct
        {
            let obj1 = SOMStruct::new();
            assert_eq!(0, obj1.len());

            let mut obj2 = SOMStruct::new();
            serialize_parse(&obj1, &mut obj2, &[]);
            assert_eq!(0, obj2.len());
        }

        // simple struct
        {
            let mut obj1 = SOMStruct::new();
            obj1.add(SOMStructMember::Bool(SOMBool::new(SOMEndian::Big, true)));
            obj1.add(SOMStructMember::U16(SOMu16::new(SOMEndian::Big, 49200u16)));
            assert_eq!(2, obj1.len());

            let mut obj2 = SOMStruct::new();
            obj2.add(SOMStructMember::Bool(SOMBool::empty(SOMEndian::Big)));
            obj2.add(SOMStructMember::U16(SOMu16::empty(SOMEndian::Big)));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x01, // Bool-Memeber
                    0xC0, 0x30, // U16-Member
                ],
            );
            assert_eq!(2, obj2.len());

            if let Some(SOMStructMember::Bool(sub)) = obj2.get(0) {
                assert_eq!(true, sub.get().unwrap());
            } else {
                panic!();
            }

            if let Some(SOMStructMember::U16(sub)) = obj2.get(1) {
                assert_eq!(49200u16, sub.get().unwrap());
            } else {
                panic!();
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 2],
                "Serializer exausted at offset: 1 for Object size: 2",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 2],
                "Parser exausted at offset: 1 for Object size: 2",
            );
        }

        // complex struct
        {
            let mut sub1 = SOMStruct::new();
            sub1.add(SOMStructMember::Bool(SOMBool::new(SOMEndian::Big, true)));
            sub1.add(SOMStructMember::U16(SOMu16::new(SOMEndian::Big, 49200u16)));

            let mut sub2 = SOMStruct::new();
            sub2.add(SOMStructMember::U16(SOMu16::new(
                SOMEndian::Little,
                49200u16,
            )));
            sub2.add(SOMStructMember::Bool(SOMBool::new(SOMEndian::Little, true)));

            let mut obj1 = SOMStruct::new();
            obj1.add(SOMStructMember::Struct(sub1));
            obj1.add(SOMStructMember::Struct(sub2));
            assert_eq!(2, obj1.len());

            let mut sub1 = SOMStruct::new();
            sub1.add(SOMStructMember::Bool(SOMBool::empty(SOMEndian::Big)));
            sub1.add(SOMStructMember::U16(SOMu16::empty(SOMEndian::Big)));

            let mut sub2 = SOMStruct::new();
            sub2.add(SOMStructMember::U16(SOMu16::empty(SOMEndian::Little)));
            sub2.add(SOMStructMember::Bool(SOMBool::empty(SOMEndian::Little)));

            let mut obj2 = SOMStruct::new();
            obj2.add(SOMStructMember::Struct(sub1));
            obj2.add(SOMStructMember::Struct(sub2));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x01, // Bool-Member
                    0xC0, 0x30, // U16-Member
                    0x30, 0xC0, // U16-Member
                    0x01, // Bool-Member
                ],
            );
            assert_eq!(2, obj2.len());

            if let Some(SOMStructMember::Struct(sub)) = obj2.get(0) {
                if let Some(SOMStructMember::Bool(subsub)) = sub.get(0) {
                    assert_eq!(true, subsub.get().unwrap());
                } else {
                    panic!();
                }

                if let Some(SOMStructMember::U16(subsub)) = sub.get(1) {
                    assert_eq!(49200u16, subsub.get().unwrap());
                } else {
                    panic!();
                }
            } else {
                panic!();
            }

            if let Some(SOMStructMember::Struct(sub)) = obj2.get(1) {
                if let Some(SOMStructMember::U16(subsub)) = sub.get(0) {
                    assert_eq!(49200u16, subsub.get().unwrap());
                } else {
                    panic!();
                }

                if let Some(SOMStructMember::Bool(subsub)) = sub.get(1) {
                    assert_eq!(true, subsub.get().unwrap());
                } else {
                    panic!();
                }
            } else {
                panic!();
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 5],
                "Serializer exausted at offset: 5 for Object size: 1",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 5],
                "Parser exausted at offset: 5 for Object size: 1",
            );

            let mut obj = SOMStruct::new();
            obj.add(SOMStructMember::Bool(SOMBool::empty(SOMEndian::Big)));

            serialize_fail(&obj, &mut [0u8; 1], "Uninitialized Type at offset: 0");
            parse_fail(&mut obj, &[0x2], "Invalid Bool value: 2 at offset: 0");
        }

        // struct with array
        {
            let mut sub1 = SOMu16Array::fixed(3);
            let mut sub2 = SOMu16Array::fixed(3);
            for i in 0..3 {
                sub1.add(SOMu16::new(SOMEndian::Big, (i + 1) as u16));
                sub2.add(SOMu16::empty(SOMEndian::Big));
            }

            let mut obj1 = SOMStruct::new();
            obj1.add(SOMStructMember::ArrayU16(sub1));
            assert_eq!(1, obj1.len());

            let mut obj2 = SOMStruct::new();
            obj2.add(SOMStructMember::ArrayU16(sub2));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x01, // Array-Member (U16)
                    0x00, 0x02, // Array-Member (U16)
                    0x00, 0x03, // Array-Member (U16)
                ],
            );
            assert_eq!(1, obj2.len());

            if let Some(SOMStructMember::ArrayU16(sub)) = obj2.get(0) {
                assert_eq!(3, sub.len());
                for i in 0..3 {
                    assert_eq!((i + 1) as u16, sub.get(i).unwrap().get().unwrap());
                }
            } else {
                panic!();
            }
        }

        // struct with array of array
        {
            let mut subsub1 = SOMu8Array::fixed(3);
            let mut subsub2 = SOMu8Array::fixed(3);
            for i in 0..3 {
                subsub1.add(SOMu8::new(SOMEndian::Big, (i + 1) as u8));
                subsub2.add(SOMu8::empty(SOMEndian::Big));
            }

            let mut sub1 = SOMArray::dynamic(SOMLengthField::U8, 0, 3);
            sub1.add(SOMArrayMember::ArrayU8(subsub1));

            let mut sub2 = SOMArray::dynamic(SOMLengthField::U8, 0, 3);
            sub2.add(SOMArrayMember::ArrayU8(subsub2));

            let mut obj1 = SOMStruct::new();
            obj1.add(SOMStructMember::Array(sub1));
            assert_eq!(1, obj1.len());

            let mut obj2 = SOMStruct::new();
            obj2.add(SOMStructMember::Array(sub2));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x03, // Length-Field (U8)
                    0x01, // Array-Mamber (U8)
                    0x02, // Array-Mamber (U8)
                    0x03, // Array-Mamber (U8)
                ],
            );
            assert_eq!(1, obj2.len());

            if let Some(SOMStructMember::Array(sub)) = obj2.get(0) {
                assert_eq!(1, sub.len());
                if let Some(SOMArrayMember::ArrayU8(subsub)) = sub.get(0) {
                    assert_eq!(3, subsub.len());
                    for i in 0..3 {
                        assert_eq!((i + 1) as u8, subsub.get(i).unwrap().get().unwrap());
                    }
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
        }

        //  struct with union
        {
            let mut sub1 = SOMUnion::new(SOMTypeField::U8);
            sub1.add(SOMUnionMember::Bool(SOMBool::empty(SOMEndian::Big)));
            sub1.add(SOMUnionMember::U16(SOMu16::empty(SOMEndian::Big)));

            if let Some(SOMUnionMember::U16(subsub)) = sub1.get_mut(2) {
                subsub.set(49200u16);
            } else {
                panic!();
            }
            assert!(sub1.has_value());

            let mut obj1 = SOMStruct::new();
            obj1.add(SOMStructMember::Union(sub1));
            assert_eq!(1, obj1.len());

            let mut sub1 = SOMUnion::new(SOMTypeField::U8);
            sub1.add(SOMUnionMember::Bool(SOMBool::empty(SOMEndian::Big)));
            sub1.add(SOMUnionMember::U16(SOMu16::empty(SOMEndian::Big)));

            let mut obj2 = SOMStruct::new();
            obj2.add(SOMStructMember::Union(sub1));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x02, // Type-Field (U8)
                    0xC0, 0x30, // U16-Value
                ],
            );
            assert_eq!(1, obj2.len());

            if let Some(SOMStructMember::Union(sub)) = obj2.get(0) {
                if let Some(SOMUnionMember::U16(subsub)) = sub.get() {
                    assert_eq!(49200u16, subsub.get().unwrap());
                } else {
                    panic!();
                }
            } else {
                panic!();
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 1",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 1",
            );
        }

        //  struct with enum
        {
            let mut sub1 = SOMu16Enum::new(SOMEndian::Little);
            sub1.add(String::from("A"), 49200u16);
            assert!(sub1.set(String::from("A")));

            let mut obj1 = SOMStruct::new();
            obj1.add(SOMStructMember::EnumU16(sub1));
            assert_eq!(1, obj1.len());

            let mut sub1 = SOMu16Enum::new(SOMEndian::Little);
            sub1.add(String::from("A"), 49200u16);

            let mut obj2 = SOMStruct::new();
            obj2.add(SOMStructMember::EnumU16(sub1));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x30, 0xC0, // U16-Value
                ],
            );
            assert_eq!(1, obj2.len());

            if let Some(SOMStructMember::EnumU16(sub)) = obj2.get(0) {
                assert_eq!(49200u16, sub.get().unwrap());
            } else {
                panic!();
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 2",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 2",
            );
        }
    }

    #[test]
    fn test_som_array() {
        // static array
        {
            let mut obj1 = SOMu16Array::fixed(3);
            let mut obj2 = SOMu16Array::fixed(3);
            assert!(!obj1.is_dynamic());
            assert_eq!(0, obj1.len());

            for i in 0..3 {
                obj1.add(SOMu16::new(SOMEndian::Big, (i + 1) as u16));
                obj2.add(SOMu16::empty(SOMEndian::Big));
            }
            assert_eq!(3, obj1.len());

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x01, // Array-Member (U16)
                    0x00, 0x02, // Array-Member (U16)
                    0x00, 0x03, // Array-Member (U16)
                ],
            );
            assert!(!obj2.is_dynamic());
            assert_eq!(3, obj2.len());
            for i in 0..3 {
                assert_eq!((i + 1) as u16, obj2.get(i).unwrap().get().unwrap());
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 5],
                "Serializer exausted at offset: 4 for Object size: 2",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 5],
                "Parser exausted at offset: 4 for Object size: 2",
            );
        }

        // dynamic array
        {
            let mut obj1 = SOMu16Array::dynamic(SOMLengthField::U32, 0, 3);
            let mut obj2 = SOMu16Array::dynamic(SOMLengthField::U32, 0, 3);
            assert!(obj1.is_dynamic());
            assert_eq!(0, obj1.len());

            for i in 0..3 {
                obj1.add(SOMu16::new(SOMEndian::Big, (i + 1) as u16));
                obj2.add(SOMu16::empty(SOMEndian::Big));
            }
            assert_eq!(3, obj1.len());

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x00, 0x00, 0x06, // Length-Field (U32)
                    0x00, 0x01, // Array-Member (U16)
                    0x00, 0x02, // Array-Member (U16)
                    0x00, 0x03, // Array-Member (U16)
                ],
            );
            assert!(obj2.is_dynamic());
            assert_eq!(3, obj2.len());
            for i in 0..3 {
                assert_eq!((i + 1) as u16, obj2.get(i).unwrap().get().unwrap());
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 3],
                "Serializer exausted at offset: 0 for Object size: 4",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 3],
                "Parser exausted at offset: 0 for Object size: 4",
            );

            serialize_fail(
                &obj1,
                &mut [0u8; 9],
                "Serializer exausted at offset: 8 for Object size: 2",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 9],
                "Parser exausted at offset: 8 for Object size: 2",
            );
        }

        // invalid length
        {
            let mut obj = SOMu8Array::dynamic(SOMLengthField::U8, 1, 3);

            serialize_fail(&obj, &mut [0u8; 4], "Invalid Array length: 0 at offset: 0");
            parse_fail(&mut obj, &[0u8; 4], "Invalid Array length: 0 at offset: 0");
        }

        // invalid member
        {
            let mut obj = SOMArray::fixed(3);

            obj.add(SOMArrayMember::Bool(SOMBool::new(SOMEndian::Big, true)));
            assert_eq!(1, obj.len());

            obj.add(SOMArrayMember::U8(SOMu8::new(SOMEndian::Big, 1u8)));
            assert_eq!(1, obj.len());
        }
    }

    #[test]
    fn test_som_union() {
        // empty union
        {
            let mut obj1 = SOMUnion::new(SOMTypeField::U8);
            assert_eq!(0, obj1.len());
            assert!(!obj1.has_value());
            assert!(obj1.get().is_none());
            assert!(obj1.get_mut(1).is_none());

            let mut obj2 = SOMUnion::new(SOMTypeField::U8);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, // Type-Field (U8)
                ],
            );
            assert_eq!(0, obj2.len());
            assert!(!obj2.has_value());

            let mut obj3 = SOMUnion::new(SOMTypeField::U8);
            obj3.get_mut(1); // invalid
            assert!(!obj3.has_value());

            obj3.add(SOMUnionMember::Bool(SOMBool::empty(SOMEndian::Big)));
            obj3.get_mut(1);
            assert!(obj3.has_value());

            obj3.clear();
            assert!(!obj3.has_value());
        }

        // primitive union
        {
            let mut obj1 = SOMUnion::new(SOMTypeField::U8);
            obj1.add(SOMUnionMember::Bool(SOMBool::empty(SOMEndian::Big)));
            obj1.add(SOMUnionMember::U16(SOMu16::empty(SOMEndian::Big)));
            assert_eq!(2, obj1.len());

            if let Some(SOMUnionMember::U16(sub)) = obj1.get_mut(2) {
                sub.set(49200u16);
            } else {
                panic!();
            }
            assert!(obj1.has_value());

            let mut obj2 = SOMUnion::new(SOMTypeField::U8);
            obj2.add(SOMUnionMember::Bool(SOMBool::empty(SOMEndian::Big)));
            obj2.add(SOMUnionMember::U16(SOMu16::empty(SOMEndian::Big)));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x02, // Type-Field (U8)
                    0xC0, 0x30, // U16-Value
                ],
            );
            assert_eq!(2, obj2.len());
            assert!(obj2.has_value());

            if let Some(SOMUnionMember::U16(sub)) = obj2.get() {
                assert_eq!(49200u16, sub.get().unwrap());
            } else {
                panic!();
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 1",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 1",
            );
            serialize_fail(
                &obj1,
                &mut [0u8; 2],
                "Serializer exausted at offset: 1 for Object size: 2",
            );
            parse_fail(
                &mut obj2,
                &[0x02, 0x01],
                "Parser exausted at offset: 1 for Object size: 2",
            );
            parse_fail(&mut obj2, &[0x03], "Invalid Union index: 3 at offset: 0");

            obj1.clear();
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, // Type-Field (U8)
                ],
            );
            assert!(!obj2.has_value());
        }

        // complex union
        {
            let mut sub1 = SOMStruct::new();
            sub1.add(SOMStructMember::Bool(SOMBool::empty(SOMEndian::Big)));
            sub1.add(SOMStructMember::U16(SOMu16::empty(SOMEndian::Big)));

            let mut obj1 = SOMUnion::new(SOMTypeField::U16);
            obj1.add(SOMUnionMember::Bool(SOMBool::empty(SOMEndian::Big)));
            obj1.add(SOMUnionMember::Struct(sub1));
            assert_eq!(2, obj1.len());

            if let Some(SOMUnionMember::Struct(sub)) = obj1.get_mut(2) {
                if let Some(SOMStructMember::Bool(subsub)) = sub.get_mut(0) {
                    subsub.set(true);
                } else {
                    panic!();
                }

                if let Some(SOMStructMember::U16(subsub)) = sub.get_mut(1) {
                    subsub.set(49200u16);
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
            assert!(obj1.has_value());

            let mut sub2 = SOMStruct::new();
            sub2.add(SOMStructMember::Bool(SOMBool::empty(SOMEndian::Big)));
            sub2.add(SOMStructMember::U16(SOMu16::empty(SOMEndian::Big)));

            let mut obj2 = SOMUnion::new(SOMTypeField::U16);
            obj2.add(SOMUnionMember::Bool(SOMBool::empty(SOMEndian::Big)));
            obj2.add(SOMUnionMember::Struct(sub2));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x02, // Type-Field (U16)
                    0x01, // Struct-Value Bool-Member
                    0xC0, 0x30, // Struct-Value U16-Member
                ],
            );
            assert_eq!(2, obj2.len());
            assert!(obj2.has_value());

            if let Some(SOMUnionMember::Struct(sub)) = obj2.get() {
                if let Some(SOMStructMember::Bool(subsub)) = sub.get(0) {
                    assert_eq!(true, subsub.get().unwrap());
                } else {
                    panic!();
                }

                if let Some(SOMStructMember::U16(subsub)) = sub.get(1) {
                    assert_eq!(49200u16, subsub.get().unwrap());
                } else {
                    panic!();
                }
            } else {
                panic!();
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 2",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 2",
            );
            serialize_fail(
                &obj1,
                &mut [0u8; 4],
                "Serializer exausted at offset: 3 for Object size: 2",
            );
            parse_fail(
                &mut obj2,
                &[0x00, 0x02, 0x01],
                "Parser exausted at offset: 3 for Object size: 2",
            );
        }
    }

    #[test]
    fn test_som_enum() {
        // empty enum
        {
            let mut obj = SOMu8Enum::new(SOMEndian::Big);
            assert_eq!(SOMEndian::Big, obj.endian());
            assert_eq!(0, obj.len());
            assert!(!obj.has_value());
            assert!(obj.get().is_none());
            assert!(!obj.set(String::from("foo")));
            assert!(obj.get().is_none());
        }

        // u8 enum
        {
            let mut obj1 = SOMu8Enum::new(SOMEndian::Big);
            obj1.add(String::from("A"), 23u8);
            obj1.add(String::from("B"), 42u8);
            assert_eq!(2, obj1.len());
            assert!(!obj1.has_value());

            assert!(obj1.set(String::from("A")));
            assert!(obj1.has_value());
            assert_eq!(23u8, obj1.get().unwrap());

            let mut obj2 = SOMu8Enum::new(SOMEndian::Big);
            obj2.add(String::from("A"), 23u8);
            obj2.add(String::from("B"), 42u8);

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x17, // U8-Value
                ],
            );
            assert_eq!(2, obj2.len());
            assert!(obj2.has_value());
            assert_eq!(23u8, obj2.get().unwrap());

            serialize_fail(
                &obj1,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 1",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 1",
            );
            parse_fail(&mut obj2, &[0u8; 1], "Invalid Enum value: 0 at offset: 0");
        }

        // u16 enum
        {
            let mut obj1 = SOMu16Enum::new(SOMEndian::Big);
            obj1.add(String::from("A"), 49200u16);
            assert!(obj1.set(String::from("A")));

            let mut obj2 = SOMu16Enum::new(SOMEndian::Big);
            obj2.add(String::from("A"), 49200u16);
            serialize_parse(&obj1, &mut obj2, &[0xC0, 0x30]);
            assert_eq!(49200u16, obj2.get().unwrap());
        }

        // u32 enum
        {
            let mut obj1 = SOMu32Enum::new(SOMEndian::Big);
            obj1.add(String::from("A"), 3405691582u32);
            assert!(obj1.set(String::from("A")));

            let mut obj2 = SOMu32Enum::new(SOMEndian::Big);
            obj2.add(String::from("A"), 3405691582u32);
            serialize_parse(&obj1, &mut obj2, &[0xCA, 0xFE, 0xBA, 0xBE]);
            assert_eq!(3405691582u32, obj2.get().unwrap());
        }

        // u64 enum
        {
            let mut obj1 = SOMu64Enum::new(SOMEndian::Big);
            obj1.add(String::from("A"), 16045704242864831166u64);
            assert!(obj1.set(String::from("A")));

            let mut obj2 = SOMu64Enum::new(SOMEndian::Big);
            obj2.add(String::from("A"), 16045704242864831166u64);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0xDE, 0xAD, 0xCA, 0xFE, 0xBE, 0xEF, 0xBA, 0xBE],
            );
            assert_eq!(16045704242864831166u64, obj2.get().unwrap());
        }

        // invalid key
        {
            let mut obj1 = SOMu8Enum::new(SOMEndian::Big);

            obj1.add(String::from("A"), 23u8);
            assert_eq!(1, obj1.len());

            obj1.add(String::from("A"), 42u8);
            assert_eq!(1, obj1.len());
        }

        // invalid value
        {
            let mut obj1 = SOMu8Enum::new(SOMEndian::Big);

            obj1.add(String::from("A"), 23u8);
            assert_eq!(1, obj1.len());

            obj1.add(String::from("B"), 23u8);
            assert_eq!(1, obj1.len());
        }
    }
}
