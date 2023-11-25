<h1 style="text-align: center"><u>RustOS 2.0</u></h1>

A toy kernel written in Rust.

### Features

&nbsp;&nbsp;🦀 Pure Rust kernel with internal safety guarantees

&nbsp;&nbsp;🧮 Virtual memory management

&nbsp;&nbsp;⚙️ APIC interrupt handling

&nbsp;&nbsp;🔄 Cooperative multitasking with an async executor

&nbsp;&nbsp;🖥️ Text-mode VGA output

### Todo

- [ ] Rewrite heap allocator
- [ ] Rewrite PCI module
- [ ] Rewrite RTC
- [ ] SMP
- [ ] Preemptive multitasking with threads
- [ ] Device manager and drivers
- [ ] Network stack
- [ ] Ability to display images
- [ ] IPC
- [ ] Userland
- [ ] System calls
- [ ] Write custom `std` target
- [ ] Shell
- [ ] Transition to hybrid/microkernel?
- [ ] Support other architectures? (e.g. aarch64)