# RustKPI 

**Master Thesis Project**  
Johannes Lundberg  
KTH, Stockholm, Sweden  
2017-2018

## Introduction

TODO: Clean up files and write introduction.


## How To 

*Preliminary*


1. Clone this repo (to folder we will call `$ROOT`) on a FreeBSD 12 machine
1. Install `rustup` (https://www.rust-lang.org/en-US/install.html)
1. Set toolchain
   1. `rustup toolchain link rustkpi $ROOT/rust/toolchains/x86_64-unknown-freebsd-1.25-nightly`
   1. `cd $ROOT`
   1. `rustup override set rustkpi`
1. Build RustKPI
   1. `cd $ROOT/rust/kmod-rustkpi`
   1. `make`
1. Build Hello World module
   1. `cd $ROOT/rust/kmod-helloworld`
   1. `make`
1. Test
   1. `cd $ROOT/rust/`
   1. `sudo kldload kmod-rustkpi/rustkpi.ko`
   1. `sudo kldload kmod-helloworld/rustkpi-hello.ko`
   1. dmesg

For the e1000 driver in kmod-e1000 the following devices are supported
* bhyve with e1000 nic emulation
* I218 (found in Intel Broadwell laptops)
* I219-LM (found in Intel Skylake laptops)
 
kmod-e1000 is built the same way as described above. 

