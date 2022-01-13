use std::io::{Cursor, Read};

use byteorder::{BigEndian as BE, ReadBytesExt};

use crate::{
    types::{Header, Message, MessageId, MessageType, RequestId, ReturnCode},
    Error,
};

/// Header length in bytes PRS_SOMEIP_00030
const HEADER_LENGTH: usize = 16;

/// Takes a byte array representing a SomeIP Message and returns the SomeIP header
#[deprecated(note = "use Header::from_slice instead")]
pub fn someip_header(input: &[u8]) -> Result<Header, Error> {
    Header::from_slice(input)
}

impl Header {
    /// Parse a SomeIP header from a byte array
    pub fn from_slice(s: &[u8]) -> Result<Header, Error> {
        if s.len() < HEADER_LENGTH {
            return Err(Error::NotEnoughData {
                min: HEADER_LENGTH,
                actual: s.len(),
            });
        }
        let mut buf = Cursor::new(s);
        Header::from_reader(&mut buf)
    }

    /// Parse a SomeIP header from a ['Read']
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Header, Error> {
        let service_id = reader.read_u16::<BE>()?;
        let method_id = reader.read_u16::<BE>()?;
        let length = reader.read_u32::<BE>()?;
        let client_id = reader.read_u16::<BE>()?;
        let session_id = reader.read_u16::<BE>()?;
        let protocol_version = reader.read_u8()?;
        let interface_version = reader.read_u8()?;
        let message_type = reader.read_u8()?.try_into()?;
        let return_code = reader.read_u8()?.try_into()?;

        Ok(Header {
            message_id: MessageId {
                service_id,
                method_id,
            },
            length,
            request_id: RequestId {
                client_id,
                session_id,
            },
            protocol_version,
            interface_version,
            message_type,
            return_code,
        })
    }
}

/// Parses the different kinds of SomeIP messages (at the moment only basic type) as Result.
#[deprecated(note = "use Someip::from_slice instead")]
pub fn someip_message(input: &[u8]) -> Result<Message, Error> {
    Message::from_slice(input)
}

impl<'a> Message<'a> {
    /// Parse a SomeIP message from a byte array
    pub fn from_slice(input: &'a [u8]) -> Result<Message<'a>, Error> {
        match Header::from_slice(input) {
            Ok(Header {
                message_id:
                    MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x0000,
                    },
                length: 8,
                request_id:
                    RequestId {
                        client_id: 0xDEAD,
                        session_id: 0xBEEF,
                    },
                protocol_version: 0x01,
                interface_version: 0x01,
                message_type: MessageType::RequestNoReturn,
                return_code: ReturnCode::Ok,
            }) => Ok(Message::MagicCookieClient),

            Ok(Header {
                message_id:
                    MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x8000,
                    },
                length: 8,
                request_id:
                    RequestId {
                        client_id: 0xDEAD,
                        session_id: 0xBEEF,
                    },
                protocol_version: 0x01,
                interface_version: 0x01,
                message_type: MessageType::Notification,
                return_code: ReturnCode::Ok,
            }) => Ok(Message::MagicCookieServer),

            Ok(header) => {
                let payload_len = header.length as usize - 8; // PRS_SOMEIP_00042
                if input.len() < HEADER_LENGTH + payload_len {
                    Err(Error::NotEnoughData {
                        min: HEADER_LENGTH + payload_len,
                        actual: input.len(),
                    })
                } else {
                    let payload = &input[HEADER_LENGTH..HEADER_LENGTH + payload_len];
                    Ok(Message::Message(header, payload))
                }
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_someip_parser() {
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
        let bytes = hex::decode("010380050000000d0000000001010200").expect("invalid hex string");
        assert_eq!(
            Header::from_slice(&bytes).unwrap(),
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
            }
        )
    }
}
