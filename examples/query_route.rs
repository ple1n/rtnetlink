// SPDX-License-Identifier: MIT

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use futures::stream::TryStreamExt;
use rtnetlink::{
    new_connection, Error, Handle, IpVersion, RouteMessageBuilder,
};

#[tokio::main]
async fn main() -> Result<(), ()> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    println!("get routes for IPv4");
    if let Err(e) =
        dump_addresses(handle.clone(), "1.1.1.1".parse().unwrap()).await
    {
        eprintln!("{e}");
    }
    if let Err(e) =
        dump_addresses(handle.clone(), "127.0.0.1".parse().unwrap()).await
    {
        eprintln!("{e}");
    }
    println!();

    println!("get routes for IPv6");
    if let Err(e) =
        dump_addresses(handle.clone(), "fe80::1".parse().unwrap()).await
    {
        eprintln!("{e}");
    }
    if let Err(e) =
        dump_addresses(handle.clone(), "2409::1".parse().unwrap()).await
    {
        eprintln!("{e}");
    }
    println!();

    Ok(())
}

async fn dump_addresses(handle: Handle, ip: IpAddr) -> Result<(), Error> {
    let route = match ip {
        IpAddr::V4(ip) => RouteMessageBuilder::<Ipv4Addr>::new()
            .destination(ip, 32)
            .build(),
        IpAddr::V6(ip) => RouteMessageBuilder::<Ipv6Addr>::new()
            .destination(ip, 128)
            .build(),
    };
    let mut routes = handle.route().get(route).execute();
    while let Some(route) = routes.try_next().await? {
        println!("{route:?}");
    }
    Ok(())
}
