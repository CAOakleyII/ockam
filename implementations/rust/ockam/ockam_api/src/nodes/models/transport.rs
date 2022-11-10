use minicbor::{Decode, Encode};
use ockam_abac::ParseError;
use ockam_core::CowStr;
use std::{fmt::{self, Display}, str::FromStr};

#[cfg(feature = "tag")]
use ockam_core::TypeTag;

///////////////////-!  REQUEST BODIES

/// Request body when instructing a node to create a transport
#[derive(Debug, Clone, Decode, Encode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct CreateTransport<'a> {
    #[cfg(feature = "tag")]
    #[n(0)] tag: TypeTag<1503320>,
    /// The type of transport to create
    #[n(1)] pub tt: TransportType,
    /// The mode the transport should operate in
    #[n(2)] pub tm: TransportMode,
    /// The address payload for the transport
    #[b(3)] pub addr: CowStr<'a>,
}

impl<'a> CreateTransport<'a> {
    pub fn new<S: Into<CowStr<'a>>>(tt: TransportType, tm: TransportMode, addr: S) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            tt,
            tm,
            addr: addr.into(),
        }
    }
}

/// Request body when retrieving list of transports
#[derive(Debug, Clone, Decode, Encode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct GetTransportList {
    #[cfg(feature = "tag")]
    #[n(0)] tag: TypeTag<3333333>,
    #[b(1)] pub tts: Option<Vec<TransportType>>,
    #[b(2)] pub tms: Option<Vec<TransportMode>>,
}

impl GetTransportList {
    pub fn new(
        tts:  Option<Vec<TransportType>>,
        tms: Option<Vec<TransportMode>>,
    ) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            tts,
            tms,
        }
    }
}

/// Request to delete a transport
#[derive(Debug, Clone, Decode, Encode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct DeleteTransport<'a> {
    #[cfg(feature = "tag")]
    #[n(0)] tag: TypeTag<4739996>,
    /// The transport ID to delete
    #[b(1)] pub tid: CowStr<'a>,
    /// The user has indicated that deleting the API transport is A-OK
    #[n(2)] pub force: bool,
}

impl<'a> DeleteTransport<'a> {
    pub fn new<S: Into<CowStr<'a>>>(tid: S, force: bool) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            tid: tid.into(),
            force,
        }
    }
}

/// Encode which type of transport is being requested
// TODO: we have a TransportType in ockam_core.  Do we really want to
// mirror this kind of type here?
#[derive(Copy, Clone, Debug, Decode, Encode, PartialEq, Eq)]
#[rustfmt::skip]
#[cbor(index_only)]
pub enum TransportType {
    /// Ockam TCP transport
    #[n(0)] Tcp,
    /// Embedded BLE transport
    #[n(1)] Ble,
    /// Websocket transport
    #[n(2)] WebSocket,
}

impl Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Tcp => "TCP",
            Self::Ble => "BLE",
            Self::WebSocket => "Websocket",
        })
    }
}

impl FromStr for TransportType {
    type Err = ockam_core::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TCP" => Ok(TransportType::Tcp),
            "BLE" => Ok(TransportType::Ble),
            "WEBSOCKET" => Ok(TransportType::WebSocket),
            "WS" => Ok(TransportType::WebSocket),
            s => {
                let error = ParseError::message(format!("'{}' is not a valid TransportType", s));
                Err(ockam_core::Error::from(error))
            }
        }
    }
}

/// Encode which type of transport is being requested
#[derive(Copy, Clone, Debug, Decode, Encode, PartialEq, Eq)]
#[rustfmt::skip]
pub enum TransportMode {
    /// Listen on a set address
    #[n(0)] Listen,
    /// Connect to a remote peer
    #[n(1)] Connect,
}

impl Display for TransportMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Listen => "Listening",
            Self::Connect => "Remote connection",
        })
    }
}

impl FromStr for TransportMode {
    type Err = ockam_core::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CONNECT" => Ok(TransportMode::Connect),
            "REMOTE CONNECTION" => Ok(TransportMode::Connect),
            "LISTEN" => Ok(TransportMode::Listen),
            "LISTENING" => Ok(TransportMode::Listen),
            s => {
                let error = ParseError::message(format!("'{}' is not a valid TransportMode", s));
                Err(ockam_core::Error::from(error))
            }
        }
    }
}

///////////////////-!  RESPONSE BODIES

/// Response body when interacting with a transport
#[derive(Debug, Clone, Decode, Encode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct TransportStatus<'a> {
    #[cfg(feature = "tag")]
    #[n(0)] tag: TypeTag<1581592>,
    /// The type of transport to create
    #[n(2)] pub tt: TransportType,
    /// The mode the transport should operate in
    #[n(3)] pub tm: TransportMode,
    /// The status payload
    #[b(4)] pub payload: CowStr<'a>,
    /// Transport ID inside the node manager
    ///
    /// We use this as a kind of URI to be able to address a transport
    /// by a unique value for specific updates and deletion events.
    #[b(5)] pub tid: CowStr<'a>,
}

impl<'a> TransportStatus<'a> {
    pub fn new<S: Into<CowStr<'a>>>(
        tt: TransportType,
        tm: TransportMode,
        payload: S,
        tid: S,
    ) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            tt,
            tm,
            payload: payload.into(),
            tid: tid.into(),
        }
    }
}

/// Response body when interacting with a transport
#[derive(Debug, Clone, Decode, Encode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct TransportList<'a> {
    #[cfg(feature = "tag")]
    #[n(0)] tag: TypeTag<5212817>,
    #[b(1)] pub list: Vec<TransportStatus<'a>>
}

impl<'a> TransportList<'a> {
    pub fn new(list: Vec<TransportStatus<'a>>) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            list,
        }
    }
}
