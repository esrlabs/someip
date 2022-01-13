use std::io::{self, Write};

use byteorder::{BigEndian, WriteBytesExt};

use crate::types::{Header, Message, MessageId, MessageType, RequestId, ReturnCode};

/// Header length in bytes PRS_SOMEIP_00030
const HEADER_LENGTH: usize = 16;

impl Header {
    /// Serialization of a SomeIP header to bytes
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_LENGTH);
        self.to_writer(&mut buf).unwrap(); // Safe because it is a Vec
        buf
    }

    /// Serialization of a SomeIP header to `writer`
    pub fn to_writer<W: Write>(&self, writer: &mut W) -> Result<usize, io::Error> {
        writer.write_u16::<BigEndian>(self.message_id.service_id)?;
        writer.write_u16::<BigEndian>(self.message_id.method_id)?;
        writer.write_u32::<BigEndian>(self.length)?;
        writer.write_u16::<BigEndian>(self.request_id.client_id)?;
        writer.write_u16::<BigEndian>(self.request_id.session_id)?;
        writer.write_u8(self.protocol_version)?;
        writer.write_u8(self.interface_version)?;
        writer.write_u8(self.message_type.into())?;
        writer.write_u8(self.return_code.into())?;
        Ok(HEADER_LENGTH)
    }
}

/// Serialize a SomeIP header to a vector of bytes
#[deprecated(note = "use Header::to_vec instead")]
pub fn serialize_someip_header(header: &Header) -> Vec<u8> {
    header.to_vec()
}

impl<'a> Message<'a> {
    /// Serialize a SomeIP message to a vector of bytes
    pub fn to_vec(&self) -> Vec<u8> {
        let len = match self {
            Message::Message(_, payload) => 8 + payload.len(),
            Message::MagicCookieClient | Message::MagicCookieServer => 16,
        };
        let mut buf = Vec::with_capacity(len);
        self.to_writer(&mut buf).unwrap(); // Safe because it is a Vec
        buf
    }

    /// Serialization of a SomeIP to `writer`
    pub fn to_writer<W: Write>(&self, writer: &mut W) -> Result<usize, io::Error> {
        match self {
            Message::Message(header, payload) => {
                header.to_writer(writer)?;
                writer.write_all(payload)?;
                Ok(16 + payload.len())
            }
            Message::MagicCookieClient => {
                const MAGIC_COOKIE_CLIENT: Header = Header {
                    message_id: MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x0000,
                    },
                    length: 8,
                    request_id: RequestId {
                        client_id: 0xDEAD,
                        session_id: 0xBEEF,
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::RequestNoReturn,
                    return_code: ReturnCode::Ok,
                };
                MAGIC_COOKIE_CLIENT.to_writer(writer).map(|_| 16)
            }
            Message::MagicCookieServer => {
                const MAGIC_COOKIE: Header = Header {
                    message_id: MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x8000,
                    },
                    length: 8,
                    request_id: RequestId {
                        client_id: 0xDEAD,
                        session_id: 0xBEEF,
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::Notification,
                    return_code: ReturnCode::Ok,
                };
                MAGIC_COOKIE.to_writer(writer).map(|_| 16)
            }
        }
    }
}

/// Serialization of a SomeIP message to bytes
#[deprecated(note = "use SomeIp::to_vec instead")]
#[allow(dead_code)]
pub fn serialize_someip(someip: &Message) -> Vec<u8> {
    someip.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, MessageId, MessageType, RequestId, ReturnCode};

    #[test]
    fn check_someip_header_serializer() {
        // SOME/IP Protocol (Service ID: 0x3085, Method ID: 0x0001, Length: 8)
        //     Service ID: 0x3085
        //     Method ID: 0x0001
        //     Length: 8
        //     Client ID: 0x0000
        //     Session ID: 0x0000
        //     SOME/IP Version: 0x01
        //     Interface Version: 0x01
        //     Message Type: 0x00 (Request)
        //     Return Code: 0x00 (Ok)
        //     Payload: NONE

        let bytes = hex::decode("30850001000000080000000001010000").expect("invalid hex string");
        // let bytes_slice = bytes.as_slice();
        // let payload: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            bytes,
            Header {
                message_id: MessageId {
                    method_id: 0x0001,
                    service_id: 0x3085,
                },
                length: 8,
                request_id: RequestId {
                    client_id: 0x0000,
                    session_id: 0x0000,
                },
                protocol_version: 0x01,
                interface_version: 0x01,
                message_type: MessageType::Request,
                return_code: ReturnCode::Ok,
            }
            .to_vec()
        )
    }

    #[test]
    fn check_someip_package_serializer() {
        // SOME/IP Protocol (Service ID: 0x0103, Method ID: 0x8005, Length: 13)
        //     Service ID: 0x0103
        //     Method ID: 0x8005
        //     Length: 13
        //     Client ID: 0x0000
        //     Session ID: 0x0000
        //     SOME/IP Version: 0x01
        //     Interface Version: 0x01
        //     Message Type: 0x02 (Notification)
        //         .0.. .... = Message Type Ack Flag: False
        //         ..0. .... = Message Type TP Flag: False
        //     Return Code: 0x00 (Ok)
        //     Payload: 0100000000
        let bytes =
            hex::decode("010380050000000d00000000010102000100000000").expect("invalid hex string");
        // let bytes_slice = bytes.as_slice();
        let payload: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            bytes,
            Message::Message(
                Header {
                    message_id: MessageId {
                        method_id: 0x8005,
                        service_id: 0x0103,
                    },
                    length: 13,
                    request_id: RequestId {
                        client_id: 0x0000,
                        session_id: 0x0000,
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::Notification,
                    return_code: ReturnCode::Ok,
                },
                payload,
            )
            .to_vec()
        )
    }
}
