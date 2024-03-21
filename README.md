# Measuring performance of different instructions for MTE

This repo contains my code to measure the performance of different instructions to tag memory on a Pixel 8.
It measures the following variants on 128 MiB of memory:

| Variant     | Instruction | Granule size | Implicit zero | memset |
|-------------|-------------|--------------|---------------|--------|
| memset      | -           | -            | No            | Yes    |
| stg         | `stg`       | `16`         | No            | No     |
| stgp        | `stgp`      | `16`         | Yes           | No     |
| st2g        | `st2g`      | `32`         | No            | No     |
| stzg        | `stzg`      | `16`         | Yes           | No     |
| stg+memset  | `stg`       | `16`         | No            | Yes    |
| st2g+memset | `st2g`      | `32`         | No            | Yes    |

On your Pixel, install Termux and run the following commands to install the necessary dependencies:

```bash
pkg install rust
```

Clone this repo on a Pixel 8 and run the following commands to measure the performance of the different instructions:

```bash
for cpu in 8 4 0; do 
  for bench in "memset" "stg" "stgp" "st2g" "stzg" "stg\+memset" "st2g\+memset"; do
    taskset --cpu-list $cpu cargo bench -- '^'"$bench"'$'
    sleep 30
  done
done | tee output.txt
```

Alternatively, just run `cargo bench` to measure the performance of all the instructions.
The above script runs the benchmarks on CPUs 8, 4, and 0, and waits for 30 seconds between each benchmark to allow the
CPU to cool down.

The CPU indices correspond to the following cores:

| CPU index | Core                   |
|-----------|------------------------|
| 0-3       | Cortex-A510 (1.7 GHz)  |
| 4-7       | Cortex-A715 (2.37 GHz) |
| 8         | Cortex-X3 (2.91 GHz)   |
