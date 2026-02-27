use amdkfd::*;

#[test]
fn create_queue() {
    let kfd = Kfd::open().unwrap();
    let gpu_aperture = kfd.devices().unwrap()[0];
    let kfd_node = KfdNode::from_aperture(&kfd, &gpu_aperture);

    let _ = kfd_node;
    // let eop_buffer = todo!();
    // let ctx_save_restore_buffer = todo!();
}
