<h1 style="text-align: center"><u>RustOS 2.0</u></h1>

A toy kernel written in Rust.

### Features

- Pure Rust kernel with internal safety guarantees
- Virtual memory management
- APIC interrupt handling
- Cooperative multitasking with an async executor
- Text-mode VGA output

### Todo

- [ ] Rewrite heap allocator
- [ ] Rewrite PCI module
- [ ] Network stack
- [ ] SMP
- [ ] Preemptive multitasking with threads
- [ ] RTC and CMOS support
- [ ] Device manager and drivers
- [ ] Ability to display images
- [ ] IPC
- [ ] Userland
- [ ] System calls
- [ ] Write custom `std` target
- [ ] Shell
- [ ] Transition to hybrid/microkernel?