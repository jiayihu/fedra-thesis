- [xPack qemu-arm](https://xpack.github.io/qemu-arm/)
- [Eclipse debugging components](https://gnu-mcu-eclipse.github.io/debug/install/)
- [qemu-eclipse options](https://gnu-mcu-eclipse.github.io/qemu/options/)
- [qemu-doc](https://www.qemu.org/docs/master/qemu-doc.html)

- [Rust embedded book](https://rust-embedded.github.io/book/)
- [Rust embedded Discovery](https://github.com/rust-embedded/discovery)
- [embedonomicon](https://docs.rust-embedded.org/embedonomicon/preface.html)
- [Rust embedded FAQ](https://docs.rust-embedded.org/faq.html#my-program-just-halts-without-connected-debugger-what-am-i-doing-wrong)

- [UART, SPI, RS232, USB, I2C etc.](https://electronics.stackexchange.com/questions/37814/usart-uart-rs232-usb-spi-i2c-ttl-etc-what-are-all-of-these-and-how-do-th)

- [GDB cheatsheet](https://darkdust.net/files/GDB%20Cheat%20Sheet.pdf)
- [GDB to LLDB](https://lldb.llvm.org/use/map.html)
- [GDB print current line](https://stackoverflow.com/questions/14581837/gdb-how-to-print-the-current-line-or-find-the-current-line-number)
- [GDB manual](https://ftp.gnu.org/old-gnu/Manuals/gdb/html_node/gdb_toc.html)
- [Debugonomicon](https://rust-embedded.github.io/debugonomicon/overview.html)
- [Software vs Hardware breakpoints](https://stackoverflow.com/a/55867982/6860493)
- [OpenOCD Commands](http://openocd.org/doc/html/General-Commands.html)

- [embedded-time](https://github.com/FluenTech/embedded-time)
- [Real-time Interrupt-driven Concurrency](https://github.com/rtic-rs/cortex-m-rtic)

- [stm32-rs](https://github.com/stm32-rs/stm32-rs): Embedded Rust device crates for STM32 microcontrollers
- [cortex-m-quickstart](https://github.com/rust-embedded/cortex-m-quickstart/blob/master/examples/hello.rs)
- [stm32f429i-disc](https://github.com/stm32-rs/stm32f429i-disc/)
- [Rust embedded stopwatch](https://www.mcu.by/rust-embedded-stopwatch/)

## Debugging

```
openocd -f board/stm32f4discovery.cfg
```

```
arm-none-eabi-gdb -q -x ocd.gdbinit
```

## Emulation

Run qemu with gdb

```
qemu-arm --board STM32F429I-Discovery --gdb tcp::1234 --verbose --verbose -d unimp,guest_errors --semihosting-config enable=on,target=native
```

Run gdb, check `.gdbinit` for executed commands

```
arm-none-eabi-gdb

# To rerun .gdbinit
source .gdbinit
```

## Inspection

Check the ELF headers

```
cargo readobj --bin app -- -file-headers
```

Check the size of the linker sections

```
cargo size --bin fedra --release -- -A
```

Diassemble the binary

```
cargo objdump --bin fedra -- --disassemble
```
