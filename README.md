# x86 Rust Kernel
 
This is an implementation of an x86 multi-threading kernel written in Rust.

The kernel currently supports kernel level threads, a virtual file system 
(inspired by Linux) and a primitive driver framework (inspired by Windows).

Todo:
- [ ] IDE driver.
- [ ] ELF parser.
- [ ] User space loader.
- [ ] System calls.
  - [ ] Process lifetime (fork/exec/wait/vanish).
  - [ ] Scheduling (yield/deschedule/make_runnable/sleep).
  - [ ] Memory management (mmap/munmap).
  - [ ] IPC (???).
- [ ] Safety improvements (remove unwraps, etc).
- [ ] Dynamic loading, kernel modules.

## Building

The kernel must be built by a post 1.1.0 Rust compiler built using the nightly
configuration in order to support unstable features and the `drop_in_place` 
function. 

The build links against a pre-compiled version of libcore in the `lib` 
directory. If the version of the compiler differs from that used to compile
libcore, it will probably panic. If that happens, recompile libcore and replace
the version in lib/. 

Obviously this is suboptimal and eventually building libcore should be
integrated into the build process so that this can be built with any Rust
compiler supporting unstable features.

## Running

Currently the kernel has only been tested in Qemu but would probably work in 
Bochs or Simics (or maybe even VirtualBox). To run with Qemu and view logs, run
with the following command.

```
qemu-system-i386 bin/kernel.iso -serial file:/dev/stdout
```

