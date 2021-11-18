// someip builder

use crate::fibex::*;
use crate::som::*;

pub struct FibexTypes;

impl FibexTypes {
    pub fn build(type_decl: &FibexTypeDeclaration) -> Result<Box<dyn SOMType>, FibexError> {
        if let Some(result) = types::get_type(type_decl) {
            return Ok(result);
        }

        Err(FibexError::Model(format!(
            "Unsupported type {:?}",
            &type_decl
        )))
    }
}

mod types {
    use super::*;

    pub fn get_type(type_decl: &FibexTypeDeclaration) -> Option<Box<dyn SOMType>> {
        if type_decl.is_array() {
            return get_array_type(type_decl);
        } else if let Some(type_ref) = &type_decl.type_ref {
            let type_def = type_ref.borrow();

            let name = type_decl.name.clone();
            let description = type_def.name.clone();

            match &type_def.datatype {
                FibexDatatype::Primitive(datatype) => {
                    let endian = get_type_endian(type_decl.is_high_low_byte_order());
                    return get_primitive_type(name, description, datatype, endian);
                }
                FibexDatatype::Struct(datatype) => {
                    return get_struct_type(name, description, datatype);
                }
                FibexDatatype::Enum(datatype) => {
                    let endian = get_type_endian(type_decl.is_high_low_byte_order());
                    return get_enum_type(name, description, datatype, endian);
                }
                FibexDatatype::String(datatype) => {
                    return get_string_type(
                        name,
                        description,
                        datatype,
                        get_type_endian(type_decl.is_high_low_byte_order()),
                        type_decl.get_length_field_size(),
                        type_decl.get_bit_length(),
                        type_decl.get_min_bit_length(),
                        type_decl.get_max_bit_length(),
                    );
                }
                _ => {
                    warn!("nyi: {:?}", &type_def.datatype);
                }
            }
        }

        None
    }

    pub fn get_array_type(type_decl: &FibexTypeDeclaration) -> Option<Box<dyn SOMType>> {
        let multidim = type_decl.is_multidim_array();

        if let Some(dimension) = type_decl.get_array_dimension(0) {
            let mut type_decl = type_decl.downdim_array();

            if let Some(type_ref) = &type_decl.type_ref {
                let type_def = type_ref.borrow();

                let name = type_decl.name.clone();
                let description = type_def.name.clone();

                type_decl.name = String::from("");

                if let Some(element_type) = get_type(&type_decl) {
                    let lengthfield = match dimension.is_dynamic() {
                        true => get_length_field(type_decl.get_array_length_field_size()),
                        false => SOMLengthField::None,
                    };

                    match &type_def.datatype {
                        FibexDatatype::Primitive(datatype) => {
                            if multidim {
                                return get_multidim_primitive_array_type(
                                    name,
                                    description,
                                    element_type,
                                    datatype,
                                    dimension,
                                    lengthfield,
                                );
                            } else {
                                return get_primitive_array_type(
                                    name,
                                    description,
                                    element_type,
                                    datatype,
                                    dimension,
                                    lengthfield,
                                );
                            }
                        }
                        FibexDatatype::Struct(_) => {
                            if let Some(item) = (*element_type).as_any().downcast_ref::<SOMStruct>()
                            {
                                return Some(Box::new(
                                    SOMArray::dynamic(
                                        lengthfield,
                                        SOMArrayMember::Struct(item.clone()),
                                        dimension.min,
                                        dimension.max,
                                    )
                                    .with_meta(SOMTypeMeta::from(name, description)),
                                ));
                            };
                        }
                        FibexDatatype::Enum(_) => {
                            if let Some(item) = (*element_type).as_any().downcast_ref::<SOMu8Enum>()
                            {
                                return Some(Box::new(
                                    SOMArray::dynamic(
                                        lengthfield,
                                        SOMArrayMember::EnumU8(item.clone()),
                                        dimension.min,
                                        dimension.max,
                                    )
                                    .with_meta(SOMTypeMeta::from(name, description)),
                                ));
                            }
                            if let Some(item) =
                                (*element_type).as_any().downcast_ref::<SOMu16Enum>()
                            {
                                return Some(Box::new(
                                    SOMArray::dynamic(
                                        lengthfield,
                                        SOMArrayMember::EnumU16(item.clone()),
                                        dimension.min,
                                        dimension.max,
                                    )
                                    .with_meta(SOMTypeMeta::from(name, description)),
                                ));
                            };
                            if let Some(item) =
                                (*element_type).as_any().downcast_ref::<SOMu32Enum>()
                            {
                                return Some(Box::new(
                                    SOMArray::dynamic(
                                        lengthfield,
                                        SOMArrayMember::EnumU32(item.clone()),
                                        dimension.min,
                                        dimension.max,
                                    )
                                    .with_meta(SOMTypeMeta::from(name, description)),
                                ));
                            };
                            if let Some(item) =
                                (*element_type).as_any().downcast_ref::<SOMu64Enum>()
                            {
                                return Some(Box::new(
                                    SOMArray::dynamic(
                                        lengthfield,
                                        SOMArrayMember::EnumU64(item.clone()),
                                        dimension.min,
                                        dimension.max,
                                    )
                                    .with_meta(SOMTypeMeta::from(name, description)),
                                ));
                            };
                        }
                        FibexDatatype::String(_) => {
                            if let Some(item) = (*element_type).as_any().downcast_ref::<SOMString>()
                            {
                                return Some(Box::new(
                                    SOMArray::dynamic(
                                        lengthfield,
                                        SOMArrayMember::String(item.clone()),
                                        dimension.min,
                                        dimension.max,
                                    )
                                    .with_meta(SOMTypeMeta::from(name, description)),
                                ));
                            };
                        }
                        _ => {
                            warn!("nyi: {:?}", &type_def.datatype);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_multidim_primitive_array_type(
        name: String,
        description: String,
        element: Box<dyn SOMType>,
        datatype: &FibexPrimitive,
        dimension: &FibexArrayDimension,
        lengthfield: SOMLengthField,
    ) -> Option<Box<dyn SOMType>> {
        match datatype {
            FibexPrimitive::Bool => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMBoolArray>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayBool(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint8 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu8Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayU8(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int8 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi8Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayI8(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint16 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu16Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayU16(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int16 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi16Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayI16(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint24 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu24Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayU24(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int24 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi24Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayI24(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint32 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu32Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayU32(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int32 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi32Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayI32(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint64 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu64Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayU64(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int64 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi64Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayI64(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Float32 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMf32Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayF32(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Float64 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMf64Array>() {
                    return Some(Box::new(
                        SOMArray::dynamic(
                            lengthfield,
                            SOMArrayMember::ArrayF64(item.clone()),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Unknown => {
                return None;
            }
        };

        None
    }

    pub fn get_primitive_array_type(
        name: String,
        description: String,
        element: Box<dyn SOMType>,
        datatype: &FibexPrimitive,
        dimension: &FibexArrayDimension,
        lengthfield: SOMLengthField,
    ) -> Option<Box<dyn SOMType>> {
        match datatype {
            FibexPrimitive::Bool => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMBool>() {
                    return Some(Box::new(
                        SOMBoolArray::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint8 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu8>() {
                    return Some(Box::new(
                        SOMu8Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int8 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi8>() {
                    return Some(Box::new(
                        SOMi8Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint16 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu16>() {
                    return Some(Box::new(
                        SOMu16Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int16 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi16>() {
                    return Some(Box::new(
                        SOMi16Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint24 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu24>() {
                    return Some(Box::new(
                        SOMu24Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int24 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi24>() {
                    return Some(Box::new(
                        SOMi24Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint32 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu32>() {
                    return Some(Box::new(
                        SOMu32Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int32 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi32>() {
                    return Some(Box::new(
                        SOMi32Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Uint64 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMu64>() {
                    return Some(Box::new(
                        SOMu64Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Int64 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMi64>() {
                    return Some(Box::new(
                        SOMi64Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Float32 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMf32>() {
                    return Some(Box::new(
                        SOMf32Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Float64 => {
                if let Some(item) = (*element).as_any().downcast_ref::<SOMf64>() {
                    return Some(Box::new(
                        SOMf64Array::dynamic(
                            lengthfield,
                            item.clone(),
                            dimension.min,
                            dimension.max,
                        )
                        .with_meta(SOMTypeMeta::from(name, description)),
                    ));
                };
            }
            FibexPrimitive::Unknown => {
                return None;
            }
        };

        None
    }

    fn get_primitive_type(
        name: String,
        description: String,
        datatype: &FibexPrimitive,
        endian: SOMEndian,
    ) -> Option<Box<dyn SOMType>> {
        match datatype {
            FibexPrimitive::Bool => Some(Box::new(
                SOMBool::empty().with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Uint8 => Some(Box::new(
                SOMu8::empty().with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Int8 => Some(Box::new(
                SOMi8::empty().with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Uint16 => Some(Box::new(
                SOMu16::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Int16 => Some(Box::new(
                SOMi16::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Uint24 => Some(Box::new(
                SOMu24::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Int24 => Some(Box::new(
                SOMi24::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Uint32 => Some(Box::new(
                SOMu32::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Int32 => Some(Box::new(
                SOMi32::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Uint64 => Some(Box::new(
                SOMu64::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Int64 => Some(Box::new(
                SOMi64::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Float32 => Some(Box::new(
                SOMf32::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Float64 => Some(Box::new(
                SOMf64::empty(endian).with_meta(SOMTypeMeta::from(name, description)),
            )),
            FibexPrimitive::Unknown => None,
        }
    }

    fn get_struct_type(
        name: String,
        description: String,
        datatype: &FibexStruct,
    ) -> Option<Box<dyn SOMType>> {
        let mut elements = Vec::new();

        for member in &datatype.members {
            if let Some(type_ref) = &member.type_ref {
                let type_def = type_ref.borrow();

                if let Some(member_type) = get_type(member) {
                    if member.is_array() {
                        if let Some(array_type) = get_array_type(member) {
                            if member.is_multidim_array() {
                                if let Some(element) =
                                    (*array_type).as_any().downcast_ref::<SOMArray>()
                                {
                                    elements.push(SOMStructMember::Array(element.clone()));
                                };
                            } else {
                                match &type_def.datatype {
                                    FibexDatatype::Primitive(datatype) => {
                                        if let Some(element) =
                                            get_primitive_array_struct_member(array_type, datatype)
                                        {
                                            elements.push(element);
                                        }
                                    }
                                    FibexDatatype::Struct(_) => {
                                        if let Some(element) =
                                            (*member_type).as_any().downcast_ref::<SOMArray>()
                                        {
                                            elements.push(SOMStructMember::Array(element.clone()));
                                        };
                                    }
                                    FibexDatatype::Enum(_) => {
                                        if let Some(element) =
                                            (*member_type).as_any().downcast_ref::<SOMArray>()
                                        {
                                            elements.push(SOMStructMember::Array(element.clone()));
                                        };
                                    }
                                    FibexDatatype::String(_) => {
                                        if let Some(element) =
                                            (*member_type).as_any().downcast_ref::<SOMArray>()
                                        {
                                            elements.push(SOMStructMember::Array(element.clone()));
                                        };
                                    }
                                    _ => {
                                        warn!("nyi: {:?}", &type_def.datatype);
                                    }
                                }
                            }
                        }
                    } else {
                        match &type_def.datatype {
                            FibexDatatype::Primitive(datatype) => {
                                if let Some(element) =
                                    get_primitive_struct_member(member_type, datatype)
                                {
                                    elements.push(element);
                                }
                            }
                            FibexDatatype::Struct(_) => {
                                if let Some(element) =
                                    (*member_type).as_any().downcast_ref::<SOMStruct>()
                                {
                                    elements.push(SOMStructMember::Struct(element.clone()));
                                };
                            }
                            FibexDatatype::Enum(_) => {
                                if let Some(element) =
                                    (*member_type).as_any().downcast_ref::<SOMu8Enum>()
                                {
                                    elements.push(SOMStructMember::EnumU8(element.clone()));
                                };
                                if let Some(element) =
                                    (*member_type).as_any().downcast_ref::<SOMu16Enum>()
                                {
                                    elements.push(SOMStructMember::EnumU16(element.clone()));
                                };
                                if let Some(element) =
                                    (*member_type).as_any().downcast_ref::<SOMu32Enum>()
                                {
                                    elements.push(SOMStructMember::EnumU32(element.clone()));
                                };
                                if let Some(element) =
                                    (*member_type).as_any().downcast_ref::<SOMu64Enum>()
                                {
                                    elements.push(SOMStructMember::EnumU64(element.clone()));
                                };
                            }
                            FibexDatatype::String(_) => {
                                if let Some(element) =
                                    (*member_type).as_any().downcast_ref::<SOMString>()
                                {
                                    elements.push(SOMStructMember::String(element.clone()));
                                };
                            }
                            _ => {
                                warn!("nyi: {:?}", &type_def.datatype);
                            }
                        }
                    }
                }
            }
        }

        Some(Box::new(
            SOMStruct::from(elements).with_meta(SOMTypeMeta::from(name, description)),
        ))
    }

    fn get_primitive_array_struct_member(
        array: Box<dyn SOMType>,
        datatype: &FibexPrimitive,
    ) -> Option<SOMStructMember> {
        match datatype {
            FibexPrimitive::Bool => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMBoolArray>() {
                    return Some(SOMStructMember::ArrayBool(item.clone()));
                };
            }
            FibexPrimitive::Uint8 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMu8Array>() {
                    return Some(SOMStructMember::ArrayU8(item.clone()));
                };
            }
            FibexPrimitive::Int8 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMi8Array>() {
                    return Some(SOMStructMember::ArrayI8(item.clone()));
                };
            }
            FibexPrimitive::Uint16 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMu16Array>() {
                    return Some(SOMStructMember::ArrayU16(item.clone()));
                };
            }
            FibexPrimitive::Int16 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMi16Array>() {
                    return Some(SOMStructMember::ArrayI16(item.clone()));
                };
            }
            FibexPrimitive::Uint24 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMu24Array>() {
                    return Some(SOMStructMember::ArrayU24(item.clone()));
                };
            }
            FibexPrimitive::Int24 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMi24Array>() {
                    return Some(SOMStructMember::ArrayI24(item.clone()));
                };
            }
            FibexPrimitive::Uint32 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMu32Array>() {
                    return Some(SOMStructMember::ArrayU32(item.clone()));
                };
            }
            FibexPrimitive::Int32 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMi32Array>() {
                    return Some(SOMStructMember::ArrayI32(item.clone()));
                };
            }
            FibexPrimitive::Uint64 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMu64Array>() {
                    return Some(SOMStructMember::ArrayU64(item.clone()));
                };
            }
            FibexPrimitive::Int64 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMi64Array>() {
                    return Some(SOMStructMember::ArrayI64(item.clone()));
                };
            }
            FibexPrimitive::Float32 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMf32Array>() {
                    return Some(SOMStructMember::ArrayF32(item.clone()));
                };
            }
            FibexPrimitive::Float64 => {
                if let Some(item) = (*array).as_any().downcast_ref::<SOMf64Array>() {
                    return Some(SOMStructMember::ArrayF64(item.clone()));
                };
            }
            FibexPrimitive::Unknown => {
                return None;
            }
        }

        None
    }

    fn get_primitive_struct_member(
        member: Box<dyn SOMType>,
        datatype: &FibexPrimitive,
    ) -> Option<SOMStructMember> {
        match datatype {
            FibexPrimitive::Bool => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMBool>() {
                    return Some(SOMStructMember::Bool(item.clone()));
                };
            }
            FibexPrimitive::Uint8 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMu8>() {
                    return Some(SOMStructMember::U8(item.clone()));
                };
            }
            FibexPrimitive::Int8 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMi8>() {
                    return Some(SOMStructMember::I8(item.clone()));
                };
            }
            FibexPrimitive::Uint16 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMu16>() {
                    return Some(SOMStructMember::U16(item.clone()));
                };
            }
            FibexPrimitive::Int16 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMi16>() {
                    return Some(SOMStructMember::I16(item.clone()));
                };
            }
            FibexPrimitive::Uint24 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMu24>() {
                    return Some(SOMStructMember::U24(item.clone()));
                };
            }
            FibexPrimitive::Int24 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMi24>() {
                    return Some(SOMStructMember::I24(item.clone()));
                };
            }
            FibexPrimitive::Uint32 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMu32>() {
                    return Some(SOMStructMember::U32(item.clone()));
                };
            }
            FibexPrimitive::Int32 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMi32>() {
                    return Some(SOMStructMember::I32(item.clone()));
                };
            }
            FibexPrimitive::Uint64 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMu64>() {
                    return Some(SOMStructMember::U64(item.clone()));
                };
            }
            FibexPrimitive::Int64 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMi64>() {
                    return Some(SOMStructMember::I64(item.clone()));
                };
            }
            FibexPrimitive::Float32 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMf32>() {
                    return Some(SOMStructMember::F32(item.clone()));
                };
            }
            FibexPrimitive::Float64 => {
                if let Some(item) = (*member).as_any().downcast_ref::<SOMf64>() {
                    return Some(SOMStructMember::F64(item.clone()));
                };
            }
            FibexPrimitive::Unknown => {
                return None;
            }
        }

        None
    }

    fn get_enum_type(
        name: String,
        description: String,
        datatype: &FibexEnum,
        endian: SOMEndian,
    ) -> Option<Box<dyn SOMType>> {
        match &datatype.primitive {
            FibexPrimitive::Uint8 => {
                let mut items = Vec::new();
                for variant in &datatype.variants {
                    if let Ok(value) = variant.value.parse::<u8>() {
                        items.push(SOMEnum::from(variant.name.clone(), value));
                    }
                }
                Some(Box::new(
                    SOMu8Enum::from(items).with_meta(SOMTypeMeta::from(name, description)),
                ))
            }
            FibexPrimitive::Uint16 => {
                let mut items = Vec::new();
                for variant in &datatype.variants {
                    if let Ok(value) = variant.value.parse::<u16>() {
                        items.push(SOMEnum::from(variant.name.clone(), value));
                    }
                }
                Some(Box::new(
                    SOMu16Enum::from(endian, items).with_meta(SOMTypeMeta::from(name, description)),
                ))
            }
            FibexPrimitive::Uint32 => {
                let mut items = Vec::new();
                for variant in &datatype.variants {
                    if let Ok(value) = variant.value.parse::<u32>() {
                        items.push(SOMEnum::from(variant.name.clone(), value));
                    }
                }
                Some(Box::new(
                    SOMu32Enum::from(endian, items).with_meta(SOMTypeMeta::from(name, description)),
                ))
            }
            FibexPrimitive::Uint64 => {
                let mut items = Vec::new();
                for variant in &datatype.variants {
                    if let Ok(value) = variant.value.parse::<u64>() {
                        items.push(SOMEnum::from(variant.name.clone(), value));
                    }
                }
                Some(Box::new(
                    SOMu64Enum::from(endian, items).with_meta(SOMTypeMeta::from(name, description)),
                ))
            }
            _ => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn get_string_type(
        name: String,
        description: String,
        datatype: &FibexString,
        endian: SOMEndian,
        lengthfield: usize,
        bit_length: Option<usize>,
        min_bit_length: Option<usize>,
        max_bit_length: Option<usize>,
    ) -> Option<Box<dyn SOMType>> {
        let is_dynamic = datatype.is_dynamic;

        let encoding = match datatype.encoding {
            FibexStringEncoding::UTF8 => SOMStringEncoding::Utf8,
            FibexStringEncoding::UTF16 => match endian {
                SOMEndian::Big => SOMStringEncoding::Utf16Be,
                SOMEndian::Little => SOMStringEncoding::Utf16Le,
            },
            _ => {
                return None;
            }
        };

        let format = match (datatype.has_bom, datatype.has_termination) {
            (false, false) => SOMStringFormat::Plain,
            (true, false) => SOMStringFormat::WithBOM,
            (false, true) => SOMStringFormat::WithTermination,
            (true, true) => SOMStringFormat::WithBOMandTermination,
        };

        let min = match min_bit_length {
            Some(value) => value / 8,
            None => datatype.min_length.unwrap_or(0),
        };

        let max = match bit_length {
            Some(value) => value / 8,
            None => match max_bit_length {
                Some(value) => value / 8,
                None => match datatype.bit_length {
                    Some(value) => value / 8,
                    None => datatype.max_length.unwrap_or(0),
                },
            },
        };

        match is_dynamic {
            true => Some(Box::new(
                SOMString::dynamic(get_length_field(lengthfield), encoding, format, min, max)
                    .with_meta(SOMTypeMeta::from(name, description)),
            )),
            false => Some(Box::new(
                SOMString::fixed(encoding, format, max)
                    .with_meta(SOMTypeMeta::from(name, description)),
            )),
        }
    }

    fn get_type_endian(high_low_byte_order: bool) -> SOMEndian {
        match high_low_byte_order {
            true => SOMEndian::Big,
            false => SOMEndian::Little,
        }
    }

    fn get_length_field(size: usize) -> SOMLengthField {
        match size {
            1 => SOMLengthField::U8,
            2 => SOMLengthField::U16,
            4 => SOMLengthField::U32,
            _ => SOMLengthField::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn flatten_str(string: &str) -> String {
        string.replace(" ", "").replace("\n", "")
    }

    fn assert_str(expected: &str, actual: &str) {
        assert_eq!(flatten_str(expected), flatten_str(actual), "\n{}\n", actual);
    }

    #[test]
    fn test_parse_primitive_event() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(32768)
            .unwrap()
            .get_request()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x01, // U8-Member
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                value (UINT8) : 1,
            }
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_struct_of_primitives_request() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(4)
            .unwrap()
            .get_request()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x01, // Bool-Member
            0xC0, 0x30, // U16-Member
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                input (AStruct) {
                    member1 (BOOL) : true,
                    member2 (UINT16) : 49200,
                },
            }
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_array_of_primitives_response() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(7)
            .unwrap()
            .get_response()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x00, 0x03, // U16 Length-Field
            0x01, // U8-Element
            0x02, // U8-Element
            0x03, // U8-Element
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                output (UINT8) [
                    1,
                    2,
                    3,
                ],
            }
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_array_of_structs_request() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(5)
            .unwrap()
            .get_request()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x01, 0xC0, 0x30, // Bool/U16-Element
            0x00, 0xC0, 0x31, // Bool/U16-Element
            0x01, 0xC0, 0x32, // Bool/U16-Element
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                input (AStruct) [
                    {
                        member1 (BOOL) : true,
                        member2 (UINT16) : 49200,
                    },
                    {
                        member1 (BOOL) : false,
                        member2 (UINT16) : 49201,
                    },
                    {
                        member1 (BOOL) : true,
                        member2 (UINT16) : 49202,
                    },
                ],
            }
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_primitive_field_setter() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(11)
            .unwrap()
            .get_request()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x01, // U8-Member
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                testField (UINT8) : 1,
            }
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_primitive_field_getter() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(10)
            .unwrap()
            .get_response()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x01, // U8-Member
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                testField (UINT8) : 1,
            }
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_enum_request() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(2)
            .unwrap()
            .get_request()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x00, 0x01, // U16 Enum-Member
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                input (AEnum) {
                    'A' : 1
                },
            }
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_array_of_enum_response() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(2)
            .unwrap()
            .get_response()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x01, 0x00, // U16 Enum-Member LE
            0x02, 0x00, // U16 Enum-Member LE
            0x03, 0x00, // U16 Enum-Member LE
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                output (AEnum) [
                    {
                        'A' : 1
                    },
                    {
                        'B' : 2
                    },
                    {
                        'C' : 3
                    },
                ],
            }
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_multidim_array_field_notifier() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(32771)
            .unwrap()
            .get_request()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x06, // U8 Length-Field
            0x01, // U8-Element 0-0
            0x02, // U8-Element 0-1
            0x03, // U8-Element 0-2
            0x11, // U8-Element 1-0
            0x12, // U8-Element 1-1
            0x13, // U8-Element 1-2
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                testFieldArray (UINT8) [
                    [
                        1,
                        2,
                        3,
                    ],
                    [
                        17,
                        18,
                        19,
                    ],
                ],
            }
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_fixed_string_response() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(30)
            .unwrap()
            .get_response()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x54, 0x65, 0x73, 0x74, 0x53, 0x65, 0x72, 0x76, 0x69, 0x63,
            0x65, // Plain UTF8 String
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                serviceName (STRINGUTF8FIXED) : 'TestService',
            }        
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_dynamic_string_request() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(8)
            .unwrap()
            .get_request()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x00, 0x0A, // Lenght-Field (U16)
            0xFE, 0xFF, // BOM
            0x00, 0x66, 0x00, 0x6F, 0x00, 0x6F, // Content
            0x00, 0x00, // Termination
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                input (STRINGUTF16DYNAMIC) : 'foo',
            }        
        "#;

        assert_str(expected, &format!("{}", som_type));
    }

    #[test]
    fn test_parse_array_of_dynamic_string_response() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("fibex error");

        let fibex_type = model
            .get_service(123, 1)
            .unwrap()
            .get_method(8)
            .unwrap()
            .get_response()
            .unwrap();

        let mut som_type = FibexTypes::build(fibex_type).expect("build error");

        let payload = &[
            0x00, 0x18, // Array Lenght-Field (U16)
            0x00, 0x0A, // String Lenght-Field (U16)
            0xFF, 0xFE, // BOM
            0x66, 0x00, 0x6F, 0x00, 0x6F, 0x00, // Content
            0x00, 0x00, // Termination
            0x00, 0x0A, // String Lenght-Field (U16)
            0xFF, 0xFE, // BOM
            0x62, 0x00, 0x61, 0x00, 0x72, 0x00, // Content
            0x00, 0x00, // Termination
        ];

        let mut parser = SOMParser::new(payload);
        som_type.parse(&mut parser).expect("someip error");

        let expected = r#"
            {
                output (STRINGUTF16DYNAMIC) [
                    'foo',
                    'bar',
                ],
            }  
        "#;

        assert_str(expected, &format!("{}", som_type));
    }
}
