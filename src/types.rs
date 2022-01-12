/// Represents the header of a SomeIP message
#[derive(Debug, PartialEq)]
pub struct SomeIpHeader {
    pub message_id: MessageID,
    pub length: u32,
    pub request_id: RequestID,
    pub protocol_version: u8,
    pub interface_version: u8,
    pub message_type: MessageType,
    pub return_code: ReturnCode,
}

/// Represents a basic SomeIP message
#[derive(Debug, PartialEq)]
pub struct SomeIpMessage<'a> {
    pub header: SomeIpHeader,
    pub payload: &'a [u8],
}

/// Different types of supported SomeIP messages
#[derive(Debug, PartialEq)]
pub enum SomeIp<'a> {
    SomeIpMessage(SomeIpMessage<'a>),
    SomeIpMagicCookieClient,
    SomeIpMagicCookieServer,
}

/// Represents the MessageID within the SomeIP header.
#[derive(Debug, Default, PartialEq)]
pub struct MessageID {
    pub service_id: ServiceID,
    pub method_id: MethodID,
}

/// Represents the MethodID within the MessageID
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct MethodID(pub u16);

/// Represents the ServiceID within the MessageID
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ServiceID(pub u16);

/// Represents the RequestID within the SomeIP header
#[derive(Debug, Default, PartialEq)]
pub struct RequestID {
    pub(crate) client_id: ClientID,
    pub(crate) session_id: SessionID,
}

/// Represents the ClientID within the RequestID
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct ClientID(pub u16);

/// Represents the SessionID within the RequestID
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SessionID(pub u16);

/// Different kinds of MessagesTypes accepted in a SomeIP header
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageType {
    Request,
    RequestNoReturn,
    Notification,
    RequestACK,
    Response,
    Error,
    // RequestNoReturnACK,
    // ResponseACK,
    // ErrorACK,
    TpRequest,
    TpRequestNoReturn,
    TpNotification,
    TpResponse,
    TpError,
}

/// Different kinds of ReturnCodes accepted in a SomeIP header
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReturnCode {
    EOk,
    NotOk,
    UnknownService,
    UnknownMethod,
    NotReady,
    NotReachable,
    Timeout,
    WrongProtocolVersion,
    WrongInterfaceVersion,
    MalformedMessage,
    WrongMessageType,
    E2eRepeated,
    E2eWrongSequence,
    E2e,
    E2eNotAvailable,
    E2eNoNewData,
    // UnknownType
}

/// Different kinds of EntriesTyp accepted in a SomeIP header
pub enum EntriesType {
    FindService,
    OfferService,
    RequestService,
    RequestServiceACK,
    FindEventgroup,
    PublishEventgroup,
    SubscribeEventgroup,
    SubscribeEventgroupACK,
}

/// Transforms an u16 to a ServiceID
impl From<u16> for ServiceID {
    fn from(i: u16) -> Self {
        ServiceID(i)
    }
}

/// Transforms a ServiceId to an u16
impl From<ServiceID> for u16 {
    fn from(i: ServiceID) -> Self {
        i.0
    }
}

/// Transforms an u16 to a MethodID
impl From<u16> for MethodID {
    fn from(i: u16) -> Self {
        MethodID(i)
    }
}

/// Transforms a MethodID to an u16
impl From<MethodID> for u16 {
    fn from(i: MethodID) -> Self {
        i.0
    }
}

/// Transforms an u16 to a ClientID
impl From<u16> for ClientID {
    fn from(i: u16) -> Self {
        ClientID(i)
    }
}

/// Transforms a ClientId to an u16
impl From<ClientID> for u16 {
    fn from(i: ClientID) -> Self {
        i.0
    }
}

/// Transforms an u16 to a SessionID
impl From<u16> for SessionID {
    fn from(i: u16) -> Self {
        SessionID(i)
    }
}

/// Transforms a SessionID to an u16
impl From<SessionID> for u16 {
    fn from(i: SessionID) -> Self {
        i.0
    }
}

/// Transforms a byte slice to a MessageType
impl From<&[u8]> for MessageType {
    fn from(i: &[u8]) -> Self {
        use MessageType::*;
        match i {
            [0x00] => Request,
            [0x01] => RequestNoReturn,
            [0x02] => Notification,
            [0x80] => Response,
            [0x81] => Error,
            [0x20] => TpRequest,
            [0x21] => TpRequestNoReturn,
            [0x22] => TpNotification,
            [0xa0] => TpResponse,
            [0xa1] => TpError,
            value => unimplemented!("MessageType {:?} not implemented", value),
        }
    }
}

/// Transforms a MessageType to a bytes slice
impl<'a> From<MessageType> for &'a [u8] {
    fn from(i: MessageType) -> &'a [u8] {
        use MessageType::*;
        match i {
            Request => &[0x00],
            RequestNoReturn => &[0x01],
            Notification => &[0x02],
            Response => &[0x80],
            Error => &[0x81],
            TpRequest => &[0x20],
            TpRequestNoReturn => &[0x21],
            TpNotification => &[0x22],
            TpResponse => &[0xa0],
            TpError => &[0xa1],
            value => unimplemented!("MessageType for {:?} not implemented", value),
        }
    }
}

/// Transforms a MessageType to a u8
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
            value => unimplemented!("MessageType for {:?} not implemented", value),
        }
    }
}

/// Transforms a byte slice to a ReturnCode
impl From<&[u8]> for ReturnCode {
    fn from(i: &[u8]) -> Self {
        use ReturnCode::*;
        match i {
            [0x00] => EOk,
            [0x01] => NotOk,
            [0x02] => UnknownService,
            [0x03] => UnknownMethod,
            [0x04] => NotReady,
            [0x05] => NotReachable,
            [0x06] => Timeout,
            [0x07] => WrongProtocolVersion,
            [0x08] => WrongInterfaceVersion,
            [0x09] => MalformedMessage,
            [0x0a] => WrongMessageType,
            [0x0b] => E2eRepeated,
            [0x0c] => E2eWrongSequence,
            [0x0d] => E2e,
            [0x0e] => E2eNotAvailable,
            [0x0f] => E2eNoNewData,
            value => unimplemented!("MessageType {:?} not implemented", value),
        }
    }
}

/// Transforms a byte slice to a ReturnCode
impl From<ReturnCode> for u8 {
    fn from(i: ReturnCode) -> Self {
        use ReturnCode::*;
        match i {
            EOk => 0x00,
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
            E2eRepeated => 0x0b,
            E2eWrongSequence => 0x0c,
            E2e => 0x0d,
            E2eNotAvailable => 0x0e,
            E2eNoNewData => 0x0f,
        }
    }
}
