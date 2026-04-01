# AMDGPU Linux API

**Status: Work in Progress / Experimental**

This repository provides Rust bindings to AMDGPU Linux kernel interfaces.

The goal is to benefit from Rust when interacting with AMD GPUs on Linux,
bypassing the need for C libraries (libdrm-amdgpu, ROCm).


## Running Experiments

Experiments in the `examples/` directory can be run using Cargo.
Some experiments are interactive or designed to be run in isolation.

```sh
# Example: Run a specific experiment
cargo run --example kfd_list_devices
```

## Contributing

Contributions are welcome!

At this stage a lot of things can still change.

If you're interested,
please feel free to open an issue or a pull request.

Generally it's best to add new examples to verify rust bindings against kernel behavior.

## License

This project is licensed under the GNU General Public License v2.0 (GPL-2.0-only).

