use amdkfd::*;

#[test]
fn userptr() {
    let kfd = Kfd::open().unwrap();
    let apertures = kfd.devices().unwrap();
    let kfd_node = KfdNode::from_aperture(&kfd, &apertures[0]);

    let mut drm_file = AmdgpuDrm {
        file: std::fs::File::open("/dev/dri/renderD128").unwrap(),
    };

    let (kfd_node,) = unsafe { kfd_node.acquire_vm(&mut drm_file) }.unwrap();
    let mut mem = [0u8; 4096 * 4];

    let (kfd_node, gpu_mem) = kfd_node
        .allocate_userptr_backed_memory(&mut mem, 0x1000)
        .unwrap();

    let mut mem2 = [0u8; 4096 * 4];
    let (kfd_node, gpu_mem2) = kfd_node
        .allocate_userptr_backed_memory(&mut mem2, 0x5000)
        .unwrap();

    let device_ids: Vec<_> = apertures.iter().map(|x| x.gpu_id).collect();
    let (gpu_mem,) = gpu_mem.map_memory(&device_ids).unwrap();

    let (gpu_mem2,) = gpu_mem2.map_memory(&device_ids).unwrap();
    drop(gpu_mem);
    drop(gpu_mem2);
    drop(kfd_node);
}
