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
0.8%  44.6%  58.0KiB wasmi
0.4%  20.1%  26.2KiB parity_wasm
0.3%  17.3%  22.6KiB std
0.1%   7.6%   9.9KiB wasmi_validation
```
