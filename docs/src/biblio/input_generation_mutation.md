# Input Generation / Mutation

Target programs usually only accept certain kinds of inputs.
The goal

The usual main concerns:
- interesting inputs: input that produce new behaviour during fuzzing
    - this is really hard because it is often target-dependent
    - there can be dynamic optimizations of the mutations during the fuzzing using fuzzing metrics
- syntactically correct: input should be valid
    - can be solved given a grammar
    - grammar can be inferred
- semantically correct: input has to make sense
    - usually mutation over already valid inputs

## Tools

### Syzkaller

Current fuzzer used in Linux Kernel development
- requires a grammar for syscalls

### [FuzzNG](https://www.ndss-symposium.org/ndss-paper/no-grammar-no-problem-towards-fuzzing-the-linux-kernel-without-system-call-descriptions/)

Fuzzer for linux kernel that does not require a grammar

- FuzzNG reshapes the input space without needing a grammar
- at runtime FuzzNG aware of which file descriptors and pointers are accessed
- before function calls, FuzzND pauses this process to populate these locations

### [Darwin](https://www.ndss-symposium.org/wp-content/uploads/2023/02/ndss2023_s159_paper.pdf)

Optimization of the mutation at runtime
- issues:
    - optimal mutations are target-dependent
    - runtime algorithms to optimize mutations may have a performance cost
        that outweighs the benefits
- leverages a variant of _Evolution strategy_
    - mix between evolution and intensifications
    - evolution optimizes over a set of inputs
        - discover promising areas and avoid local optima
        - Algo:
            - start with primary population of inputs
            - natural selection of the population by selecting best inputs over a metric
            - mutate these best inputs to obtain a new population
            - reiterate the selection until certain criteria is met
    - intensification optimize a promising area
        - focus on current best solution
        - mutate to find the better neighboring solutions

### Nautilus

Combines usage of grammar + code coverage results
- improve the probability of having syntactically and semantically valid inputs

### Grimoire

TODO


