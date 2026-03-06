use amdgpu_linux_api::drm::AmdgpuDrmRender3_64;
use amdgpu_linux_api::kfd::AcquireVm;
use amdgpu_linux_api::kfd::AcquireVmResult;
use amdgpu_linux_api::kfd::AvailableMemory;
use amdgpu_linux_api::kfd::Kfd1_18;
use amdgpu_linux_api::kfd::apertures::Apertures;

fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let mut buff = Default::default();
    let gpus = kfd.apertures(&mut buff);
    let gpu = gpus[0];
    let drm_file = AmdgpuDrmRender3_64::open(128).unwrap();
    let kfd = match kfd.acquire_vm(&gpu, &drm_file) {
        AcquireVmResult::Ok(acquired_vm) => acquired_vm,
        AcquireVmResult::GpuNotFound(_) => panic!(),
        AcquireVmResult::MemoryAlreadyAcquiredWithDifferentDrmFile(acquired_vm) => acquired_vm,
        AcquireVmResult::Unexpected(_) => panic!(),
    };
    let _ = kfd.available_memory(gpu);
}
