use crate::{types::*, Error};
use byteorder::{BigEndian as BE, ReadBytesExt};
use std::io::{Cursor, Read};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

impl<'a> Message<'a> {
    /// Parse a message from a byte slice.
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
            }) => Ok(Message::CookieClient),

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
            }) => Ok(Message::CookieServer),

            Ok(header) => {
                let payload_len = header.payload_len();
                if input.len() < Header::LENGTH + payload_len {
                    return Err(Error::NotEnoughData {
                        min: Header::LENGTH + payload_len,
                        actual: input.len(),
                    });
                }

                let payload = &input[Header::LENGTH..Header::LENGTH + payload_len];

                if header.is_sd() {
                    let mut buffer = Cursor::new(payload);
                    Ok(Message::Sd(header, SdPayload::from_reader(&mut buffer)?))
                } else {
                    Ok(Message::Rpc(header, payload))
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl Header {
    /// Parse a header from a byte slice.
    pub fn from_slice(input: &[u8]) -> Result<Header, Error> {
        if input.len() < Header::LENGTH {
            return Err(Error::NotEnoughData {
                min: Header::LENGTH,
                actual: input.len(),
            });
        }

        let mut buffer = Cursor::new(input);
        Header::from_reader(&mut buffer)
    }

    /// Parse a header from a `Read`
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

impl SdPayload {
    /// Parse SD payload from a `Read`
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<SdPayload, Error> {
        let flags = reader.read_u8()?;
        reader.read_u24::<BE>()?; // reserved

        let entries_len: usize = reader.read_u32::<BE>()? as usize;
        let num_entries = entries_len / SdEntry::LENGTH;
        let mut entries: Vec<SdEntry> = Vec::with_capacity(num_entries);

        for _ in 0..num_entries {
            entries.push(SdEntry::from_reader(reader)?);
        }

        let options_len: usize = reader.read_u32::<BE>()? as usize;
        let mut options: Vec<SdOption> = Vec::new();

        let mut read_len: usize = 0;
        while read_len < options_len {
            let (option_len, option) = SdOption::from_reader(reader)?;
            if let Some(value) = option {
                options.push(value);
            }

            read_len += option_len;
        }

        Ok(SdPayload {
            flags,
            entries,
            options,
        })
    }
}

impl SdEntry {
    fn from_reader<R: Read>(reader: &mut R) -> Result<SdEntry, Error> {
        let entry_type = reader.read_u8()?;
        let index1 = reader.read_u8()?;
        let index2 = reader.read_u8()?;
        let num = reader.read_u8()?;
        let num1 = (num >> 4) & 0x0F;
        let num2 = num & 0x0F;
        let service_id = reader.read_u16::<BE>()?;
        let instance_id = reader.read_u16::<BE>()?;
        let major_version = reader.read_u8()?;
        let ttl = reader.read_u24::<BE>()?;

        let options = SdOptionRef {
            index1,
            index2,
            num1,
            num2,
        };

        if SdEntry::is_service(entry_type) {
            let minor_version = reader.read_u32::<BE>()?;

            Ok(SdEntry::try_from((
                entry_type,
                SdServiceEntry {
                    service_id,
                    instance_id,
                    major_version,
                    minor_version,
                    ttl,
                    options,
                },
            ))?)
        } else {
            reader.read_u16::<BE>()?; // reserved
            let eventgroup_id = reader.read_u16::<BE>()?;

            Ok(SdEntry::try_from((
                entry_type,
                SdEventgroupEntry {
                    service_id,
                    eventgroup_id,
                    instance_id,
                    major_version,
                    ttl,
                    options,
                },
            ))?)
        }
    }
}

impl SdOption {
    fn from_reader<R: Read>(reader: &mut R) -> Result<(usize, Option<SdOption>), Error> {
        let option_len: usize = reader.read_u16::<BE>()? as usize;
        let option_type = reader.read_u8()?;

        if !SdOption::is_ip_option(option_type) {
            let mut buffer: Vec<u8> = vec![0; option_len];
            reader.read_exact(&mut buffer)?;

            return Ok((3 + option_len, None)); // drop option
        }

        reader.read_u8()?; // reserved

        let ip: IpAddr;
        if SdOption::is_ip4_option(option_type) {
            let mut buffer = [0u8; 4];
            reader.read_exact(&mut buffer)?;
            ip = IpAddr::V4(Ipv4Addr::from(buffer));
        } else {
            let mut buffer = [0u8; 16];
            reader.read_exact(&mut buffer)?;
            ip = IpAddr::V6(Ipv6Addr::from(buffer));
        }

        reader.read_u8()?; // reserved
        let proto = IpProto::try_from(reader.read_u8()?)?;
        let port = reader.read_u16::<BE>()?;

        let option = SdOption::from(option_type, SdEndpointOption { ip, port, proto })?;
        Ok((option.len(), Some(option)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_cookie_client() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x00, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x08, // length(u32)
            0xDE, 0xAD, 0xBE, 0xEF, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x01, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];

        assert_eq!(Message::from_slice(header).unwrap(), Message::CookieClient);
    }

    #[test]
    fn parse_cookie_server() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x80, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x08, // length(u32)
            0xDE, 0xAD, 0xBE, 0xEF, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];

        assert_eq!(Message::from_slice(header).unwrap(), Message::CookieServer);
    }

    #[test]
    fn parse_empty_rpc_message() {
        let header: &[u8] = &[
            0x01, 0x03, 0x80, 0x05, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x08, // length(u32)
            0x00, 0x01, 0x00, 0x02, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];

        assert_eq!(
            Message::from_slice(header).unwrap(),
            Message::Rpc(
                Header {
                    message_id: MessageId {
                        service_id: 0x0103,
                        method_id: 0x8005,
                    },
                    length: 8,
                    request_id: RequestId {
                        client_id: 0x0001,
                        session_id: 0x0002,
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::Notification,
                    return_code: ReturnCode::Ok,
                },
                &[],
            )
        );
    }

    #[test]
    fn parse_rpc_message() {
        let header: &[u8] = &[
            0x01, 0x03, 0x80, 0x05, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x0D, // length(u32)
            0x00, 0x01, 0x00, 0x02, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];
        let payload: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05];
        let message: &[u8] = &[header, payload].concat();

        assert_eq!(
            Message::from_slice(message).unwrap(),
            Message::Rpc(
                Header {
                    message_id: MessageId {
                        service_id: 0x0103,
                        method_id: 0x8005,
                    },
                    length: 13,
                    request_id: RequestId {
                        client_id: 0x0001,
                        session_id: 0x0002,
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::Notification,
                    return_code: ReturnCode::Ok,
                },
                payload,
            )
        );
    }

    #[test]
    fn parse_empty_sd_message() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x81, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x14, // length(u32)
            0x00, 0x00, 0x00, 0x00, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];
        let payload: &[u8] = &[
            0xC0, 0x00, 0x00, 0x00, // sdFlags(08), reserved(u24)
            0x00, 0x00, 0x00, 0x00, // entries-length(u32)
            0x00, 0x00, 0x00, 0x00, // options-length(u32)
        ];
        let message: &[u8] = &[header, payload].concat();

        assert_eq!(
            Message::from_slice(message).unwrap(),
            Message::Sd(
                Header {
                    message_id: MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x8100,
                    },
                    length: 20,
                    request_id: RequestId {
                        client_id: 0x0000,
                        session_id: 0x0000,
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::Notification,
                    return_code: ReturnCode::Ok,
                },
                SdPayload {
                    flags: 0xC0,
                    entries: vec![],
                    options: vec![],
                },
            )
        );
    }

    #[test]
    fn parse_find_service_sd_message() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x81, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x2C, // length(u32)
            0x00, 0x00, 0x00, 0x00, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];
        let payload: &[u8] = &[
            0xC0, 0x00, 0x00, 0x00, // sdFlags(08), reserved(u24)
            // entries
            0x00, 0x00, 0x00, 0x10, // entries-length(u32)
            // find-service
            0x00, 0x00, 0x00, 0x00, // entryType(u8), index1(u8), index2,(u8) num1|2(u8)
            0x01, 0x03, 0x00, 0x01, // serviceId(u16), instanceId(u16)
            0x02, 0x00, 0x00, 0x03, // majorVersion(u8), ttl(u24)
            0x00, 0x00, 0x00, 0x0A, // minorVersion(u32)
            // options
            0x00, 0x00, 0x00, 0x08, // options-length(u32)
            // unrelated option (dropped)
            0x00, 0x05, 0x01, 0x00, // length(u16), optionType(u8), reserved(u8)
            0x66, 0x6F, 0x6F, 0x00, // config(String)
        ];
        let message: &[u8] = &[header, payload].concat();

        assert_eq!(
            Message::from_slice(message).unwrap(),
            Message::Sd(
                Header {
                    message_id: MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x8100,
                    },
                    length: 44,
                    request_id: RequestId {
                        client_id: 0x0000,
                        session_id: 0x0000,
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::Notification,
                    return_code: ReturnCode::Ok,
                },
                SdPayload {
                    flags: 0xC0,
                    entries: vec![SdEntry::FindService(SdServiceEntry {
                        service_id: 0x0103,
                        instance_id: 0x0001,
                        major_version: 0x02,
                        minor_version: 0x0000000A,
                        ttl: 0x00000003,
                        options: SdOptionRef {
                            index1: 0,
                            index2: 0,
                            num1: 0,
                            num2: 0,
                        },
                    })],
                    options: vec![],
                },
            )
        );
    }

    #[test]
    fn parse_offer_service_sd_message() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x81, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x48, // length(u32)
            0x00, 0x00, 0x00, 0x00, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];
        let payload: &[u8] = &[
            0xC0, 0x00, 0x00, 0x00, // sdFlags(08), reserved(u24)
            // entries
            0x00, 0x00, 0x00, 0x10, // entries-length(u32)
            // offer-service
            0x01, 0x00, 0x00, 0x20, // entryType(u8), index1(u8), index2,(u8) num1|2(u8)
            0x01, 0x03, 0x00, 0x01, // serviceId(u16), instanceId(u16)
            0x02, 0x00, 0x00, 0x03, // majorVersion(u8), ttl(u24)
            0x00, 0x00, 0x00, 0x0A, // minorVersion(u32)
            // options
            0x00, 0x00, 0x00, 0x24, // options-length(u32)
            // ip-4 endpoint
            0x00, 0x09, 0x04, 0x00, // length(u16), optionType(u8), reserved(u8)
            0x7F, 0x00, 0x00, 0x01, // ip4(u32)
            0x00, 0x11, 0x75, 0x30, // reserved(u8), proto(u8), port(u16)
            // ip-6 endpoint
            0x00, 0x15, 0x06, 0x00, // length(u16), optionType(u8), reserved(u8)
            0xFF, 0x0E, 0x00, 0x00, // ip6(u32)
            0x00, 0x00, 0x00, 0x00, // ip6(u32)
            0x00, 0x00, 0xFF, 0xFF, // ip6(u32)
            0xEF, 0xC0, 0xFF, 0xFB, // ip6(u32)
            0x00, 0x06, 0x75, 0x30, // reserved(u8), proto(u8), port(u16)
        ];
        let message: &[u8] = &[header, payload].concat();

        assert_eq!(
            Message::from_slice(message).unwrap(),
            Message::Sd(
                Header {
                    message_id: MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x8100,
                    },
                    length: 72,
                    request_id: RequestId {
                        client_id: 0x0000,
                        session_id: 0x0000,
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::Notification,
                    return_code: ReturnCode::Ok,
                },
                SdPayload {
                    flags: 0xC0,
                    entries: vec![SdEntry::OfferService(SdServiceEntry {
                        service_id: 0x0103,
                        instance_id: 0x0001,
                        major_version: 0x02,
                        minor_version: 0x0000000A,
                        ttl: 0x00000003,
                        options: SdOptionRef {
                            index1: 0,
                            index2: 0,
                            num1: 2,
                            num2: 0,
                        },
                    })],
                    options: vec![
                        SdOption::Ip4Unicast(SdEndpointOption {
                            ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                            port: 30000,
                            proto: IpProto::UDP,
                        }),
                        SdOption::Ip6Unicast(SdEndpointOption {
                            ip: IpAddr::V6(
                                Ipv6Addr::from_str("FF0E:0000:0000:0000:0000:FFFF:EFC0:FFFB")
                                    .unwrap()
                            ),
                            port: 30000,
                            proto: IpProto::TCP,
                        }),
                    ],
                },
            )
        );
    }

    #[test]
    fn parse_subscribe_eventgroup_and_ack_sd_message() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x81, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x48, // length(u32)
            0x00, 0x00, 0x00, 0x00, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];
        let payload: &[u8] = &[
            0xC0, 0x00, 0x00, 0x00, // sdFlags(08), reserved(u24)
            // entries
            0x00, 0x00, 0x00, 0x20, // entries-length(u32)
            // subscribe-eventgroup
            0x06, 0x01, 0x00, 0x10, // entryType(u8), index1(u8), index2,(u8) num1|2(u8)
            0x01, 0x03, 0x00, 0x01, // serviceId(u16), instanceId(u16)
            0x02, 0x00, 0x00, 0x03, // majorVersion(u8), ttl(u24)
            0x00, 0x00, 0x01, 0xC8, // reserved(u16), eventgroupId(u16)
            // subscribe-eventgroup-ack
            0x07, 0x00, 0x01, 0x01, // entryType(u8), index1(u8), index2,(u8) num1|2(u8)
            0x01, 0x03, 0x00, 0x01, // serviceId(u16), instanceId(u16)
            0x02, 0x00, 0x00, 0x03, // majorVersion(u8), ttl(u24)
            0x00, 0x00, 0x01, 0xC8, // reserved(u16), eventgroupId(u16)
            // options
            0x00, 0x00, 0x00, 0x14, // options-length(u32)
            // unrelated option (dropped)
            0x00, 0x05, 0x01, 0x00, // length(u16), optionType(u8), reserved(u8)
            0x66, 0x6F, 0x6F, 0x00, // config(String)
            // ip-4 endpoint
            0x00, 0x09, 0x04, 0x00, // length(u16), optionType(u8), reserved(u8)
            0x7F, 0x00, 0x00, 0x01, // ip4(u32)
            0x00, 0x11, 0x75, 0x30, // reserved(u8), proto(u8), port(u16)
        ];
        let message: &[u8] = &[header, payload].concat();

        assert_eq!(
            Message::from_slice(message).unwrap(),
            Message::Sd(
                Header {
                    message_id: MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x8100,
                    },
                    length: 72,
                    request_id: RequestId {
                        client_id: 0x0000,
                        session_id: 0x0000,
                    },
                    protocol_version: 0x01,
                    interface_version: 0x01,
                    message_type: MessageType::Notification,
                    return_code: ReturnCode::Ok,
                },
                SdPayload {
                    flags: 0xC0,
                    entries: vec![
                        SdEntry::SubscribeEventgroup(SdEventgroupEntry {
                            service_id: 0x0103,
                            eventgroup_id: 0x01C8,
                            instance_id: 0x0001,
                            major_version: 0x02,
                            ttl: 0x00000003,
                            options: SdOptionRef {
                                index1: 1,
                                index2: 0,
                                num1: 1,
                                num2: 0,
                            },
                        }),
                        SdEntry::SubscribeEventgroupAck(SdEventgroupEntry {
                            service_id: 0x0103,
                            eventgroup_id: 0x01C8,
                            instance_id: 0x0001,
                            major_version: 0x02,
                            ttl: 0x00000003,
                            options: SdOptionRef {
                                index1: 0,
                                index2: 1,
                                num1: 0,
                                num2: 1,
                            },
                        })
                    ],
                    options: vec![SdOption::Ip4Unicast(SdEndpointOption {
                        ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                        port: 30000,
                        proto: IpProto::UDP,
                    })],
                },
            )
        );
    }
}
