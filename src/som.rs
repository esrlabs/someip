// wip: someip types

use byteorder::{BigEndian, ByteOrder, LittleEndian};

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
        self.check_size(size).unwrap();

        self.buffer[self.offset] = match value {
            true => 1,
            false => 0,
        };

        self.offset += size;
        Ok(())
    }

    fn write_u8(&mut self, value: u8) -> Result<(), SOMError> {
        let size = std::mem::size_of::<u8>();
        self.check_size(size).unwrap();

        self.buffer[self.offset] = value;

        self.offset += size;
        Ok(())
    }

    fn write_u16(&mut self, value: u16, endian: SOMEndian) -> Result<(), SOMError> {
        let size = std::mem::size_of::<u16>();
        self.check_size(size).unwrap();

        match endian {
            SOMEndian::Big => BigEndian::write_u16(&mut self.buffer[self.offset..], value),
            SOMEndian::Little => LittleEndian::write_u16(&mut self.buffer[self.offset..], value),
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
        self.check_size(size).unwrap();

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
        self.check_size(size).unwrap();

        let result = self.buffer[self.offset];

        self.offset += size;
        Ok(result)
    }

    fn read_u16(&mut self, endian: SOMEndian) -> Result<u16, SOMError> {
        let size = std::mem::size_of::<u16>();
        self.check_size(size).unwrap();

        let result = match endian {
            SOMEndian::Big => BigEndian::read_u16(&self.buffer[self.offset..]),
            SOMEndian::Little => LittleEndian::read_u16(&self.buffer[self.offset..]),
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

    fn size_of(_: &SOMPrimitive<T>) -> usize {
        std::mem::size_of::<T>()
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
            Some(value) => serializer.write_bool(value).unwrap(),
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

        self.value = Some(parser.read_bool().unwrap());

        Ok(parser.offset() - offset)
    }

    fn size(&self) -> usize {
        SOMPrimitive::size_of(self)
    }
}

impl SOMType for SOMPrimitive<u8> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_u8(value).unwrap(),
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

        self.value = Some(parser.read_u8().unwrap());

        Ok(parser.offset() - offset)
    }

    fn size(&self) -> usize {
        SOMPrimitive::size_of(self)
    }
}

impl SOMType for SOMPrimitive<u16> {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        let offset = serializer.offset();

        match self.value {
            Some(value) => serializer.write_u16(value, self.endian()).unwrap(),
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

        self.value = Some(parser.read_u16(self.endian()).unwrap());

        Ok(parser.offset() - offset)
    }

    fn size(&self) -> usize {
        SOMPrimitive::size_of(self)
    }
}

pub enum SOMStructMember {
    Bool(SOMPrimitive<bool>),
    U8(SOMPrimitive<u8>),
    U16(SOMPrimitive<u16>),
    Struct(SOMStruct<SOMStructMember>),
}

impl SOMType for SOMStructMember {
    fn serialize(&self, serializer: &mut SOMSerializer) -> Result<usize, SOMError> {
        match self {
            SOMStructMember::Bool(obj) => obj.serialize(serializer),
            SOMStructMember::U8(obj) => obj.serialize(serializer),
            SOMStructMember::U16(obj) => obj.serialize(serializer),
            SOMStructMember::Struct(obj) => obj.serialize(serializer),
        }
    }

    fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
        match self {
            SOMStructMember::Bool(obj) => obj.parse(parser),
            SOMStructMember::U8(obj) => obj.parse(parser),
            SOMStructMember::U16(obj) => obj.parse(parser),
            SOMStructMember::Struct(obj) => obj.parse(parser),
        }
    }

    fn size(&self) -> usize {
        match self {
            SOMStructMember::Bool(obj) => obj.size(),
            SOMStructMember::U8(obj) => obj.size(),
            SOMStructMember::U16(obj) => obj.size(),
            SOMStructMember::Struct(obj) => obj.size(),
        }
    }
}

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
            member.serialize(serializer).unwrap();
        }

        Ok(serializer.offset() - offset)
    }

    fn parse(&mut self, parser: &mut SOMParser) -> Result<usize, SOMError> {
        let offset = parser.offset();

        for member in &mut self.members {
            member.parse(parser).unwrap();
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

    #[test]
    fn test_som_primitive_bool() {
        let obj: SOMPrimitive<bool> = SOMPrimitive::new(SOMEndian::Big, true);
        assert_eq!(SOMEndian::Big, obj.endian());
        assert_eq!(true, obj.get().unwrap());

        let mut obj: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Big);
        assert_eq!(SOMEndian::Big, obj.endian());
        assert_eq!(None, obj.get());
        obj.set(true);
        assert_eq!(true, obj.get().unwrap());
    }

    #[test]
    fn test_som_primitive_bool_serialization() {
        let obj: SOMPrimitive<bool> = SOMPrimitive::new(SOMEndian::Big, true);
        assert_eq!(1, obj.size());

        let mut buffer: [u8; 1] = [0; 1];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(1, obj.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, [0x01]);

        let mut obj: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Big);
        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(1, obj.parse(&mut parser).unwrap());
        assert_eq!(true, obj.get().unwrap());

        let obj: SOMPrimitive<bool> = SOMPrimitive::new(SOMEndian::Big, false);
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(1, obj.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, [0x00]);

        let mut obj: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Big);
        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(1, obj.parse(&mut parser).unwrap());
        assert_eq!(false, obj.get().unwrap());

        let obj: SOMPrimitive<bool> = SOMPrimitive::new(SOMEndian::Little, true);
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(1, obj.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, [0x01]);

        let mut obj: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Little);
        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(1, obj.parse(&mut parser).unwrap());
        assert_eq!(true, obj.get().unwrap());
    }

    #[test]
    #[should_panic(expected = "Uninitialized Type at offset: 0")]
    fn test_som_primitive_bool_invalid_type() {
        let obj: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Big);
        let mut buffer: [u8; 1] = [0; 1];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        obj.serialize(&mut serializer).unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid Bool value: 2 at offset: 0")]
    fn test_som_primitive_bool_invalid_payload() {
        let mut obj: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Big);
        let buffer: [u8; 1] = [2; 1];
        let mut parser = SOMParser::new(&buffer[..]);
        obj.parse(&mut parser).unwrap();
    }

    #[test]
    #[should_panic(expected = "Serializer exausted at offset: 0 for Object size: 1")]
    fn test_som_primitive_bool_invalid_output_buffer() {
        let obj: SOMPrimitive<bool> = SOMPrimitive::new(SOMEndian::Big, true);
        let mut buffer: [u8; 0] = [0; 0];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        obj.serialize(&mut serializer).unwrap();
    }

    #[test]
    #[should_panic(expected = "Parser exausted at offset: 0 for Object size: 1")]
    fn test_som_primitive_bool_invalid_input_buffer() {
        let mut obj: SOMPrimitive<bool> = SOMPrimitive::empty(SOMEndian::Big);
        let buffer: [u8; 0] = [0; 0];
        let mut parser = SOMParser::new(&buffer[..]);
        obj.parse(&mut parser).unwrap();
    }

    #[test]
    fn test_som_primitive_u8() {
        let obj: SOMPrimitive<u8> = SOMPrimitive::new(SOMEndian::Big, 1);
        assert_eq!(SOMEndian::Big, obj.endian());
        assert_eq!(1, obj.get().unwrap());

        let mut obj: SOMPrimitive<u8> = SOMPrimitive::empty(SOMEndian::Big);
        assert_eq!(SOMEndian::Big, obj.endian());
        assert_eq!(None, obj.get());
        obj.set(1);
        assert_eq!(1, obj.get().unwrap());
    }

    #[test]
    fn test_som_primitive_u8_serialization() {
        let obj: SOMPrimitive<u8> = SOMPrimitive::new(SOMEndian::Big, 195);
        assert_eq!(1, obj.size());

        let mut buffer: [u8; 1] = [0; 1];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(1, obj.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, [0xC3]);

        let mut obj: SOMPrimitive<u8> = SOMPrimitive::empty(SOMEndian::Big);
        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(1, obj.parse(&mut parser).unwrap());
        assert_eq!(195, obj.get().unwrap());

        let obj: SOMPrimitive<u8> = SOMPrimitive::new(SOMEndian::Little, 195);
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(1, obj.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, [0xC3]);

        let mut obj: SOMPrimitive<u8> = SOMPrimitive::empty(SOMEndian::Little);
        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(1, obj.parse(&mut parser).unwrap());
        assert_eq!(195, obj.get().unwrap());
    }

    #[test]
    #[should_panic(expected = "Uninitialized Type at offset: 0")]
    fn test_som_primitive_u8_invalid_type() {
        let obj: SOMPrimitive<u8> = SOMPrimitive::empty(SOMEndian::Big);
        let mut buffer: [u8; 1] = [0; 1];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        obj.serialize(&mut serializer).unwrap();
    }

    #[test]
    #[should_panic(expected = "Serializer exausted at offset: 0 for Object size: 1")]
    fn test_som_primitive_u8_invalid_output_buffer() {
        let obj: SOMPrimitive<u8> = SOMPrimitive::new(SOMEndian::Big, 0);
        let mut buffer: [u8; 0] = [0; 0];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        obj.serialize(&mut serializer).unwrap();
    }

    #[test]
    #[should_panic(expected = "Parser exausted at offset: 0 for Object size: 1")]
    fn test_som_primitive_u8_invalid_input_buffer() {
        let mut obj: SOMPrimitive<u8> = SOMPrimitive::empty(SOMEndian::Big);
        let buffer: [u8; 0] = [0; 0];
        let mut parser = SOMParser::new(&buffer[..]);
        obj.parse(&mut parser).unwrap();
    }

    #[test]
    fn test_som_primitive_u16() {
        let obj: SOMPrimitive<u16> = SOMPrimitive::new(SOMEndian::Big, 1);
        assert_eq!(SOMEndian::Big, obj.endian());
        assert_eq!(1, obj.get().unwrap());

        let mut obj: SOMPrimitive<u16> = SOMPrimitive::empty(SOMEndian::Big);
        assert_eq!(SOMEndian::Big, obj.endian());
        assert_eq!(None, obj.get());
        obj.set(1);
        assert_eq!(1, obj.get().unwrap());
    }

    #[test]
    fn test_som_primitive_u16_serialization() {
        let obj: SOMPrimitive<u16> = SOMPrimitive::new(SOMEndian::Big, 49200);
        assert_eq!(2, obj.size());

        let mut buffer: [u8; 2] = [0; 2];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(2, obj.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, [0xC0, 0x30]);

        let mut obj: SOMPrimitive<u16> = SOMPrimitive::empty(SOMEndian::Big);
        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(2, obj.parse(&mut parser).unwrap());
        assert_eq!(49200, obj.get().unwrap());

        let obj: SOMPrimitive<u16> = SOMPrimitive::new(SOMEndian::Little, 49200);
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(2, obj.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, [0x30, 0xC0]);

        let mut obj: SOMPrimitive<u16> = SOMPrimitive::empty(SOMEndian::Little);
        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(2, obj.parse(&mut parser).unwrap());
        assert_eq!(49200, obj.get().unwrap());
    }

    #[test]
    #[should_panic(expected = "Uninitialized Type at offset: 0")]
    fn test_som_primitive_u16_invalid_type() {
        let obj: SOMPrimitive<u16> = SOMPrimitive::empty(SOMEndian::Big);
        let mut buffer: [u8; 1] = [0; 1];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        obj.serialize(&mut serializer).unwrap();
    }

    #[test]
    #[should_panic(expected = "Serializer exausted at offset: 0 for Object size: 2")]
    fn test_som_primitive_u16_invalid_output_buffer() {
        let obj: SOMPrimitive<u16> = SOMPrimitive::new(SOMEndian::Big, 0);
        let mut buffer: [u8; 1] = [0; 1];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        obj.serialize(&mut serializer).unwrap();
    }

    #[test]
    #[should_panic(expected = "Parser exausted at offset: 0 for Object size: 2")]
    fn test_som_primitive_u16_invalid_input_buffer() {
        let mut obj: SOMPrimitive<u16> = SOMPrimitive::empty(SOMEndian::Big);
        let buffer: [u8; 1] = [0; 1];
        let mut parser = SOMParser::new(&buffer[..]);
        obj.parse(&mut parser).unwrap();
    }

    #[test]
    fn test_som_struct() {
        let obj: SOMStruct<SOMStructMember> = SOMStruct::new();
        assert_eq!(0, obj.len());
        assert_eq!(0, obj.size());
    }

    #[test]
    fn test_som_struct_of_primitives_serialization() {
        let mut obj: SOMStruct<SOMStructMember> = SOMStruct::new();
        obj.add(SOMStructMember::Bool(SOMPrimitive::new(
            SOMEndian::Big,
            true,
        )));
        obj.add(SOMStructMember::U16(SOMPrimitive::new(
            SOMEndian::Big,
            49200u16,
        )));
        assert_eq!(2, obj.len());
        assert_eq!(3, obj.size());

        let mut buffer: [u8; 3] = [0; 3];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(3, obj.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, [0x01, 0xC0, 0x30]);

        let mut obj: SOMStruct<SOMStructMember> = SOMStruct::new();
        obj.add(SOMStructMember::Bool(SOMPrimitive::empty(SOMEndian::Big)));
        obj.add(SOMStructMember::U16(SOMPrimitive::empty(SOMEndian::Big)));

        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(3, obj.parse(&mut parser).unwrap());
        assert_eq!(2, obj.len());

        if let Some(SOMStructMember::Bool(child)) = obj.get(0) {
            assert_eq!(true, child.get().unwrap());
        } else {
            panic!();
        }

        if let Some(SOMStructMember::U16(child)) = obj.get(1) {
            assert_eq!(49200, child.get().unwrap());
        } else {
            panic!();
        }
    }

    #[test]
    fn test_som_struct_of_structs_serialization() {
        let mut child1: SOMStruct<SOMStructMember> = SOMStruct::new();
        child1.add(SOMStructMember::Bool(SOMPrimitive::new(
            SOMEndian::Big,
            true,
        )));
        child1.add(SOMStructMember::U16(SOMPrimitive::new(
            SOMEndian::Big,
            49200u16,
        )));

        let mut child2: SOMStruct<SOMStructMember> = SOMStruct::new();
        child2.add(SOMStructMember::U16(SOMPrimitive::new(
            SOMEndian::Little,
            49200u16,
        )));
        child2.add(SOMStructMember::Bool(SOMPrimitive::new(
            SOMEndian::Little,
            true,
        )));

        let mut obj: SOMStruct<SOMStructMember> = SOMStruct::new();
        obj.add(SOMStructMember::Struct(child1));
        obj.add(SOMStructMember::Struct(child2));
        assert_eq!(2, obj.len());
        assert_eq!(6, obj.size());

        let mut buffer: [u8; 6] = [0; 6];
        let mut serializer = SOMSerializer::new(&mut buffer[..]);
        assert_eq!(6, obj.serialize(&mut serializer).unwrap());
        assert_eq!(buffer, [0x01, 0xC0, 0x30, 0x30, 0xC0, 0x01,]);

        let mut child1: SOMStruct<SOMStructMember> = SOMStruct::new();
        child1.add(SOMStructMember::Bool(SOMPrimitive::empty(SOMEndian::Big)));
        child1.add(SOMStructMember::U16(SOMPrimitive::empty(SOMEndian::Big)));

        let mut child2: SOMStruct<SOMStructMember> = SOMStruct::new();
        child2.add(SOMStructMember::U16(SOMPrimitive::empty(SOMEndian::Little)));
        child2.add(SOMStructMember::Bool(SOMPrimitive::empty(
            SOMEndian::Little,
        )));

        let mut obj: SOMStruct<SOMStructMember> = SOMStruct::new();
        obj.add(SOMStructMember::Struct(child1));
        obj.add(SOMStructMember::Struct(child2));

        let mut parser = SOMParser::new(&buffer[..]);
        assert_eq!(6, obj.parse(&mut parser).unwrap());
        assert_eq!(2, obj.len());

        if let Some(SOMStructMember::Struct(sub)) = obj.get(0) {
            if let Some(SOMStructMember::Bool(child)) = sub.get(0) {
                assert_eq!(true, child.get().unwrap());
            } else {
                panic!();
            }

            if let Some(SOMStructMember::U16(child)) = sub.get(1) {
                assert_eq!(49200, child.get().unwrap());
            } else {
                panic!();
            }
        } else {
            panic!();
        }

        if let Some(SOMStructMember::Struct(sub)) = obj.get(1) {
            if let Some(SOMStructMember::U16(child)) = sub.get(0) {
                assert_eq!(49200, child.get().unwrap());
            } else {
                panic!();
            }

            if let Some(SOMStructMember::Bool(child)) = sub.get(1) {
                assert_eq!(true, child.get().unwrap());
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}
