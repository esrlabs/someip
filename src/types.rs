use crate::Error;
use derive_builder::Builder;

use std::net::IpAddr;

/// Length trait for items with a fixed length.
pub trait Length {
    /// The fixed length of an item
    const LENGTH: usize;
}

/// Different kinds of SOME/IP messages.
#[derive(Debug, PartialEq)]
pub enum Message<'a> {
    /// RPC Message
    Rpc(Header, RpcPayload<'a>),
    /// SD Message
    Sd(Header, SdPayload),
    /// Magic Cookie Client
    /// RS_SOMEIP_00010
    CookieClient,
    /// Magic Cookie Server
    /// RS_SOMEIP_00010
    CookieServer,
}

/// Represents the header of a SOME/IP message.
#[derive(Builder, Clone, Debug, PartialEq)]
#[builder(pattern = "mutable")]
pub struct Header {
    /// Message id
    pub message_id: MessageId,
    /// Message length
    pub length: u32,
    /// Request id
    pub request_id: RequestId,
    /// Protocol version
    #[builder(default = "1")]
    pub protocol_version: ProtocolVersion,
    /// Interface version
    pub interface_version: InterfaceVersion,
    /// Message type
    pub message_type: MessageType,
    /// Return code
    pub return_code: ReturnCode,
}

impl Header {
    /// Creates a new header with the given message id,
    /// length, request id, protocol version, interface
    /// version, message type and return code.
    pub fn new(
        message_id: MessageId,
        length: u32,
        request_id: RequestId,
        protocol_version: ProtocolVersion,
        interface_version: InterfaceVersion,
        message_type: MessageType,
        return_code: ReturnCode,
    ) -> Self {
        Self {
            message_id,
            length,
            request_id,
            protocol_version,
            interface_version,
            message_type,
            return_code,
        }
    }

    /// Get message id
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Raw length field
    pub fn length(&self) -> u32 {
        self.length
    }

    /// The length of the message associated with the header
    pub fn message_len(&self) -> usize {
        self.length as usize + 8
    }

    /// The length of the payload associated with the header
    pub fn payload_len(&self) -> usize {
        self.length as usize - 8
    }

    /// Get request id
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Get client id
    pub fn client_id(&self) -> ClientId {
        self.request_id.client_id()
    }

    /// Get session id
    pub fn session_id(&self) -> SessionId {
        self.request_id.session_id()
    }

    /// Get protocol version
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }

    /// Get interface version
    pub fn interface_version(&self) -> InterfaceVersion {
        self.interface_version
    }

    /// Get message type
    pub fn message_type(&self) -> MessageType {
        self.message_type
    }

    /// Get return code
    pub fn return_code(&self) -> ReturnCode {
        self.return_code
    }

    /// Returns true if the header indicates a SD message
    pub fn is_sd(&self) -> bool {
        matches!(
            self,
            Header {
                message_id: MessageId {
                    service_id: 0xFFFF,
                    method_id: 0x8100,
                },
                length: _,
                request_id: _,
                protocol_version: 0x01,
                interface_version: 0x01,
                message_type: MessageType::Notification,
                return_code: ReturnCode::Ok,
            }
        )
    }
}

/// Provides the fixed length of a Header
impl Length for Header {
    /// Fixed length
    const LENGTH: usize = 16; // PRS_SOMEIP_00030
}

/// Represents the MessageId within the header.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MessageId {
    /// Service id
    pub service_id: ServiceId,
    /// Method id
    pub method_id: MethodId,
}

impl MessageId {
    /// Construct a new MessageId
    pub fn new(service_id: ServiceId, method_id: MethodId) -> Self {
        Self {
            service_id,
            method_id,
        }
    }
}

/// Transforms a ServiceId and MethodId to a MessageId.
impl From<(ServiceId, MethodId)> for MessageId {
    fn from(value: (ServiceId, MethodId)) -> Self {
        Self {
            service_id: value.0,
            method_id: value.1,
        }
    }
}

/// Transforms a u32 to a MessageId.
impl From<u32> for MessageId {
    fn from(value: u32) -> Self {
        Self {
            service_id: (value >> 16) as u16,
            method_id: (value & 0xFFFF) as u16,
        }
    }
}

/// Represents the ServiceId within the MessageId.
pub type ServiceId = u16;

/// Represents the MethodId within the MessageId.
pub type MethodId = u16;

/// Represents the RequestId within the header.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RequestId {
    /// Client id
    pub client_id: ClientId,
    /// Session id
    pub session_id: SessionId,
}

impl RequestId {
    /// Construct a new RequestId
    pub fn new(client_id: ClientId, session_id: SessionId) -> Self {
        Self {
            client_id,
            session_id,
        }
    }

    /// Get client id
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Get session id
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }
}

/// Transforms a ClientId and SessionId to a RequestId.
impl From<(ClientId, SessionId)> for RequestId {
    fn from(value: (ClientId, SessionId)) -> Self {
        Self {
            client_id: value.0,
            session_id: value.1,
        }
    }
}

/// Transforms a u32 to a RequestId.
impl From<u32> for RequestId {
    fn from(value: u32) -> Self {
        Self {
            client_id: (value >> 16) as u16,
            session_id: (value & 0xFFFF) as u16,
        }
    }
}

/// Represents the ClientId within the RequestID.
pub type ClientId = u16;

/// Represents the SessionId within the RequestID.
pub type SessionId = u16;

/// Represents the ProtocolVersion within the header.
pub type ProtocolVersion = u8;

/// Represents the InterfaceVersion within the header.
pub type InterfaceVersion = u8;

/// Different kinds of MessageType accepted in a header.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageType {
    /// Request
    Request,
    /// Rquest no return
    RequestNoReturn,
    /// Notification
    Notification,
    /// Repsonse
    Response,
    /// Error
    Error,
    /// Tp request
    TpRequest,
    /// Tp request no return
    TpRequestNoReturn,
    /// Tp notification
    TpNotification,
    /// Tp response
    TpResponse,
    /// Tp error
    TpError,
}

/// Different kinds of EntriesTyp accepted in a SdPayload.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EntriesType {
    /// Find service
    FindService,
    /// Offer service
    OfferService,
    /// Subscribe event group
    SubscribeEventgroup,
    /// Subscribe event group ack
    SubscribeEventgroupACK,
}

/// Transforms a u8 representing the type to a MessageType.
impl TryFrom<u8> for MessageType {
    type Error = Error;

    fn try_from(i: u8) -> Result<Self, Error> {
        match i {
            0x00 => Ok(Self::Request),
            0x01 => Ok(Self::RequestNoReturn),
            0x02 => Ok(Self::Notification),
            0x80 => Ok(Self::Response),
            0x81 => Ok(Self::Error),
            0x20 => Ok(Self::TpRequest),
            0x21 => Ok(Self::TpRequestNoReturn),
            0x22 => Ok(Self::TpNotification),
            0xa0 => Ok(Self::TpResponse),
            0xa1 => Ok(Self::TpError),
            value => Err(Error::InvalidMessageType(value)),
        }
    }
}

/// Transforms a MessageType to a u8 representing the type.
impl From<MessageType> for u8 {
    fn from(i: MessageType) -> u8 {
        use MessageType::*;
        match i {
            Request => 0x00,
            RequestNoReturn => 0x01,
            Notification => 0x02,
            Response => 0x80,
            Error => 0x81,
            TpRequest => 0x20,
            TpRequestNoReturn => 0x21,
            TpNotification => 0x22,
            TpResponse => 0xa0,
            TpError => 0xa1,
        }
    }
}

/// Different kinds of ReturnCode accepted in a header.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReturnCode {
    /// No error occurred
    Ok,
    /// An unspecified error occurred
    NotOk,
    /// The requested Service ID is unknown.
    UnknownService,
    /// The requested Method ID is unknown. Service ID is known.
    UnknownMethod,
    /// Service ID and Method ID are known. Application not running.
    NotReady,
    /// System running the service is not reachable (internal error code only).
    NotReachable,
    /// A timeout occurred (internal error code only).
    Timeout,
    /// Version of SOME/IP protocol not supported
    WrongProtocolVersion,
    /// Interface version mismatch
    WrongInterfaceVersion,
    /// Deserialization error, so that payload cannot be deserialized.
    MalformedMessage,
    /// An unexpected message type was received (e.g. REQUEST_NO_RETURN for a method defined as REQUEST.)
    WrongMessageType,
    /// Reserved for generic SOME/IP errors. These errors will be specified in future versions of this document.
    ReservedGeneric(u8),
    /// Reserved for specific errors of services and meth- ods. These errors are specified by the interface specification.
    ReservedSpecific(u8),
}

/// Transforms a u8 representing the type to a ReturnCode.
impl TryFrom<u8> for ReturnCode {
    type Error = Error;

    fn try_from(i: u8) -> Result<Self, Self::Error> {
        match i {
            0x00 => Ok(Self::Ok),
            0x01 => Ok(Self::NotOk),
            0x02 => Ok(Self::UnknownService),
            0x03 => Ok(Self::UnknownMethod),
            0x04 => Ok(Self::NotReady),
            0x05 => Ok(Self::NotReachable),
            0x06 => Ok(Self::Timeout),
            0x07 => Ok(Self::WrongProtocolVersion),
            0x08 => Ok(Self::WrongInterfaceVersion),
            0x09 => Ok(Self::MalformedMessage),
            0x0a => Ok(Self::WrongMessageType),
            0x0b..=0x1f => Ok(Self::ReservedGeneric(i)),
            0x20..=0x5e => Ok(Self::ReservedSpecific(i)),
            value => Err(Error::InvalidReturnCode(value)),
        }
    }
}

/// Transforms a ReturnCode to a u8 representing the type.
impl From<ReturnCode> for u8 {
    fn from(i: ReturnCode) -> Self {
        use ReturnCode::*;
        match i {
            Ok => 0x00,
            NotOk => 0x01,
            UnknownService => 0x02,
            UnknownMethod => 0x03,
            NotReady => 0x04,
            NotReachable => 0x05,
            Timeout => 0x06,
            WrongProtocolVersion => 0x07,
            WrongInterfaceVersion => 0x08,
            MalformedMessage => 0x09,
            WrongMessageType => 0x0a,
            ReservedGeneric(i) => i,
            ReservedSpecific(i) => i,
        }
    }
}

/// Represents the RpcPayload within a RPC message.
pub type RpcPayload<'a> = &'a [u8];

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::len_without_is_empty)]
/// Represents the SdPayload within a SD message.
pub struct SdPayload {
    /// Flags
    pub flags: SdFlags,
    /// Entries
    pub entries: Vec<SdEntry>,
    /// Options
    pub options: Vec<SdOption>,
}

impl SdPayload {
    /// Length of the payload in bytes
    pub fn len(&self) -> usize {
        12 + self.entries_len() + self.options_len()
    }

    /// Length of the payload's entries in bytes
    pub fn entries_len(&self) -> usize {
        SdEntry::LENGTH * self.entries.len()
    }

    /// Length of the payload's options in bytes
    pub fn options_len(&self) -> usize {
        self.options.iter().map(SdOption::len).sum()
    }

    /// Returns true if the reboot flag is set
    pub fn reboot_flag(&self) -> bool {
        self.flags & 0x80 != 0x00
    }

    /// Returns true if the unicast flag is set
    pub fn unicast_flag(&self) -> bool {
        self.flags & 0x40 != 0x00
    }

    /// Returns the associated options of an entry
    pub fn options(&self, entry_index: usize) -> Vec<&SdEndpointOption> {
        use SdEntry::*;
        match self.entries.get(entry_index).unwrap() {
            FindService(entry) => self.get_options(&entry.options),
            OfferService(entry) => self.get_options(&entry.options),
            SubscribeEventgroup(entry) => self.get_options(&entry.options),
            SubscribeEventgroupAck(entry) => self.get_options(&entry.options),
        }
    }

    fn get_options(&self, option_ref: &SdOptionRef) -> Vec<&SdEndpointOption> {
        let mut options: Vec<&SdEndpointOption> = Vec::new();

        for refs in [
            (option_ref.index1, option_ref.num1),
            (option_ref.index2, option_ref.num2),
        ] {
            for i in 0..refs.1 {
                if let Some(item) = self.options.get(refs.0 as usize + i as usize) {
                    options.push(match item {
                        SdOption::Ip4Unicast(value) => value,
                        SdOption::Ip4Multicast(value) => value,
                        SdOption::Ip6Unicast(value) => value,
                        SdOption::Ip6Multicast(value) => value,
                    });
                }
            }
        }

        options
    }
}

/// Represents the SdFlags within a SdPayload.
pub type SdFlags = u8;

/// Represents the Ttl (Time to live) within a SdPayload.
pub type Ttl = u32;

/// Different kinds of SdEntry accepted in a SdPayload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SdEntry {
    /// Find service entry
    FindService(SdServiceEntry),
    /// Offer service entry
    OfferService(SdServiceEntry),
    /// Subscribe eventgroup entry
    SubscribeEventgroup(SdEventgroupEntry),
    /// Subscribe eventgroup ack entry
    SubscribeEventgroupAck(SdEventgroupEntry),
}

impl SdEntry {
    /// Returns true if the entry type relates to a service
    pub fn is_service(entry_type: u8) -> bool {
        entry_type < 0x04
    }
}

/// Provides the fixed length of a SdEntry
impl Length for SdEntry {
    /// Fixed length
    const LENGTH: usize = 16;
}

/// Transforms a SdEntry to a u8 representing the type.
impl From<&SdEntry> for u8 {
    fn from(entry: &SdEntry) -> Self {
        use SdEntry::*;
        match entry {
            FindService(_) => 0x00,
            OfferService(_) => 0x01,
            SubscribeEventgroup(_) => 0x06,
            SubscribeEventgroupAck(_) => 0x07,
        }
    }
}

/// Transforms a entry type and SdServiceEntry to a SdEntry.
impl TryFrom<(u8, SdServiceEntry)> for SdEntry {
    type Error = Error;

    fn try_from(i: (u8, SdServiceEntry)) -> Result<Self, Error> {
        let (entry_type, entry) = i;
        use SdEntry::*;
        match entry_type {
            0x00 => Ok(FindService(entry)),
            0x01 => Ok(OfferService(entry)),
            entry_type => Err(Error::UnknownSdEntry(entry_type)),
        }
    }
}

/// Transforms a entry type and SdEventgroupEntry to a SdEntry.
impl TryFrom<(u8, SdEventgroupEntry)> for SdEntry {
    type Error = Error;

    fn try_from(i: (u8, SdEventgroupEntry)) -> Result<Self, Error> {
        let (entry_type, entry) = i;
        use SdEntry::*;
        match entry_type {
            0x06 => Ok(SubscribeEventgroup(entry)),
            0x07 => Ok(SubscribeEventgroupAck(entry)),
            entry_type => Err(Error::UnknownSdEntry(entry_type)),
        }
    }
}

/// Represents a SdServiceEntry within a SdPayload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SdServiceEntry {
    /// Service id
    pub service_id: ServiceId,
    /// Instance id
    pub instance_id: InstanceId,
    /// Major version
    pub major_version: MajorVersion,
    /// Minor version
    pub minor_version: MinorVersion,
    /// TTL
    pub ttl: Ttl,
    /// The options referenced by this entry
    pub options: SdOptionRef,
}

impl SdServiceEntry {
    /// Returns true if the entry has a positive ttl
    pub fn has_ttl(&self) -> bool {
        self.ttl != 0
    }
}

/// Represents a SdEventgroupEntry within a SdPayload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SdEventgroupEntry {
    /// Service id
    pub service_id: ServiceId,
    /// Instance id
    pub instance_id: InstanceId,
    /// Eventgroup id
    pub eventgroup_id: EventgroupId,
    /// Major version
    pub major_version: MajorVersion,
    /// TTL
    pub ttl: Ttl,
    /// The options referenced by this entry
    pub options: SdOptionRef,
}

impl SdEventgroupEntry {
    /// Returns true if the entry has a positive ttl
    pub fn has_ttl(&self) -> bool {
        self.ttl != 0
    }
}

/// Represents the InstanceId within a SdEntry.
pub type InstanceId = u16;

/// Represents the EventgroupId within a SdEntry.
pub type EventgroupId = u16;

/// Represents the MajorVersion within a SdEntry.
pub type MajorVersion = u8;

/// Represents the MinorVersion within a SdEntry.
pub type MinorVersion = u32;

/// Represents the referenced options of a SdEntry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SdOptionRef {
    /// Index start of first options set
    pub index1: u8,
    /// Index start of second options set
    pub index2: u8,
    /// Number of options within the first options set
    pub num1: u8,
    /// Number of options within the second options set
    pub num2: u8,
}

/// Different kinds of SdOption accepted in a SdPayload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SdOption {
    /// Ip4 unicast endpoint option
    Ip4Unicast(SdEndpointOption),
    /// Ip4 multicast endpoint option
    Ip4Multicast(SdEndpointOption),
    /// Ip6 unicast endpoint option
    Ip6Unicast(SdEndpointOption),
    /// Ip6 multicast endpoint option
    Ip6Multicast(SdEndpointOption),
}

impl SdOption {
    /// Returns true if this is a IP option
    pub fn is_ip_option(option_type: u8) -> bool {
        Self::is_ip4_option(option_type) || Self::is_ip6_option(option_type)
    }

    /// Returns true if this is a IPv4 option
    pub fn is_ip4_option(option_type: u8) -> bool {
        matches!(option_type, 0x04 | 0x14)
    }

    /// Returns true if this is a IPv6 option
    pub fn is_ip6_option(option_type: u8) -> bool {
        matches!(option_type, 0x06 | 0x16)
    }

    /// Returns true if this is a multicast option
    pub fn is_multicast_option(option_type: u8) -> bool {
        matches!(option_type, 0x14 | 0x16)
    }

    /// Construct a new SD option from type and endpoint
    pub fn from(option_type: u8, option: SdEndpointOption) -> Result<Self, Error> {
        use SdOption::*;
        match option_type {
            0x04 => Ok(Ip4Unicast(option)),
            0x14 => Ok(Ip4Multicast(option)),
            0x06 => Ok(Ip6Unicast(option)),
            0x16 => Ok(Ip6Multicast(option)),
            option_type => Err(Error::UnknownSdOption(option_type)),
        }
    }

    /// Length of the option in bytes
    pub(crate) fn len(&self) -> usize {
        use SdOption::*;
        match self {
            Ip4Unicast(_) => 12,
            Ip4Multicast(_) => 12,
            Ip6Unicast(_) => 24,
            Ip6Multicast(_) => 24,
        }
    }
}

/// Transforms a SdOption to a u8 representing the type.
impl From<&SdOption> for u8 {
    fn from(option: &SdOption) -> Self {
        use SdOption::*;
        match option {
            Ip4Unicast(_) => 0x04,
            Ip4Multicast(_) => 0x14,
            Ip6Unicast(_) => 0x06,
            Ip6Multicast(_) => 0x16,
        }
    }
}

/// Represents a SdEndpointOption within a SdPayload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SdEndpointOption {
    /// IP Address
    pub ip: IpAddr,
    /// Port number
    pub port: u16,
    /// IP Protocol
    pub proto: IpProto,
}

/// Different kinds of IpProto accepted in a SdEndpointOption.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpProto {
    /// User Datagram Protocol (UDP)
    UDP,
    /// Transmission Control Protocol (TCP)
    TCP,
}

/// Transforms a u8 representing the type to a IpProto.
impl TryFrom<u8> for IpProto {
    type Error = Error;

    fn try_from(i: u8) -> Result<Self, Error> {
        match i {
            0x11 => Ok(Self::UDP),
            0x06 => Ok(Self::TCP),
            value => Err(Error::InvalidIpProto(value)),
        }
    }
}

/// Transforms a IpProto to a u8 representing the type.
impl From<IpProto> for u8 {
    fn from(i: IpProto) -> u8 {
        use IpProto::*;
        match i {
            UDP => 0x11,
            TCP => 0x06,
        }
    }
}

#[cfg(feature = "url")]
impl From<SdEndpointOption> for url::Url {
    fn from(option: SdEndpointOption) -> url::Url {
        let port = option.port;
        let scheme = match option.proto {
            IpProto::UDP => "udp",
            IpProto::TCP => "tcp",
        };
        let url = match option.ip {
            IpAddr::V4(ip) => {
                format!("{}://{}:{}", scheme, ip, port)
            }
            IpAddr::V6(ip) => {
                format!("{}://[{}]:{}", scheme, ip, port)
            }
        };

        url::Url::parse(&url).unwrap() // safe - url constructed from known values
    }
}

#[cfg(feature = "url")]
impl TryFrom<url::Url> for SdEndpointOption {
    type Error = Error;

    fn try_from(url: url::Url) -> Result<Self, Self::Error> {
        let ip: IpAddr = url
            .host()
            .ok_or(Error::InvalidUrl("invalid URL: missing host"))
            .and_then(|host| match host {
                url::Host::Domain(domain) => domain
                    .parse::<IpAddr>()
                    .map_err(|_| Error::InvalidUrl("invalid URL: host not an ip")),
                url::Host::Ipv4(ip) => Ok(ip.into()),
                url::Host::Ipv6(ip) => Ok(ip.into()),
            })?;

        let port = url
            .port()
            .ok_or(Error::InvalidUrl("invalid URL: missing port"))?;

        let proto = match url.scheme() {
            "tcp" => Ok(IpProto::TCP),
            "udp" => Ok(IpProto::UDP),
            _ => Err(Error::InvalidUrl("invalid URL: invalid scheme")),
        }?;

        Ok(SdEndpointOption { ip, port, proto })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(feature = "url")]
    use std::net::Ipv6Addr;
    use std::net::{IpAddr, Ipv4Addr};
    use std::str::FromStr;

    #[cfg(test)]
    mod return_code {
        proptest::proptest! {
            #[test]
            fn try_from_u8(generic in 0x0bu8..0x1f, specific in 0x20u8..0x5e, error in 0x60u8..0xff) {
                use super::ReturnCode;
                use super::Error;

                let values = [
                    (0x00, Ok(ReturnCode::Ok)),
                    (0x01, Ok(ReturnCode::NotOk)),
                    (0x02, Ok(ReturnCode::UnknownService)),
                    (0x03, Ok(ReturnCode::UnknownMethod)),
                    (0x04, Ok(ReturnCode::NotReady)),
                    (0x05, Ok(ReturnCode::NotReachable)),
                    (0x06, Ok(ReturnCode::Timeout)),
                    (0x07, Ok(ReturnCode::WrongProtocolVersion)),
                    (0x08, Ok(ReturnCode::WrongInterfaceVersion)),
                    (0x09, Ok(ReturnCode::MalformedMessage)),
                    (0x0a, Ok(ReturnCode::WrongMessageType)),
                    (generic, Ok(ReturnCode::ReservedGeneric(generic))),
                    (specific, Ok(ReturnCode::ReservedSpecific(specific))),
                    (error, Err(Error::InvalidReturnCode(error))),
                ];
                for (input, expected) in values.iter() {
                    let result = ReturnCode::try_from(*input);
                    // TODO: Error cannot implmenet PartialEq because of io::Error
                    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
                }
            }
        }
    }

    #[test]
    fn header_builder_smoke() {
        super::HeaderBuilder::default()
            .message_id(MessageId::new(0x1234, 0x1234))
            .length(0x1234)
            .request_id(RequestId::new(0x1234, 0x1234))
            .protocol_version(1)
            .interface_version(1)
            .message_type(MessageType::Request)
            .return_code(ReturnCode::NotOk)
            .build()
            .unwrap();
    }

    #[allow(unused)]
    fn sd_payload() {
        let payload = SdPayload {
            flags: 0x00,
            entries: vec![],
            options: vec![],
        };

        assert!(!payload.reboot_flag());
        assert!(!payload.unicast_flag());
        assert!(payload.options(0).is_empty());

        let payload = SdPayload {
            flags: 0xC0,
            entries: vec![
                SdEntry::OfferService(SdServiceEntry {
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
                }),
                SdEntry::OfferService(SdServiceEntry {
                    service_id: 0x0103,
                    instance_id: 0x0002,
                    major_version: 0x02,
                    minor_version: 0x0000000A,
                    ttl: 0x00000003,
                    options: SdOptionRef {
                        index1: 2,
                        index2: 3,
                        num1: 1,
                        num2: 1,
                    },
                }),
            ],
            options: vec![
                SdOption::Ip4Unicast(SdEndpointOption {
                    ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                    port: 30000,
                    proto: IpProto::UDP,
                }),
                SdOption::Ip4Unicast(SdEndpointOption {
                    ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                    port: 30001,
                    proto: IpProto::UDP,
                }),
                SdOption::Ip4Unicast(SdEndpointOption {
                    ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                    port: 30002,
                    proto: IpProto::UDP,
                }),
                SdOption::Ip4Unicast(SdEndpointOption {
                    ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                    port: 30003,
                    proto: IpProto::UDP,
                }),
            ],
        };

        assert!(payload.reboot_flag());
        assert!(payload.unicast_flag());

        let options = payload.options(0);
        assert_eq!(2, options.len());
        assert_eq!(30000, options.get(0).unwrap().port);
        assert_eq!(30001, options.get(1).unwrap().port);

        let options = payload.options(1);
        assert_eq!(2, options.len());
        assert_eq!(30002, options.get(0).unwrap().port);
        assert_eq!(30003, options.get(1).unwrap().port);
    }

    #[cfg(feature = "url")]
    #[test]
    fn endpoint_url_v4() {
        let option = SdEndpointOption {
            ip: Ipv4Addr::LOCALHOST.into(),
            port: 1234,
            proto: IpProto::TCP,
        };
        let url: url::Url = option.clone().into();
        assert_eq!(option, url.try_into().unwrap());
    }

    #[cfg(feature = "url")]
    #[test]
    fn endpoint_url_v6() {
        let option = SdEndpointOption {
            ip: Ipv6Addr::LOCALHOST.into(),
            port: 5555,
            proto: IpProto::UDP,
        };
        let url: url::Url = option.clone().into();
        assert_eq!(option, url.try_into().unwrap());
    }
}
