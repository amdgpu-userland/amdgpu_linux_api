use amdgpu_linux_api::drm::ioctl::amd::*;
use amdgpu_linux_api::drm::ioctl::drm::*;
use amdgpu_linux_api::drm::*;
use amdgpu_linux_api::kfd::apertures::*;
use amdgpu_linux_api::kfd::ioctl::*;
use amdgpu_linux_api::kfd::*;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;

fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let devs = kfd.all_apertures().unwrap();
    let drm = AmdgpuDrmRender3_64::open(128).unwrap();
    let kfd = match kfd.acquire_vm(&devs[0], &drm) {
        AcquireVmResult::Ok(x) => x,
        _ => panic!(),
    };
    let gpu_id = devs[0].gpu_id;
    let fd = kfd.as_fd().as_raw_fd();
    let mut args = AllocMemoryOfGpuArgs {
        va_addr: 0x10_000,
        size: 0x1000,
        handle: 0,
        mmap_offset: 0,
        gpu_id,
        flags: alloc_domain::VRAM,
    };
    let res = unsafe { alloc_memory_of_gpu(fd, &mut args) };
    assert!(res.is_ok());

    let handle = args.handle;
    println!("Allocating Vram in kfd, handle: {handle}");

    let mut args = ExportDmabufArgs {
        handle,
        flags: 0,
        dmabuf_fd: 0,
    };
    let res = unsafe { export_dmabuf(fd, &mut args) };
    assert!(res.is_ok());

    let dmabuf_fd = args.dmabuf_fd;
    println!("Exporting kfd's memory as dmabuf: {dmabuf_fd}");

    let mut args = ImportDmabufArgs {
        va_addr: 0x20_000,
        handle: 0,
        gpu_id,
        dmabuf_fd,
    };
    let res = unsafe { import_dmabuf(fd, &mut args) };
    assert!(res.is_ok());

    let handle2 = args.handle;
    println!("Importing dmabuf in kfd, handle: {handle2}");

    let dev_ids = [gpu_id];
    let mut args = MapMemoryToGpuArgs {
        handle: handle2,
        device_ids_array_ptr: dev_ids.as_ptr(),
        n_devices: dev_ids.len() as u32,
        n_success: 0,
    };
    let res = unsafe { map_memory_to_gpu(fd, &mut args) };
    assert!(res.is_ok());

    args.handle = handle;
    args.n_success = 0;
    let res = unsafe { map_memory_to_gpu(fd, &mut args) };
    assert!(res.is_ok());

    println!("Mapping the same memory to different VAs");

    let drm_fd = drm.as_fd().as_raw_fd();

    let mut args = GemCreate {
        input: GemCreateIn {
            bo_size: 0x1_000,
            alignment: 0,
            domains: gem_domain::GTT,
            domain_flags: 0,
        },
    };
    let res = unsafe { gem_create(drm_fd, &mut args) };
    assert!(res.is_ok());
    let gem_handle = unsafe { args.output.handle };

    println!("Allocating GTT in GEM, handle: {gem_handle}");
    let mut args = PrimeHandle {
        handle: gem_handle,
        flags: 0,
        fd: 0,
    };
    let res = unsafe { prime_handle_to_fd(drm_fd, &mut args) };
    assert!(res.is_ok());
    let gem_fd = args.fd;

    let mut args = ImportDmabufArgs {
        va_addr: 0x30_000,
        handle: 0,
        gpu_id,
        dmabuf_fd: gem_fd,
    };
    let res = unsafe { import_dmabuf(fd, &mut args) };
    assert!(res.is_ok());

    println!("Importing GEM dmabuf in KFD, handle: {}", args.handle);

    let mut args = MapMemoryToGpuArgs {
        handle: args.handle,
        device_ids_array_ptr: dev_ids.as_ptr(),
        n_devices: dev_ids.len() as u32,
        n_success: 0,
    };
    let res = unsafe { map_memory_to_gpu(fd, &mut args) };
    assert!(res.is_ok());
    println!("Mapped GEM dmabuf to VA in KFD");

    let mut args = PrimeHandle {
        handle: 0,
        flags: 0,
        fd: dmabuf_fd,
    };
    let res = unsafe { prime_fd_to_handle(drm_fd, &mut args) };
    assert!(res.is_ok());
    let gem_imported_handle = args.handle;
    println!("Importing kfd's dmabuf as GEM obj, gem_handle: {gem_imported_handle}",);

    let mut metadata = [0; 64];
    metadata[0] = 0xBAD;
    metadata[1] = 0x3;
    let mut args = GemMetadata {
        handle: gem_imported_handle,
        op: MetadataOp::Set,
        data: GemMetadataData {
            flags: 0x0CAFEBABE,
            tiling_info: 0,
            data: metadata,
            data_size_bytes: metadata.len() as u32,
        },
    };
    let res = unsafe { gem_metadata(drm_fd, &mut args) };
    assert!(res.is_ok());
    println!("Setting metadata on imported GEM");

    let mut metadata = [0u32; 64];
    let mut args = GetDmabufInfoArgs {
        size: 0,
        metadata_ptr: metadata.as_mut_ptr(),
        metadata_size: metadata.len() as u32,
        gpu_id,
        flags: 0,
        dmabuf_fd,
    };
    let res = unsafe { get_dmabuf_info(fd, &mut args) };
    assert!(res.is_ok());
    assert_eq!(args.flags, alloc_domain::VRAM);
    assert_eq!(args.size, 0x1_000);
    assert_eq!(metadata[0], 0xBAD);
    assert_eq!(metadata[1], 0x3);
    println!("Retrieved metadata in kfd");
}
