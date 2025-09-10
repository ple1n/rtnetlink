// SPDX-License-Identifier: MIT

use std::net::Ipv4Addr;

use futures::{
    future::{self, Either},
    stream::{Stream, StreamExt},
    FutureExt,
};
use netlink_packet_core::{NetlinkMessage, NLM_F_DUMP, NLM_F_REQUEST};
use netlink_packet_route::{
    RouteFlags, RouteMessage, RtnlMessage, AF_INET, AF_INET6, RTN_UNSPEC,
    RTPROT_UNSPEC, RT_SCOPE_UNIVERSE, RT_TABLE_UNSPEC,
    route::nlas::Nla,
    route::{RouteAttribute, RouteMessage},
    AddressFamily, RouteNetlinkMessage,
};

use crate::{try_rtnl, Error, Handle};

#[derive(Debug, Clone)]
pub struct RouteGetRequest {
    handle: Handle,
    message: RouteMessage,
}

/// Internet Protocol (IP) version.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub enum IpVersion {
    /// IPv4
    V4,
    /// IPv6
    V6,
}

impl IpVersion {
    pub(crate) fn family(self) -> AddressFamily {
        match self {
            IpVersion::V4 => AddressFamily::Inet,
            IpVersion::V6 => AddressFamily::Inet6,
        }
    }
}

impl RouteGetRequest {
    pub(crate) fn new(handle: Handle, message: RouteMessage) -> Self {
        RouteGetRequest { handle, message }
    }

    pub fn message_mut(&mut self) -> &mut RouteMessage {
        &mut self.message
    }

    pub fn lookup(self) -> impl TryStream<Ok = RouteMessage, Error = Error> {
        let RouteGetRequest {
            mut handle,
            message,
        } = self;

        let mut req = NetlinkMessage::from(RtnlMessage::GetRoute(message));
        req.header.flags = NLM_F_REQUEST;

        match handle.request(req) {
            Ok(response) => Either::Left(
                response
                    .map(move |msg| Ok(try_rtnl!(msg, RtnlMessage::NewRoute))),
            ),
            Err(e) => Either::Right(
                future::err::<RouteMessage, Error>(e).into_stream(),
            ),
        }
    }

    pub fn execute(self) -> impl TryStream<Ok = RouteMessage, Error = Error> {
        let RouteGetRequest {
            mut handle,
            message,
        } = self;

        let has_dest = message
            .attributes
            .iter()
            .any(|attr| matches!(attr, RouteAttribute::Destination(_)));

        let mut req =
            NetlinkMessage::from(RouteNetlinkMessage::GetRoute(message));
        req.header.flags = NLM_F_REQUEST;

        if !has_dest {
            req.header.flags |= NLM_F_DUMP;
        }

        match handle.request(req) {
            Ok(response) => Either::Left(response.map(move |msg| {
                Ok(try_rtnl!(msg, RouteNetlinkMessage::NewRoute))
            })),
            Err(e) => Either::Right(
                future::err::<RouteMessage, Error>(e).into_stream(),
            ),
        }
    }
}

pub struct RouteGetResolve<IP> {
    handle: Handle,
    message: RouteMessage,
    ip: IP,
}

impl RouteGetResolve<Ipv4Addr> {
    pub fn new(handle: Handle, ip: Ipv4Addr) -> Self {
        let mut message = RouteMessage::default();
        message.header.address_family = IpVersion::V4.family();
        message.header.destination_prefix_length = 32;
        message.header.source_prefix_length = 0;
        message.header.flags = RouteFlags::RTM_F_LOOKUP_TABLE;
        message.header.scope = RT_SCOPE_UNIVERSE;
        message.header.kind = RTN_UNSPEC;
        message.header.table = RT_TABLE_UNSPEC;
        message.header.protocol = RTPROT_UNSPEC;
        message
            .nlas
            .push(Nla::Destination(ip.octets().to_vec()));
        RouteGetResolve {
            handle,
            message,
            ip,
        }
    }
}

impl<T> RouteGetResolve<T> {
    pub fn message_mut(&mut self) -> &mut RouteMessage {
        &mut self.message
    }

    pub fn lookup(self) -> impl TryStream<Ok = RouteMessage, Error = Error> {
        let RouteGetResolve {
            mut handle,
            message,
            ip,
        } = self;

        let mut req = NetlinkMessage::from(RtnlMessage::GetRoute(message));
        req.header.flags = NLM_F_REQUEST;

        match handle.request(req) {
            Ok(response) => Either::Left(
                response
                    .map(move |msg| Ok(try_rtnl!(msg, RtnlMessage::NewRoute))),
            ),
            Err(e) => Either::Right(
                future::err::<RouteMessage, Error>(e).into_stream(),
            ),
        }
    }
}
