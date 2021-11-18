// someip output

use crate::som::*;
use std::any::Any;
use std::fmt::{Display, Formatter, Result as FmtResult};

const TAB_SPACES: &str = "    ";

mod primitives {
    use super::*;
    use crate::som::primitives::*;

    impl<T: Display + Copy + PartialEq> Display for SOMPrimitiveType<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let mut d = match self.meta() {
                Some(meta) => meta.to_str(),
                None => std::any::type_name::<T>().to_string(),
            };

            if let Some(value) = self.get() {
                if d.is_empty() {
                    d = format!("{}", value);
                } else {
                    d = format!("{} : {}", d, value);
                }
            } else if d.is_empty() {
                d = std::any::type_name::<T>().to_string();
            }

            write!(f, "{}", d)
        }
    }

    impl<T: Display + Copy + PartialEq> Display for SOMPrimitiveTypeWithEndian<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            write!(f, "{}", self.primitive())
        }
    }
}

mod arrays {
    use super::*;
    use crate::som::arrays::*;

    impl<T: Display + SOMType + Any + Clone> Display for SOMArrayType<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let mut d = match self.meta() {
                Some(meta) => meta.to_str(),
                None => String::from("array"),
            };

            let mut dd = String::from("");
            for i in 0..self.len() {
                if let Some(element) = self.get(i) {
                    let ddd = format!("{}", element);
                    dd = format!("{}\n\t{},", dd, ddd.replace("\n", "\n\t"));
                }
            }

            if !dd.is_empty() {
                if d.is_empty() {
                    d = format!("[{}\n]", dd);
                } else {
                    d = format!("{} [{}\n]", d, dd);
                }
            } else if d.is_empty() {
                d = String::from("[]");
            }

            write!(f, "{}", d.replace("\t", TAB_SPACES))
        }
    }
}

mod structs {
    use super::*;
    use crate::som::structs::*;

    impl<T: Display + SOMType> Display for SOMStructType<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let mut d = match self.meta() {
                Some(meta) => meta.to_str(),
                None => String::from("struct"),
            };

            let mut dd = String::from("");
            for i in 0..self.len() {
                if let Some(member) = self.get(i) {
                    let ddd = format!("{}", member);
                    dd = format!("{}\n\t{},", dd, ddd.replace("\n", "\n\t"));
                }
            }

            if !dd.is_empty() {
                if d.is_empty() {
                    d = format!("{{{}\n}}", dd);
                } else {
                    d = format!("{} {{{}\n}}", d, dd);
                }
            } else if d.is_empty() {
                d = String::from("{}");
            }

            write!(f, "{}", d.replace("\t", TAB_SPACES))
        }
    }
}

mod unions {
    use super::*;
    use crate::som::unions::*;

    impl<T: Display + SOMType + Any> Display for SOMUnionType<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let mut d = match self.meta() {
                Some(meta) => meta.to_str(),
                None => String::from("union"),
            };

            let mut dd = String::from("");
            if self.len() > 0 {
                if let Some(value) = &self.get() {
                    let ddd = format!("{}", value);
                    dd = format!("\n\t{}", ddd.replace("\n", "\n\t"));
                } else {
                    dd = String::from("\n\t?");
                }
            }

            if !dd.is_empty() {
                if d.is_empty() {
                    d = format!("{{{}\n}}", dd);
                } else {
                    d = format!("{} {{{}\n}}", d, dd);
                }
            } else if d.is_empty() {
                d = String::from("{?}");
            }

            write!(f, "{}", d.replace("\t", TAB_SPACES))
        }
    }
}

mod enums {
    use super::*;
    use crate::som::enums::*;

    impl<T: Display + Copy> Display for SOMEnumTypeItem<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let (key, value) = self.get();
            write!(f, "'{}' : {}", key, value)
        }
    }

    impl<T: Display + Copy + PartialEq> Display for SOMEnumType<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let mut d = match self.meta() {
                Some(meta) => meta.to_str(),
                None => String::from("enum"),
            };

            let mut dd = String::from("");
            if self.len() > 0 {
                if let Some(value) = &self.value() {
                    dd = format!("\n\t{}", value);
                } else {
                    dd = String::from("\n\t'?'");
                }
            }

            if !dd.is_empty() {
                if d.is_empty() {
                    d = format!("{{{}\n}}", dd);
                } else {
                    d = format!("{} {{{}\n}}", d, dd);
                }
            } else if d.is_empty() {
                d = String::from("{'?'}");
            }

            write!(f, "{}", d.replace("\t", TAB_SPACES))
        }
    }

    impl<T: Display + Copy + PartialEq> Display for SOMEnumTypeWithEndian<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            write!(f, "{}", self.enumeration())
        }
    }
}

mod strings {
    use super::*;
    use crate::som::strings::*;

    impl Display for SOMStringType {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let mut d = match self.meta() {
                Some(meta) => meta.to_str(),
                None => String::from("string"),
            };

            let dd = self.get();
            if !dd.is_empty() {
                if d.is_empty() {
                    d = format!("'{}'", dd);
                } else {
                    d = format!("{} : '{}'", d, dd);
                }
            } else if d.is_empty() {
                d = String::from("''");
            }

            write!(f, "{}", d)
        }
    }
}

mod optionals {
    use super::*;
    use crate::som::optionals::*;

    impl<T: Display + SOMType> Display for SOMOptionalTypeItem<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let (key, value) = self.get();
            write!(f, "<{}> {}", key, value)
        }
    }

    impl<T: Display + SOMType> Display for SOMOptionalType<T> {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let mut d = match self.meta() {
                Some(meta) => meta.to_str(),
                None => String::from("optional"),
            };

            let mut dd = String::from("");
            for member in self.members() {
                if member.is_set() {
                    let ddd = format!("{}", member);
                    dd = format!("{}\n\t{},", dd, ddd.replace("\n", "\n\t"));
                }
            }

            if !dd.is_empty() {
                if d.is_empty() {
                    d = format!("{{{}\n}}", dd);
                } else {
                    d = format!("{} {{{}\n}}", d, dd);
                }
            } else if d.is_empty() {
                d = String::from("{}");
            }

            write!(f, "{}", d.replace("\t", TAB_SPACES))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives_output() {
        let mut obj = SOMu8::empty();
        assert_eq!("u8", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::empty());
        assert_eq!("u8", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::from(String::from("name"), String::from("")));

        assert_eq!("name", format!("{}", obj));

        obj = SOMu8::from(1u8);
        assert_eq!("u8 : 1", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::empty());
        assert_eq!("1", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::from(
            String::from("name"),
            String::from("description"),
        ));

        assert_eq!("name (description) : 1", format!("{}", obj));

        let obj = SOMu16::from(SOMEndian::Big, 1u16);
        assert_eq!("u16 : 1", format!("{}", obj));
    }

    #[test]
    fn test_arrays_output() {
        let mut obj = SOMu8Array::from(SOMLengthField::U8, 0, 2, vec![]);
        assert_eq!("array", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::empty());
        assert_eq!("[]", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::from(String::from("name"), String::from("")));

        assert_eq!("name", format!("{}", obj));

        obj = SOMu8Array::from(
            SOMLengthField::U8,
            0,
            2,
            vec![SOMu8::from(1u8), SOMu8::from(2u8)],
        );

        assert_eq!(
            "array [\n\tu8 : 1,\n\tu8 : 2,\n]".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::empty());

        assert_eq!(
            "[\n\tu8 : 1,\n\tu8 : 2,\n]".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::from(
            String::from("name"),
            String::from("description"),
        ));

        assert_eq!(
            "name (description) [\n\tu8 : 1,\n\tu8 : 2,\n]".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        let obj = SOMArray::from(
            SOMLengthField::None,
            1,
            1,
            vec![SOMArrayMember::ArrayU8(SOMu8Array::from(
                SOMLengthField::U8,
                0,
                2,
                vec![SOMu8::from(1u8), SOMu8::from(2u8)],
            ))],
        );

        assert_eq!(
            "array [\n\tarray [\n\t\tu8 : 1,\n\t\tu8 : 2,\n\t],\n]".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );
    }

    #[test]
    fn test_structs_output() {
        let mut obj = SOMStruct::from(vec![]);
        assert_eq!("struct", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::empty());
        assert_eq!("{}", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::from(String::from("name"), String::from("")));

        assert_eq!("name", format!("{}", obj));

        obj = SOMStruct::from(vec![
            SOMStructMember::Bool(SOMBool::from(true)),
            SOMStructMember::U16(SOMu16::from(SOMEndian::Big, 49200u16)),
        ]);

        assert_eq!(
            "struct {\n\tbool : true,\n\tu16 : 49200,\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::empty());

        assert_eq!(
            "{\n\tbool : true,\n\tu16 : 49200,\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::from(
            String::from("name"),
            String::from("description"),
        ));

        assert_eq!(
            "name (description) {\n\tbool : true,\n\tu16 : 49200,\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        let obj = SOMStruct::from(vec![SOMStructMember::Struct(SOMStruct::from(vec![
            SOMStructMember::Bool(SOMBool::from(true)),
            SOMStructMember::U16(SOMu16::from(SOMEndian::Big, 49200u16)),
        ]))]);

        assert_eq!(
            "struct {\n\tstruct {\n\t\tbool : true,\n\t\tu16 : 49200,\n\t},\n}"
                .replace("\t", TAB_SPACES),
            format!("{}", obj)
        );
    }

    #[test]
    fn test_unions_output() {
        let mut obj = SOMUnion::from(SOMTypeField::U8, vec![]);
        assert_eq!("union", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::empty());
        assert_eq!("{?}", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::from(String::from("name"), String::from("")));

        assert_eq!("name", format!("{}", obj));

        obj = SOMUnion::from(
            SOMTypeField::U8,
            vec![
                SOMUnionMember::Bool(SOMBool::empty()),
                SOMUnionMember::U16(SOMu16::empty(SOMEndian::Big)),
            ],
        );

        assert_eq!(
            "union {\n\t?\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        if let Some(SOMUnionMember::U16(sub)) = obj.get_mut(2) {
            sub.set(49200u16);
        } else {
            panic!();
        }

        assert_eq!(
            "union {\n\tu16 : 49200\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::empty());

        assert_eq!(
            "{\n\tu16 : 49200\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::from(
            String::from("name"),
            String::from("description"),
        ));

        assert_eq!(
            "name (description) {\n\tu16 : 49200\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );
    }

    #[test]
    fn test_enums_output() {
        let mut obj = SOMu8Enum::from(vec![]);
        assert_eq!("enum", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::empty());
        assert_eq!("{'?'}", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::from(String::from("name"), String::from("")));

        assert_eq!("name", format!("{}", obj));

        obj = SOMu8Enum::from(vec![
            SOMEnum::from(String::from("A"), 23u8),
            SOMEnum::from(String::from("B"), 42u8),
        ]);

        assert_eq!(
            "enum {\n\t'?'\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        assert!(obj.set(String::from("A")));

        assert_eq!(
            "enum {\n\t'A' : 23\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::empty());

        assert_eq!(
            "{\n\t'A' : 23\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::from(
            String::from("name"),
            String::from("description"),
        ));

        assert_eq!(
            "name (description) {\n\t'A' : 23\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        let mut obj = SOMu16Enum::from(
            SOMEndian::Big,
            vec![
                SOMEnum::from(String::from("A"), 49200u16),
                SOMEnum::from(String::from("B"), 49201u16),
            ],
        );

        assert!(obj.set(String::from("B")));

        assert_eq!(
            "enum {\n\t'B' : 49201\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );
    }

    #[test]
    fn test_strings_output() {
        let mut obj = SOMString::fixed(
            SOMStringEncoding::Utf8,
            SOMStringFormat::WithBOMandTermination,
            7,
        );
        assert_eq!("string", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::empty());
        assert_eq!("''", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::from(String::from("name"), String::from("")));

        assert_eq!("name", format!("{}", obj));

        let mut obj = SOMString::fixed(
            SOMStringEncoding::Utf8,
            SOMStringFormat::WithBOMandTermination,
            7,
        );

        assert!(obj.set(String::from("foo")));
        assert_eq!("string : 'foo'", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::empty());
        assert_eq!("'foo'", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::from(
            String::from("name"),
            String::from("description"),
        ));

        assert_eq!("name (description) : 'foo'", format!("{}", obj));
    }

    #[test]
    fn test_optionals_output() {
        let mut obj = SOMOptional::from(SOMLengthField::U32, vec![]);
        assert_eq!("optional", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::empty());
        assert_eq!("{}", format!("{}", obj));

        obj = obj.with_meta(SOMTypeMeta::from(String::from("name"), String::from("")));

        assert_eq!("name", format!("{}", obj));

        let mut obj = SOMOptional::from(
            SOMLengthField::U32,
            vec![
                SOMOptional::required(1, SOMOptionalMember::U16(SOMu16::empty(SOMEndian::Big)))
                    .unwrap(),
            ],
        );

        if let Some(SOMUnionMember::U16(sub)) = obj.get_mut(1) {
            sub.set(49200u16);
        } else {
            panic!();
        }

        assert_eq!(
            "optional {\n\t<1> u16 : 49200,\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::empty());

        assert_eq!(
            "{\n\t<1> u16 : 49200,\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        obj = obj.with_meta(SOMTypeMeta::from(
            String::from("name"),
            String::from("description"),
        ));

        assert_eq!(
            "name (description) {\n\t<1> u16 : 49200,\n}".replace("\t", TAB_SPACES),
            format!("{}", obj)
        );

        let mut obj = SOMOptional::from(
            SOMLengthField::U32,
            vec![SOMOptional::required(
                1,
                SOMOptionalMember::Struct(SOMStruct::from(vec![
                    SOMStructMember::Bool(SOMBool::empty()),
                    SOMStructMember::U16(SOMu16::empty(SOMEndian::Big)),
                ])),
            )
            .unwrap()],
        );

        if let Some(SOMStructMember::Struct(sub)) = obj.get_mut(1) {
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

        assert_eq!(
            "optional {\n\t<1> struct {\n\t\tbool : true,\n\t\tu16 : 49200,\n\t},\n}"
                .replace("\t", TAB_SPACES),
            format!("{}", obj)
        );
    }
}
