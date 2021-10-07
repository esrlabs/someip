// someip types

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::any::Any;
use ux::{i24, u24};

pub trait SOMType {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError>;
    fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError>;
    fn size(&self) -> usize;
    fn info(&self) -> SOMTypeInfo {
        SOMTypeInfo::FixedLength
    }
}

#[derive(Debug)]
pub enum SOMError {
    BufferExhausted(String),
    InvalidPayload(String),
    InvalidType(String),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SOMTypeInfo {
    FixedLength,
    ImplicitLength,
    ExplicitLength,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SOMEndian {
    Big,
    Little,
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

    fn skip(&mut self, size: usize) -> Result<usize, SOMError> {
        self.check_size(size)?;
        self.offset += size;
        Ok(size)
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
        pub value: Option<T>,
    }

    impl<T: Copy + Clone + PartialEq> SOMPrimitiveType<T> {
        pub fn empty() -> SOMPrimitiveType<T> {
            SOMPrimitiveType { value: None }
        }

        pub fn new(value: T) -> SOMPrimitiveType<T> {
            SOMPrimitiveType { value: Some(value) }
        }

        pub fn set(&mut self, value: T) {
            self.value = Some(value);
        }

        pub fn get(&self) -> Option<T> {
            self.value
        }
    }

    #[derive(Debug)]
    pub struct SOMPrimitiveTypeWithEndian<T> {
        pub primitive: SOMPrimitiveType<T>,
        pub endian: SOMEndian,
    }

    impl<T: Copy + Clone + PartialEq> SOMPrimitiveTypeWithEndian<T> {
        pub fn empty(endian: SOMEndian) -> SOMPrimitiveTypeWithEndian<T> {
            SOMPrimitiveTypeWithEndian {
                primitive: SOMPrimitiveType::empty(),
                endian,
            }
        }

        pub fn new(endian: SOMEndian, value: T) -> SOMPrimitiveTypeWithEndian<T> {
            SOMPrimitiveTypeWithEndian {
                endian,
                primitive: SOMPrimitiveType::new(value),
            }
        }

        pub fn set(&mut self, value: T) {
            self.primitive.set(value);
        }

        pub fn get(&self) -> Option<T> {
            self.primitive.get()
        }
    }

    impl SOMType for SOMPrimitiveType<bool> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
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

            self.set(parser.read_bool()?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<bool>()
        }
    }

    impl SOMType for SOMPrimitiveType<u8> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
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

            self.set(parser.read_u8()?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u8>()
        }
    }

    impl SOMType for SOMPrimitiveType<i8> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
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

            self.set(parser.read_i8()?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i8>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<u16> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_u16(value, self.endian)?,
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

            self.set(parser.read_u16(self.endian)?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u16>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<i16> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_i16(value, self.endian)?,
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

            self.set(parser.read_i16(self.endian)?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i16>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<u24> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_u24(value, self.endian)?,
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

            self.set(parser.read_u24(self.endian)?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u16>() + std::mem::size_of::<u8>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<i24> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_i24(value, self.endian)?,
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

            self.set(parser.read_i24(self.endian)?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i16>() + std::mem::size_of::<i8>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<u32> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_u32(value, self.endian)?,
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

            self.set(parser.read_u32(self.endian)?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u32>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<i32> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_i32(value, self.endian)?,
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

            self.set(parser.read_i32(self.endian)?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i32>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<u64> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_u64(value, self.endian)?,
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

            self.set(parser.read_u64(self.endian)?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<u64>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<i64> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_i64(value, self.endian)?,
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

            self.set(parser.read_i64(self.endian)?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<i64>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<f32> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_f32(value, self.endian)?,
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

            self.set(parser.read_f32(self.endian)?);

            Ok(parser.offset() - offset)
        }

        fn size(&self) -> usize {
            std::mem::size_of::<f32>()
        }
    }

    impl SOMType for SOMPrimitiveTypeWithEndian<f64> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            match self.get() {
                Some(value) => serializer.write_f64(value, self.endian)?,
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

            self.set(parser.read_f64(self.endian)?);

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
pub type SOMu16 = primitives::SOMPrimitiveTypeWithEndian<u16>;
pub type SOMi16 = primitives::SOMPrimitiveTypeWithEndian<i16>;
pub type SOMu24 = primitives::SOMPrimitiveTypeWithEndian<u24>;
pub type SOMi24 = primitives::SOMPrimitiveTypeWithEndian<i24>;
pub type SOMu32 = primitives::SOMPrimitiveTypeWithEndian<u32>;
pub type SOMi32 = primitives::SOMPrimitiveTypeWithEndian<i32>;
pub type SOMu64 = primitives::SOMPrimitiveTypeWithEndian<u64>;
pub type SOMi64 = primitives::SOMPrimitiveTypeWithEndian<i64>;
pub type SOMf32 = primitives::SOMPrimitiveTypeWithEndian<f32>;
pub type SOMf64 = primitives::SOMPrimitiveTypeWithEndian<f64>;

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
            let length: usize = self.elements.len();
            if (length < self.min) || (length > self.max) {
                return Err(SOMError::InvalidType(format!(
                    "Invalid Array length: {} at offset: {}",
                    length, offset
                )));
            }

            Ok(())
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

            let lengthfield_value = parser.read_lengthfield(self.lengthfield)?;

            for element in &mut self.elements {
                element.parse(parser)?;
            }

            let size = parser.offset() - offset;
            if self.is_dynamic() && (lengthfield_value != (size - self.lengthfield.size())) {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Length-Field value: {} at offset: {}",
                    lengthfield_value, offset
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

        fn info(&self) -> SOMTypeInfo {
            if self.is_dynamic() {
                SOMTypeInfo::ExplicitLength
            } else {
                SOMTypeInfo::ImplicitLength
            }
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

        fn info(&self) -> SOMTypeInfo {
            SOMTypeInfo::ImplicitLength
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

        pub fn len(&self) -> usize {
            self.variants.len()
        }

        pub fn add(&mut self, obj: T) {
            self.variants.push(obj);
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

        fn info(&self) -> SOMTypeInfo {
            SOMTypeInfo::ImplicitLength
        }
    }
}

pub type SOMUnionMember = wrapper::SOMTypeWrapper;
pub type SOMUnion = unions::SOMUnionType<SOMUnionMember>;

mod enums {
    use super::*;

    #[derive(Debug)]
    struct SOMEnumTypeItem<T> {
        key: String,
        value: T,
    }

    #[derive(Debug)]
    pub struct SOMEnumType<T> {
        variants: Vec<SOMEnumTypeItem<T>>,
        index: usize,
    }

    impl<T: Copy + Clone + PartialEq> SOMEnumType<T> {
        pub fn new() -> SOMEnumType<T> {
            SOMEnumType {
                variants: Vec::<SOMEnumTypeItem<T>>::new(),
                index: 0,
            }
        }

        pub fn len(&self) -> usize {
            self.variants.len()
        }

        pub fn add(&mut self, key: String, value: T) {
            for variant in &self.variants {
                if (variant.key == key) || (variant.value == value) {
                    return;
                }
            }

            self.variants.push(SOMEnumTypeItem { key, value });
        }

        pub fn has_value(&self) -> bool {
            self.index != 0
        }

        pub fn get(&self) -> Option<T> {
            if self.has_value() {
                let variant = self.variants.get(self.index - 1).unwrap();
                Some(variant.value)
            } else {
                None
            }
        }

        pub fn set(&mut self, key: String) -> bool {
            let mut index: usize = 0;
            for variant in &self.variants {
                index += 1;
                if variant.key == key {
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
                if variant.value == value {
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

    #[derive(Debug)]
    pub struct SOMEnumTypeWithEndian<T> {
        enumeration: SOMEnumType<T>,
        endian: SOMEndian,
    }

    impl<T: Copy + Clone + PartialEq> SOMEnumTypeWithEndian<T> {
        pub fn new(endian: SOMEndian) -> SOMEnumTypeWithEndian<T> {
            SOMEnumTypeWithEndian {
                enumeration: SOMEnumType::new(),
                endian,
            }
        }

        pub fn len(&self) -> usize {
            self.enumeration.len()
        }

        pub fn add(&mut self, key: String, value: T) {
            self.enumeration.add(key, value);
        }

        pub fn has_value(&self) -> bool {
            self.enumeration.has_value()
        }

        pub fn get(&self) -> Option<T> {
            self.enumeration.get()
        }

        pub fn set(&mut self, key: String) -> bool {
            self.enumeration.set(key)
        }

        fn set_value(&mut self, value: T) -> bool {
            self.enumeration.set_value(value)
        }

        pub fn clear(&mut self) {
            self.enumeration.clear()
        }
    }

    impl SOMType for SOMEnumType<u8> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            if self.has_value() {
                let mut temp = SOMu8::empty();
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

            let mut temp = SOMu8::empty();
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

    impl SOMType for SOMEnumTypeWithEndian<u16> {
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

    impl SOMType for SOMEnumTypeWithEndian<u32> {
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

    impl SOMType for SOMEnumTypeWithEndian<u64> {
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
pub type SOMu16Enum = enums::SOMEnumTypeWithEndian<u16>;
pub type SOMu32Enum = enums::SOMEnumTypeWithEndian<u32>;
pub type SOMu64Enum = enums::SOMEnumTypeWithEndian<u64>;

mod strings {
    use super::*;

    const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];
    const UTF8_TERMINATION: [u8; 1] = [0x00];
    const UTF16_BOM_BE: [u8; 2] = [0xFE, 0xFF];
    const UTF16_BOM_LE: [u8; 2] = [0xFF, 0xFE];
    const UTF16_TERMINATION: [u8; 2] = [0x00, 0x00];

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum SOMStringEncoding {
        Utf8,
        Utf16Be,
        Utf16Le,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum SOMStringFormat {
        Plain,
        WithBOM,
        WithTermination,
        WithBOMandTermination,
    }

    fn char_size(encoding: SOMStringEncoding) -> usize {
        match encoding {
            SOMStringEncoding::Utf8 => std::mem::size_of::<u8>(),
            _ => std::mem::size_of::<u16>(),
        }
    }

    fn string_len(encoding: SOMStringEncoding, bytes: &[u8]) -> usize {
        let bytes_len = bytes.len();
        let char_size = char_size(encoding);

        let len = bytes_len / char_size;
        if (bytes_len % char_size) != 0 {
            return len + 1;
        }
        len
    }

    #[derive(Debug)]
    pub struct SOMStringType {
        lengthfield: SOMLengthField,
        encoding: SOMStringEncoding,
        format: SOMStringFormat,
        value: String,
        min: usize,
        max: usize,
    }

    impl SOMStringType {
        pub fn fixed(
            encoding: SOMStringEncoding,
            format: SOMStringFormat,
            size: usize,
        ) -> SOMStringType {
            SOMStringType {
                lengthfield: SOMLengthField::None,
                encoding,
                format,
                value: String::from(""),
                min: size,
                max: size,
            }
        }

        pub fn dynamic(
            lengthfield: SOMLengthField,
            encoding: SOMStringEncoding,
            format: SOMStringFormat,
            min: usize,
            max: usize,
        ) -> SOMStringType {
            SOMStringType {
                lengthfield,
                encoding,
                format,
                value: String::from(""),
                min,
                max,
            }
        }

        pub fn len(&self) -> usize {
            self.string_len(&self.value)
        }

        fn string_len(&self, value: &String) -> usize {
            let bom_len = string_len(self.encoding, &self.bom());
            let termination_len = string_len(self.encoding, &self.termination());

            value.len()
                + match self.format {
                    SOMStringFormat::Plain => 0,
                    SOMStringFormat::WithBOM => bom_len,
                    SOMStringFormat::WithTermination => termination_len,
                    SOMStringFormat::WithBOMandTermination => bom_len + termination_len,
                }
        }

        fn bom(&self) -> Vec<u8> {
            match self.encoding {
                SOMStringEncoding::Utf8 => UTF8_BOM.to_vec(),
                SOMStringEncoding::Utf16Be => UTF16_BOM_BE.to_vec(),
                SOMStringEncoding::Utf16Le => UTF16_BOM_LE.to_vec(),
            }
        }

        fn termination(&self) -> Vec<u8> {
            match self.encoding {
                SOMStringEncoding::Utf8 => UTF8_TERMINATION.to_vec(),
                _ => UTF16_TERMINATION.to_vec(),
            }
        }

        pub fn is_dynamic(&self) -> bool {
            (self.min != self.max) || (self.lengthfield != SOMLengthField::None)
        }

        pub fn has_bom(&self) -> bool {
            match self.format {
                SOMStringFormat::WithBOM => true,
                SOMStringFormat::WithBOMandTermination => true,
                _ => false,
            }
        }

        pub fn has_termination(&self) -> bool {
            match self.format {
                SOMStringFormat::WithTermination => true,
                SOMStringFormat::WithBOMandTermination => true,
                _ => false,
            }
        }

        pub fn set(&mut self, value: String) -> bool {
            if self.string_len(&value) <= self.max {
                self.value = value;
                return true;
            }

            false
        }

        pub fn get(&self) -> &str {
            &self.value
        }

        fn endian(&self) -> SOMEndian {
            match self.encoding {
                SOMStringEncoding::Utf8 => SOMEndian::Big,
                SOMStringEncoding::Utf16Be => SOMEndian::Big,
                SOMStringEncoding::Utf16Le => SOMEndian::Little,
            }
        }

        fn check_length(&self, offset: usize) -> Result<(), SOMError> {
            let length: usize = self.len();

            let valid: bool;
            if self.is_dynamic() {
                valid = (self.min <= length) && (length <= self.max);
            } else {
                valid = length <= self.max;
            }

            if !valid {
                return Err(SOMError::InvalidType(format!(
                    "Invalid String length: {} at offset: {}",
                    length, offset
                )));
            }

            Ok(())
        }
    }

    impl SOMType for SOMStringType {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();
            self.check_length(offset)?;

            let lengthfield_promise = serializer.promise(self.lengthfield.size())?;

            let char_size = char_size(self.encoding);
            let mut string_size = 0usize;

            if self.has_bom() {
                for item in self.bom() {
                    serializer.write_u8(item)?;
                }
                string_size += char_size * string_len(self.encoding, &self.bom());
            }

            match self.encoding {
                SOMStringEncoding::Utf8 => {
                    let bytes: Vec<u8> = self.value.clone().into_bytes();
                    for item in bytes {
                        serializer.write_u8(item)?;
                        string_size += char_size;
                    }
                }
                _ => {
                    let bytes: Vec<u16> = self.value.encode_utf16().collect();
                    for item in bytes {
                        serializer.write_u16(item, self.endian())?;
                        string_size += char_size;
                    }
                }
            }

            if self.has_termination() {
                for item in self.termination() {
                    serializer.write_u8(item)?;
                }
                string_size += char_size * string_len(self.encoding, &self.termination());
            }

            let size;
            if self.is_dynamic() {
                size = serializer.offset() - offset;
                serializer.write_lengthfield(
                    lengthfield_promise,
                    self.lengthfield,
                    size - self.lengthfield.size(),
                )?;
            } else {
                let max_size = char_size * self.max;
                while string_size < max_size {
                    serializer.write_u8(0x00)?;
                    string_size += std::mem::size_of::<u8>();
                }
                size = serializer.offset() - offset;
            }

            Ok(size)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            let lengthfield_value = parser.read_lengthfield(self.lengthfield)?;

            let char_size = char_size(self.encoding);
            let mut string_size = lengthfield_value;

            if !self.is_dynamic() {
                string_size = char_size * self.max;
            }

            if self.has_termination() {
                string_size -= self.termination().len();
            }

            let mut valid = true;
            if self.has_bom() {
                for item in self.bom() {
                    let value = parser.read_u8()?;
                    if value != item {
                        valid = false;
                        break;
                    }
                    string_size -= std::mem::size_of::<u8>();
                }
            }
            if !valid {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid String-BOM at offset: {}",
                    parser.offset()
                )));
            }

            let value: String;
            match self.encoding {
                SOMStringEncoding::Utf8 => {
                    let mut bytes: Vec<u8> = vec![];
                    while string_size >= char_size {
                        bytes.push(parser.read_u8()?);
                        string_size -= char_size;
                    }
                    value = String::from_utf8(bytes).unwrap();
                }
                _ => {
                    let mut bytes: Vec<u16> = vec![];
                    while string_size >= char_size {
                        bytes.push(parser.read_u16(self.endian())?);
                        string_size -= char_size;
                    }
                    value = String::from_utf16(&bytes).unwrap()
                }
            }
            self.value = value.trim_end_matches(char::from(0x00)).to_string();

            if self.has_termination() {
                for item in self.termination() {
                    let value = parser.read_u8()?;
                    if value != item {
                        valid = false;
                        break;
                    }
                }
            }
            if !valid {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid String-Termination at offset: {}",
                    parser.offset()
                )));
            }

            let size = parser.offset() - offset;
            if self.is_dynamic() && (lengthfield_value != (size - self.lengthfield.size())) {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Length-Field value: {} at offset: {}",
                    lengthfield_value, offset
                )));
            }

            self.check_length(offset)?;
            Ok(size)
        }

        fn size(&self) -> usize {
            let mut size: usize = 0;

            if self.is_dynamic() {
                size += self.lengthfield.size();
                size += char_size(self.encoding) * self.len();
            } else {
                size += char_size(self.encoding) * self.max;
            }

            size
        }

        fn info(&self) -> SOMTypeInfo {
            if self.is_dynamic() {
                SOMTypeInfo::ExplicitLength
            } else {
                SOMTypeInfo::ImplicitLength
            }
        }
    }
}

pub type SOMStringEncoding = strings::SOMStringEncoding;
pub type SOMStringFormat = strings::SOMStringFormat;
pub type SOMString = strings::SOMStringType;

mod optionals {
    use super::*;

    const TAG_MASK: u16 = 0x7FFF;

    fn wire_type<T: SOMType>(value: &T) -> Option<usize> {
        match value.info() {
            SOMTypeInfo::FixedLength => match value.size() {
                1 => Some(0),
                2 => Some(1),
                4 => Some(2),
                8 => Some(3),
                _ => None,
            },
            _ => Some(4),
        }
    }

    fn wire_size(wiretype: usize) -> Option<usize> {
        match wiretype {
            0 => Some(1),
            1 => Some(2),
            2 => Some(4),
            3 => Some(8),
            _ => None,
        }
    }

    #[derive(Debug)]
    struct SOMOptionalTypeItem<T: SOMType> {
        wiretype: usize,
        key: usize,
        value: T,
        mandatory: bool,
        set: bool,
    }

    impl<T: SOMType> SOMOptionalTypeItem<T> {
        fn tag(&self) -> u16 {
            TAG_MASK & (((self.wiretype as u16) << 12) | ((self.key as u16) & 0x0FFF))
        }
    }

    #[derive(Debug)]
    pub struct SOMOptionalType<T: SOMType> {
        lengthfield: SOMLengthField,
        members: Vec<SOMOptionalTypeItem<T>>,
    }

    impl<T: SOMType> SOMOptionalType<T> {
        pub fn new(lengthfield: SOMLengthField) -> SOMOptionalType<T> {
            SOMOptionalType {
                lengthfield,
                members: Vec::<SOMOptionalTypeItem<T>>::new(),
            }
        }

        pub fn len(&self) -> usize {
            self.members.len()
        }

        pub fn add(&mut self, key: usize, value: T, mandatory: bool) {
            for member in &self.members {
                if member.key == key {
                    return;
                }
            }

            if let Some(wiretype) = wire_type(&value) {
                self.members.push(SOMOptionalTypeItem {
                    wiretype,
                    key,
                    value,
                    mandatory,
                    set: false,
                });
            }
        }

        pub fn is_mandatory(&self, key: usize) -> bool {
            for member in &self.members {
                if (member.key == key) && member.mandatory {
                    return true;
                }
            }

            false
        }

        pub fn is_set(&self, key: usize) -> bool {
            for member in &self.members {
                if (member.key == key) && member.set {
                    return true;
                }
            }

            false
        }

        pub fn get(&self, key: usize) -> Option<&T> {
            for member in &self.members {
                if (member.key == key) && member.set {
                    return Some(&member.value);
                }
            }

            None
        }

        pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
            for member in &mut self.members {
                if member.key == key {
                    member.set = true;
                    return Some(&mut member.value);
                }
            }

            None
        }

        pub fn clear(&mut self) {
            for member in &mut self.members {
                member.set = false;
            }
        }
    }

    impl<T: SOMType + Any> SOMType for SOMOptionalType<T> {
        fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
            let offset = serializer.offset();

            let type_lengthfield = serializer.promise(self.lengthfield.size())?;

            for member in &self.members {
                if member.set {
                    serializer.write_u16(member.tag(), SOMEndian::Big)?;
                    if member.value.info() == SOMTypeInfo::ImplicitLength {
                        let member_lengthfield = serializer.promise(self.lengthfield.size())?;
                        let member_start = serializer.offset();
                        member.value.serialize(serializer)?;
                        serializer.write_lengthfield(
                            member_lengthfield,
                            self.lengthfield,
                            serializer.offset() - member_start,
                        )?;
                    } else {
                        member.value.serialize(serializer)?;
                    }
                } else if member.mandatory {
                    return Err(SOMError::InvalidType(format!(
                        "Uninitialized mandatory member: {} at offset: {}",
                        member.key, offset
                    )));
                }
            }

            let size = serializer.offset() - offset;
            serializer.write_lengthfield(
                type_lengthfield,
                self.lengthfield,
                size - self.lengthfield.size(),
            )?;

            Ok(size)
        }

        fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
            let offset = parser.offset();

            let type_lengthfield = parser.read_lengthfield(self.lengthfield)?;
            let type_start = parser.offset();

            self.clear();
            while (parser.offset() - type_start) < type_lengthfield {
                let tag: u16 = parser.read_u16(SOMEndian::Big)? & TAG_MASK;
                let mut found: bool = false;
                for member in &mut self.members {
                    if !member.set && member.tag() == tag {
                        if member.value.info() == SOMTypeInfo::ImplicitLength {
                            let member_lengthfield = parser.read_lengthfield(self.lengthfield)?;
                            let member_start = parser.offset();
                            member.value.parse(parser)?;
                            if parser.offset() != (member_start + member_lengthfield) {
                                return Err(SOMError::InvalidPayload(format!(
                                    "Invalid Length-Field value: {} at offset: {}",
                                    member_lengthfield, member_start
                                )));
                            }
                        } else {
                            member.value.parse(parser)?;
                        }
                        member.set = true;
                        found = true;
                        break;
                    }
                }

                if !found {
                    let wiretype: usize = ((tag >> 8) & 0xFF) as usize;
                    let wiresize: Option<usize> = wire_size(wiretype);
                    if wiresize.is_some() {
                        parser.skip(wiresize.unwrap())?;
                    } else {
                        let skip = parser.read_lengthfield(self.lengthfield)?;
                        parser.skip(skip)?;
                    }
                }
            }

            for member in &mut self.members {
                if member.mandatory && !member.set {
                    return Err(SOMError::InvalidPayload(format!(
                        "Uninitialized mandatory member: : {} at offset: {}",
                        member.key, offset
                    )));
                }
            }

            let size = parser.offset() - offset;
            if type_lengthfield != (size - self.lengthfield.size()) {
                return Err(SOMError::InvalidPayload(format!(
                    "Invalid Length-Field value: {} at offset: {}",
                    type_lengthfield, offset
                )));
            }

            Ok(size)
        }

        fn size(&self) -> usize {
            let mut size: usize = 0;

            size += self.lengthfield.size();
            for member in &self.members {
                if member.set {
                    size += std::mem::size_of::<u16>(); // tag
                    if member.value.info() == SOMTypeInfo::ImplicitLength {
                        size += self.lengthfield.size();
                    }
                    size += member.value.size();
                }
            }

            size
        }

        fn info(&self) -> SOMTypeInfo {
            SOMTypeInfo::ExplicitLength
        }
    }
}

pub type SOMOptionalMember = wrapper::SOMTypeWrapper;
pub type SOMOptional = optionals::SOMOptionalType<SOMOptionalMember>;

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

                fn info(&self) -> SOMTypeInfo {
                    match self {
                        $(SOMTypeWrapper::$value(obj) => obj.info(),)*
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
        EnumU8(SOMu8Enum),
        EnumU16(SOMu16Enum),
        EnumU32(SOMu32Enum),
        EnumU64(SOMu64Enum),
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
        Struct(SOMStruct),
        Union(SOMUnion),
        String(SOMString),
        Optional(SOMOptional),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serialize_parse<T: SOMType>(obj1: &T, obj2: &mut T, data: &[u8]) {
        serialize(obj1, data);
        parse(obj2, data);
    }

    fn serialize<T: SOMType>(obj1: &T, data: &[u8]) {
        let size = data.len();
        assert_eq!(size, obj1.size());

        let mut buffer = vec![0u8; size];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(size, obj1.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, data);
    }

    fn parse<T: SOMType>(obj2: &mut T, data: &[u8]) {
        let mut parser = SOMParser::new(data);
        let size = data.len();
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
            let obj = SOMu8::new(1u8);
            assert_eq!(1u8, obj.get().unwrap());

            let mut obj = SOMu8::empty();
            assert_eq!(None, obj.get());
            obj.set(1u8);
            assert_eq!(1u8, obj.get().unwrap());

            let obj = SOMu16::new(SOMEndian::Big, 1u16);
            assert_eq!(1u16, obj.get().unwrap());

            let mut obj = SOMu16::empty(SOMEndian::Big);
            assert_eq!(None, obj.get());
            obj.set(1u16);
            assert_eq!(1u16, obj.get().unwrap());
        }

        // bool
        {
            let obj1 = SOMBool::new(true);
            let mut obj2 = SOMBool::empty();
            serialize_parse(&obj1, &mut obj2, &[0x01]);
            assert_eq!(true, obj2.get().unwrap());

            let obj1 = SOMBool::new(false);
            let mut obj2 = SOMBool::empty();
            serialize_parse(&obj1, &mut obj2, &[0x00]);
            assert_eq!(false, obj2.get().unwrap());

            let mut obj = SOMBool::new(true);
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

            let mut obj = SOMBool::empty();
            serialize_fail(&obj, &mut [0u8; 1], "Uninitialized Type at offset: 0");
            parse_fail(&mut obj, &[0x2], "Invalid Bool value: 2 at offset: 0");
        }

        // u8
        {
            let obj1 = SOMu8::new(195u8);
            let mut obj2 = SOMu8::empty();
            serialize_parse(&obj1, &mut obj2, &[0xC3]);
            assert_eq!(195u8, obj2.get().unwrap());

            let mut obj = SOMu8::new(195u8);
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

            let obj = SOMu8::empty();
            serialize_fail(&obj, &mut [0u8; 1], "Uninitialized Type at offset: 0");
        }

        // i8
        {
            let obj1 = SOMi8::new(-95i8);
            let mut obj2 = SOMi8::empty();
            serialize_parse(&obj1, &mut obj2, &[0xA1]);
            assert_eq!(-95i8, obj2.get().unwrap());

            let mut obj = SOMi8::new(-95i8);
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

            let obj = SOMi8::empty();
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
            obj1.add(SOMStructMember::Bool(SOMBool::new(true)));
            obj1.add(SOMStructMember::U16(SOMu16::new(SOMEndian::Big, 49200u16)));
            assert_eq!(2, obj1.len());

            let mut obj2 = SOMStruct::new();
            obj2.add(SOMStructMember::Bool(SOMBool::empty()));
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
            sub1.add(SOMStructMember::Bool(SOMBool::new(true)));
            sub1.add(SOMStructMember::U16(SOMu16::new(SOMEndian::Big, 49200u16)));

            let mut sub2 = SOMStruct::new();
            sub2.add(SOMStructMember::U16(SOMu16::new(
                SOMEndian::Little,
                49200u16,
            )));
            sub2.add(SOMStructMember::Bool(SOMBool::new(true)));

            let mut obj1 = SOMStruct::new();
            obj1.add(SOMStructMember::Struct(sub1));
            obj1.add(SOMStructMember::Struct(sub2));
            assert_eq!(2, obj1.len());

            let mut sub1 = SOMStruct::new();
            sub1.add(SOMStructMember::Bool(SOMBool::empty()));
            sub1.add(SOMStructMember::U16(SOMu16::empty(SOMEndian::Big)));

            let mut sub2 = SOMStruct::new();
            sub2.add(SOMStructMember::U16(SOMu16::empty(SOMEndian::Little)));
            sub2.add(SOMStructMember::Bool(SOMBool::empty()));

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
            obj.add(SOMStructMember::Bool(SOMBool::empty()));

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
                subsub1.add(SOMu8::new((i + 1) as u8));
                subsub2.add(SOMu8::empty());
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
            sub1.add(SOMUnionMember::Bool(SOMBool::empty()));
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
            sub1.add(SOMUnionMember::Bool(SOMBool::empty()));
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

        // struct with string
        {
            let mut obj1 = SOMStruct::new();
            obj1.add(SOMStructMember::String(SOMString::fixed(
                SOMStringEncoding::Utf8,
                SOMStringFormat::Plain,
                3,
            )));
            obj1.add(SOMStructMember::String(SOMString::dynamic(
                SOMLengthField::U8,
                SOMStringEncoding::Utf16Be,
                SOMStringFormat::Plain,
                1,
                3,
            )));
            assert_eq!(2, obj1.len());

            if let Some(SOMStructMember::String(sub)) = obj1.get_mut(0) {
                sub.set(String::from("foo"));
            } else {
                panic!();
            }

            if let Some(SOMStructMember::String(sub)) = obj1.get_mut(1) {
                sub.set(String::from("bar"));
            } else {
                panic!();
            }

            let mut obj2 = SOMStruct::new();
            obj2.add(SOMStructMember::String(SOMString::fixed(
                SOMStringEncoding::Utf8,
                SOMStringFormat::Plain,
                3,
            )));
            obj2.add(SOMStructMember::String(SOMString::dynamic(
                SOMLengthField::U8,
                SOMStringEncoding::Utf16Be,
                SOMStringFormat::Plain,
                1,
                3,
            )));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x66, 0x6F, 0x6F, // String-Memeber (UTF8)
                    0x06, // Lenght-Field (U8)
                    0x00, 0x62, 0x00, 0x61, 0x00, 0x72, // String-Memeber (UTF16)
                ],
            );
            assert_eq!(2, obj2.len());

            if let Some(SOMStructMember::String(sub)) = obj2.get(0) {
                assert_eq!(String::from("foo"), sub.get());
            } else {
                panic!();
            }

            if let Some(SOMStructMember::String(sub)) = obj2.get(1) {
                assert_eq!(String::from("bar"), sub.get());
            } else {
                panic!();
            }
        }

        // struct with optional
        {
            let mut sub1 = SOMOptional::new(SOMLengthField::U32);
            sub1.add(1, SOMOptionalMember::Bool(SOMBool::empty()), true);
            assert_eq!(1, sub1.len());

            if let Some(SOMOptionalMember::Bool(sub)) = sub1.get_mut(1) {
                sub.set(true);
            } else {
                panic!();
            }

            let mut obj1 = SOMStruct::new();
            obj1.add(SOMStructMember::Optional(sub1));
            assert_eq!(1, obj1.len());

            let mut sub2 = SOMOptional::new(SOMLengthField::U32);
            sub2.add(1, SOMOptionalMember::Bool(SOMBool::empty()), true);

            let mut obj2 = SOMStruct::new();
            obj2.add(SOMStructMember::Optional(sub2));

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x00, 0x00, 0x03, // Lenght-Field (U32)
                    0x00, 0x01, // TLV-Tag (U16)
                    0x01, // Bool-Member
                ],
            );
            assert_eq!(1, obj2.len());

            if let Some(SOMStructMember::Optional(sub)) = obj2.get(0) {
                assert_eq!(1, sub.len());
                if let Some(SOMStructMember::Bool(subsub)) = sub.get(1) {
                    assert!(subsub.get().unwrap());
                } else {
                    panic!();
                }
            } else {
                panic!();
            }
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

            obj.add(SOMArrayMember::Bool(SOMBool::new(true)));
            assert_eq!(1, obj.len());

            obj.add(SOMArrayMember::U8(SOMu8::new(1u8)));
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

            obj3.add(SOMUnionMember::Bool(SOMBool::empty()));
            obj3.get_mut(1);
            assert!(obj3.has_value());

            obj3.clear();
            assert!(!obj3.has_value());
        }

        // primitive union
        {
            let mut obj1 = SOMUnion::new(SOMTypeField::U8);
            obj1.add(SOMUnionMember::Bool(SOMBool::empty()));
            obj1.add(SOMUnionMember::U16(SOMu16::empty(SOMEndian::Big)));
            assert_eq!(2, obj1.len());

            if let Some(SOMUnionMember::U16(sub)) = obj1.get_mut(2) {
                sub.set(49200u16);
            } else {
                panic!();
            }
            assert!(obj1.has_value());

            let mut obj2 = SOMUnion::new(SOMTypeField::U8);
            obj2.add(SOMUnionMember::Bool(SOMBool::empty()));
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
            sub1.add(SOMStructMember::Bool(SOMBool::empty()));
            sub1.add(SOMStructMember::U16(SOMu16::empty(SOMEndian::Big)));

            let mut obj1 = SOMUnion::new(SOMTypeField::U16);
            obj1.add(SOMUnionMember::Bool(SOMBool::empty()));
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
            sub2.add(SOMStructMember::Bool(SOMBool::empty()));
            sub2.add(SOMStructMember::U16(SOMu16::empty(SOMEndian::Big)));

            let mut obj2 = SOMUnion::new(SOMTypeField::U16);
            obj2.add(SOMUnionMember::Bool(SOMBool::empty()));
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
            let mut obj = SOMu8Enum::new();
            assert_eq!(0, obj.len());
            assert!(!obj.has_value());
            assert!(obj.get().is_none());
            assert!(!obj.set(String::from("foo")));
            assert!(obj.get().is_none());
        }

        // u8 enum
        {
            let mut obj1 = SOMu8Enum::new();
            obj1.add(String::from("A"), 23u8);
            obj1.add(String::from("B"), 42u8);
            assert_eq!(2, obj1.len());
            assert!(!obj1.has_value());

            assert!(obj1.set(String::from("A")));
            assert!(obj1.has_value());
            assert_eq!(23u8, obj1.get().unwrap());

            let mut obj2 = SOMu8Enum::new();
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

            let mut obj3 = SOMu16Enum::new(SOMEndian::Little);
            obj3.add(String::from("A"), 49200u16);
            assert!(obj3.set(String::from("A")));

            let mut obj4 = SOMu16Enum::new(SOMEndian::Little);
            obj4.add(String::from("A"), 49200u16);
            serialize_parse(&obj3, &mut obj4, &[0x30, 0xC0]);
            assert_eq!(49200u16, obj4.get().unwrap());
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
            let mut obj1 = SOMu8Enum::new();

            obj1.add(String::from("A"), 23u8);
            assert_eq!(1, obj1.len());

            obj1.add(String::from("A"), 42u8);
            assert_eq!(1, obj1.len());
        }

        // invalid value
        {
            let mut obj1 = SOMu8Enum::new();

            obj1.add(String::from("A"), 23u8);
            assert_eq!(1, obj1.len());

            obj1.add(String::from("B"), 23u8);
            assert_eq!(1, obj1.len());
        }
    }

    #[test]
    fn test_som_string() {
        // empty strings
        {
            let obj1 = SOMString::fixed(SOMStringEncoding::Utf8, SOMStringFormat::Plain, 0);
            assert!(!obj1.is_dynamic());
            assert_eq!(0, obj1.len());
            assert_eq!(0, obj1.size());

            let obj2 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf8,
                SOMStringFormat::Plain,
                0,
                3,
            );
            assert!(obj2.is_dynamic());
            assert_eq!(0, obj2.len());
            assert_eq!(4, obj2.size());

            let obj3 = SOMString::fixed(
                SOMStringEncoding::Utf8,
                SOMStringFormat::WithBOMandTermination,
                4,
            );
            assert!(!obj3.is_dynamic());
            assert_eq!(4, obj3.len());
            assert_eq!(4, obj3.size());

            let obj4 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf8,
                SOMStringFormat::WithBOMandTermination,
                4,
                7,
            );
            assert!(obj4.is_dynamic());
            assert_eq!(4, obj4.len());
            assert_eq!(8, obj4.size());

            let obj5 = SOMString::fixed(SOMStringEncoding::Utf16Be, SOMStringFormat::Plain, 0);
            assert!(!obj5.is_dynamic());
            assert_eq!(0, obj5.len());
            assert_eq!(0, obj5.size());

            let obj6 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf16Be,
                SOMStringFormat::Plain,
                0,
                3,
            );
            assert!(obj6.is_dynamic());
            assert_eq!(0, obj6.len());
            assert_eq!(4, obj6.size());

            let obj7 = SOMString::fixed(
                SOMStringEncoding::Utf16Le,
                SOMStringFormat::WithBOMandTermination,
                5,
            );
            assert!(!obj7.is_dynamic());
            assert_eq!(2, obj7.len());
            assert_eq!(4 + 6, obj7.size());

            let obj8 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf16Le,
                SOMStringFormat::WithBOMandTermination,
                2,
                5,
            );
            assert!(obj8.is_dynamic());
            assert_eq!(2, obj8.len());
            assert_eq!(8, obj8.size());
        }

        // fixed utf8 string without bom and termination
        {
            let mut obj1 = SOMString::fixed(SOMStringEncoding::Utf8, SOMStringFormat::Plain, 3);
            assert!(obj1.set(String::from("foo")));
            assert_eq!(3, obj1.len());
            assert_eq!(3, obj1.size());

            let mut obj2 = SOMString::fixed(SOMStringEncoding::Utf8, SOMStringFormat::Plain, 3);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x66, 0x6F, 0x6F, // Content
                ],
            );

            assert_eq!(String::from("foo"), obj2.get());
            assert_eq!(3, obj2.len());
            assert_eq!(3, obj2.size());

            serialize_fail(
                &obj2,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 1",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 1",
            );
        }

        // fixed utf8 string with bom and termination
        {
            let mut obj1 = SOMString::fixed(
                SOMStringEncoding::Utf8,
                SOMStringFormat::WithBOMandTermination,
                7,
            );
            assert_eq!(3 + 1, obj1.len());
            assert_eq!(3 + 3 + 1, obj1.size());

            assert!(obj1.set(String::from("foo")));
            assert_eq!(3 + 3 + 1, obj1.len());
            assert_eq!(3 + 3 + 1, obj1.size());

            let mut obj2 = SOMString::fixed(
                SOMStringEncoding::Utf8,
                SOMStringFormat::WithBOMandTermination,
                7,
            );
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0xEF, 0xBB, 0xBF, // BOM
                    0x66, 0x6F, 0x6F, // Content
                    0x00, // Termination
                ],
            );

            assert_eq!(String::from("foo"), obj2.get());
            assert_eq!(3 + 3 + 1, obj2.len());
            assert_eq!(3 + 3 + 1, obj2.size());
        }

        // fixed utf16-be string with bom and termination
        {
            let mut obj1 = SOMString::fixed(
                SOMStringEncoding::Utf16Be,
                SOMStringFormat::WithBOMandTermination,
                5,
            );
            assert_eq!(1 + 1, obj1.len());
            assert_eq!((1 + 3 + 1) * 2, obj1.size());

            assert!(obj1.set(String::from("foo")));
            assert_eq!(1 + 3 + 1, obj1.len());
            assert_eq!((1 + 3 + 1) * 2, obj1.size());

            let mut obj2 = SOMString::fixed(
                SOMStringEncoding::Utf16Be,
                SOMStringFormat::WithBOMandTermination,
                5,
            );
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0xFE, 0xFF, // BOM
                    0x00, 0x66, 0x00, 0x6F, 0x00, 0x6F, // Content
                    0x00, 0x00, // Termination
                ],
            );

            assert_eq!(String::from("foo"), obj2.get());
            assert_eq!(1 + 3 + 1, obj2.len());
            assert_eq!((1 + 3 + 1) * 2, obj2.size());
        }

        // incomplete fixed utf16-le string with termination only
        {
            let mut obj1 = SOMString::fixed(
                SOMStringEncoding::Utf16Le,
                SOMStringFormat::WithTermination,
                10,
            );
            assert_eq!(1, obj1.len());
            assert_eq!((9 + 1) * 2, obj1.size());

            assert!(obj1.set(String::from("foo")));
            assert_eq!(3 + 1, obj1.len());
            assert_eq!((3 + 6 + 1) * 2, obj1.size());

            let mut obj2 = SOMString::fixed(
                SOMStringEncoding::Utf16Le,
                SOMStringFormat::WithTermination,
                10,
            );
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x66, 0x00, 0x6F, 0x00, 0x6F, 0x00, // Content
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    0x00, // Filler
                    0x00, 0x00, // Termination
                ],
            );

            assert_eq!(String::from("foo"), obj2.get());
            assert_eq!(3 + 1, obj2.len());
            assert_eq!((3 + 6 + 1) * 2, obj2.size());
        }

        // dynamic utf8 string without bom and termination
        {
            let mut obj1 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf8,
                SOMStringFormat::Plain,
                0,
                3,
            );
            assert!(obj1.set(String::from("foo")));
            assert_eq!(3, obj1.len());
            assert_eq!(4 + 3, obj1.size());

            let mut obj2 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf8,
                SOMStringFormat::Plain,
                0,
                3,
            );
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x00, 0x00, 0x03, // Length-Field (U32)
                    0x66, 0x6F, 0x6F, // Content
                ],
            );

            assert_eq!(String::from("foo"), obj2.get());
            assert_eq!(3, obj2.len());
            assert_eq!(4 + 3, obj2.size());

            serialize_fail(
                &obj2,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 4",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 4",
            );
        }

        // dynamic utf8 string with bom and termination
        {
            let mut obj1 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf8,
                SOMStringFormat::WithBOMandTermination,
                4,
                7,
            );
            assert_eq!(3 + 1, obj1.len());
            assert_eq!(4 + 3 + 1, obj1.size());

            assert!(obj1.set(String::from("foo")));
            assert_eq!(3 + 3 + 1, obj1.len());
            assert_eq!(4 + 3 + 3 + 1, obj1.size());

            let mut obj2 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf8,
                SOMStringFormat::WithBOMandTermination,
                4,
                7,
            );
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x00, 0x00, 0x07, // Length-Field (U32)
                    0xEF, 0xBB, 0xBF, // BOM
                    0x66, 0x6F, 0x6F, // Content
                    0x00, // Termination
                ],
            );

            assert_eq!(String::from("foo"), obj2.get());
            assert_eq!(3 + 3 + 1, obj2.len());
            assert_eq!(4 + 3 + 3 + 1, obj2.size());
        }

        // dynamic utf16-be string with bom and termination
        {
            let mut obj1 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf16Be,
                SOMStringFormat::WithBOMandTermination,
                2,
                5,
            );
            assert_eq!(1 + 1, obj1.len());
            assert_eq!(4 + (1 + 1) * 2, obj1.size());

            assert!(obj1.set(String::from("foo")));
            assert_eq!(1 + 3 + 1, obj1.len());
            assert_eq!(4 + (1 + 3 + 1) * 2, obj1.size());

            let mut obj2 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf16Be,
                SOMStringFormat::WithBOMandTermination,
                2,
                5,
            );
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x00, 0x00, 0x0A, // Length-Field (U32)
                    0xFE, 0xFF, // BOM
                    0x00, 0x66, 0x00, 0x6F, 0x00, 0x6F, // Content
                    0x00, 0x00, // Termination
                ],
            );

            assert_eq!(String::from("foo"), obj2.get());
            assert_eq!(1 + 3 + 1, obj2.len());
            assert_eq!(4 + (1 + 3 + 1) * 2, obj2.size());
        }

        // incomplete dynamic utf16-le string with bom only
        {
            let mut obj1 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf16Le,
                SOMStringFormat::WithBOM,
                0,
                10,
            );
            assert_eq!(1, obj1.len());
            assert_eq!(4 + 1 * 2, obj1.size());

            assert!(obj1.set(String::from("foo")));
            assert_eq!(1 + 3, obj1.len());
            assert_eq!(4 + (1 + 3) * 2, obj1.size());

            let mut obj2 = SOMString::dynamic(
                SOMLengthField::U32,
                SOMStringEncoding::Utf16Le,
                SOMStringFormat::WithBOM,
                0,
                10,
            );
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x00, 0x00, 0x08, // Length-Field (U32)
                    0xFF, 0xFE, // BOM
                    0x66, 0x00, 0x6F, 0x00, 0x6F, 0x00, // Content
                ],
            );

            assert_eq!(String::from("foo"), obj2.get());
            assert_eq!(1 + 3, obj2.len());
            assert_eq!(4 + (1 + 3) * 2, obj2.size());
        }

        // incomplete length
        {
            let mut obj1 = SOMString::fixed(SOMStringEncoding::Utf8, SOMStringFormat::Plain, 3);
            assert!(!obj1.set(String::from("foobar")));
            assert_eq!(0, obj1.len());
            assert_eq!(3, obj1.size());
            assert!(obj1.set(String::from("f")));
            assert_eq!(1, obj1.len());
            assert_eq!(3, obj1.size());

            serialize(&obj1, &mut [0x66, 0x00, 0x00]);

            let mut obj2 = SOMString::dynamic(
                SOMLengthField::U8,
                SOMStringEncoding::Utf8,
                SOMStringFormat::Plain,
                2,
                3,
            );
            assert!(!obj2.set(String::from("foobar")));
            assert_eq!(0, obj2.len());
            assert_eq!(1, obj2.size());
            assert!(obj2.set(String::from("f")));
            assert_eq!(1, obj2.len());
            assert_eq!(1 + 1, obj2.size());

            serialize_fail(
                &obj2,
                &mut [0u8; 2],
                "Invalid String length: 1 at offset: 0",
            );
        }
    }

    #[test]
    fn test_som_optional() {
        // empty optional
        {
            let obj1 = SOMOptional::new(SOMLengthField::U32);
            assert_eq!(0, obj1.len());

            let mut obj2 = SOMOptional::new(SOMLengthField::U32);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x00, 0x00, 0x00, // Length-Field (U32)
                ],
            );
            assert_eq!(0, obj2.len());
        }

        // simple optional
        {
            let mut obj1 = SOMOptional::new(SOMLengthField::U32);
            obj1.add(1, SOMOptionalMember::Bool(SOMBool::empty()), true);
            obj1.add(
                2,
                SOMOptionalMember::U16(SOMu16::empty(SOMEndian::Big)),
                false,
            );
            assert_eq!(2, obj1.len());

            if let Some(SOMUnionMember::Bool(sub)) = obj1.get_mut(1) {
                sub.set(true);
            } else {
                panic!();
            }

            if let Some(SOMUnionMember::U16(sub)) = obj1.get_mut(2) {
                sub.set(49200u16);
            } else {
                panic!();
            }

            let mut obj2 = SOMOptional::new(SOMLengthField::U32);
            obj2.add(1, SOMOptionalMember::Bool(SOMBool::empty()), true);
            obj2.add(
                2,
                SOMOptionalMember::U16(SOMu16::empty(SOMEndian::Big)),
                false,
            );

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x00, 0x00, 0x07, // Length-Field (U32)
                    0x00, 0x01, // TLV-Tag (U16)
                    0x01, // Bool-Memeber
                    0x10, 0x02, // TLV-Tag (U16)
                    0xC0, 0x30, // U16-Member
                ],
            );
            assert_eq!(2, obj2.len());

            if let Some(SOMStructMember::Bool(sub)) = obj2.get(1) {
                assert_eq!(true, sub.get().unwrap());
            } else {
                panic!();
            }

            if let Some(SOMStructMember::U16(sub)) = obj2.get(2) {
                assert_eq!(49200u16, sub.get().unwrap());
            } else {
                panic!();
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 0],
                "Serializer exausted at offset: 0 for Object size: 4",
            );
            parse_fail(
                &mut obj2,
                &[0u8; 0],
                "Parser exausted at offset: 0 for Object size: 4",
            );
        }

        // complex optional
        {
            let sub11 = SOMString::fixed(SOMStringEncoding::Utf8, SOMStringFormat::Plain, 3);
            let sub12 = SOMu16Array::dynamic(SOMLengthField::U8, 1, 3);
            let mut obj1 = SOMOptional::new(SOMLengthField::U32);
            obj1.add(1, SOMOptionalMember::Bool(SOMBool::empty()), true);
            obj1.add(2, SOMOptionalMember::String(sub11), true);
            obj1.add(3, SOMOptionalMember::ArrayU16(sub12), false);
            assert_eq!(3, obj1.len());

            if let Some(SOMUnionMember::Bool(sub)) = obj1.get_mut(1) {
                sub.set(true);
            } else {
                panic!();
            }

            if let Some(SOMUnionMember::String(sub)) = obj1.get_mut(2) {
                sub.set(String::from("foo"));
            } else {
                panic!();
            }

            if let Some(SOMUnionMember::ArrayU16(sub)) = obj1.get_mut(3) {
                for i in 0..3 {
                    sub.add(SOMu16::new(SOMEndian::Big, (i + 1) as u16));
                }
            } else {
                panic!();
            }

            let sub21 = SOMString::fixed(SOMStringEncoding::Utf8, SOMStringFormat::Plain, 3);
            let sub22 = SOMu16Array::dynamic(SOMLengthField::U8, 1, 3);
            let mut obj2 = SOMOptional::new(SOMLengthField::U32);
            obj2.add(1, SOMOptionalMember::Bool(SOMBool::empty()), true);
            obj2.add(2, SOMOptionalMember::String(sub21), true);
            obj2.add(3, SOMOptionalMember::ArrayU16(sub22), false);
            assert_eq!(3, obj2.len());

            if let Some(SOMUnionMember::ArrayU16(sub)) = obj2.get_mut(3) {
                for _ in 0..3 {
                    sub.add(SOMu16::empty(SOMEndian::Big));
                }
            } else {
                panic!();
            }

            serialize_parse(
                &obj1,
                &mut obj2,
                &[
                    0x00, 0x00, 0x00, 0x15, // Length-Field (U32)
                    0x00, 0x01, // TLV-Tag (U16)
                    0x01, // Bool-Memeber
                    0x40, 0x02, // TLV-Tag (U16)
                    0x00, 0x00, 0x00, 0x03, // Length-Field (U32)
                    0x66, 0x6F, 0x6F, // String-Member
                    0x40, 0x03, // TLV-Tag (U16)
                    0x06, // Length-Field (U8)
                    0x00, 0x01, // Array-Mamber (U16)
                    0x00, 0x02, // Array-Mamber (U16)
                    0x00, 0x03, // Array-Mamber (U16)
                ],
            );
            assert_eq!(3, obj2.len());

            if let Some(SOMStructMember::Bool(sub)) = obj2.get(1) {
                assert_eq!(true, sub.get().unwrap());
            } else {
                panic!();
            }

            if let Some(SOMStructMember::String(sub)) = obj2.get(2) {
                assert_eq!(String::from("foo"), sub.get());
            } else {
                panic!();
            }

            if let Some(SOMStructMember::ArrayU16(sub)) = obj2.get(3) {
                assert_eq!(3, sub.len());
                for i in 0..3 {
                    assert_eq!((i + 1) as u16, sub.get(i).unwrap().get().unwrap());
                }
            } else {
                panic!();
            }
        }

        // missing mandatory
        {
            let mut obj1 = SOMOptional::new(SOMLengthField::U32);
            obj1.add(1, SOMOptionalMember::Bool(SOMBool::empty()), true);
            obj1.add(
                2,
                SOMOptionalMember::U16(SOMu16::empty(SOMEndian::Big)),
                true,
            );

            serialize_fail(
                &obj1,
                &mut [0u8; 11],
                "Uninitialized mandatory member: 1 at offset: 0",
            );

            if let Some(SOMUnionMember::Bool(sub)) = obj1.get_mut(1) {
                sub.set(true);
            } else {
                panic!();
            }

            serialize_fail(
                &obj1,
                &mut [0u8; 11],
                "Uninitialized mandatory member: 2 at offset: 0",
            );

            parse_fail(
                &mut obj1,
                &[
                    0x00, 0x00, 0x00, 0x00, // Length-Field (U32)
                ],
                "Uninitialized mandatory member: : 1 at offset: 0",
            );

            parse_fail(
                &mut obj1,
                &[
                    0x00, 0x00, 0x00, 0x03, // Length-Field (U32)
                    0x00, 0x01, // TLV-Tag (U16)
                    0x01, // Bool-Memeber
                ],
                "Uninitialized mandatory member: : 2 at offset: 0",
            );
        }
    }
}
