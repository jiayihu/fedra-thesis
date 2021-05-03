## Running

Running the code on constrained devices requires to set the envinronment variable `WASMI_VALUE_STACK_LIMIT` and `WASMI_CALL_STACK_LIMIT` so that `wasmi` doesn't allocate the default 1MB heap for the module stacks.

These values are set within `Makefile.toml.` if `cargo make` is used.

```
cargo make run
```

## General instructions

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

##

```
File  .text     Size Crate
1.0%  27.1%  43.5KiB std
0.9%  24.4%  39.2KiB fedra_iot
0.6%  17.6%  28.3KiB wasmi
0.4%  10.2%  16.3KiB [Unknown]
0.2%   5.7%   9.2KiB smoltcp
0.2%   4.8%   7.7KiB parity_wasm
0.2%   4.5%   7.2KiB wasmi_validation
0.1%   3.0%   4.9KiB coap_lite
0.0%   0.6%    1016B libm
0.0%   0.6%     936B stm32_eth
0.0%   0.3%     472B rtt_target
0.0%   0.2%     364B alloc_cortex_m
0.0%   0.2%     316B linked_list_allocator
0.0%   0.1%     228B log
0.0%   0.1%     172B cortex_m_rt
0.0%   0.1%     148B stm32f4xx_hal
0.0%   0.1%     144B byteorder
0.0%   0.1%     106B heapless
0.0%   0.1%      88B cast
0.0%   0.1%      86B cortex_m
0.0%   0.0%      26B And 1 more crates. Use -n N to show more.
3.6% 100.0% 160.3KiB .text section size, the file size is 4.4MiB
```
