use nom::bytes::complete::take;
use nom::number::complete::{be_u16, be_u32, be_u8};
use nom::sequence::tuple;
use nom::{IResult, Needed};

use crate::types::{
    ClientID, MessageID, MessageType, MethodID, RequestID, ReturnCode, ServiceID, SessionID,
    SomeIp, SomeIpHeader, SomeIpMessage,
};

type Result<T> = std::result::Result<T, SomeIpError>;

/// Custom Error type
#[derive(Debug, Clone)]
pub struct SomeIpError;

/// Takes a byte array representing a SomeIP Message and returns the SomeIP header and the rest of the
/// message representing the payload.
pub fn someip_header(input: &[u8]) -> IResult<&[u8], SomeIpHeader> {
    let (input, message_id) = message_id(input)?;
    let (input, length) = length(input)?;
    let (input, request_id) = request_id(input)?;
    let (input, protocol_version) = protocol_version(input)?;
    let (input, interface_version) = interface_version(input)?;
    let (input, message_type) = message_type(input)?;
    let (payload, return_code) = return_code(input)?;

    let payload_length = (length - 8) as usize;
    if payload_length > payload.len() {
        Err(nom::Err::Incomplete(Needed::new(
            payload_length - payload.len(),
        )))
    } else {
        Ok((
            payload,
            SomeIpHeader {
                message_id,
                length,
                request_id,
                protocol_version,
                interface_version,
                message_type,
                return_code,
            },
        ))
    }
}

/// Parses the different kinds of SomeIP messages (at the moment only basic type) as Result.
pub fn someip_message(input: &[u8]) -> Result<SomeIp> {
    match someip_header(input) {
        Ok((
            _,
            SomeIpHeader {
                message_id:
                    MessageID {
                        service_id: ServiceID(0xFFFF),
                        method_id: MethodID(0x0000),
                    },
                length: 8,
                request_id:
                    RequestID {
                        client_id: ClientID(0xDEAD),
                        session_id: SessionID(0xBEEF),
                    },
                protocol_version: 0x01,
                interface_version: 0x01,
                message_type: MessageType::RequestNoReturn,
                return_code: ReturnCode::EOk,
            },
        )) => Ok(SomeIp::SomeIpMagicCookieClient),

        Ok((
            _,
            SomeIpHeader {
                message_id:
                    MessageID {
                        service_id: ServiceID(0xFFFF),
                        method_id: MethodID(0x8000),
                    },
                length: 8,
                request_id:
                    RequestID {
                        client_id: ClientID(0xDEAD),
                        session_id: SessionID(0xBEEF),
                    },
                protocol_version: 0x01,
                interface_version: 0x01,
                message_type: MessageType::Notification,
                return_code: ReturnCode::EOk,
            },
        )) => Ok(SomeIp::SomeIpMagicCookieServer),

        Ok((payload, header)) => {
            return Ok(SomeIp::SomeIpMessage(SomeIpMessage { header, payload }));
        }

        Err(_) => Err(SomeIpError),
    }
}

/// Takes a byte array representing a SomeIP Message starting with the MessageID and returns the
/// MessageId and the rest of the message.
fn message_id(i: &[u8]) -> IResult<&[u8], MessageID> {
    tuple((service_id, method_id))(i).map(|(next_input, res)| {
        (
            next_input,
            MessageID {
                service_id: res.0,
                method_id: res.1,
            },
        )
    })
}

/// Takes a byte array representing a SomeIP Message starting with the ServiceID field and returns
/// the ServiceID and the rest of the message.
fn service_id(i: &[u8]) -> IResult<&[u8], ServiceID> {
    be_u16(i).map(|(next_input, res)| (next_input, res.into()))
}

/// Takes a byte array representing a SomeIP Message starting with the MethodID field and returns
/// the MethodID and the rest of the message.
fn method_id(i: &[u8]) -> IResult<&[u8], MethodID> {
    be_u16(i).map(|(next_input, res)| (next_input, res.into()))
}

/// Takes a byte array representing a part of a SomeIp message starting with the length and returns
/// the length field and the rest of the message.
fn length(i: &[u8]) -> IResult<&[u8], u32> {
    be_u32(i)
}

/// Takes a byte array representing a part of a SomeIp message starting with the RequestID and
/// returns the requestId field and the rest of the message.
fn request_id(i: &[u8]) -> IResult<&[u8], RequestID> {
    tuple((client_id, session_id))(i).map(|(next_input, res)| {
        (
            next_input,
            RequestID {
                client_id: res.0,
                session_id: res.1,
            },
        )
    })
}

/// Takes a byte array representing a part of a SomeIp message starting with the ClientID and
/// returns the ClientID field and the rest of the message.
fn client_id(i: &[u8]) -> IResult<&[u8], ClientID> {
    be_u16(i).map(|(next_input, res)| (next_input, res.into()))
}

/// Takes a byte array representing a part of a SomeIp message starting with the SessionID and
/// returns the SessionID field and the rest of the message.
fn session_id(i: &[u8]) -> IResult<&[u8], SessionID> {
    be_u16(i).map(|(next_input, res)| (next_input, res.into()))
}

/// Takes a byte array representing a part of a SomeIp message starting with the protocol version
/// and returns the protocol version field and the rest of the message.
fn protocol_version(i: &[u8]) -> IResult<&[u8], u8> {
    be_u8(i)
}

/// Takes a byte array representing a part of a SomeIp message starting with the interface version
/// and returns the interface version field and the rest of the message.
fn interface_version(i: &[u8]) -> IResult<&[u8], u8> {
    be_u8(i)
}

/// Takes a byte array representing a part of a SomeIp message starting with the MessageType and
/// returns the MessageType field and the rest of the message.
fn message_type(i: &[u8]) -> IResult<&[u8], MessageType> {
    take(1u8)(i).map(|(next_input, res)| (next_input, res.into()))
}

/// Takes a byte array representing a part of a SomeIp message starting with the return code and
/// returns the ReturnCode field and the rest of the message.
fn return_code(i: &[u8]) -> IResult<&[u8], ReturnCode> {
    take(1u8)(i).map(|(next_input, res)| (next_input, res.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    // use hex::decode;

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
        let bytes =
            hex::decode("010380050000000d00000000010102000100000000").expect("invalid hex string");
        let bytes_slice = bytes.as_slice();
        let payload: &[u8] = &[0x01, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            someip_header(bytes_slice),
            Ok((
                payload,
                SomeIpHeader {
                    message_id: MessageID {
                        method_id: MethodID(0x8005),
                        service_id: ServiceID(0x0103),
                    },
                    length: 13,
                    request_id: RequestID {
                        client_id: ClientID(0x0000),
                        session_id: SessionID(0x0000),
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::Notification,
                    return_code: ReturnCode::EOk,
                }
            ))
        )
    }

    #[test]
    fn check_elemental_functions() {
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
        let bytes_slice = bytes.as_slice();
        let (output, message_id) = message_id(bytes_slice).unwrap_or_default();
        let (output, length) = length(output).unwrap_or_default();
        let (output, request_id) = request_id(output).unwrap_or_default();
        let (output, protocol_version) = protocol_version(output).unwrap_or_default();
        let (output, interface_version) = interface_version(output).unwrap_or_default();
        let (output, message_type) = message_type(output).unwrap();
        let (payload, return_code) = return_code(output).unwrap();
        assert_eq!(
            message_id,
            MessageID {
                method_id: MethodID(0x8005),
                service_id: ServiceID(0x0103),
            }
        );
        assert_eq!(length, 13);
        assert_eq!(
            request_id,
            RequestID {
                client_id: ClientID(0x0000),
                session_id: SessionID(0x0000),
            }
        );
        assert_eq!(protocol_version, 0x01);
        assert_eq!(interface_version, 0x01);
        assert_eq!(message_type, MessageType::Notification);
        assert_eq!(return_code, ReturnCode::EOk);
        assert_eq!(payload, [0x01, 0x00, 0x00, 0x00, 0x00])
    }
}
