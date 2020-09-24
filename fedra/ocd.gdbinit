file "./target/thumbv7em-none-eabi/debug/fedra"

target remote :3333

# print demangled symbols
set print asm-demangle on

# perform a hard reset
monitor reset init

# Load symbols
load

# Enable semihosting
monitor arm semihosting enable

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

# start the process but immediately halt the processor
stepi
