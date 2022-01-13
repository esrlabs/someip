use crate::Error;

/// Represents the header of a SomeIP message
#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub message_id: MessageId,
    pub length: u32,
    pub request_id: RequestId,
    pub protocol_version: ProtocolVersion,
    pub interface_version: InterfaceVersion,
    pub message_type: MessageType,
    pub return_code: ReturnCode,
}

/// Different types of supported SomeIP messages
#[derive(Debug, PartialEq)]
pub enum Message<'a> {
    /// Message
    Message(Header, &'a [u8]),
    /// Magic Cookie Client
    /// RS_SOMEIP_00010
    MagicCookieClient,
    /// Magic Cookie Server
    /// RS_SOMEIP_00010
    MagicCookieServer,
}

/// Represents the MessageID within the SomeIP header.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MessageId {
    pub service_id: ServiceId,
    pub method_id: MethodId,
}

/// Represents the MethodID within the MessageID
pub type MethodId = u16;

/// Represents the ServiceID within the MessageID
pub type ServiceId = u16;

/// Represents the RequestID within the SomeIP header
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RequestId {
    pub client_id: ClientId,
    pub session_id: SessionId,
}

/// Protocol version
pub type ProtocolVersion = u8;

/// Interface version
pub type InterfaceVersion = u8;

/// Represents the ClientID within the RequestID
pub type ClientId = u16;

/// Represents the SessionID within the RequestID
pub type SessionId = u16;

/// Different kinds of MessagesTypes accepted in a SomeIP header
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageType {
    Request,
    RequestNoReturn,
    Notification,
    Response,
    Error,
    TpRequest,
    TpRequestNoReturn,
    TpNotification,
    TpResponse,
    TpError,
}

/// Different kinds of ReturnCodes accepted in a SomeIP header
/// RS_SOMEIP_00008
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

/// Transforms a byte slice to a MessageType
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
            value => Err(Error::UnknownMessageType(value)),
        }
    }
}

/// Transforms a MessageType to a bytes slice
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

/// Transforms a byte slice to a ReturnCode
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
            value => Err(Error::UnknownReturnCode(value)),
        }
    }
}

/// Transforms a byte slice to a ReturnCode
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

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    mod return_code {
        proptest::proptest! {
            #[test]
            fn try_from_u8(generic in 0x0bu8..0x1f, specific in 0x20u8..0x5e, error in 0x60u8..0xff) {
                use super::ReturnCode;
                use super::Error;

                let values = [
                    (0x00u8, Ok(ReturnCode::Ok)),
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
                    (error, Err(Error::UnknownReturnCode(error))),
                ];
                for (input, expected) in values.iter() {
                    let result = ReturnCode::try_from(*input);
                    // TODO: Error cannot implmenet PartialEq because of io::Error
                    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
                }
            }
        }
    }
}
