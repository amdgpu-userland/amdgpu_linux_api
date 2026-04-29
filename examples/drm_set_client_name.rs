use amdgpu_linux_api::drm::{
    AmdgpuDrmPrimary3_64, AmdgpuDrmRender3_64, ClientName, set_client_name,
};

fn main() {
    let drm = AmdgpuDrmRender3_64::open(128).unwrap();
    set_client_name(&drm, const { ClientName::new("awesome_client!") });
    let drm_primary = AmdgpuDrmPrimary3_64::open(1).unwrap();
    set_client_name(&drm_primary, const { ClientName::new("primary_one") });
    println!(
        "Check /sys/kernel/debug/dri/0/clients for two clients with custom names.
Hit enter to exit."
    );
    let _ = std::io::stdin().read_line(&mut String::new());
}
