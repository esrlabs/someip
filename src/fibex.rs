// fibex parser

use quick_xml::{
    events::{BytesStart, Event as XmlEvent},
    Reader as XmlReader,
};
use regex::Regex;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    rc::Rc,
};
use thiserror::Error;
use voca_rs::case::{capitalize, decapitalize};

const ERROR_TAG: &str = "FIBEX Error";

#[derive(Error, Debug)]
pub enum FibexError {
    #[error("{}: {0}", ERROR_TAG)]
    Parse(String),
    #[error("{}: {0}", ERROR_TAG)]
    Model(String),
    #[error("{}: {0:?}", ERROR_TAG)]
    Xml(#[from] quick_xml::Error),
}

#[derive(Debug)]
pub struct FibexModel {
    pub services: Vec<FibexServiceInterface>,
    pub types: Vec<FibexTypeReference>,
    codings: Vec<FibexTypeCoding>,
}

impl FibexModel {
    pub fn new() -> Self {
        FibexModel {
            services: Vec::new(),
            types: Vec::new(),
            codings: Vec::new(),
        }
    }

    pub fn get_service(
        &self,
        service_id: usize,
        service_version: usize,
    ) -> Option<&FibexServiceInterface> {
        self.services.iter().find(|&service| {
            service.service_id == service_id && service.major_version == service_version
        })
    }

    pub fn pack(&mut self, strict: bool) -> Result<(), FibexError> {
        let mut types: HashMap<String, FibexTypeReference> = HashMap::new();

        for item in &mut self.types {
            let mut borrow = item.borrow_mut();

            if let Some(coding_ref) = borrow.coding_ref.as_ref() {
                let coding_ref = coding_ref.clone();

                if let FibexDatatype::Unknown = &borrow.datatype {
                    for coding in &self.codings {
                        if *coding.id == *coding_ref {
                            if let Some(datatype) = coding.resolve() {
                                borrow.datatype = datatype;
                            }
                        }
                    }
                }
                if let FibexDatatype::Enum(enumeration) = &mut borrow.datatype {
                    for coding in &self.codings {
                        if *coding.id == *coding_ref {
                            if let Some(FibexDatatype::Primitive(primitive)) = coding.resolve() {
                                enumeration.primitive = primitive;
                            }
                        }
                    }
                }
            }

            if let FibexDatatype::Unknown = &borrow.datatype {
                warn!(
                    "{}",
                    format!("Unknown type {} ({})", borrow.id, borrow.name)
                );
            } else {
                types.insert(borrow.id.clone(), item.clone());
            }
        }

        for item in &mut self.types {
            let mut borrow = item.borrow_mut();
            if let FibexDatatype::Struct(datatype) = &mut borrow.datatype {
                for member in &mut datatype.members {
                    Self::resolve_reference(&types, member, strict)?;
                }
            }
        }

        Ok(())
    }

    fn resolve_reference(
        types: &HashMap<String, FibexTypeReference>,
        declaration: &mut FibexTypeDeclaration,
        strict: bool,
    ) -> Result<(), FibexError> {
        if declaration.type_ref.is_none() {
            if let Some(type_ref) = types.get(&declaration.id_ref) {
                declaration.type_ref = Some(type_ref.clone());
            } else {
                Self::unresolved_reference(&declaration.id_ref, &declaration.id, strict)?;
            }
        }

        Ok(())
    }

    fn unresolved_reference(id_ref: &str, id: &str, strict: bool) -> Result<(), FibexError> {
        let message = format!("Unresolved reference {} at {}", id_ref, id);

        match strict {
            true => Err(FibexError::Parse(message)),
            false => {
                warn!("{}", message);
                Ok(())
            }
        }
    }
}

impl Default for FibexModel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct FibexServiceInterface {
    pub id: String,
    pub name: String,
    pub service_id: usize,
    pub major_version: usize,
    pub minor_version: usize,
    pub methods: Vec<FibexServiceMethod>,
}

impl FibexServiceInterface {
    pub fn get_method(&self, method_id: usize) -> Option<&FibexServiceMethod> {
        self.methods
            .iter()
            .find(|&method| method.method_id == method_id)
    }
}

#[derive(Debug)]
pub struct FibexServiceMethod {
    pub id: String,
    pub name: String,
    pub method_id: usize,
    pub request: Option<FibexTypeDeclaration>,
    pub response: Option<FibexTypeDeclaration>,
}

impl FibexServiceMethod {
    pub fn get_request(&self) -> Option<&FibexTypeDeclaration> {
        self.request.as_ref()
    }

    pub fn get_response(&self) -> Option<&FibexTypeDeclaration> {
        self.response.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct FibexTypeDeclaration {
    pub id: String,
    pub name: String,
    pub id_ref: String,
    pub type_ref: Option<FibexTypeReference>,
    pub attributes: Vec<FibexTypeAttribute>,
}

impl FibexTypeDeclaration {
    pub fn is_high_low_byte_order(&self) -> bool {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::HighLowByteOrder(value) = attribute {
                return *value;
            }
        }

        true // default
    }

    pub fn is_array(&self) -> bool {
        self.num_array_dimensions() > 0
    }

    pub fn is_multidim_array(&self) -> bool {
        self.num_array_dimensions() > 1
    }

    pub fn num_array_dimensions(&self) -> usize {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::ArrayDeclaration(dimensions) = attribute {
                return dimensions.len();
            }
        }

        0
    }

    pub fn get_array_dimension(&self, index: usize) -> Option<&FibexArrayDimension> {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::ArrayDeclaration(dimensions) = attribute {
                if let Some(dimension) = dimensions.get(index) {
                    return Some(dimension);
                }
            }
        }

        None
    }

    pub fn downdim_array(&self) -> Self {
        let mut declaration = self.clone();

        for attribute in &mut declaration.attributes {
            if let FibexTypeAttribute::ArrayDeclaration(dimensions) = attribute {
                if !dimensions.is_empty() {
                    dimensions.remove(0);
                }
            }
        }

        declaration
    }

    pub fn get_length_field_size(&self) -> usize {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::LengthField(value) = attribute {
                return *value / 8;
            }
        }

        4 // default
    }

    pub fn get_array_length_field_size(&self) -> usize {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::ArrayLengthField(value) = attribute {
                return *value / 8;
            }
        }

        4 // default
    }

    pub fn get_type_length_field_size(&self) -> usize {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::TypeLengthField(value) = attribute {
                return *value / 8;
            }
        }

        4 // default
    }

    pub fn get_bit_length(&self) -> Option<usize> {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::BitLength(value) = attribute {
                return Some(*value);
            }
        }

        None
    }

    pub fn get_min_bit_length(&self) -> Option<usize> {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::MinBitLength(value) = attribute {
                return Some(*value);
            }
        }

        None
    }

    pub fn get_max_bit_length(&self) -> Option<usize> {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::MaxBitLength(value) = attribute {
                return Some(*value);
            }
        }

        None
    }

    pub fn get_position(&self) -> usize {
        for attribute in &self.attributes {
            if let FibexTypeAttribute::Position(value) = attribute {
                return *value;
            }
        }

        0
    }
}

#[derive(Debug, Clone)]
pub enum FibexTypeAttribute {
    HighLowByteOrder(bool),
    LengthField(usize),
    ArrayLengthField(usize),
    ArrayDeclaration(Vec<FibexArrayDimension>),
    TypeLengthField(usize),
    BitLength(usize),
    MinBitLength(usize),
    MaxBitLength(usize),
    Position(usize),
}

#[derive(Debug, Clone)]
pub struct FibexArrayDimension {
    pub index: usize,
    pub min: usize,
    pub max: usize,
}

impl FibexArrayDimension {
    pub fn is_dynamic(&self) -> bool {
        self.min != self.max
    }
}

pub type FibexTypeReference = Rc<RefCell<FibexTypeInstance>>;

#[derive(Debug)]
pub struct FibexTypeInstance {
    pub id: String,
    pub name: String,
    pub datatype: FibexDatatype,
    coding_ref: Option<String>,
}

#[derive(Debug)]
pub enum FibexDatatype {
    Unknown,
    Primitive(FibexPrimitive),
    Struct(FibexStruct),
    Enum(FibexEnum),
    String(FibexString),
    // Union,
    // Optional,
}

impl PartialEq for FibexDatatype {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, PartialEq)]
pub enum FibexPrimitive {
    Unknown,
    Bool,
    Uint8,
    Uint16,
    Uint24,
    Uint32,
    Uint64,
    Int8,
    Int16,
    Int24,
    Int32,
    Int64,
    Float32,
    Float64,
}

impl From<&str> for FibexPrimitive {
    fn from(name: &str) -> Self {
        match name {
            "BOOL" => FibexPrimitive::Bool,
            "UINT8" => FibexPrimitive::Uint8,
            "UINT16" => FibexPrimitive::Uint16,
            "UINT24" => FibexPrimitive::Uint24,
            "UINT32" => FibexPrimitive::Uint32,
            "UINT64" => FibexPrimitive::Uint64,
            "INT8" => FibexPrimitive::Int8,
            "INT16" => FibexPrimitive::Int16,
            "INT24" => FibexPrimitive::Int24,
            "INT32" => FibexPrimitive::Int32,
            "INT64" => FibexPrimitive::Int64,
            "FLOAT32" => FibexPrimitive::Float32,
            "FLOAT64" => FibexPrimitive::Float64,
            _ => FibexPrimitive::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct FibexStruct {
    pub members: Vec<FibexTypeDeclaration>,
}

#[derive(Debug)]
pub struct FibexEnum {
    pub primitive: FibexPrimitive,
    pub variants: Vec<FibexEnumDeclaration>,
}

#[derive(Debug)]
pub struct FibexEnumDeclaration {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct FibexString {
    pub encoding: FibexStringEncoding,
    pub is_dynamic: bool,
    pub has_bom: bool,
    pub has_termination: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub bit_length: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub enum FibexStringEncoding {
    Unknown,
    UTF8,
    UTF16,
}

impl From<&str> for FibexStringEncoding {
    fn from(name: &str) -> Self {
        match name {
            STRING_ENCODING_UTF8 => FibexStringEncoding::UTF8,
            STRING_ENCODING_UTF16 => FibexStringEncoding::UTF16,
            STRING_ENCODING_UCS2 => FibexStringEncoding::UTF16,
            _ => FibexStringEncoding::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct FibexTypeCoding {
    pub id: String,
    pub name: String,
    pub attributes: Vec<FibexCodingAttribute>,
}

const STRING_CATEGORY_DYNAMIC: &str = "LEADING-LENGTH-INFO-TYPE";
const STRING_ENCODING_UTF8: &str = "UTF-8";
const STRING_ENCODING_UTF16: &str = "UTF-16";
const STRING_ENCODING_UCS2: &str = "UCS-2"; // subset of UTF16
const STRING_BOM: &str = "EXPLICIT";
const STRING_TERMINATION: &str = "ZERO";

impl FibexTypeCoding {
    fn resolve(&self) -> Option<FibexDatatype> {
        if self.is_string() {
            return Some(FibexDatatype::String(FibexString {
                encoding: self.encoding(),
                is_dynamic: self.is_dynamic(),
                has_bom: self.has_bom(),
                has_termination: self.has_termination(),
                min_length: self.min_length(),
                max_length: self.max_length(),
                bit_length: self.bit_length(),
            }));
        } else {
            let primitive = FibexPrimitive::from(&*self.name);
            if FibexPrimitive::Unknown != primitive {
                return Some(FibexDatatype::Primitive(primitive));
            }

            if let Ok(regex) = Regex::new(r"^A_(.*)$") {
                if let Some(capture) = regex.captures(&*self.base_type()) {
                    if let Some(matcher) = capture.get(1) {
                        let primitive = FibexPrimitive::from(&*matcher.as_str());
                        if FibexPrimitive::Unknown != primitive {
                            return Some(FibexDatatype::Primitive(primitive));
                        }
                    }
                }
            }
        }

        None
    }

    fn is_string(&self) -> bool {
        self.encoding() != FibexStringEncoding::Unknown
    }

    fn is_dynamic(&self) -> bool {
        for attribute in &self.attributes {
            if let FibexCodingAttribute::Category(value) = attribute {
                return !value.is_empty() && (value == STRING_CATEGORY_DYNAMIC);
            }
        }

        false
    }

    fn encoding(&self) -> FibexStringEncoding {
        for attribute in &self.attributes {
            if let FibexCodingAttribute::Encoding(value) = attribute {
                return FibexStringEncoding::from(&*(*value));
            }
        }

        FibexStringEncoding::Unknown
    }

    fn has_bom(&self) -> bool {
        for attribute in &self.attributes {
            if let FibexCodingAttribute::Bom(value) = attribute {
                return !value.is_empty() && (value == STRING_BOM);
            }
        }

        true // default
    }

    fn has_termination(&self) -> bool {
        for attribute in &self.attributes {
            if let FibexCodingAttribute::Termination(value) = attribute {
                return !value.is_empty() && (value == STRING_TERMINATION);
            }
        }

        true // default
    }

    fn min_length(&self) -> Option<usize> {
        for attribute in &self.attributes {
            if let FibexCodingAttribute::MinLength(value) = attribute {
                return Some(*value);
            }
        }

        None
    }

    fn max_length(&self) -> Option<usize> {
        for attribute in &self.attributes {
            if let FibexCodingAttribute::MaxLength(value) = attribute {
                return Some(*value);
            }
        }

        None
    }

    fn bit_length(&self) -> Option<usize> {
        for attribute in &self.attributes {
            if let FibexCodingAttribute::BitLength(value) = attribute {
                return Some(*value);
            }
        }

        None
    }

    fn base_type(&self) -> String {
        for attribute in &self.attributes {
            if let FibexCodingAttribute::BaseType(value) = attribute {
                return value.clone();
            }
        }

        String::from("")
    }
}

#[derive(Debug, Clone)]
pub enum FibexCodingAttribute {
    BaseType(String),
    Category(String),
    Encoding(String),
    Bom(String),
    Termination(String),
    MinLength(usize),
    MaxLength(usize),
    BitLength(usize),
}

mod parser {
    use super::xml::FibexEvent;
    use super::*;

    pub struct FibexXmlParser;

    impl FibexXmlParser {
        pub fn parse<R: Read>(reader: FibexReader<BufReader<R>>) -> Result<FibexModel, FibexError> {
            parse_fibex(reader, true)
        }

        pub fn try_parse<R: Read>(
            reader: FibexReader<BufReader<R>>,
        ) -> Result<FibexModel, FibexError> {
            parse_fibex(reader, false)
        }
    }

    fn parse_fibex<R: Read>(
        mut reader: FibexReader<BufReader<R>>,
        strict: bool,
    ) -> Result<FibexModel, FibexError> {
        let mut model = FibexModel::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::ServiceStart(id) => {
                    let (service, mut service_types) = parse_service_interface(&mut reader, id)?;
                    model.services.push(service);
                    model.types.append(&mut service_types);
                }
                FibexEvent::DatatypeStart(id, datatype_type) => {
                    if datatype_type == "fx:COMMON-DATATYPE-TYPE" {
                        model.types.push(Rc::new(RefCell::new(parse_common_datatype(
                            &mut reader,
                            id,
                        )?)));
                    } else if datatype_type == "fx:COMPLEX-DATATYPE-TYPE" {
                        model
                            .types
                            .push(Rc::new(RefCell::new(parse_complex_datatype(
                                &mut reader,
                                id,
                            )?)));
                    } else if datatype_type == "fx:ENUM-DATATYPE-TYPE" {
                        model
                            .types
                            .push(Rc::new(RefCell::new(parse_enum_datatype(&mut reader, id)?)));
                    }
                }
                FibexEvent::ProcessingInfoStart => {
                    model
                        .codings
                        .append(&mut parse_processing_info(&mut reader, "ProcessingInfo")?);
                }
                FibexEvent::Eof => break,
                _ => {}
            }
        }

        model.pack(strict)?;

        Ok(model)
    }

    fn parse_service_interface<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: String,
    ) -> Result<(FibexServiceInterface, Vec<FibexTypeReference>), FibexError> {
        let mut service_name: Option<String> = None;
        let mut service_id: Option<usize> = None;
        let mut major_version: Option<usize> = None;
        let mut minor_version: Option<usize> = None;
        let mut methods: Vec<FibexServiceMethod> = Vec::new();
        let mut types: Vec<FibexTypeReference> = Vec::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::ShortName(name) => {
                    service_name = Some(name);
                }
                FibexEvent::ServiceIdentifier(identifier) => {
                    service_id = Some(parse_number(reader, &identifier)?);
                }
                FibexEvent::MajorVersion(version) => {
                    major_version = Some(parse_number(reader, &version)?);
                }
                FibexEvent::MinorVersion(version) => {
                    minor_version = Some(parse_number(reader, &version)?);
                }
                FibexEvent::MethodStart(id) => {
                    let (method, mut method_types) = parse_service_method(reader, id)?;
                    methods.push(method);
                    types.append(&mut method_types);
                }
                FibexEvent::FieldStart(id) => {
                    let (mut field_methods, field) = parse_service_field(reader, id)?;
                    methods.append(&mut field_methods);
                    types.push(field);
                }
                FibexEvent::ServiceEnd => {
                    return Ok((
                        FibexServiceInterface {
                            id,
                            name: first_to_upper(&get_value(reader, service_name)?),
                            service_id: get_value(reader, service_id)?,
                            major_version: get_value(reader, major_version)?,
                            minor_version: get_value(reader, minor_version)?,
                            methods,
                        },
                        types,
                    ));
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(&id));
                }
                _ => {}
            }
        }
    }

    fn parse_service_method<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: String,
    ) -> Result<(FibexServiceMethod, Vec<FibexTypeReference>), FibexError> {
        let mut method_name: Option<String> = None;
        let mut method_id: Option<usize> = None;
        let mut request: Option<FibexTypeDeclaration> = None;
        let mut response: Option<FibexTypeDeclaration> = None;
        let mut types: Vec<FibexTypeReference> = Vec::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::ShortName(name) => {
                    method_name = Some(name);
                }
                FibexEvent::MethodIdentifier(identifier) => {
                    method_id = Some(parse_number(reader, &identifier)?);
                }
                FibexEvent::MethodInputStart => {
                    let (parameter, attributes) = parse_method_parameter(reader, &id, "Request")?;
                    request = Some(FibexTypeDeclaration {
                        id: format!("{}/Request", id),
                        name: String::from(""),
                        id_ref: parameter.borrow().id.clone(),
                        type_ref: Some(parameter.clone()),
                        attributes,
                    });
                    types.push(parameter);
                }
                FibexEvent::MethodOutputStart => {
                    let (parameter, attributes) = parse_method_parameter(reader, &id, "Response")?;
                    response = Some(FibexTypeDeclaration {
                        id: format!("{}/Response", id),
                        name: String::from(""),
                        id_ref: parameter.borrow().id.clone(),
                        type_ref: Some(parameter.clone()),
                        attributes,
                    });
                    types.push(parameter);
                }
                FibexEvent::MethodEnd => {
                    return Ok((
                        FibexServiceMethod {
                            id,
                            name: first_to_lower(&get_value(reader, method_name)?),
                            method_id: get_value(reader, method_id)?,
                            request,
                            response,
                        },
                        types,
                    ));
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(&id));
                }
                _ => {}
            }
        }
    }

    fn parse_method_parameter<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: &str,
        parameter: &str,
    ) -> Result<(FibexTypeReference, Vec<FibexTypeAttribute>), FibexError> {
        let mut members: Vec<FibexTypeDeclaration> = Vec::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::DatatypeMemberStart(id) => {
                    members.push(parse_datatype_member(reader, id)?);
                }
                FibexEvent::MethodParamEnd => {
                    members.sort_by_key(|m| m.get_position());
                    return Ok((
                        Rc::new(RefCell::new(FibexTypeInstance {
                            id: format!("{}/{}", id, parameter),
                            name: String::from(""),
                            datatype: FibexDatatype::Struct(FibexStruct { members }),
                            coding_ref: None,
                        })),
                        Vec::new(),
                    ));
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(id));
                }
                _ => {}
            }
        }
    }

    fn parse_service_field<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: String,
    ) -> Result<(Vec<FibexServiceMethod>, FibexTypeReference), FibexError> {
        let mut field_name: Option<String> = None;
        let mut field_type: Option<FibexTypeReference> = None;
        let mut methods: Vec<FibexServiceMethod> = Vec::new();
        let mut attributes: Vec<FibexTypeAttribute> = Vec::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::ShortName(name) => {
                    field_name = Some(name);
                }
                FibexEvent::DatatypeRef(id_ref) => {
                    field_type = Some(Rc::new(RefCell::new(FibexTypeInstance {
                        id: id.clone(),
                        name: String::from(""),
                        datatype: FibexDatatype::Struct(FibexStruct {
                            members: vec![FibexTypeDeclaration {
                                id: id.clone(),
                                name: first_to_lower(
                                    &ref_value(reader, field_name.as_ref())?.clone(),
                                ),
                                id_ref,
                                type_ref: None,
                                attributes: Vec::new(),
                            }],
                        }),
                        coding_ref: None,
                    })));
                }
                FibexEvent::ArrayDeclarationStart => {
                    attributes.push(parse_array_declaration(reader, &id)?);
                }
                FibexEvent::UtilizationStart => {
                    attributes.append(&mut parse_type_utilization(reader, &id)?);
                }
                FibexEvent::FieldGetterStart => {
                    methods.push(parse_field_accessor(
                        reader,
                        &id,
                        "Getter",
                        &format!(
                            "get{}",
                            first_to_upper(ref_value(reader, field_name.as_ref())?)
                        ),
                        ref_value(reader, field_type.as_ref())?,
                        false,
                        true,
                    )?);
                }
                FibexEvent::FieldSetterStart => {
                    methods.push(parse_field_accessor(
                        reader,
                        &id,
                        "Setter",
                        &format!(
                            "set{}",
                            first_to_upper(ref_value(reader, field_name.as_ref())?)
                        ),
                        ref_value(reader, field_type.as_ref())?,
                        true,
                        true,
                    )?);
                }
                FibexEvent::FieldNotifierStart => {
                    methods.push(parse_field_accessor(
                        reader,
                        &id,
                        "Notifier",
                        &first_to_lower(ref_value(reader, field_name.as_ref())?),
                        ref_value(reader, field_type.as_ref())?,
                        true,
                        false,
                    )?);
                }
                FibexEvent::FieldEnd => {
                    if let Some(item) = &mut field_type {
                        if let FibexDatatype::Struct(struct_type) = &mut item.borrow_mut().datatype
                        {
                            if let Some(member) = struct_type.members.get_mut(0) {
                                member.attributes.append(&mut attributes);
                            }
                        }
                    }

                    return Ok((methods, get_value(reader, field_type)?));
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(&id));
                }
                _ => {}
            }
        }
    }

    fn parse_field_accessor<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: &str,
        accessor: &str,
        name: &str,
        type_ref: &FibexTypeReference,
        has_request: bool,
        has_response: bool,
    ) -> Result<FibexServiceMethod, FibexError> {
        let mut method_id: Option<usize> = None;

        loop {
            match reader.read_fibex()? {
                FibexEvent::MethodIdentifier(identifier) => {
                    method_id = Some(parse_number(reader, &identifier)?);
                }
                FibexEvent::NotificationIdentifier(identifier) => {
                    method_id = Some(parse_number(reader, &identifier)?);
                }
                FibexEvent::FieldAccessorEnd => {
                    return Ok(FibexServiceMethod {
                        id: format!("{}/{}", id, accessor),
                        name: String::from(name),
                        method_id: get_value(reader, method_id)?,
                        request: match has_request {
                            false => None,
                            true => Some(FibexTypeDeclaration {
                                id: format!("{}/{}", id, accessor),
                                name: String::from(""),
                                id_ref: String::from(id),
                                type_ref: Some(type_ref.clone()),
                                attributes: vec![],
                            }),
                        },
                        response: match has_response {
                            false => None,
                            true => Some(FibexTypeDeclaration {
                                id: format!("{}/{}", id, accessor),
                                name: String::from(""),
                                id_ref: String::from(id),
                                type_ref: Some(type_ref.clone()),
                                attributes: vec![],
                            }),
                        },
                    });
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(id));
                }
                _ => {}
            }
        }
    }

    fn parse_processing_info<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: &str,
    ) -> Result<Vec<FibexTypeCoding>, FibexError> {
        let mut codings: Vec<FibexTypeCoding> = Vec::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::CodingInfoStart(id) => codings.push(parse_coding_info(reader, id)?),
                FibexEvent::ProcessingInfoEnd => {
                    return Ok(codings);
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(id));
                }
                _ => {}
            }
        }
    }

    fn parse_coding_info<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: String,
    ) -> Result<FibexTypeCoding, FibexError> {
        let mut coding_name: Option<String> = None;
        let mut attributes = Vec::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::ShortName(name) => {
                    coding_name = Some(name);
                }
                FibexEvent::CodedType(base_type, category, encoding, bom, termination) => {
                    if !base_type.is_empty() {
                        attributes.push(FibexCodingAttribute::BaseType(base_type));
                    }
                    if !category.is_empty() {
                        attributes.push(FibexCodingAttribute::Category(category));
                    }
                    if !encoding.is_empty() {
                        attributes.push(FibexCodingAttribute::Encoding(encoding));
                    }
                    if !bom.is_empty() {
                        attributes.push(FibexCodingAttribute::Bom(bom));
                    }
                    if !termination.is_empty() {
                        attributes.push(FibexCodingAttribute::Termination(termination));
                    }
                }
                FibexEvent::MinLength(value) => {
                    if !value.is_empty() {
                        attributes.push(FibexCodingAttribute::MinLength(parse_number(
                            reader, &*value,
                        )?));
                    }
                }
                FibexEvent::MaxLength(value) => {
                    if !value.is_empty() {
                        attributes.push(FibexCodingAttribute::MaxLength(parse_number(
                            reader, &*value,
                        )?));
                    }
                }
                FibexEvent::BitLength(value) => {
                    if !value.is_empty() {
                        attributes.push(FibexCodingAttribute::BitLength(parse_number(
                            reader, &*value,
                        )?));
                    }
                }
                FibexEvent::CodingInfoEnd => {
                    return Ok(FibexTypeCoding {
                        id,
                        name: get_value(reader, coding_name)?,
                        attributes,
                    });
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(&id));
                }
                _ => {}
            }
        }
    }

    fn parse_common_datatype<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: String,
    ) -> Result<FibexTypeInstance, FibexError> {
        let mut datatype_name: Option<String> = None;
        let mut datatype_type = FibexDatatype::Unknown;
        let mut datatype_coding: Option<String> = None;

        loop {
            match reader.read_fibex()? {
                FibexEvent::ShortName(name) => {
                    let primitive = FibexPrimitive::from(&*name);
                    if FibexPrimitive::Unknown != primitive {
                        datatype_type = FibexDatatype::Primitive(primitive);
                    }

                    datatype_name = Some(name);
                }
                FibexEvent::CodingRef(id_ref) => {
                    datatype_coding = Some(id_ref);
                }
                FibexEvent::DatatypeEnd => {
                    return Ok(FibexTypeInstance {
                        id,
                        name: first_to_upper(&get_value(reader, datatype_name)?),
                        datatype: datatype_type,
                        coding_ref: datatype_coding,
                    });
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(&id));
                }
                _ => {}
            }
        }
    }

    fn parse_complex_datatype<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: String,
    ) -> Result<FibexTypeInstance, FibexError> {
        let mut datatype_name: Option<String> = None;
        let mut datatype_type = FibexDatatype::Unknown;

        loop {
            match reader.read_fibex()? {
                FibexEvent::ShortName(name) => {
                    datatype_name = Some(name);
                }
                FibexEvent::DatatypeClass(datatype_class) => {
                    if datatype_class == "STRUCTURE" || datatype_class == "TYPEDEF" {
                        datatype_type = FibexDatatype::Struct(FibexStruct {
                            members: Vec::new(),
                        });
                    }
                }
                FibexEvent::DatatypeMemberStart(id) => {
                    if let FibexDatatype::Struct(datatype) = &mut datatype_type {
                        datatype.members.push(parse_datatype_member(reader, id)?);
                    }
                }
                FibexEvent::DatatypeEnd => {
                    if let FibexDatatype::Struct(datatype) = &mut datatype_type {
                        datatype.members.sort_by_key(|m| m.get_position());
                    }
                    return Ok(FibexTypeInstance {
                        id,
                        name: first_to_upper(&get_value(reader, datatype_name)?),
                        datatype: datatype_type,
                        coding_ref: None,
                    });
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(&id));
                }
                _ => {}
            }
        }
    }

    fn parse_datatype_member<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: String,
    ) -> Result<FibexTypeDeclaration, FibexError> {
        let mut member_name: Option<String> = None;
        let mut member_type: Option<String> = None;
        let mut attributes: Vec<FibexTypeAttribute> = Vec::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::ShortName(name) => {
                    member_name = Some(name);
                }
                FibexEvent::DatatypeRef(id_ref) => {
                    member_type = Some(id_ref);
                }
                FibexEvent::ArrayDeclarationStart => {
                    attributes.push(parse_array_declaration(reader, &id)?);
                }
                FibexEvent::UtilizationStart => {
                    attributes.append(&mut parse_type_utilization(reader, &id)?);
                }
                FibexEvent::Position(value) => {
                    attributes.push(FibexTypeAttribute::Position(parse_number(reader, &value)?));
                }
                FibexEvent::DatatypeMemberEnd => {
                    return Ok(FibexTypeDeclaration {
                        id,
                        name: first_to_lower(&get_value(reader, member_name)?),
                        id_ref: get_value(reader, member_type)?,
                        type_ref: None,
                        attributes,
                    });
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(&id));
                }
                _ => {}
            }
        }
    }

    fn parse_enum_datatype<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: String,
    ) -> Result<FibexTypeInstance, FibexError> {
        let mut enum_name: Option<String> = None;
        let mut enum_coding: Option<String> = None;
        let mut variants = Vec::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::ShortName(name) => {
                    enum_name = Some(name);
                }
                FibexEvent::CodingRef(id_ref) => {
                    enum_coding = Some(id_ref);
                }
                FibexEvent::DatatypeEnd => {
                    return Ok(FibexTypeInstance {
                        id,
                        name: first_to_upper(&get_value(reader, enum_name)?),
                        datatype: FibexDatatype::Enum(FibexEnum {
                            primitive: FibexPrimitive::Unknown,
                            variants,
                        }),
                        coding_ref: Some(get_value(reader, enum_coding)?),
                    });
                }
                FibexEvent::EnumStart => {
                    variants.push(parse_enum_element(reader, &id)?);
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(&id));
                }
                _ => {}
            }
        }
    }

    fn parse_enum_element<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: &str,
    ) -> Result<FibexEnumDeclaration, FibexError> {
        let mut element_name: Option<String> = None;
        let mut element_value: Option<String> = None;

        loop {
            match reader.read_fibex()? {
                FibexEvent::EnumName(name) => {
                    element_name = Some(name);
                }
                FibexEvent::EnumValue(value) => {
                    element_value = Some(value);
                }
                FibexEvent::EnumEnd => {
                    return Ok(FibexEnumDeclaration {
                        name: get_value(reader, element_name)?,
                        value: get_value(reader, element_value)?,
                    });
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(id));
                }
                _ => {}
            }
        }
    }

    fn parse_array_declaration<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: &str,
    ) -> Result<FibexTypeAttribute, FibexError> {
        let mut dimensions = Vec::new();
        let mut index: Option<usize> = None;
        let mut min: Option<usize> = None;
        let mut max: Option<usize> = None;

        loop {
            match reader.read_fibex()? {
                FibexEvent::ArrayDimensionStart => {
                    index = None;
                    min = None;
                    max = None;
                }
                FibexEvent::ArrayDimension(value) => {
                    index = Some(parse_number(reader, &value)?);
                }
                FibexEvent::MinimumSize(value) => {
                    min = Some(parse_number(reader, &value)?);
                }
                FibexEvent::MaximumSize(value) => {
                    max = Some(parse_number(reader, &value)?);
                }
                FibexEvent::ArrayDimensionEnd => {
                    dimensions.push(FibexArrayDimension {
                        index: get_value(reader, index)?,
                        min: min.unwrap_or(0),
                        max: max.unwrap_or(0),
                    });
                }
                FibexEvent::ArrayDeclarationEnd => {
                    return Ok(FibexTypeAttribute::ArrayDeclaration(dimensions));
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(id));
                }
                _ => {}
            }
        }
    }

    fn parse_type_utilization<R: Read>(
        reader: &mut FibexReader<BufReader<R>>,
        id: &str,
    ) -> Result<Vec<FibexTypeAttribute>, FibexError> {
        let mut attributes = Vec::new();

        loop {
            match reader.read_fibex()? {
                FibexEvent::HighLowByteOrder(value) => {
                    attributes.push(FibexTypeAttribute::HighLowByteOrder(parse_bool(
                        reader, &value,
                    )?));
                }
                FibexEvent::LengthField(value) => {
                    attributes.push(FibexTypeAttribute::LengthField(parse_number(
                        reader, &value,
                    )?));
                }
                FibexEvent::ArrayLengthField(value) => {
                    attributes.push(FibexTypeAttribute::ArrayLengthField(parse_number(
                        reader, &value,
                    )?));
                }
                FibexEvent::TypeLengthField(value) => {
                    attributes.push(FibexTypeAttribute::TypeLengthField(parse_number(
                        reader, &value,
                    )?));
                }
                FibexEvent::BitLength(value) => {
                    attributes.push(FibexTypeAttribute::BitLength(parse_number(reader, &value)?));
                }
                FibexEvent::MinBitLength(value) => {
                    attributes.push(FibexTypeAttribute::MinBitLength(parse_number(
                        reader, &value,
                    )?));
                }
                FibexEvent::MaxBitLength(value) => {
                    attributes.push(FibexTypeAttribute::MaxBitLength(parse_number(
                        reader, &value,
                    )?));
                }
                FibexEvent::UtilizationEnd => {
                    return Ok(attributes);
                }
                FibexEvent::Eof => {
                    return Err(unexpected_eof(id));
                }
                _ => {}
            }
        }
    }

    fn unexpected_eof(id: &str) -> FibexError {
        FibexError::Parse(format!("Unexpected EOF at {}", id))
    }

    fn parse_bool<R: Read>(
        reader: &FibexReader<BufReader<R>>,
        value: &str,
    ) -> Result<bool, FibexError> {
        if let Ok(result) = value.parse::<bool>() {
            return Ok(result);
        }

        Err(FibexError::Model(format!(
            "Invalid bool {} at {}",
            value,
            reader.position(),
        )))
    }

    fn parse_number<R: Read>(
        reader: &FibexReader<BufReader<R>>,
        value: &str,
    ) -> Result<usize, FibexError> {
        if let Ok(result) = value.parse::<usize>() {
            return Ok(result);
        }

        Err(FibexError::Model(format!(
            "Invalid number {} at {}",
            value,
            reader.position(),
        )))
    }

    fn get_value<R: Read, T>(
        reader: &FibexReader<BufReader<R>>,
        value: Option<T>,
    ) -> Result<T, FibexError> {
        if let Some(result) = value {
            return Ok(result);
        }

        Err(FibexError::Model(format!(
            "Missing value at {}",
            reader.position()
        )))
    }

    fn ref_value<'a, R: Read, T>(
        reader: &FibexReader<BufReader<R>>,
        value: Option<&'a T>,
    ) -> Result<&'a T, FibexError> {
        if let Some(result) = value {
            return Ok(result);
        }

        Err(FibexError::Model(format!(
            "Missing value at {}",
            reader.position()
        )))
    }

    fn first_to_upper(string: &str) -> String {
        capitalize(string, false)
    }

    fn first_to_lower(string: &str) -> String {
        decapitalize(string, false)
    }
}

pub type FibexParser = parser::FibexXmlParser;

mod xml {
    use super::*;

    const B_ID: &[u8] = b"ID";
    const B_ID_REF: &[u8] = b"ID-REF";
    const B_SHORT_NAME: &[u8] = b"SHORT-NAME";

    const B_SERVICE_INTERFACE: &[u8] = b"SERVICE-INTERFACE";
    const B_SERVICE_IDENTIFIER: &[u8] = b"SERVICE-IDENTIFIER";
    const B_SERVICE_VERSION_MAJOR: &[u8] = b"MAJOR";
    const B_SERVICE_VERSION_MINOR: &[u8] = b"MINOR";
    const B_SERVICE_METHOD: &[u8] = b"METHOD";
    const B_SERVICE_EVENT: &[u8] = b"EVENT";
    const B_SERVICE_FIELD: &[u8] = b"FIELD";

    const B_METHOD_IDENTIFIER: &[u8] = b"METHOD-IDENTIFIER";
    const B_METHOD_INPUT: &[u8] = b"INPUT-PARAMETERS";
    const B_METHOD_INPUT_PARAMETER: &[u8] = b"INPUT-PARAMETER";
    const B_METHOD_OUTPUT: &[u8] = b"RETURN-PARAMETERS";
    const B_METHOD_OUTPUT_PARAMETER: &[u8] = b"RETURN-PARAMETER";

    const B_FIELD_GETTER: &[u8] = b"GETTER";
    const B_FIELD_SETTER: &[u8] = b"SETTER";
    const B_FIELD_NOTIFIER: &[u8] = b"NOTIFIER";
    const B_NOTIFICATION_IDENTIFIER: &[u8] = b"NOTIFICATION-IDENTIFIER";

    const B_ARRAY_DECLARATION: &[u8] = b"ARRAY-DECLARATION";
    const B_ARRAY_DIMENSION: &[u8] = b"ARRAY-DIMENSION";
    const B_DIMENSION: &[u8] = b"DIMENSION";
    const B_MINIMUM_SIZE: &[u8] = b"MINIMUM-SIZE";
    const B_MAXIMUM_SIZE: &[u8] = b"MAXIMUM-SIZE";

    const B_UTILIZATION: &[u8] = b"UTILIZATION";
    const B_HIGH_LOW_BYTE_ORDER: &[u8] = b"IS-HIGH-LOW-BYTE-ORDER";
    const B_LENGTH_FIELD: &[u8] = b"LENGTH-FIELD-SIZE";
    const B_ARRAY_LENGTH_FIELD: &[u8] = b"ARRAY-LENGTH-FIELD-SIZE";
    const B_TYPE_LENGTH_FIELD: &[u8] = b"TYPE-FIELD-SIZE";

    const B_EVENT_GROUP: &[u8] = b"EVENT-GROUP";

    const B_DATATYPE: &[u8] = b"DATATYPE";
    const B_DATATYPE_TYPE: &[u8] = b"xsi:type";
    const B_DATATYPE_MEMBER: &[u8] = b"MEMBER";
    const B_DATATYPE_CLASS: &[u8] = b"COMPLEX-DATATYPE-CLASS";
    const B_DATATYPE_REF: &[u8] = b"DATATYPE-REF";
    const B_CODING_REF: &[u8] = b"CODING-REF";
    const B_POSITION: &[u8] = b"POSITION";

    const B_ENUM: &[u8] = b"ENUM-ELEMENT";
    const B_ENUM_NAME: &[u8] = b"SYNONYM";
    const B_ENUM_VALUE: &[u8] = b"VALUE";

    const B_PROCESSING_INFO: &[u8] = b"PROCESSING-INFORMATION";
    const B_CODING_INFO: &[u8] = b"CODING";
    const B_CODED_TYPE: &[u8] = b"CODED-TYPE";
    const B_CODED_BASE_TYPE: &[u8] = b"ho:BASE-DATA-TYPE";
    const B_CODED_CATEGORY: &[u8] = b"CATEGORY";
    const B_CODED_ENCODING: &[u8] = b"ENCODING";
    const B_CODED_BOM: &[u8] = b"BYTE-ORDER-MARK";
    const B_CODED_TERMINATION: &[u8] = b"TERMINATION";

    const B_MIN_LENGTH: &[u8] = b"MIN-LENGTH";
    const B_MAX_LENGTH: &[u8] = b"MAX-LENGTH";
    const B_BIT_LENGTH: &[u8] = b"BIT-LENGTH";
    const B_MIN_BIT_LENGTH: &[u8] = b"MIN-BIT-LENGTH";
    const B_MAX_BIT_LENGTH: &[u8] = b"MAX-BIT-LENGTH";

    const B_MANUFACTURER_EXTENSION: &[u8] = b"MANUFACTURER-EXTENSION";
    const B_COMPU_METHODS: &[u8] = b"COMPU-METHODS";

    #[derive(Debug)]
    pub enum FibexEvent {
        ShortName(String),
        ServiceStart(String),
        ServiceEnd,
        ServiceIdentifier(String),
        MethodIdentifier(String),
        NotificationIdentifier(String),
        MajorVersion(String),
        MinorVersion(String),
        MethodStart(String),
        MethodEnd,
        MethodInputStart,
        MethodOutputStart,
        MethodParamEnd,
        FieldStart(String),
        FieldEnd,
        FieldGetterStart,
        FieldSetterStart,
        FieldNotifierStart,
        FieldAccessorEnd,
        ArrayDeclarationStart,
        ArrayDeclarationEnd,
        ArrayDimensionStart,
        ArrayDimension(String),
        ArrayDimensionEnd,
        MinimumSize(String),
        MaximumSize(String),
        UtilizationStart,
        UtilizationEnd,
        HighLowByteOrder(String),
        LengthField(String),
        ArrayLengthField(String),
        TypeLengthField(String),
        DatatypeStart(String, String),
        DatatypeEnd,
        DatatypeMemberStart(String),
        DatatypeMemberEnd,
        DatatypeClass(String),
        DatatypeRef(String),
        CodingRef(String),
        Position(String),
        EnumStart,
        EnumEnd,
        EnumName(String),
        EnumValue(String),
        ProcessingInfoStart,
        ProcessingInfoEnd,
        CodingInfoStart(String),
        CodingInfoEnd,
        CodedType(String, String, String, String, String),
        MinLength(String),
        MaxLength(String),
        BitLength(String),
        MinBitLength(String),
        MaxBitLength(String),
        Eou,
        Eof,
    }

    pub struct FibexXmlReader<B: BufRead> {
        reader: XmlReader<B>,
        buffer: Vec<u8>,
        buffer2: Vec<u8>,
    }

    impl<B: BufRead> FibexXmlReader<B> {
        pub fn from_reader(reader: B) -> Result<Self, FibexError> {
            Ok(FibexXmlReader {
                reader: XmlReader::from_reader(reader),
                buffer: Vec::new(),
                buffer2: Vec::new(),
            })
        }

        pub fn position(&self) -> usize {
            self.reader.buffer_position()
        }
    }

    impl FibexXmlReader<BufReader<File>> {
        pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, FibexError> {
            Ok(FibexXmlReader {
                reader: XmlReader::from_file(path)?,
                buffer: Vec::new(),
                buffer2: Vec::new(),
            })
        }
    }

    impl<B: BufRead> FibexXmlReader<B> {
        pub fn read_fibex(&mut self) -> Result<FibexEvent, FibexError> {
            loop {
                match self.reader.read_event(&mut self.buffer)? {
                    XmlEvent::Start(ref event) => match event.local_name() {
                        B_SHORT_NAME => {
                            return Ok(FibexEvent::ShortName(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_SERVICE_INTERFACE => {
                            return Ok(FibexEvent::ServiceStart(get_attribute(
                                &self.reader,
                                event,
                                B_ID,
                            )?));
                        }
                        B_SERVICE_IDENTIFIER => {
                            return Ok(FibexEvent::ServiceIdentifier(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_SERVICE_VERSION_MAJOR => {
                            return Ok(FibexEvent::MajorVersion(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_SERVICE_VERSION_MINOR => {
                            return Ok(FibexEvent::MinorVersion(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_SERVICE_METHOD => {
                            return Ok(FibexEvent::MethodStart(get_attribute(
                                &self.reader,
                                event,
                                B_ID,
                            )?));
                        }
                        B_SERVICE_EVENT => {
                            return Ok(FibexEvent::MethodStart(get_attribute(
                                &self.reader,
                                event,
                                B_ID,
                            )?));
                        }
                        B_SERVICE_FIELD => {
                            return Ok(FibexEvent::FieldStart(get_attribute(
                                &self.reader,
                                event,
                                B_ID,
                            )?));
                        }
                        B_METHOD_IDENTIFIER => {
                            return Ok(FibexEvent::MethodIdentifier(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_METHOD_INPUT => {
                            return Ok(FibexEvent::MethodInputStart);
                        }
                        B_METHOD_INPUT_PARAMETER => {
                            return Ok(FibexEvent::DatatypeMemberStart(get_attribute(
                                &self.reader,
                                event,
                                B_ID,
                            )?));
                        }
                        B_METHOD_OUTPUT => {
                            return Ok(FibexEvent::MethodOutputStart);
                        }
                        B_METHOD_OUTPUT_PARAMETER => {
                            return Ok(FibexEvent::DatatypeMemberStart(get_attribute(
                                &self.reader,
                                event,
                                B_ID,
                            )?));
                        }
                        B_FIELD_GETTER => {
                            return Ok(FibexEvent::FieldGetterStart);
                        }
                        B_FIELD_SETTER => {
                            return Ok(FibexEvent::FieldSetterStart);
                        }
                        B_FIELD_NOTIFIER => {
                            return Ok(FibexEvent::FieldNotifierStart);
                        }
                        B_NOTIFICATION_IDENTIFIER => {
                            return Ok(FibexEvent::NotificationIdentifier(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_ARRAY_DECLARATION => {
                            return Ok(FibexEvent::ArrayDeclarationStart);
                        }
                        B_ARRAY_DIMENSION => {
                            return Ok(FibexEvent::ArrayDimensionStart);
                        }
                        B_DIMENSION => {
                            return Ok(FibexEvent::ArrayDimension(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_MINIMUM_SIZE => {
                            return Ok(FibexEvent::MinimumSize(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_MAXIMUM_SIZE => {
                            return Ok(FibexEvent::MaximumSize(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_UTILIZATION => {
                            return Ok(FibexEvent::UtilizationStart);
                        }
                        B_HIGH_LOW_BYTE_ORDER => {
                            return Ok(FibexEvent::HighLowByteOrder(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_LENGTH_FIELD => {
                            return Ok(FibexEvent::LengthField(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_ARRAY_LENGTH_FIELD => {
                            return Ok(FibexEvent::ArrayLengthField(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_TYPE_LENGTH_FIELD => {
                            return Ok(FibexEvent::TypeLengthField(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_MIN_LENGTH => {
                            return Ok(FibexEvent::MinLength(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_MAX_LENGTH => {
                            return Ok(FibexEvent::MaxLength(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_BIT_LENGTH => {
                            return Ok(FibexEvent::BitLength(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_MIN_BIT_LENGTH => {
                            return Ok(FibexEvent::MinBitLength(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_MAX_BIT_LENGTH => {
                            return Ok(FibexEvent::MaxBitLength(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_EVENT_GROUP => match self.skip_unrelated()? {
                            FibexEvent::Eou => {}
                            FibexEvent::Eof => {
                                return Ok(FibexEvent::Eof);
                            }
                            _ => {}
                        },
                        B_DATATYPE => {
                            return Ok(FibexEvent::DatatypeStart(
                                get_attribute(&self.reader, event, B_ID)?,
                                get_attribute(&self.reader, event, B_DATATYPE_TYPE)?,
                            ));
                        }
                        B_DATATYPE_CLASS => {
                            return Ok(FibexEvent::DatatypeClass(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_DATATYPE_MEMBER => {
                            return Ok(FibexEvent::DatatypeMemberStart(get_attribute(
                                &self.reader,
                                event,
                                B_ID,
                            )?));
                        }
                        B_POSITION => {
                            return Ok(FibexEvent::Position(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_ENUM => {
                            return Ok(FibexEvent::EnumStart);
                        }
                        B_ENUM_NAME => {
                            return Ok(FibexEvent::EnumName(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_ENUM_VALUE => {
                            return Ok(FibexEvent::EnumValue(get_text(
                                &mut self.reader,
                                &mut self.buffer2,
                                event,
                            )?));
                        }
                        B_PROCESSING_INFO => {
                            return Ok(FibexEvent::ProcessingInfoStart);
                        }
                        B_CODING_INFO => {
                            return Ok(FibexEvent::CodingInfoStart(get_attribute(
                                &self.reader,
                                event,
                                B_ID,
                            )?));
                        }
                        B_CODED_TYPE => {
                            return Ok(FibexEvent::CodedType(
                                get_attribute(&self.reader, event, B_CODED_BASE_TYPE)?,
                                get_attribute(&self.reader, event, B_CODED_CATEGORY)?,
                                get_attribute(&self.reader, event, B_CODED_ENCODING)?,
                                get_attribute(&self.reader, event, B_CODED_BOM)?,
                                get_attribute(&self.reader, event, B_CODED_TERMINATION)?,
                            ));
                        }
                        B_COMPU_METHODS => match self.skip_unrelated()? {
                            FibexEvent::Eou => {}
                            FibexEvent::Eof => {
                                return Ok(FibexEvent::Eof);
                            }
                            _ => {}
                        },
                        B_MANUFACTURER_EXTENSION => match self.skip_unrelated()? {
                            FibexEvent::Eou => {}
                            FibexEvent::Eof => {
                                return Ok(FibexEvent::Eof);
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    XmlEvent::Empty(ref event) => match event.local_name() {
                        B_DATATYPE_REF => {
                            return Ok(FibexEvent::DatatypeRef(get_attribute(
                                &self.reader,
                                event,
                                B_ID_REF,
                            )?));
                        }
                        B_CODING_REF => {
                            return Ok(FibexEvent::CodingRef(get_attribute(
                                &self.reader,
                                event,
                                B_ID_REF,
                            )?));
                        }
                        _ => {}
                    },
                    XmlEvent::End(ref event) => match event.local_name() {
                        B_SERVICE_INTERFACE => {
                            return Ok(FibexEvent::ServiceEnd);
                        }
                        B_SERVICE_METHOD => {
                            return Ok(FibexEvent::MethodEnd);
                        }
                        B_SERVICE_EVENT => {
                            return Ok(FibexEvent::MethodEnd);
                        }
                        B_SERVICE_FIELD => {
                            return Ok(FibexEvent::FieldEnd);
                        }
                        B_METHOD_INPUT => {
                            return Ok(FibexEvent::MethodParamEnd);
                        }
                        B_METHOD_INPUT_PARAMETER => {
                            return Ok(FibexEvent::DatatypeMemberEnd);
                        }
                        B_METHOD_OUTPUT => {
                            return Ok(FibexEvent::MethodParamEnd);
                        }
                        B_METHOD_OUTPUT_PARAMETER => {
                            return Ok(FibexEvent::DatatypeMemberEnd);
                        }
                        B_FIELD_GETTER => {
                            return Ok(FibexEvent::FieldAccessorEnd);
                        }
                        B_FIELD_SETTER => {
                            return Ok(FibexEvent::FieldAccessorEnd);
                        }
                        B_FIELD_NOTIFIER => {
                            return Ok(FibexEvent::FieldAccessorEnd);
                        }
                        B_ARRAY_DECLARATION => {
                            return Ok(FibexEvent::ArrayDeclarationEnd);
                        }
                        B_ARRAY_DIMENSION => {
                            return Ok(FibexEvent::ArrayDimensionEnd);
                        }
                        B_UTILIZATION => {
                            return Ok(FibexEvent::UtilizationEnd);
                        }
                        B_DATATYPE => {
                            return Ok(FibexEvent::DatatypeEnd);
                        }
                        B_DATATYPE_MEMBER => {
                            return Ok(FibexEvent::DatatypeMemberEnd);
                        }
                        B_ENUM => {
                            return Ok(FibexEvent::EnumEnd);
                        }
                        B_PROCESSING_INFO => {
                            return Ok(FibexEvent::ProcessingInfoEnd);
                        }
                        B_CODING_INFO => {
                            return Ok(FibexEvent::CodingInfoEnd);
                        }
                        _ => {}
                    },
                    XmlEvent::Eof => return Ok(FibexEvent::Eof),
                    _ => {}
                }
            }
        }

        fn skip_unrelated(&mut self) -> Result<FibexEvent, FibexError> {
            loop {
                match self.reader.read_event(&mut self.buffer2)? {
                    XmlEvent::End(ref event) => match event.local_name() {
                        B_EVENT_GROUP => return Ok(FibexEvent::Eou),
                        B_COMPU_METHODS => return Ok(FibexEvent::Eou),
                        B_MANUFACTURER_EXTENSION => return Ok(FibexEvent::Eou),
                        _ => {}
                    },
                    XmlEvent::Eof => return Ok(FibexEvent::Eof),
                    _ => {}
                }
            }
        }
    }

    fn get_text<B: BufRead>(
        reader: &mut XmlReader<B>,
        buffer: &mut Vec<u8>,
        event: &BytesStart<'_>,
    ) -> Result<String, FibexError> {
        Ok(reader.read_text(event.name(), buffer)?)
    }

    fn get_attribute<B: BufRead>(
        reader: &XmlReader<B>,
        event: &BytesStart<'_>,
        name: &[u8],
    ) -> Result<String, FibexError> {
        for attribute in event.attributes() {
            let attribute = attribute?;
            if attribute.key == name {
                return Ok(attribute.unescape_and_decode_value(reader)?);
            }
        }

        Ok(String::from(""))
    }
}

pub type FibexReader<B> = xml::FibexXmlReader<B>;

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    macro_rules! match_if {
        ($expression:expr, $( $pattern:pat )+, $result:expr) => {
            match $expression {
                $( $pattern => $result, )+
                _ => { panic!(); }
            }
        }
    }

    macro_rules! assert_if {
        ($expression:expr, $( $pattern:pat )+) => {
            assert!(matches!($expression, $( $pattern )+));
        }
    }

    #[test]
    fn test_parse_service() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:SERVICE-INTERFACE ID="/SOMEIP/TEST/ServiceInterface_TestService">
                <ho:SHORT-NAME>TestService</ho:SHORT-NAME>
                <fx:SERVICE-IDENTIFIER>123</fx:SERVICE-IDENTIFIER>
                <fx:PACKAGE-REF ID-REF="/SOMEIP/TEST"/>
                <service:API-VERSION>
                    <service:MAJOR>1</service:MAJOR>
                    <service:MINOR>2</service:MINOR>
                </service:API-VERSION>
            </fx:SERVICE-INTERFACE>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(0, model.types.len());
        assert_eq!(1, model.services.len());

        let service = model.services.get(0).unwrap();

        assert_eq!(service.id, "/SOMEIP/TEST/ServiceInterface_TestService");
        assert_eq!(service.name, "TestService");
        assert_eq!(123, service.service_id);
        assert_eq!(1, service.major_version);
        assert_eq!(2, service.minor_version);
        assert_eq!(0, service.methods.len());
    }

    #[test]
    fn test_parse_service_method() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:SERVICE-INTERFACE ID="/SOMEIP/TEST/ServiceInterface_TestService">
                <ho:SHORT-NAME>TestService</ho:SHORT-NAME>
                <fx:SERVICE-IDENTIFIER>123</fx:SERVICE-IDENTIFIER>
                <fx:PACKAGE-REF ID-REF="/SOMEIP/TEST"/>
                <service:API-VERSION>
                    <service:MAJOR>1</service:MAJOR>
                    <service:MINOR>2</service:MINOR>
                </service:API-VERSION>
                <service:METHODS>
                    <service:METHOD ID="/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod">
                        <ho:SHORT-NAME>TestMethod</ho:SHORT-NAME>
                        <service:METHOD-IDENTIFIER>100</service:METHOD-IDENTIFIER>
                        <service:INPUT-PARAMETERS>
                            <service:INPUT-PARAMETER ID="/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod/in/Parameter_Input">
                                <ho:SHORT-NAME>Input</ho:SHORT-NAME>
                                <fx:DATATYPE-REF ID-REF="/CommonDatatype_UINT8"/>
                                <fx:UTILIZATION>
                                    <fx:IS-HIGH-LOW-BYTE-ORDER>true</fx:IS-HIGH-LOW-BYTE-ORDER>
                                </fx:UTILIZATION>
                                <service:POSITION>0</service:POSITION>
                            </service:INPUT-PARAMETER>
                        </service:INPUT-PARAMETERS>
                        <service:RETURN-PARAMETERS>
                            <service:RETURN-PARAMETER ID="/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod/ret/Parameter_Output">
                                <ho:SHORT-NAME>Output</ho:SHORT-NAME>
                                <fx:DATATYPE-REF ID-REF="/CommonDatatype_UINT8"/>
                                <fx:UTILIZATION>
                                    <fx:IS-HIGH-LOW-BYTE-ORDER>false</fx:IS-HIGH-LOW-BYTE-ORDER>
                                </fx:UTILIZATION>
                                <service:POSITION>0</service:POSITION>
                            </service:RETURN-PARAMETER>
                        </service:RETURN-PARAMETERS>
                    </service:METHOD>
                </service:METHODS>
            </fx:SERVICE-INTERFACE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_UINT8">
                <ho:SHORT-NAME>UINT8</ho:SHORT-NAME>
            </fx:DATATYPE>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(3, model.types.len());
        assert_eq!(1, model.services.len());

        let service = model.services.get(0).unwrap();
        assert_eq!(1, service.methods.len());

        let method = service.methods.get(0).unwrap();
        assert_eq!(
            method.id,
            "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod"
        );
        assert_eq!(method.name, "testMethod");
        assert_eq!(100, method.method_id);

        // request
        {
            let request = method.request.as_ref().unwrap();
            assert_eq!(
                request.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod/Request"
            );
            assert!(request.name.is_empty());

            let request_type = request.type_ref.as_ref().unwrap().borrow();
            assert_eq!(
                request_type.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod/Request"
            );
            assert!(request_type.name.is_empty());

            let struct_type = match_if!(
                &request_type.datatype,
                FibexDatatype::Struct(datatype),
                datatype
            );

            assert_eq!(1, struct_type.members.len());
            let member = struct_type.members.get(0).unwrap();

            assert_eq!(
                member.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod/in/Parameter_Input"
            );
            assert_eq!(member.name, "input");
            assert_eq!(member.id_ref, "/CommonDatatype_UINT8");

            assert!(member.is_high_low_byte_order());
            assert!(!member.is_array());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_UINT8");
            assert_if!(
                member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Uint8)
            );
        }

        // response
        {
            let response = method.response.as_ref().unwrap();
            assert_eq!(
                response.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod/Response"
            );
            assert!(response.name.is_empty());

            let response_type = response.type_ref.as_ref().unwrap().borrow();
            assert_eq!(
                response_type.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod/Response"
            );
            assert!(response_type.name.is_empty());

            let struct_type = match_if!(
                &response_type.datatype,
                FibexDatatype::Struct(datatype),
                datatype
            );

            assert_eq!(1, struct_type.members.len());
            let member = struct_type.members.get(0).unwrap();

            assert_eq!(
                member.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod/ret/Parameter_Output"
            );
            assert_eq!(member.name, "output");
            assert_eq!(member.id_ref, "/CommonDatatype_UINT8");
            assert!(!member.is_high_low_byte_order());
            assert!(!member.is_array());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_UINT8");
            assert_if!(
                member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Uint8)
            );
        }
    }

    #[test]
    fn test_parse_service_method_array() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:SERVICE-INTERFACE ID="/SOMEIP/TEST/ServiceInterface_TestService">
                <ho:SHORT-NAME>TestService</ho:SHORT-NAME>
                <fx:SERVICE-IDENTIFIER>123</fx:SERVICE-IDENTIFIER>
                <fx:PACKAGE-REF ID-REF="/SOMEIP/TEST"/>
                <service:API-VERSION>
                    <service:MAJOR>1</service:MAJOR>
                    <service:MINOR>2</service:MINOR>
                </service:API-VERSION>
                <service:METHODS>
                    <service:METHOD ID="/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod">
                        <ho:SHORT-NAME>TestMethod</ho:SHORT-NAME>
                        <service:METHOD-IDENTIFIER>100</service:METHOD-IDENTIFIER>
                        <service:INPUT-PARAMETERS>
                            <service:INPUT-PARAMETER ID="/SOMEIP/TEST/ServiceInterface_TestService/Method_TestMethod/in/Parameter_Input">
                                <ho:SHORT-NAME>Input</ho:SHORT-NAME>
                                <fx:DATATYPE-REF ID-REF="/CommonDatatype_UINT8"/>
                                <fx:ARRAY-DECLARATION>
                                    <fx:ARRAY-DIMENSION>
                                    <fx:DIMENSION>1</fx:DIMENSION>
                                    <fx:MINIMUM-SIZE>3</fx:MINIMUM-SIZE>
                                    <fx:MAXIMUM-SIZE>5</fx:MAXIMUM-SIZE>
                                    </fx:ARRAY-DIMENSION>
                                </fx:ARRAY-DECLARATION>
                                <fx:UTILIZATION>
                                    <fx:IS-HIGH-LOW-BYTE-ORDER>true</fx:IS-HIGH-LOW-BYTE-ORDER>
                                    <fx:SERIALIZATION-ATTRIBUTES>
                                        <fx:ARRAY-LENGTH-FIELD-SIZE>8</fx:ARRAY-LENGTH-FIELD-SIZE>
                                    </fx:SERIALIZATION-ATTRIBUTES>
                                </fx:UTILIZATION>
                                <service:POSITION>0</service:POSITION>
                            </service:INPUT-PARAMETER>
                        </service:INPUT-PARAMETERS>
                    </service:METHOD>
                </service:METHODS>
            </fx:SERVICE-INTERFACE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_UINT8">
                <ho:SHORT-NAME>UINT8</ho:SHORT-NAME>
            </fx:DATATYPE>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(1, model.services.len());
        let service = model.services.get(0).unwrap();

        assert_eq!(1, service.methods.len());
        let method = service.methods.get(0).unwrap();

        // request
        {
            let request = method.request.as_ref().unwrap();
            let request_type = request.type_ref.as_ref().unwrap().borrow();

            let struct_type = match_if!(
                &request_type.datatype,
                FibexDatatype::Struct(datatype),
                datatype
            );

            assert_eq!(1, struct_type.members.len());
            let member = struct_type.members.get(0).unwrap();

            assert!(member.is_high_low_byte_order());
            assert!(member.is_array());

            assert_eq!(1, member.num_array_dimensions());
            let array_dimension = member.get_array_dimension(0).unwrap();
            assert_eq!(1, array_dimension.index);
            assert_eq!(3, array_dimension.min);
            assert_eq!(5, array_dimension.max);

            assert_eq!(1, member.get_array_length_field_size());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_if!(
                member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Uint8)
            );
        }
    }

    #[test]
    fn test_parse_service_event() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:SERVICE-INTERFACE ID="/SOMEIP/TEST/ServiceInterface_TestService">
                <ho:SHORT-NAME>TestService</ho:SHORT-NAME>
                <fx:SERVICE-IDENTIFIER>123</fx:SERVICE-IDENTIFIER>
                <fx:PACKAGE-REF ID-REF="/SOMEIP/TEST"/>
                <service:API-VERSION>
                    <service:MAJOR>1</service:MAJOR>
                    <service:MINOR>2</service:MINOR>
                </service:API-VERSION>
                <service:EVENTS>
                    <service:EVENT ID="/SOMEIP/TEST/ServiceInterface_TestService/Method_TestEvent">
                        <ho:SHORT-NAME>TestEvent</ho:SHORT-NAME>
                        <service:METHOD-IDENTIFIER>32768</service:METHOD-IDENTIFIER>
                        <service:CALL-SEMANTIC>FIRE_AND_FORGET</service:CALL-SEMANTIC>
                        <service:INPUT-PARAMETERS>
                            <service:INPUT-PARAMETER ID="/SOMEIP/TEST/ServiceInterface_TestService/Method_TestEvent/in/Parameter_Value2">
                                <ho:SHORT-NAME>Value2</ho:SHORT-NAME>
                                <fx:DATATYPE-REF ID-REF="/CommonDatatype_UINT8"/>
                                <fx:UTILIZATION>
                                    <fx:IS-HIGH-LOW-BYTE-ORDER>false</fx:IS-HIGH-LOW-BYTE-ORDER>
                                </fx:UTILIZATION>
                                <service:POSITION>1</service:POSITION>
                            </service:INPUT-PARAMETER>
                            <service:INPUT-PARAMETER ID="/SOMEIP/TEST/ServiceInterface_TestService/Method_TestEvent/in/Parameter_Value1">
                                <ho:SHORT-NAME>Value1</ho:SHORT-NAME>
                                <fx:DATATYPE-REF ID-REF="/CommonDatatype_UINT8"/>
                                <fx:UTILIZATION>
                                    <fx:IS-HIGH-LOW-BYTE-ORDER>false</fx:IS-HIGH-LOW-BYTE-ORDER>
                                </fx:UTILIZATION>
                                <service:POSITION>0</service:POSITION>
                            </service:INPUT-PARAMETER>
                        </service:INPUT-PARAMETERS>
                    </service:EVENT>
                </service:EVENTS>
            </fx:SERVICE-INTERFACE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_UINT8">
                <ho:SHORT-NAME>UINT8</ho:SHORT-NAME>
            </fx:DATATYPE>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(2, model.types.len());
        assert_eq!(1, model.services.len());

        let service = model.services.get(0).unwrap();
        assert_eq!(1, service.methods.len());

        let method = service.methods.get(0).unwrap();
        assert_eq!(
            method.id,
            "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestEvent"
        );
        assert_eq!(method.name, "testEvent");
        assert_eq!(32768, method.method_id);

        // request
        {
            let request = method.request.as_ref().unwrap();
            assert_eq!(
                request.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestEvent/Request"
            );
            assert!(request.name.is_empty());

            let request_type = request.type_ref.as_ref().unwrap().borrow();
            assert_eq!(
                request_type.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestEvent/Request"
            );
            assert!(request_type.name.is_empty());

            let struct_type = match_if!(
                &request_type.datatype,
                FibexDatatype::Struct(datatype),
                datatype
            );

            assert_eq!(2, struct_type.members.len());

            let member = struct_type.members.get(0).unwrap();
            assert_eq!(
                member.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestEvent/in/Parameter_Value1"
            );
            assert_eq!(member.name, "value1");
            assert_eq!(member.id_ref, "/CommonDatatype_UINT8");
            assert!(!member.is_high_low_byte_order());
            assert!(!member.is_array());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_UINT8");
            assert_if!(
                member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Uint8)
            );

            let member = struct_type.members.get(1).unwrap();
            assert_eq!(
                member.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Method_TestEvent/in/Parameter_Value2"
            );
            assert_eq!(member.name, "value2");
            assert_eq!(member.id_ref, "/CommonDatatype_UINT8");
            assert!(!member.is_high_low_byte_order());
            assert!(!member.is_array());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_UINT8");
            assert_if!(
                member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Uint8)
            );
        }

        assert!(method.response.is_none());
    }

    #[test]
    fn test_parse_service_field() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:SERVICE-INTERFACE ID="/SOMEIP/TEST/ServiceInterface_TestService">
                <ho:SHORT-NAME>TestService</ho:SHORT-NAME>
                <fx:SERVICE-IDENTIFIER>123</fx:SERVICE-IDENTIFIER>
                <fx:PACKAGE-REF ID-REF="/SOMEIP/TEST"/>
                <service:API-VERSION>
                    <service:MAJOR>1</service:MAJOR>
                    <service:MINOR>2</service:MINOR>
                </service:API-VERSION>
                <service:FIELDS>
                    <service:FIELD ID="/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField">
                        <ho:SHORT-NAME>TestField</ho:SHORT-NAME>
                        <fx:DATATYPE-REF ID-REF="/CommonDatatype_UINT8"/>
                        <fx:UTILIZATION>
                            <fx:IS-HIGH-LOW-BYTE-ORDER>false</fx:IS-HIGH-LOW-BYTE-ORDER>
                        </fx:UTILIZATION>
                        <service:ACCESS-PERMISSION>NOTIFY_READ_WRITE</service:ACCESS-PERMISSION>
                        <service:GETTER>
                            <service:METHOD-IDENTIFIER>10</service:METHOD-IDENTIFIER>
                        </service:GETTER>
                        <service:SETTER>
                            <service:METHOD-IDENTIFIER>11</service:METHOD-IDENTIFIER>
                        </service:SETTER>
                        <service:NOTIFIER>
                            <service:NOTIFICATION-IDENTIFIER>32770</service:NOTIFICATION-IDENTIFIER>
                        </service:NOTIFIER>
                    </service:FIELD>
                </service:FIELDS>
            </fx:SERVICE-INTERFACE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_UINT8">
                <ho:SHORT-NAME>UINT8</ho:SHORT-NAME>
            </fx:DATATYPE>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(2, model.types.len());
        assert_eq!(1, model.services.len());

        let service = model.services.get(0).unwrap();
        assert_eq!(3, service.methods.len());

        // getter
        {
            let method = service.methods.get(0).unwrap();
            assert_eq!(
                method.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField/Getter"
            );
            assert_eq!(method.name, "getTestField");
            assert_eq!(10, method.method_id);

            assert!(method.request.is_none());

            let response = method.response.as_ref().unwrap();
            assert_eq!(
                response.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField/Getter"
            );
            assert!(response.name.is_empty());

            let response_type = response.type_ref.as_ref().unwrap().borrow();
            assert_test_field(&response_type);
        }

        // setter
        {
            let method = service.methods.get(1).unwrap();
            assert_eq!(
                method.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField/Setter"
            );
            assert_eq!(method.name, "setTestField");
            assert_eq!(11, method.method_id);

            let request = method.request.as_ref().unwrap();
            assert_eq!(
                request.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField/Setter"
            );
            assert!(request.name.is_empty());

            let request_type = request.type_ref.as_ref().unwrap().borrow();
            assert_test_field(&request_type);

            let response = method.response.as_ref().unwrap();
            assert_eq!(
                response.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField/Setter"
            );
            assert!(response.name.is_empty());

            let response_type = response.type_ref.as_ref().unwrap().borrow();
            assert_test_field(&response_type);
        }

        // notifier
        {
            let method = service.methods.get(2).unwrap();
            assert_eq!(
                method.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField/Notifier"
            );
            assert_eq!(method.name, "testField");
            assert_eq!(32770, method.method_id);

            let request = method.request.as_ref().unwrap();
            assert_eq!(
                request.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField/Notifier"
            );
            assert!(request.name.is_empty());

            let request_type = request.type_ref.as_ref().unwrap().borrow();
            assert_test_field(&request_type);

            assert!(method.response.is_none());
        }

        fn assert_test_field(field_type: &FibexTypeInstance) {
            assert_eq!(
                field_type.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField"
            );
            assert!(field_type.name.is_empty());

            let struct_type = match_if!(
                &field_type.datatype,
                FibexDatatype::Struct(datatype),
                datatype
            );

            assert_eq!(1, struct_type.members.len());
            let member = struct_type.members.get(0).unwrap();

            assert_eq!(
                member.id,
                "/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField"
            );
            assert_eq!(member.name, "testField");
            assert_eq!(member.id_ref, "/CommonDatatype_UINT8");
            assert!(!member.is_high_low_byte_order());
            assert!(!member.is_array());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_UINT8");
            assert_if!(
                member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Uint8)
            );
        }
    }

    #[test]
    fn test_parse_service_field_array() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:SERVICE-INTERFACE ID="/SOMEIP/TEST/ServiceInterface_TestService">
                <ho:SHORT-NAME>TestService</ho:SHORT-NAME>
                <fx:SERVICE-IDENTIFIER>123</fx:SERVICE-IDENTIFIER>
                <fx:PACKAGE-REF ID-REF="/SOMEIP/TEST"/>
                <service:API-VERSION>
                    <service:MAJOR>1</service:MAJOR>
                    <service:MINOR>2</service:MINOR>
                </service:API-VERSION>
                <service:FIELDS>
                    <service:FIELD ID="/SOMEIP/TEST/ServiceInterface_TestService/Field_TestField">
                        <ho:SHORT-NAME>TestField</ho:SHORT-NAME>
                        <fx:DATATYPE-REF ID-REF="/CommonDatatype_UINT8"/>
                        <fx:ARRAY-DECLARATION>
                            <fx:ARRAY-DIMENSION>
                            <fx:DIMENSION>1</fx:DIMENSION>
                            <fx:MAXIMUM-SIZE>5</fx:MAXIMUM-SIZE>
                            </fx:ARRAY-DIMENSION>
                        </fx:ARRAY-DECLARATION>
                        <fx:UTILIZATION>
                            <fx:IS-HIGH-LOW-BYTE-ORDER>false</fx:IS-HIGH-LOW-BYTE-ORDER>
                        </fx:UTILIZATION>
                        <service:ACCESS-PERMISSION>NOTIFY_READ_WRITE</service:ACCESS-PERMISSION>
                        <service:GETTER>
                            <service:METHOD-IDENTIFIER>10</service:METHOD-IDENTIFIER>
                        </service:GETTER>
                        <service:SETTER>
                            <service:METHOD-IDENTIFIER>11</service:METHOD-IDENTIFIER>
                        </service:SETTER>
                        <service:NOTIFIER>
                            <service:NOTIFICATION-IDENTIFIER>32770</service:NOTIFICATION-IDENTIFIER>
                        </service:NOTIFIER>
                    </service:FIELD>
                </service:FIELDS>
            </fx:SERVICE-INTERFACE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_UINT8">
                <ho:SHORT-NAME>UINT8</ho:SHORT-NAME>
            </fx:DATATYPE>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(1, model.services.len());
        let service = model.services.get(0).unwrap();

        assert_eq!(3, service.methods.len());

        // getter
        {
            let method = service.methods.get(0).unwrap();

            assert!(method.request.is_none());

            let response = method.response.as_ref().unwrap();
            let response_type = response.type_ref.as_ref().unwrap().borrow();
            assert_test_field_array(&response_type);
        }

        // setter
        {
            let method = service.methods.get(1).unwrap();

            let request = method.request.as_ref().unwrap();
            let request_type = request.type_ref.as_ref().unwrap().borrow();
            assert_test_field_array(&request_type);

            let response = method.response.as_ref().unwrap();
            let response_type = response.type_ref.as_ref().unwrap().borrow();
            assert_test_field_array(&response_type);
        }

        // notifier
        {
            let method = service.methods.get(2).unwrap();

            let request = method.request.as_ref().unwrap();
            let request_type = request.type_ref.as_ref().unwrap().borrow();
            assert_test_field_array(&request_type);

            assert!(method.response.is_none());
        }

        fn assert_test_field_array(payload: &FibexTypeInstance) {
            let struct_type =
                match_if!(&payload.datatype, FibexDatatype::Struct(datatype), datatype);

            assert_eq!(1, struct_type.members.len());
            let member = struct_type.members.get(0).unwrap();

            assert!(!member.is_high_low_byte_order());
            assert!(member.is_array());

            assert_eq!(1, member.num_array_dimensions());
            let array_dimension = member.get_array_dimension(0).unwrap();
            assert_eq!(1, array_dimension.index);
            assert_eq!(0, array_dimension.min);
            assert_eq!(5, array_dimension.max);

            assert_eq!(4, member.get_array_length_field_size());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_if!(
                member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Uint8)
            );
        }
    }

    #[test]
    fn test_parse_primitives() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_BOOL">
                <ho:SHORT-NAME>BOOL</ho:SHORT-NAME>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_UINT8">
                <ho:SHORT-NAME>UINT8</ho:SHORT-NAME>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_FLOAT32">
                <ho:SHORT-NAME>FLOAT32</ho:SHORT-NAME>
            </fx:DATATYPE>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(3, model.types.len());

        let common_type = model.types.get(0).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_BOOL");
        assert_eq!(common_type.name, "BOOL");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Bool)
        );

        let common_type = model.types.get(1).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_UINT8");
        assert_eq!(common_type.name, "UINT8");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Uint8)
        );

        let common_type = model.types.get(2).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_FLOAT32");
        assert_eq!(common_type.name, "FLOAT32");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Float32)
        );
    }

    #[test]
    fn test_parse_coded_primitives() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_AUint16">
                <ho:SHORT-NAME>AUint16</ho:SHORT-NAME>
                <fx:CODING-REF ID-REF="/Coding_UINT16"/>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_AUint32">
                <ho:SHORT-NAME>AUint32</ho:SHORT-NAME>
                <fx:CODING-REF ID-REF="/Coding_UINT32"/>
            </fx:DATATYPE>
            <fx:PROCESSING-INFORMATION>
                <fx:CODINGS>
                    <fx:CODING ID="/Coding_UINT16">
                        <ho:SHORT-NAME>UINT16</ho:SHORT-NAME>
                    </fx:CODING>
                    <fx:CODING ID="/Coding_UINT32">
                        <ho:SHORT-NAME>AUint32Coding</ho:SHORT-NAME>
                        <ho:CODED-TYPE ho:BASE-DATA-TYPE="A_UINT32" CATEGORY="STANDARD-LENGTH-TYPE">
                            <ho:BIT-LENGTH>32</ho:BIT-LENGTH>
                        </ho:CODED-TYPE>
                    </fx:CODING>
                </fx:CODINGS>
            </fx:PROCESSING-INFORMATION>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(2, model.types.len());
        assert_eq!(2, model.codings.len());

        let common_type = model.types.get(0).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_AUint16");
        assert_eq!(common_type.name, "AUint16");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Uint16)
        );

        let common_type = model.types.get(1).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_AUint32");
        assert_eq!(common_type.name, "AUint32");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Uint32)
        );
    }

    #[test]
    fn test_parse_struct_of_primitives() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_UINT8">
                <ho:SHORT-NAME>UINT8</ho:SHORT-NAME>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_UINT16">
                <ho:SHORT-NAME>UINT16</ho:SHORT-NAME>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMPLEX-DATATYPE-TYPE" ID="/ComplexDatatype_Struct">
                <ho:SHORT-NAME>Struct</ho:SHORT-NAME>
                <fx:COMPLEX-DATATYPE-CLASS>STRUCTURE</fx:COMPLEX-DATATYPE-CLASS>
                <fx:MEMBERS>
                    <fx:MEMBER ID="/ComplexDatatype_Struct/ComplexDatatypeMember_Member2">
                    <ho:SHORT-NAME>Member2</ho:SHORT-NAME>
                    <fx:DATATYPE-REF ID-REF="/CommonDatatype_UINT16"/>
                    <fx:UTILIZATION>
                        <fx:IS-HIGH-LOW-BYTE-ORDER>false</fx:IS-HIGH-LOW-BYTE-ORDER>
                    </fx:UTILIZATION>
                    <fx:POSITION>1</fx:POSITION>
                    </fx:MEMBER>
                    <fx:MEMBER ID="/ComplexDatatype_Struct/ComplexDatatypeMember_Member1">
                    <ho:SHORT-NAME>Member1</ho:SHORT-NAME>
                    <fx:DATATYPE-REF ID-REF="/CommonDatatype_UINT8"/>
                    <fx:UTILIZATION>
                        <fx:IS-HIGH-LOW-BYTE-ORDER>true</fx:IS-HIGH-LOW-BYTE-ORDER>
                    </fx:UTILIZATION>
                    <fx:POSITION>0</fx:POSITION>
                    </fx:MEMBER>
                </fx:MEMBERS>
            </fx:DATATYPE>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(3, model.types.len());

        let common_type = model.types.get(0).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_UINT8");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Uint8)
        );

        let common_type = model.types.get(1).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_UINT16");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Uint16)
        );

        let complex_type = model.types.get(2).unwrap().borrow();
        assert_eq!(complex_type.id, "/ComplexDatatype_Struct");
        assert_eq!(complex_type.name, "Struct");

        let struct_type = match_if!(
            &complex_type.datatype,
            FibexDatatype::Struct(datatype),
            datatype
        );
        assert_eq!(2, struct_type.members.len());

        // member 1
        {
            let member = struct_type.members.get(0).unwrap();
            assert_eq!(
                member.id,
                "/ComplexDatatype_Struct/ComplexDatatypeMember_Member1"
            );
            assert_eq!(member.name, "member1");
            assert_eq!(member.id_ref, "/CommonDatatype_UINT8");

            assert!(member.is_high_low_byte_order());
            assert!(!member.is_array());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_UINT8");
            assert_if!(
                &member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Uint8)
            );
        }

        // member 2
        {
            let member = struct_type.members.get(1).unwrap();
            assert_eq!(
                member.id,
                "/ComplexDatatype_Struct/ComplexDatatypeMember_Member2"
            );
            assert_eq!(member.name, "member2");
            assert_eq!(member.id_ref, "/CommonDatatype_UINT16");
            assert!(!member.is_high_low_byte_order());
            assert!(!member.is_array());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_UINT16");
            assert_if!(
                &member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Uint16)
            );
        }
    }

    #[test]
    fn test_parse_struct_of_structs_with_multidim_array() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_BOOL">
                <ho:SHORT-NAME>BOOL</ho:SHORT-NAME>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_INT16">
                <ho:SHORT-NAME>INT16</ho:SHORT-NAME>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_INT32">
                <ho:SHORT-NAME>INT32</ho:SHORT-NAME>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMPLEX-DATATYPE-TYPE" ID="/ComplexDatatype_Struct">
                <ho:SHORT-NAME>Struct</ho:SHORT-NAME>
                <fx:COMPLEX-DATATYPE-CLASS>STRUCTURE</fx:COMPLEX-DATATYPE-CLASS>
                <fx:MEMBERS>
                    <fx:MEMBER ID="/ComplexDatatype_Struct/ComplexDatatypeMember_Member1">
                    <ho:SHORT-NAME>Member1</ho:SHORT-NAME>
                    <fx:DATATYPE-REF ID-REF="/CommonDatatype_BOOL"/>
                    <fx:POSITION>0</fx:POSITION>
                    </fx:MEMBER>
                    <fx:MEMBER ID="/ComplexDatatype_Struct/ComplexDatatypeMember_Member2">
                    <ho:SHORT-NAME>Member2</ho:SHORT-NAME>
                    <fx:DATATYPE-REF ID-REF="/ComplexDatatype_SubStruct"/>
                    <fx:POSITION>1</fx:POSITION>
                    </fx:MEMBER>
                </fx:MEMBERS>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMPLEX-DATATYPE-TYPE" ID="/ComplexDatatype_SubStruct">
                <ho:SHORT-NAME>SubStruct</ho:SHORT-NAME>
                <fx:COMPLEX-DATATYPE-CLASS>STRUCTURE</fx:COMPLEX-DATATYPE-CLASS>
                <fx:MEMBERS>
                    <fx:MEMBER ID="/ComplexDatatype_SubStruct/ComplexDatatypeMember_Member1">
                    <ho:SHORT-NAME>Member1</ho:SHORT-NAME>
                    <fx:DATATYPE-REF ID-REF="/CommonDatatype_INT16"/>
                    <fx:UTILIZATION>
                        <fx:IS-HIGH-LOW-BYTE-ORDER>true</fx:IS-HIGH-LOW-BYTE-ORDER>
                    </fx:UTILIZATION>
                    <fx:POSITION>0</fx:POSITION>
                    </fx:MEMBER>
                    <fx:MEMBER ID="/ComplexDatatype_SubStruct/ComplexDatatypeMember_Member2">
                    <ho:SHORT-NAME>Member2</ho:SHORT-NAME>
                    <fx:DATATYPE-REF ID-REF="/CommonDatatype_INT32"/>
                    <fx:ARRAY-DECLARATION>
                        <fx:ARRAY-DIMENSION>
                            <fx:DIMENSION>1</fx:DIMENSION>
                            <fx:MAXIMUM-SIZE>3</fx:MAXIMUM-SIZE>
                        </fx:ARRAY-DIMENSION>
                        <fx:ARRAY-DIMENSION>
                            <fx:DIMENSION>2</fx:DIMENSION>
                            <fx:MINIMUM-SIZE>1</fx:MINIMUM-SIZE>
                            <fx:MAXIMUM-SIZE>5</fx:MAXIMUM-SIZE>
                        </fx:ARRAY-DIMENSION>
                    </fx:ARRAY-DECLARATION>
                    <fx:UTILIZATION>
                        <fx:IS-HIGH-LOW-BYTE-ORDER>false</fx:IS-HIGH-LOW-BYTE-ORDER>
                    </fx:UTILIZATION>
                    <fx:POSITION>1</fx:POSITION>
                    </fx:MEMBER>
                </fx:MEMBERS>
            </fx:DATATYPE>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(5, model.types.len());

        let common_type = model.types.get(0).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_BOOL");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Bool)
        );

        let common_type = model.types.get(1).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_INT16");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Int16)
        );

        let common_type = model.types.get(2).unwrap().borrow();
        assert_eq!(common_type.id, "/CommonDatatype_INT32");
        assert_if!(
            &common_type.datatype,
            FibexDatatype::Primitive(FibexPrimitive::Int32)
        );

        let complex_type = model.types.get(3).unwrap().borrow();
        assert_eq!(complex_type.id, "/ComplexDatatype_Struct");
        assert_eq!(complex_type.name, "Struct");

        let struct_type = match_if!(
            &complex_type.datatype,
            FibexDatatype::Struct(datatype),
            datatype
        );
        assert_eq!(2, struct_type.members.len());

        // member 1
        {
            let member = struct_type.members.get(0).unwrap();
            assert_eq!(
                member.id,
                "/ComplexDatatype_Struct/ComplexDatatypeMember_Member1"
            );
            assert_eq!(member.name, "member1");
            assert_eq!(member.id_ref, "/CommonDatatype_BOOL");
            assert!(!member.is_array());

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_BOOL");
            assert_if!(
                &member_type.datatype,
                FibexDatatype::Primitive(FibexPrimitive::Bool)
            );
        }

        // member 2
        {
            let member = struct_type.members.get(1).unwrap();
            assert_eq!(
                member.id,
                "/ComplexDatatype_Struct/ComplexDatatypeMember_Member2"
            );
            assert_eq!(member.id_ref, "/ComplexDatatype_SubStruct");
            assert!(!member.is_array());

            let member_type = member.type_ref.as_ref().unwrap().borrow();

            let struct_type = match_if!(
                &member_type.datatype,
                FibexDatatype::Struct(datatype),
                datatype
            );
            assert_eq!(2, struct_type.members.len());

            // sub member 1
            {
                let member = struct_type.members.get(0).unwrap();
                assert_eq!(
                    member.id,
                    "/ComplexDatatype_SubStruct/ComplexDatatypeMember_Member1"
                );
                assert_eq!(member.name, "member1");
                assert_eq!(member.id_ref, "/CommonDatatype_INT16");

                assert!(member.is_high_low_byte_order());
                assert!(!member.is_array());

                let member_type = member.type_ref.as_ref().unwrap().borrow();
                assert_eq!(member_type.id, "/CommonDatatype_INT16");
                assert_if!(
                    &member_type.datatype,
                    FibexDatatype::Primitive(FibexPrimitive::Int16)
                );
            }

            // sub member 2
            {
                let member = struct_type.members.get(1).unwrap();
                assert_eq!(
                    member.id,
                    "/ComplexDatatype_SubStruct/ComplexDatatypeMember_Member2"
                );
                assert_eq!(member.name, "member2");
                assert_eq!(member.id_ref, "/CommonDatatype_INT32");
                assert!(!member.is_high_low_byte_order());
                assert!(member.is_array());

                assert_eq!(2, member.num_array_dimensions());
                let array_dimension = member.get_array_dimension(0).unwrap();
                assert_eq!(1, array_dimension.index);
                assert_eq!(0, array_dimension.min);
                assert_eq!(3, array_dimension.max);

                let array_dimension = member.get_array_dimension(1).unwrap();
                assert_eq!(2, array_dimension.index);
                assert_eq!(1, array_dimension.min);
                assert_eq!(5, array_dimension.max);

                let member_type = member.type_ref.as_ref().unwrap().borrow();
                assert_eq!(member_type.id, "/CommonDatatype_INT32");
                assert_if!(
                    &member_type.datatype,
                    FibexDatatype::Primitive(FibexPrimitive::Int32)
                );
            }
        }
    }

    #[test]
    fn test_parse_enum() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:DATATYPE xsi:type="fx:ENUM-DATATYPE-TYPE" ID="/SOMEIP/TEST/EnumDatatype_AEnum">
                <ho:SHORT-NAME>AEnum</ho:SHORT-NAME>
                <fx:PACKAGE-REF ID-REF="/SOMEIP/TEST"/>
                <fx:CODING-REF ID-REF="/Coding_UINT16"/>
                <fx:ENUMERATION-ELEMENTS>
                    <fx:ENUM-ELEMENT>
                        <fx:VALUE>1</fx:VALUE>
                        <fx:SYNONYM>A</fx:SYNONYM>
                    </fx:ENUM-ELEMENT>
                    <fx:ENUM-ELEMENT>
                        <fx:VALUE>2</fx:VALUE>
                        <fx:SYNONYM>B</fx:SYNONYM>
                    </fx:ENUM-ELEMENT>
                    <fx:ENUM-ELEMENT>
                        <fx:VALUE>3</fx:VALUE>
                        <fx:SYNONYM>C</fx:SYNONYM>
                    </fx:ENUM-ELEMENT>
                </fx:ENUMERATION-ELEMENTS>
            </fx:DATATYPE>
            <fx:PROCESSING-INFORMATION>
                <fx:CODINGS>
                    <fx:CODING ID="/Coding_UINT16">
                        <ho:SHORT-NAME>UINT16</ho:SHORT-NAME>
                    </fx:CODING>
                </fx:CODINGS>
            </fx:PROCESSING-INFORMATION>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(1, model.types.len());
        assert_eq!(1, model.codings.len());

        let complex_type = model.types.get(0).unwrap().borrow();
        assert_eq!(complex_type.id, "/SOMEIP/TEST/EnumDatatype_AEnum");
        assert_eq!(complex_type.name, "AEnum");

        let enum_type = match_if!(
            &complex_type.datatype,
            FibexDatatype::Enum(datatype),
            datatype
        );
        assert_eq!(3, enum_type.variants.len());

        // primitive
        {
            let primitive = &enum_type.primitive;

            assert_if!(&primitive, FibexPrimitive::Uint16);
        }

        // variants
        {
            let enum_variant = enum_type.variants.get(0).unwrap();
            assert_eq!(enum_variant.name, "A");
            assert_eq!(enum_variant.value, "1");

            let enum_variant = enum_type.variants.get(1).unwrap();
            assert_eq!(enum_variant.name, "B");
            assert_eq!(enum_variant.value, "2");

            let enum_variant = enum_type.variants.get(2).unwrap();
            assert_eq!(enum_variant.name, "C");
            assert_eq!(enum_variant.value, "3");
        }
    }

    #[test]
    fn test_parse_struct_of_strings() {
        use stringreader::StringReader;

        let xml = r#"
            <fx:DATATYPE xsi:type="fx:COMPLEX-DATATYPE-TYPE" ID="/ComplexDatatype_Struct">
                <ho:SHORT-NAME>Struct</ho:SHORT-NAME>
                <fx:COMPLEX-DATATYPE-CLASS>STRUCTURE</fx:COMPLEX-DATATYPE-CLASS>
                <fx:MEMBERS>
                    <fx:MEMBER ID="/ComplexDatatype_Struct/ComplexDatatypeMember_Member1">
                    <ho:SHORT-NAME>Member1</ho:SHORT-NAME>
                    <fx:DATATYPE-REF ID-REF="/CommonDatatype_STRINGUTF8FIXED"/>
                    <fx:UTILIZATION>
                        <fx:BIT-LENGTH>40</fx:BIT-LENGTH>
                        <fx:IS-HIGH-LOW-BYTE-ORDER>true</fx:IS-HIGH-LOW-BYTE-ORDER>
                    </fx:UTILIZATION>
                    <fx:POSITION>0</fx:POSITION>
                    </fx:MEMBER>
                    <fx:MEMBER ID="/ComplexDatatype_Struct/ComplexDatatypeMember_Member2">
                    <ho:SHORT-NAME>Member2</ho:SHORT-NAME>
                    <fx:DATATYPE-REF ID-REF="/CommonDatatype_STRINGUTF16DYNAMIC"/>
                    <fx:UTILIZATION>
                        <fx:MAX-BIT-LENGTH>80</fx:MAX-BIT-LENGTH>
                        <fx:IS-HIGH-LOW-BYTE-ORDER>false</fx:IS-HIGH-LOW-BYTE-ORDER>
                        <fx:SERIALIZATION-ATTRIBUTES>
                            <fx:LENGTH-FIELD-SIZE>8</fx:LENGTH-FIELD-SIZE>
                        </fx:SERIALIZATION-ATTRIBUTES>
                    </fx:UTILIZATION>
                    <fx:POSITION>1</fx:POSITION>
                    </fx:MEMBER>
                </fx:MEMBERS>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_STRINGUTF8FIXED">
                <ho:SHORT-NAME>STRINGUTF8FIXED</ho:SHORT-NAME>
                <fx:CODING-REF ID-REF="/Coding_STRINGUTF8FIXED"/>
            </fx:DATATYPE>
            <fx:DATATYPE xsi:type="fx:COMMON-DATATYPE-TYPE" ID="/CommonDatatype_STRINGUTF16DYNAMIC">
                <ho:SHORT-NAME>STRINGUTF16DYNAMIC</ho:SHORT-NAME>
                <fx:CODING-REF ID-REF="/Coding_STRINGUTF16DYNAMIC"/>
            </fx:DATATYPE>
            <fx:PROCESSING-INFORMATION>
                <fx:CODINGS>
                    <fx:CODING ID="/Coding_STRINGUTF8FIXED">
                        <ho:SHORT-NAME>STRINGUTF8FIXED</ho:SHORT-NAME>
                        <ho:CODED-TYPE 
                                ho:BASE-DATA-TYPE="A_UNICODE2STRING" 
                                CATEGORY="STANDARD-LENGTH-TYPE" 
                                ENCODING="UTF-8" 
                                BYTE-ORDER-MARK="NONE"
                                TERMINATION="NONE"
                        >
                        </ho:CODED-TYPE>
                    </fx:CODING>
                    <fx:CODING ID="/Coding_STRINGUTF16DYNAMIC">
                        <ho:SHORT-NAME>STRINGUTF16DYNAMIC</ho:SHORT-NAME>
                        <ho:CODED-TYPE 
                            ho:BASE-DATA-TYPE="A_UNICODE2STRING" 
                            CATEGORY="LEADING-LENGTH-INFO-TYPE" 
                            ENCODING="UTF-16" 
                            BYTE-ORDER-MARK="EXPLICIT"
                            TERMINATION="ZERO"
                        >
                            <ho:MIN-LENGTH>2</ho:MIN-LENGTH>
                        </ho:CODED-TYPE>
                    </fx:CODING>
                </fx:CODINGS>
            </fx:PROCESSING-INFORMATION>
        "#;

        let reader = FibexReader::from_reader(BufReader::new(StringReader::new(xml))).unwrap();
        let model = FibexParser::parse(reader).expect("parse failed");

        assert_eq!(3, model.types.len());
        assert_eq!(2, model.codings.len());

        let complex_type = model.types.get(0).unwrap().borrow();

        let struct_type = match_if!(
            &complex_type.datatype,
            FibexDatatype::Struct(datatype),
            datatype
        );
        assert_eq!(2, struct_type.members.len());

        // member 1
        {
            let member = struct_type.members.get(0).unwrap();
            assert_eq!(member.id_ref, "/CommonDatatype_STRINGUTF8FIXED");

            assert!(member.is_high_low_byte_order());
            assert_eq!(member.get_bit_length().unwrap(), 40);

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_STRINGUTF8FIXED");

            let string_type = match_if!(
                &member_type.datatype,
                FibexDatatype::String(datatype),
                datatype
            );

            assert!(!string_type.is_dynamic);
            assert_eq!(string_type.encoding, FibexStringEncoding::UTF8);
            assert!(!string_type.has_bom);
            assert!(!string_type.has_termination);
        }

        // member 2
        {
            let member = struct_type.members.get(1).unwrap();
            assert_eq!(member.id_ref, "/CommonDatatype_STRINGUTF16DYNAMIC");

            assert!(!member.is_high_low_byte_order());
            assert_eq!(member.get_length_field_size(), 1);
            assert_eq!(member.get_max_bit_length().unwrap(), 80);

            let member_type = member.type_ref.as_ref().unwrap().borrow();
            assert_eq!(member_type.id, "/CommonDatatype_STRINGUTF16DYNAMIC");

            let string_type = match_if!(
                &member_type.datatype,
                FibexDatatype::String(datatype),
                datatype
            );

            assert!(string_type.is_dynamic);
            assert_eq!(string_type.encoding, FibexStringEncoding::UTF16);
            assert!(string_type.has_bom);
            assert!(string_type.has_termination);
            assert_eq!(string_type.min_length.unwrap(), 2);
        }
    }

    #[test]
    fn test_parse_fibex_file() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fibex-model.xml");

        let reader = FibexReader::from_file(file).unwrap();
        let model = FibexParser::try_parse(reader).expect("parse failed");

        assert_eq!(1, model.services.len());
        assert_eq!(17, model.services.get(0).unwrap().methods.len());
        assert_eq!(33, model.types.len());
        assert_eq!(3, model.codings.len());

        let mut num_primitives = 0;
        let mut num_structs = 0;
        let mut num_enums = 0;
        let mut num_strings = 0;

        for item in model.types {
            if let FibexDatatype::Primitive(_) = item.borrow().datatype {
                num_primitives += 1;
            }
            if let FibexDatatype::Struct(_) = item.borrow().datatype {
                num_structs += 1;
            }
            if let FibexDatatype::Enum(_) = item.borrow().datatype {
                num_enums += 1;
            }
            if let FibexDatatype::String(_) = item.borrow().datatype {
                num_strings += 1;
            }
        }

        assert_eq!(8, num_primitives);
        assert_eq!(22, num_structs);
        assert_eq!(1, num_enums);
        assert_eq!(2, num_strings);
    }
}
