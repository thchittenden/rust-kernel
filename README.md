# x86 Rust Kernel
 
This is an implementation of an x86 multi-threading kernel written in Rust.

## Building

The kernel must be built by the Rust 1.0.0 compiler built using the nightly
configuration in order to support unstable features (such as core). The build 
will probably panic if using any other compiler because it tries to link 
against the precompiled core library in lib/ that was built with the 1.0.0
compiler.

Obviously this is suboptimal and eventually building libcore should be
integrated into the build process so that this can be built with any Rust
compiler supporting unstable features.

If you already have your own Rust nightly compiler, you can replace
lib/libcore.rlib with a version you built yourself using the same flags as
in the Makefile.

## Running

Currently the kernel has only been tested in Qemu but would probably work in 
Bochs or Simics (or maybe even VirtualBox). To run with Qemu and view logs, run
with the following command.

```
qemu-system-i386 bin/kernel.iso -serial file:/dev/stdout
```

