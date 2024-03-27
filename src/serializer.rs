use crate::{types::*, Error};
use byteorder::{BigEndian, WriteBytesExt};
use std::io::Write;
use std::net::IpAddr;

impl<'a> Message<'a> {
    /// Serializes the message into a byte array.
    pub fn to_vec(&self) -> Vec<u8> {
        let len = match self {
            Message::Rpc(_, payload) => Header::LENGTH + payload.len(),
            Message::Sd(_, payload) => Header::LENGTH + payload.len(),
            Message::CookieClient | Message::CookieServer => Header::LENGTH,
        };

        let mut buffer = Vec::with_capacity(len);
        self.to_writer(&mut buffer).unwrap(); // Safe because it is a Vec
        buffer
    }

    /// Serialize the message into a writer.
    pub fn to_writer<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        match self {
            Message::Rpc(header, payload) => {
                header.to_writer(&mut writer)?;
                writer.write_all(payload)?;
                Ok(Header::LENGTH + payload.len())
            }
            Message::Sd(header, payload) => {
                header.to_writer(&mut writer)?;
                payload.to_writer(&mut writer)?;
                Ok(Header::LENGTH + payload.len())
            }
            Message::CookieClient => {
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
                MAGIC_COOKIE_CLIENT
                    .to_writer(writer)
                    .map(|_| Header::LENGTH)
            }
            Message::CookieServer => {
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
                MAGIC_COOKIE.to_writer(writer).map(|_| Header::LENGTH)
            }
        }
    }
}

impl Header {
    /// Serializes the header into a byte array.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(Header::LENGTH);
        self.to_writer(&mut buffer).unwrap(); // Safe because it is a Vec
        buffer
    }

    /// Serialize the header into a writer.
    pub fn to_writer<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        writer.write_u16::<BigEndian>(self.message_id.service_id)?;
        writer.write_u16::<BigEndian>(self.message_id.method_id)?;
        writer.write_u32::<BigEndian>(self.length)?;
        writer.write_u16::<BigEndian>(self.request_id.client_id)?;
        writer.write_u16::<BigEndian>(self.request_id.session_id)?;
        writer.write_u8(self.protocol_version)?;
        writer.write_u8(self.interface_version)?;
        writer.write_u8(self.message_type.into())?;
        writer.write_u8(self.return_code.into())?;

        Ok(Header::LENGTH)
    }
}

impl SdPayload {
    /// Serializes the payload into a writer.
    pub fn to_writer<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        writer.write_u8(self.flags)?;
        writer.write_u24::<BigEndian>(0x000000)?; // reserved

        writer.write_u32::<BigEndian>(self.entries_len() as u32)?;
        for entry in &self.entries {
            entry.to_writer(&mut writer)?;
        }

        writer.write_u32::<BigEndian>(self.options_len() as u32)?;
        for option in &self.options {
            option.to_writer(&mut writer)?;
        }

        Ok(self.len())
    }
}

impl SdEntry {
    fn to_writer<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        let entry_type: u8 = self.into();

        match self {
            SdEntry::FindService(item) => item.to_writer(entry_type, &mut writer),
            SdEntry::OfferService(item) => item.to_writer(entry_type, &mut writer),
            SdEntry::SubscribeEventgroup(item) => item.to_writer(entry_type, &mut writer),
            SdEntry::SubscribeEventgroupAck(item) => item.to_writer(entry_type, &mut writer),
        }
    }
}

impl SdServiceEntry {
    fn to_writer<W: Write>(&self, entry_type: u8, writer: &mut W) -> Result<usize, Error> {
        writer.write_u8(entry_type)?;
        self.options.to_writer(writer)?;

        writer.write_u16::<BigEndian>(self.service_id)?;
        writer.write_u16::<BigEndian>(self.instance_id)?;
        writer.write_u8(self.major_version)?;
        writer.write_u24::<BigEndian>(self.ttl)?;
        writer.write_u32::<BigEndian>(self.minor_version)?;

        Ok(SdEntry::LENGTH)
    }
}

impl SdEventgroupEntry {
    fn to_writer<W: Write>(&self, entry_type: u8, writer: &mut W) -> Result<usize, Error> {
        writer.write_u8(entry_type)?;
        self.options.to_writer(writer)?;
        writer.write_u16::<BigEndian>(self.service_id)?;
        writer.write_u16::<BigEndian>(self.instance_id)?;
        writer.write_u8(self.major_version)?;
        writer.write_u24::<BigEndian>(self.ttl)?;
        writer.write_u16::<BigEndian>(0x0000)?; // reserved
        writer.write_u16::<BigEndian>(self.eventgroup_id)?;

        Ok(SdEntry::LENGTH)
    }
}

impl SdOptionRef {
    fn to_writer<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        writer.write_u8(self.index1)?;
        writer.write_u8(self.index2)?;
        let mut num: u8 = 0x00;
        num |= (self.num1 << 4) & 0xF0;
        num |= self.num2 & 0x0F;
        writer.write_u8(num)?;

        Ok(3)
    }
}

impl SdOption {
    fn to_writer<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        let option_type: u8 = self.into();
        let option_len = self.len();

        match self {
            SdOption::IpUnicast(item) => item.to_writer(option_type, option_len, &mut writer),
            SdOption::IpMulticast(item) => item.to_writer(option_type, option_len, &mut writer),
        }
    }
}

impl SdEndpointOption {
    fn to_writer<W: Write>(
        &self,
        option_type: u8,
        option_len: usize,
        writer: &mut W,
    ) -> Result<usize, Error> {
        writer.write_u16::<BigEndian>(option_len as u16 - 3)?;
        writer.write_u8(option_type)?;
        writer.write_u8(0x00)?; // reserved

        match &self.ip {
            IpAddr::V4(ip4) => {
                for octet in ip4.octets() {
                    writer.write_u8(octet)?;
                }
            }
            IpAddr::V6(ip6) => {
                for segement in ip6.segments() {
                    writer.write_u16::<BigEndian>(segement)?;
                }
            }
        }

        writer.write_u8(0x00)?; // reserved
        writer.write_u8(self.proto.into())?;
        writer.write_u16::<BigEndian>(self.port)?;

        Ok(option_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use std::str::FromStr;

    #[test]
    fn serialize_cookie_client() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x00, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x08, // length(u32)
            0xDE, 0xAD, 0xBE, 0xEF, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x01, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];

        assert_eq!(header, Message::CookieClient.to_vec());
    }

    #[test]
    fn serialize_cookie_server() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x80, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x08, // length(u32)
            0xDE, 0xAD, 0xBE, 0xEF, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];

        assert_eq!(header, Message::CookieServer.to_vec());
    }

    #[test]
    fn serialize_empty_rpc_message() {
        let header: &[u8] = &[
            0x01, 0x03, 0x80, 0x05, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x08, // length(u32)
            0x00, 0x01, 0x00, 0x02, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];

        assert_eq!(
            header,
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
            .to_vec()
        );
    }

    #[test]
    fn serialize_rpc_message() {
        let header: &[u8] = &[
            0x01, 0x03, 0x80, 0x05, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x0D, // length(u32)
            0x00, 0x01, 0x00, 0x02, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];
        let payload: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05];
        let message: &[u8] = &[header, payload].concat();

        assert_eq!(
            message,
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
            .to_vec()
        );
    }

    #[test]
    fn serialize_empty_sd_message() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x81, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x0C, // length(u32)
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
            message,
            Message::Sd(
                Header {
                    message_id: MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x8100,
                    },
                    length: 12,
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
            .to_vec()
        );
    }

    #[test]
    fn serialize_find_service_sd_message() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x81, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x24, // length(u32)
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
            0x00, 0x00, 0x00, 0x00, // options-length(u32)
        ];
        let message: &[u8] = &[header, payload].concat();

        assert_eq!(
            message,
            Message::Sd(
                Header {
                    message_id: MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x8100,
                    },
                    length: 36,
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
            .to_vec()
        );
    }

    #[test]
    fn serialize_offer_service_sd_message() {
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
            message,
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
                        SdOption::IpUnicast(SdEndpointOption {
                            ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                            port: 30000,
                            proto: IpProto::UDP,
                        }),
                        SdOption::IpUnicast(SdEndpointOption {
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
            .to_vec()
        );
    }

    #[test]
    fn serialize_subscribe_eventgroup_and_ack_sd_message() {
        let header: &[u8] = &[
            0xFF, 0xFF, 0x81, 0x00, // serviceId(u16), methodId(u16)
            0x00, 0x00, 0x00, 0x40, // length(u32)
            0x00, 0x00, 0x00, 0x00, // clientId(u16), sessionId(u16)
            0x01, 0x01, 0x02, 0x00, // proto(u8), version(u8), messageType,(u8) returnCode(u8)
        ];
        let payload: &[u8] = &[
            0xC0, 0x00, 0x00, 0x00, // sdFlags(08), reserved(u24)
            // entries
            0x00, 0x00, 0x00, 0x20, // entries-length(u32)
            // subscribe-eventgroup
            0x06, 0x00, 0x00, 0x10, // entryType(u8), index1(u8), index2,(u8) num1|2(u8)
            0x01, 0x03, 0x00, 0x01, // serviceId(u16), instanceId(u16)
            0x02, 0x00, 0x00, 0x03, // majorVersion(u8), ttl(u24)
            0x00, 0x00, 0x01, 0xC8, // reserved(u16), eventgroupId(u16)
            // subscribe-eventgroup-ack
            0x07, 0x00, 0x00, 0x01, // entryType(u8), index1(u8), index2,(u8) num1|2(u8)
            0x01, 0x03, 0x00, 0x01, // serviceId(u16), instanceId(u16)
            0x02, 0x00, 0x00, 0x03, // majorVersion(u8), ttl(u24)
            0x00, 0x00, 0x01, 0xC8, // reserved(u16), eventgroupId(u16)
            // options
            0x00, 0x00, 0x00, 0x0C, // options-length(u32)
            // ip-4 endpoint
            0x00, 0x09, 0x04, 0x00, // length(u16), optionType(u8), reserved(u8)
            0x7F, 0x00, 0x00, 0x01, // ip4(u32)
            0x00, 0x11, 0x75, 0x30, // reserved(u8), proto(u8), port(u16)
        ];
        let message: &[u8] = &[header, payload].concat();

        assert_eq!(
            message,
            Message::Sd(
                Header {
                    message_id: MessageId {
                        service_id: 0xFFFF,
                        method_id: 0x8100,
                    },
                    length: 64,
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
                                index1: 0,
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
                                index2: 0,
                                num1: 0,
                                num2: 1,
                            },
                        })
                    ],
                    options: vec![SdOption::IpUnicast(SdEndpointOption {
                        ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                        port: 30000,
                        proto: IpProto::UDP,
                    })],
                },
            )
            .to_vec()
        );
    }
}
