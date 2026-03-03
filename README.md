## Linux's C header files
You can find them in kernel-headers package on fedora.

In kernel source code these are the paths:
### KFD
`include/uapi/linux/kfd_sysfs.h`
`include/uapi/linux/kfd_ioctl.h`

### DRM
`include/uapi/drm/amdgpu_drm.h`

## Why not to simply use bindgen?
Bindgen fails to generate important constants and produces bloat by default because how unhygenic
include file dependencies are.

## Why not to use existing C libraries?
Because we can.

### Some tests are design to be run alone and are interactive
Example 
```sh
cargo test calling_acquire_vm_twice_the_same_file -- --no-capture
```
