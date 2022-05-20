use futures::stream::TryStreamExt;
use rtnetlink::{new_connection, Error, Handle};
use std::process::Command;

fn clean_env_for_update_vlan() {
    // ip link del veth0
    Command::new("ip")
        .args(["link", "del", "veth0"])
        .output()
        .expect("failed to clean update vlan env")
}

fn prepare_env_for_update_vlan() {
    clean_env_for_update_vlan();
    // ip link add type veth
    Command::new("ip")
        .args(["link", "add", "type", "veth"])
        .output()
        .expect("failed to add veth pair");

    // ip link set veth0 up
    Command::new("ip")
        .args(["link", "set", "veth0", "up"])
        .output()
        .expect("failed to up veth0");

    // ip link set veth1 up
    Command::new("ip")
        .args(["link", "set", "veth1", "up"])
        .output()
        .expect("failed to up veth1");
}

#[tokio::main]
async fn main() -> Result<(), String> {
    // let args: Vec<String> = env::args().collect();
    // if args.len() != 2 {
    //     usage();
    //     return Ok(());
    // }
    let link_name = "veth0";
    prepare_env_for_update_vlan();

    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    create_vlan(handle, link_name.to_string())
        .await
        .map_err(|e| format!("{}", e))
}

async fn create_vlan(handle: Handle, veth_name: String) -> Result<(), Error> {
    let mut links = handle.link().get().match_name(veth_name.clone()).execute();
    if let Some(link) = links.try_next().await? {
        // hard code mode: 4u32 i.e bridge mode
        let request = handle
            .link()
            .add()
            .vlan("vlan.3000", link.header.index, vlan_id);
        request.execute().await?
    } else {
        println!("no link link {} found", veth_name);
    }
    Ok(())
}

// fn usage() {
//     eprintln!(
//         "usage:
//     cargo run --example create_macvlan -- <link name>
// Note that you need to run this program as root. Instead of running cargo as root,
// build the example normally:
//     cd netlink-ip ; cargo build --example create_macvlan
// Then find the binary in the target directory:
//     cd ../target/debug/example ; sudo ./create_macvlan <link_name>"
//     );
// }
