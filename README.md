## Running

The project uses [probe-run](https://github.com/knurling-rs/probe-run) as runner. [cargo-embed](https://probe.rs/guide/1_tools/cargo-embed/) can be used as alternative.

```
cargo run
```

## Debugging with OCD

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
