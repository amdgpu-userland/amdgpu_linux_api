#[path = "common/helpers.rs"] mod helpers;
use helpers::RawPrimaryNode;
use helpers::RawRenderNode;
use amdgpu_linux_api::drm::{
    ClientName, set_client_name,
};

fn main() {
    let drm = RawRenderNode::open(128).unwrap();
    set_client_name(&drm, const { ClientName::new("awesome_client!") });
    let drm_primary = RawPrimaryNode::open(1).unwrap();
    set_client_name(&drm_primary, const { ClientName::new("primary_one") });
    println!(
        "Check /sys/kernel/debug/dri/0/clients for two clients with custom names.
Hit enter to exit."
    );
    let _ = std::io::stdin().read_line(&mut String::new());
}
