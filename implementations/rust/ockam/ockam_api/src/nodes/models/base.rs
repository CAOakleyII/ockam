//! Nodemanager API types

// TODO: split up this file into sub modules

use minicbor::{Decode, Encode};
use ockam_core::CowStr;

#[cfg(feature = "tag")]
use ockam_core::TypeTag;

use super::{services::ServiceList, identity::ShortIdentityResponse, transport::TransportList, portal::{InletList, OutletList}};

///////////////////-!  REQUEST BODIES
/// Request body when retrieving node status
#[derive(Debug, Clone, Decode, Encode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct GetNodeStatusRequest {
    #[cfg(feature = "tag")]
    #[n(0)] tag: TypeTag<2222222>,
    #[b(1)] pub detailed: Option<bool>,
}

impl GetNodeStatusRequest {
    pub fn new(
        detailed: Option<bool>
    ) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            detailed
        }
    }
}

///////////////////-!  RESPONSE BODIES

/// Extra Details for a Node
#[derive(Debug, Clone, Decode, Encode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct NodeDetails<'a> {
    #[cfg(feature = "tag")]
    #[n(0)] tag: TypeTag<6586551>,
    #[b(1)] pub status: CowStr<'a>,
    #[n(2)] pub short_identity: ShortIdentityResponse<'a>,
    #[n(3)] pub services: ServiceList<'a>,
    #[n(4)] pub transport_list: TransportList<'a>,
    #[n(5)] pub secure_channel_listeners: Vec<String>,
    #[n(6)] pub inlets: InletList<'a>,
    #[n(7)] pub outlets: OutletList<'a>,
}

impl <'a> NodeDetails<'a> {
    pub fn new(
        status: impl Into<CowStr<'a>>,
        short_identity: ShortIdentityResponse<'a>,
        services: ServiceList<'a>,
        transport_list: TransportList<'a>,
        secure_channel_listeners: Vec<String>,
        inlets: InletList<'a>,
        outlets:OutletList<'a>,
    ) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            status: status.into(),
            short_identity,
            services,
            transport_list,
            secure_channel_listeners,
            inlets,
            outlets
        }
    }
}

/// Response body for a node status
#[derive(Debug, Clone, Decode, Encode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct NodeStatus<'a> {
    #[cfg(feature = "tag")]
    #[n(0)] tag: TypeTag<6586555>,
    #[b(1)] pub node_name: CowStr<'a>,
    #[b(2)] pub status: CowStr<'a>,
    #[n(3)] pub workers: u32,
    #[n(4)] pub pid: i32,
    #[n(5)] pub transports: u32,
    #[n(6)] pub details: Option<NodeDetails<'a>>,
}

impl<'a> NodeStatus<'a> {
    pub fn new(
        node_name: impl Into<CowStr<'a>>,
        status: impl Into<CowStr<'a>>,
        workers: u32,
        pid: i32,
        transports: u32,
        details: Option<NodeDetails<'a>>,
    ) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            node_name: node_name.into(),
            status: status.into(),
            workers,
            pid,
            transports,
            details,
        }
    }
}

/// Response Body for listing nodes
#[derive(Debug, Clone, Decode, Encode)]
#[rustfmt::skip]
#[cbor(map)]
pub struct NodeList<'a> {
    #[cfg(feature = "tag")]
    #[n(0)] tag: TypeTag<5432123>,
    #[b(1)] pub list: Vec<NodeStatus<'a>>
}

impl<'a> NodeList<'a> {
    pub fn new(
        list: Vec<NodeStatus<'a>>
    ) -> Self {
        Self {
            #[cfg(feature = "tag")]
            tag: TypeTag,
            list,
        }
    }
}