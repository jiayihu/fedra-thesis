## Wasmi size

Unoptimized

```
File  .text     Size Crate
2.7%  49.6% 278.9KiB wasmi
0.9%  17.0%  95.9KiB parity_wasm
0.7%  12.2%  68.4KiB wasmi_validation
```

Release optimized w/ lto

```
File  .text     Size Crate
1.4%  34.5%  39.8KiB wasmi
0.8%  21.4%  24.7KiB parity_wasm
0.7%  18.1%  20.9KiB std
0.5%  12.9%  14.8KiB fedra
0.3%   6.8%   7.8KiB wasmi_validation
```

Release optimized w/o lto

```
File  .text     Size Crate
0.7%  36.2%  59.7KiB wasmi
0.3%  15.6%  25.8KiB parity_wasm
0.3%  14.1%  23.3KiB std
0.2%  10.0%  16.5KiB smoltcp
0.1%   6.0%   9.9KiB wasmi_validation
0.1%   4.8%   7.9KiB [Unknown]
0.1%   4.6%   7.5KiB fedra
0.1%   4.1%   6.8KiB coap_lite
0.0%   1.3%   2.2KiB stm32f4xx_hal
0.0%   1.1%   1.8KiB stm32_eth
0.0%   0.5%     906B rtt_target
0.0%   0.5%     796B libm
0.0%   0.4%     734B linked_list_allocator
0.0%   0.3%     440B cortex_m_rt
0.0%   0.2%     364B alloc_cortex_m
0.0%   0.1%     180B heapless
0.0%   0.0%      60B memory_units
0.0%   0.0%      40B num_traits
0.0%   0.0%      38B cortex_m
0.0%   0.0%       2B hash32
0.0%   0.0%       4B And 2 more crates. Use -n N to show more.
1.8% 100.0% 165.1KiB .text section size, the file size is 8.9MiB
```
