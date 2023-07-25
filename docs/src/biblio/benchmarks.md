# Benchmarks

Fuzzers efficiency are compared over different metrics/conditions
Overall there is no best fuzzer.
Fuzzers performance vary a lot depending on the fuzzing environment:
- resources given
- time give
- type of vulnerabilities
- target program

## Metrics used

From [Unifuzz](https://www.usenix.org/system/files/sec21summer_li-yuwei.pdf) paper:
- quantity of unique bugs
- quality of bugs: rare bugs are worth more
- speed of finding bugs
- stability of finding bugs: can a fuzzer find the same bug repeatedly?
- coverage
- overheard

## Tools

- Magma: real-world vulnerabilities with their ground truth to verify genuine software defects
- [Unifuzz](https://www.usenix.org/system/files/sec21summer_li-yuwei.pdf):
    real-world vulnerabilities with their ground truth to verify genuine software defects
- LAVA-M: benchs of programs with synctatically injected memory errors
- Cyber Grand Challenge: benchs of binaries with synthetic bugs
- Fuzzer Test Suite (FTS): real-world programs and vulnerabilities
- FuzzBench: improvement of FTS with a frontend for easier integration


