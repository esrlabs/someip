// wip: someip types

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use ux::{i24, u24};

pub trait SOMType {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError>;
    fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError>;
    fn size(&self) -> usize;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SOMEndian {
    Big,
    Little,
}

pub trait SOMTypeWithEndian {
    fn endian(&self) -> SOMEndian;
}

#[derive(Debug)]
pub enum SOMError {
    BufferExhausted(String),
    InvalidPayload(String),
    UninitializedType(String),
}

pub struct SOMSerializer<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}

impl<'a> SOMSerializer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> SOMSerializer<'a> {
        SOMSerializer { buffer, offset: 0 }
    }

    fn offset(&self) -> usize {
        self.offset
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

#[derive(Debug)]
pub struct SOMPrimitive<T> {
    endian: SOMEndian,
    value: Option<T>,
}

impl<T: Copy + Clone> SOMPrimitive<T> {
    pub fn empty(endian: SOMEndian) -> SOMPrimitive<T> {
        SOMPrimitive {
            endian,
            value: None,
        }
    }

    pub fn new(endian: SOMEndian, value: T) -> SOMPrimitive<T> {
        SOMPrimitive {
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

impl<T> SOMTypeWithEndian for SOMPrimitive<T> {
    fn endian(&self) -> SOMEndian {
        self.endian
    }
}

impl SOMType for SOMPrimitive<bool> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_bool(value)?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<u8> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_u8(value)?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<i8> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_i8(value)?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<u16> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_u16(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<i16> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_i16(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<u24> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_u24(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<i24> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_i24(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<u32> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_u32(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<i32> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_i32(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<u64> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_u64(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<i64> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_i64(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<f32> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_f32(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

impl SOMType for SOMPrimitive<f64> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_f64(value, self.endian())?,
            None => {
                return Err(SOMError::UninitializedType(format!(
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

pub enum SOMTypeWrapper {
    Bool(SOMPrimitive<bool>),
    U8(SOMPrimitive<u8>),
    I8(SOMPrimitive<i8>),
    U16(SOMPrimitive<u16>),
    I16(SOMPrimitive<i16>),
    U24(SOMPrimitive<u24>),
    I24(SOMPrimitive<i24>),
    U32(SOMPrimitive<u32>),
    I32(SOMPrimitive<i32>),
    U64(SOMPrimitive<u64>),
    I64(SOMPrimitive<i64>),
    F32(SOMPrimitive<f32>),
    F64(SOMPrimitive<f64>),
    Struct(SOMStruct<SOMTypeWrapper>),
}

impl SOMType for SOMTypeWrapper {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        match self {
            SOMTypeWrapper::Bool(obj) => obj.serialize(serializer),
            SOMTypeWrapper::U8(obj) => obj.serialize(serializer),
            SOMTypeWrapper::I8(obj) => obj.serialize(serializer),
            SOMTypeWrapper::U16(obj) => obj.serialize(serializer),
            SOMTypeWrapper::I16(obj) => obj.serialize(serializer),
            SOMTypeWrapper::U24(obj) => obj.serialize(serializer),
            SOMTypeWrapper::I24(obj) => obj.serialize(serializer),
            SOMTypeWrapper::U32(obj) => obj.serialize(serializer),
            SOMTypeWrapper::I32(obj) => obj.serialize(serializer),
            SOMTypeWrapper::U64(obj) => obj.serialize(serializer),
            SOMTypeWrapper::I64(obj) => obj.serialize(serializer),
            SOMTypeWrapper::F32(obj) => obj.serialize(serializer),
            SOMTypeWrapper::F64(obj) => obj.serialize(serializer),
            SOMTypeWrapper::Struct(obj) => obj.serialize(serializer),
        }
    }

    fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
        match self {
            SOMTypeWrapper::Bool(obj) => obj.parse(parser),
            SOMTypeWrapper::U8(obj) => obj.parse(parser),
            SOMTypeWrapper::I8(obj) => obj.parse(parser),
            SOMTypeWrapper::U16(obj) => obj.parse(parser),
            SOMTypeWrapper::I16(obj) => obj.parse(parser),
            SOMTypeWrapper::U24(obj) => obj.parse(parser),
            SOMTypeWrapper::I24(obj) => obj.parse(parser),
            SOMTypeWrapper::U32(obj) => obj.parse(parser),
            SOMTypeWrapper::I32(obj) => obj.parse(parser),
            SOMTypeWrapper::U64(obj) => obj.parse(parser),
            SOMTypeWrapper::I64(obj) => obj.parse(parser),
            SOMTypeWrapper::F32(obj) => obj.parse(parser),
            SOMTypeWrapper::F64(obj) => obj.parse(parser),
            SOMTypeWrapper::Struct(obj) => obj.parse(parser),
        }
    }

    fn size(&self) -> usize {
        match self {
            SOMTypeWrapper::Bool(obj) => obj.size(),
            SOMTypeWrapper::U8(obj) => obj.size(),
            SOMTypeWrapper::I8(obj) => obj.size(),
            SOMTypeWrapper::U16(obj) => obj.size(),
            SOMTypeWrapper::I16(obj) => obj.size(),
            SOMTypeWrapper::U24(obj) => obj.size(),
            SOMTypeWrapper::I24(obj) => obj.size(),
            SOMTypeWrapper::U32(obj) => obj.size(),
            SOMTypeWrapper::I32(obj) => obj.size(),
            SOMTypeWrapper::U64(obj) => obj.size(),
            SOMTypeWrapper::I64(obj) => obj.size(),
            SOMTypeWrapper::F32(obj) => obj.size(),
            SOMTypeWrapper::F64(obj) => obj.size(),
            SOMTypeWrapper::Struct(obj) => obj.size(),
        }
    }
}

#[derive(Debug)]
pub struct SOMStruct<T: SOMType> {
    members: Vec<T>,
}

impl<T: SOMType> SOMStruct<T> {
    pub fn new() -> SOMStruct<T> {
        SOMStruct {
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
}

impl<T: SOMType> SOMType for SOMStruct<T> {
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
                SOMError::UninitializedType(msg) => assert_eq!(msg, error),
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
                SOMError::UninitializedType(msg) => assert_eq!(msg, error),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_som_primitive() {
        // generic
        {
            let obj: SOMPrimitive<u8> = SOMPrimitive::new(SOMEndian::Big, 1u8);
            assert_eq!(SOMEndian::Big, obj.endian());
            assert_eq!(1u8, obj.get().unwrap());

            let mut obj: SOMPrimitive<u8> = SOMPrimitive::empty(SOMEndian::Little);
            assert_eq!(SOMEndian::Little, obj.endian());
            assert_eq!(None, obj.get());
            obj.set(1u8);
            assert_eq!(1u8, obj.get().unwrap());
        }

        // bool
        {
            let obj1: SOMPrimitive<bool> = SOMPrimitive::new(SOMEndian::Big, true);
            let mut obj2: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0x01]);
            assert_eq!(true, obj2.get().unwrap());

            let obj1: SOMPrimitive<bool> = SOMPrimitive::new(SOMEndian::Big, false);
            let mut obj2: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0x00]);
            assert_eq!(false, obj2.get().unwrap());

            let obj1: SOMPrimitive<bool> = SOMPrimitive::new(SOMEndian::Little, true);
            let mut obj2: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x01]);
            assert_eq!(true, obj2.get().unwrap());

            let mut obj: SOMPrimitive<bool> = SOMPrimitive::new(SOMEndian::Big, true);
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

            let mut obj: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 1], "Uninitialized Type at offset: 0");
            parse_fail(&mut obj, &[0x2], "Invalid Bool value: 2 at offset: 0");
        }

        // u8
        {
            let obj1: SOMPrimitive<u8> = SOMPrimitive::new(SOMEndian::Big, 195u8);
            let mut obj2: SOMPrimitive<u8> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xC3]);
            assert_eq!(195u8, obj2.get().unwrap());

            let obj1: SOMPrimitive<u8> = SOMPrimitive::new(SOMEndian::Little, 195u8);
            let mut obj2: SOMPrimitive<u8> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0xC3]);
            assert_eq!(195u8, obj2.get().unwrap());

            let mut obj: SOMPrimitive<u8> = SOMPrimitive::new(SOMEndian::Big, 195u8);
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

            let obj: SOMPrimitive<u8> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 1], "Uninitialized Type at offset: 0");
        }

        // i8
        {
            let obj1: SOMPrimitive<i8> = SOMPrimitive::new(SOMEndian::Big, -95i8);
            let mut obj2: SOMPrimitive<i8> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xA1]);
            assert_eq!(-95i8, obj2.get().unwrap());

            let obj1: SOMPrimitive<i8> = SOMPrimitive::new(SOMEndian::Little, -95i8);
            let mut obj2: SOMPrimitive<i8> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0xA1]);
            assert_eq!(-95i8, obj2.get().unwrap());

            let mut obj: SOMPrimitive<i8> = SOMPrimitive::new(SOMEndian::Big, -95i8);
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

            let obj: SOMPrimitive<i8> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 1], "Uninitialized Type at offset: 0");
        }

        // u16
        {
            let obj1: SOMPrimitive<u16> = SOMPrimitive::new(SOMEndian::Big, 49200u16);
            let mut obj2: SOMPrimitive<u16> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xC0, 0x30]);
            assert_eq!(49200u16, obj2.get().unwrap());

            let obj1: SOMPrimitive<u16> = SOMPrimitive::new(SOMEndian::Little, 49200u16);
            let mut obj2: SOMPrimitive<u16> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x30, 0xC0]);
            assert_eq!(49200u16, obj2.get().unwrap());

            let mut obj: SOMPrimitive<u16> = SOMPrimitive::new(SOMEndian::Big, 49200u16);
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

            let obj: SOMPrimitive<u16> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // i16
        {
            let obj1: SOMPrimitive<i16> = SOMPrimitive::new(SOMEndian::Big, -9200i16);
            let mut obj2: SOMPrimitive<i16> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xDC, 0x10]);
            assert_eq!(-9200i16, obj2.get().unwrap());

            let obj1: SOMPrimitive<i16> = SOMPrimitive::new(SOMEndian::Little, -9200i16);
            let mut obj2: SOMPrimitive<i16> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x10, 0xDC]);
            assert_eq!(-9200i16, obj2.get().unwrap());

            let mut obj: SOMPrimitive<i16> = SOMPrimitive::new(SOMEndian::Big, -9200i16);
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

            let obj: SOMPrimitive<i16> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // u24
        {
            let obj1: SOMPrimitive<u24> = SOMPrimitive::new(SOMEndian::Big, u24::new(12513060u32));
            let mut obj2: SOMPrimitive<u24> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xBE, 0xEF, 0x24]);
            assert_eq!(u24::new(12513060u32), obj2.get().unwrap());

            let obj1: SOMPrimitive<u24> =
                SOMPrimitive::new(SOMEndian::Little, u24::new(12513060u32));
            let mut obj2: SOMPrimitive<u24> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x24, 0xEF, 0xBE]);
            assert_eq!(u24::new(12513060u32), obj2.get().unwrap());

            let mut obj: SOMPrimitive<u24> =
                SOMPrimitive::new(SOMEndian::Big, u24::new(12513060u32));
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

            let obj: SOMPrimitive<u24> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // i24
        {
            let obj1: SOMPrimitive<i24> = SOMPrimitive::new(SOMEndian::Big, i24::new(-2513060i32));
            let mut obj2: SOMPrimitive<i24> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xD9, 0xA7, 0x5C]);
            assert_eq!(i24::new(-2513060i32), obj2.get().unwrap());

            let obj1: SOMPrimitive<i24> =
                SOMPrimitive::new(SOMEndian::Little, i24::new(-2513060i32));
            let mut obj2: SOMPrimitive<i24> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x5C, 0xA7, 0xD9]);
            assert_eq!(i24::new(-2513060i32), obj2.get().unwrap());

            let mut obj: SOMPrimitive<i24> =
                SOMPrimitive::new(SOMEndian::Big, i24::new(-2513060i32));
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

            let obj: SOMPrimitive<i24> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // u32
        {
            let obj1: SOMPrimitive<u32> = SOMPrimitive::new(SOMEndian::Big, 3405691582u32);
            let mut obj2: SOMPrimitive<u32> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xCA, 0xFE, 0xBA, 0xBE]);
            assert_eq!(3405691582u32, obj2.get().unwrap());

            let obj1: SOMPrimitive<u32> = SOMPrimitive::new(SOMEndian::Little, 3405691582u32);
            let mut obj2: SOMPrimitive<u32> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0xBE, 0xBA, 0xFE, 0xCA]);
            assert_eq!(3405691582u32, obj2.get().unwrap());

            let mut obj: SOMPrimitive<u32> = SOMPrimitive::new(SOMEndian::Big, 3405691582u32);
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

            let obj: SOMPrimitive<u32> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // i32
        {
            let obj1: SOMPrimitive<i32> = SOMPrimitive::new(SOMEndian::Big, -405691582i32);
            let mut obj2: SOMPrimitive<i32> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0xE7, 0xD1, 0xA3, 0x42]);
            assert_eq!(-405691582i32, obj2.get().unwrap());

            let obj1: SOMPrimitive<i32> = SOMPrimitive::new(SOMEndian::Little, -405691582i32);
            let mut obj2: SOMPrimitive<i32> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x42, 0xA3, 0xD1, 0xE7]);
            assert_eq!(-405691582i32, obj2.get().unwrap());

            let mut obj: SOMPrimitive<i32> = SOMPrimitive::new(SOMEndian::Big, -405691582i32);
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

            let obj: SOMPrimitive<i32> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // u64
        {
            let obj1: SOMPrimitive<u64> =
                SOMPrimitive::new(SOMEndian::Big, 16045704242864831166u64);
            let mut obj2: SOMPrimitive<u64> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0xDE, 0xAD, 0xCA, 0xFE, 0xBE, 0xEF, 0xBA, 0xBE],
            );
            assert_eq!(16045704242864831166u64, obj2.get().unwrap());

            let obj1: SOMPrimitive<u64> =
                SOMPrimitive::new(SOMEndian::Little, 16045704242864831166u64);
            let mut obj2: SOMPrimitive<u64> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0xBE, 0xBA, 0xEF, 0xBE, 0xFE, 0xCA, 0xAD, 0xDE],
            );
            assert_eq!(16045704242864831166u64, obj2.get().unwrap());

            let mut obj: SOMPrimitive<u64> =
                SOMPrimitive::new(SOMEndian::Big, 16045704242864831166u64);
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

            let obj: SOMPrimitive<u64> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // i64
        {
            let obj1: SOMPrimitive<i64> =
                SOMPrimitive::new(SOMEndian::Big, -6045704242864831166i64);
            let mut obj2: SOMPrimitive<i64> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0xAC, 0x19, 0x58, 0x05, 0xCA, 0xF8, 0x45, 0x42],
            );
            assert_eq!(-6045704242864831166i64, obj2.get().unwrap());

            let obj1: SOMPrimitive<i64> =
                SOMPrimitive::new(SOMEndian::Little, -6045704242864831166i64);
            let mut obj2: SOMPrimitive<i64> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0x42, 0x45, 0xF8, 0xCA, 0x05, 0x58, 0x19, 0xAC],
            );
            assert_eq!(-6045704242864831166i64, obj2.get().unwrap());

            let mut obj: SOMPrimitive<i64> =
                SOMPrimitive::new(SOMEndian::Big, -6045704242864831166i64);
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

            let obj: SOMPrimitive<i64> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // f32
        {
            let obj1: SOMPrimitive<f32> = SOMPrimitive::new(SOMEndian::Big, 1.0f32);
            let mut obj2: SOMPrimitive<f32> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(&obj1, &mut obj2, &[0x3F, 0x80, 0x00, 0x00]);
            assert_eq!(1.0f32, obj2.get().unwrap());

            let obj1: SOMPrimitive<f32> = SOMPrimitive::new(SOMEndian::Little, 1.0f32);
            let mut obj2: SOMPrimitive<f32> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(&obj1, &mut obj2, &[0x00, 0x00, 0x80, 0x3F]);
            assert_eq!(1.0f32, obj2.get().unwrap());

            let mut obj: SOMPrimitive<f32> = SOMPrimitive::new(SOMEndian::Big, 1.0f32);
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

            let obj: SOMPrimitive<f32> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }

        // f64
        {
            let obj1: SOMPrimitive<f64> = SOMPrimitive::new(SOMEndian::Big, 1.0f64);
            let mut obj2: SOMPrimitive<f64> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            );
            assert_eq!(1.0f64, obj2.get().unwrap());

            let obj1: SOMPrimitive<f64> = SOMPrimitive::new(SOMEndian::Little, 1.0f64);
            let mut obj2: SOMPrimitive<f64> = SOMPrimitive::empty(SOMEndian::Little);
            serialize_parse(
                &obj1,
                &mut obj2,
                &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F],
            );
            assert_eq!(1.0f64, obj2.get().unwrap());

            let mut obj: SOMPrimitive<f64> = SOMPrimitive::new(SOMEndian::Big, 1.0f64);
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

            let obj: SOMPrimitive<f64> = SOMPrimitive::empty(SOMEndian::Big);
            serialize_fail(&obj, &mut [0u8; 2], "Uninitialized Type at offset: 0");
        }
    }

    #[test]
    fn test_som_struct() {
        // empty struct
        {
            let obj1: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            assert_eq!(0, obj1.len());

            let mut obj2: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            serialize_parse(&obj1, &mut obj2, &[]);
            assert_eq!(0, obj2.len());
        }

        // simple struct
        {
            let mut obj1: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            obj1.add(SOMTypeWrapper::Bool(SOMPrimitive::new(
                SOMEndian::Big,
                true,
            )));
            obj1.add(SOMTypeWrapper::U16(SOMPrimitive::new(
                SOMEndian::Big,
                49200u16,
            )));
            assert_eq!(2, obj1.len());

            let mut obj2: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            obj2.add(SOMTypeWrapper::Bool(SOMPrimitive::empty(SOMEndian::Big)));
            obj2.add(SOMTypeWrapper::U16(SOMPrimitive::empty(SOMEndian::Big)));

            serialize_parse(&obj1, &mut obj2, &[0x01, 0xC0, 0x30]);
            assert_eq!(2, obj2.len());

            if let Some(SOMTypeWrapper::Bool(child)) = obj2.get(0) {
                assert_eq!(true, child.get().unwrap());
            } else {
                panic!();
            }

            if let Some(SOMTypeWrapper::U16(child)) = obj2.get(1) {
                assert_eq!(49200, child.get().unwrap());
            } else {
                panic!();
            }
        }

        // complex struct
        {
            let mut sub1: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            sub1.add(SOMTypeWrapper::Bool(SOMPrimitive::new(
                SOMEndian::Big,
                true,
            )));
            sub1.add(SOMTypeWrapper::U16(SOMPrimitive::new(
                SOMEndian::Big,
                49200u16,
            )));

            let mut sub2: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            sub2.add(SOMTypeWrapper::U16(SOMPrimitive::new(
                SOMEndian::Little,
                49200u16,
            )));
            sub2.add(SOMTypeWrapper::Bool(SOMPrimitive::new(
                SOMEndian::Little,
                true,
            )));

            let mut obj1: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            obj1.add(SOMTypeWrapper::Struct(sub1));
            obj1.add(SOMTypeWrapper::Struct(sub2));
            assert_eq!(2, obj1.len());

            let mut sub1: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            sub1.add(SOMTypeWrapper::Bool(SOMPrimitive::empty(SOMEndian::Big)));
            sub1.add(SOMTypeWrapper::U16(SOMPrimitive::empty(SOMEndian::Big)));

            let mut sub2: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            sub2.add(SOMTypeWrapper::U16(SOMPrimitive::empty(SOMEndian::Little)));
            sub2.add(SOMTypeWrapper::Bool(SOMPrimitive::empty(SOMEndian::Little)));

            let mut obj2: SOMStruct<SOMTypeWrapper> = SOMStruct::new();
            obj2.add(SOMTypeWrapper::Struct(sub1));
            obj2.add(SOMTypeWrapper::Struct(sub2));

            serialize_parse(&obj1, &mut obj2, &[0x01, 0xC0, 0x30, 0x30, 0xC0, 0x01]);
            assert_eq!(2, obj2.len());

            if let Some(SOMTypeWrapper::Struct(sub)) = obj2.get(0) {
                if let Some(SOMTypeWrapper::Bool(child)) = sub.get(0) {
                    assert_eq!(true, child.get().unwrap());
                } else {
                    panic!();
                }

                if let Some(SOMTypeWrapper::U16(child)) = sub.get(1) {
                    assert_eq!(49200, child.get().unwrap());
                } else {
                    panic!();
                }
            } else {
                panic!();
            }

            if let Some(SOMTypeWrapper::Struct(sub)) = obj2.get(1) {
                if let Some(SOMTypeWrapper::U16(child)) = sub.get(0) {
                    assert_eq!(49200, child.get().unwrap());
                } else {
                    panic!();
                }

                if let Some(SOMTypeWrapper::Bool(child)) = sub.get(1) {
                    assert_eq!(true, child.get().unwrap());
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
        }
    }
}
