# Web applications fuzzing

## Challenges

What challenges are specific to web applications?

- webapps have many components that we don't want to fuzz
    - web server that takes HTTP request
    - data storage
    - most likely a code runtime
    - the app we want to test
- __Enabling fuzzing for webapps__
    - detecting inputs that triggers vulnerabilities
        - binary fuzzing usually detects segfault
    - generating valid inputs for end-to-end execution
        - inputs need to be valid HTTP requests
        - inputs need to possess the necessary input parameters for the webapp logic
- __Improving fuzzing for webapps__
    - collecting coverage information
        - not always possible with web applications
    - mutating inputs effectively
        - little research has been done on mutation strategy on webapps currently

## Types of fuzzer for webapps

Fuzzing in web apps is still young.
- Blackbox
    - Pros/Cons
        - ++ you don't need source code
        - -- the inputs space is restrained in webapps and need manual meddling
        - -- vulnerabilities are inferred based on the output of the webapp which is not precise
- Whitebox
    - no recent papers using this approach
    - Pros/Cons
        - -- requires source code
        - -- usually uses language model making them language-specific
        - -- requires more effort to implement
        - -- does not scale well to real-word applications
        - ++ the fuzzing is the most complete
- Greybox
    - really few papers of this type but it looks promising
    - Pros/Cons 
        - ++ you don't necessarily need source code
        - ++ extra information makes the fuzzing more efficient
        - ++ scales well

## Industry solution

- [Burp](https://portswigger.net/burp/documentation/desktop/tools/burps-browser)
- [Skipfish](https://portswigger.net/burp/documentation/desktop/tools/burps-browser)

## [WebFuzz](https://www.researchgate.net/publication/354942205_webFuzz_Grey-Box_Fuzzing_for_Web_Applications)

Date: 2021
[Github](https://github.com/ovanr/webFuzz)

Greybox fuzzer targeted at PHP web applications __specialized for XSS vulnerabilities__

### Contributions

- greybox fuzzer targeted at PHP web applications specialized for XSS vulnerabilities
- bug injection technique in PHP code
    - useful to evaluate webFuzz and other bug-finding techniques in webapps

### Fuzzer

- uses edge coverage on PHP server code
- __workflow__
    - fuzzer fetches any GET or POST request that has been uncovered by a crawler
    - sends the request to the webapp
    - reads its HTTP response and coverage feedback
        - http is parsed to uncover new potential HTTP requests and XSS vulnerabilities
        - if feedback is favorable, store the HTTP request for further mutations
    - loop
- __HTTP requests mutation__
    - modify parameters of POST and GET request
    - 5 mutations techniques are employed
        - insertion of real XSS payloads
        - mixing GET or POST parameters from previously interesting requests
        - insertion of randomly generated strings
        - insertion of HTML, JS or PHP tokens
        - altering the type of a parameter
- __web crawling__
    - HTTP responses are parsed and analysed to crawl the whole app
    - extract new fuzz targets from `anchor` and `form` elements
    - retrieve inputs from `input`, `textarea` and `option` elements
- __vulnerability detection__
    - look for stored and reflective XSS vulnerabilities
        - stored XSS when JS is stored in the webapp data
        - reflective XSS vuln when JS from an HTTP request is reflected on the webapp
    - HTML responses are parsed and analysed to discover code in
        - link attribute (e.g. `href`) that start with the `javascrip:` label
        - executable attribute that starts with the `on` prefix (e.g. `onclick`)
        - script elements
    - fuzzer injects XSS payloads in the HTTP requests to call `alert()`
        - fuzzer detector check for any calls to `alert()`
- __corpus selection__ criteria
    - coverage score: number of labels triggered
    - mutated score: difference of code coverage with its parent request it
    was mutated from
    - sinks present: if the request managed to find their way in the HTTPS response
    - execution time: round-trip time of the request
    - size: number of char in the request
    - picked score: number of times it was picked for further mutations


## [Witcher](https://adamdoupe.com/publications/witcher-oakland2023.pdf)

Date: 2023
Greybox fuzzing

Really good paper. 
- Context and challenges are explained clearly.
- first paper to fuzz against SQL and code injection
- bibliography is pleasant to read

### Contributions

- framework to ease the integration of coverage-guided fuzzing on webapps
- fuzzer that can detect multiple type of vulnerabilities in both server-side binary
and interpreted web applications
    - SQL injection, command injection, memory corruption vulnerability (in C)

### Enable fuzzing in webapp for SQL and command injection

#### Fault Escalator

We want to detect when an input makes the webapp transitions into an unsafe state.
Usually for binary fuzzing we detect segfault and memory corruption.
_Witcher_ uses fault escalation of syntax errors to detect when a SQL or code injection 
has been executed by the fuzzer.

##### SQL fault escalation

- instrument an SQL database to trigger a segfault when a syntax error
has been triggered
- illegal sql injection from the fuzzer has a high change to trigger a syntax error
- valid sql access shouldn't form ill-formed requests

##### Command injection escalation 

- `dash` is instrumented to escalate parsing error to segfault 
- any code injection that calls `exec()`, `system()` or `passthru()` will be 
passed to `dash`
- Witcher version of `dash` has 3 lines of code difference from the original

##### Extend fault escalation 

Syntax errors have been used for both SQL and command injection. 
This can apply also to any type of warning, error or pattern.
__Ex: detect file system usage by triggering segfault when a non-ascii value has been used__

##### XSS 

- Not handled 
- browsers are really permissive when parsing HTML
- makes XSS vulnerabilities hard to detect 

#### Request Crawler 

Uses `Reqr`
- extracts HTTP requests from all types of web application.
- uses `Puppeteer` to simulate user actions 
- static analyze the rendered HTML to detect HTML elements that create 
HTTP requests or parameters
- trigger all HTML elements that trigger user action 
- randomly fires user event inputs

#### Request Harness

Witcher’s HTTP harnesses translates fuzzer generated inputs into valid requests
- CGI requests are used for PHP and CGI binaries
- HTTP requests are used for Python, Jave, Node.js and Qemu-based binaries

#### Translating fuzzer input into a Request

- create seeds to fuzz 
    - field for cookies
    - query parameters 
    - post variables
    - header values 
- sets the variables for the webapp to operate correctly (e.g. cookies) 

### Augmenting Fuzzing for web injection vulnerabilities

#### Coverage Accountant

It is hard to do code coverage for interpreted languages.
Instrumentations to the interpreters add unnecessary noises.

- augmented bytecode interpreter for interpreted languages 
    - linenumber, opcode and parameters are collected at runtime
- CGI binaries 
    - source code available, uses AFL instrumentation 
    - without source code uses dynamic QEMU instrumentation

#### HTTP-specific Input mutations

Add two HTTP-specific mutations stages to AFL
- HTTP parameter mutator 
    - cross-pollinates unique parameter name and values between
    interesting test cases stored in the corpus 
    - more likely to trigger new execution rather than random
    byte mutations 
- HTTP dictionary mutator 
    - endpoints usually serve multiple purposes hence an endpoint
    may have several requests that use different HTTP variables
    - for a given endpoint, `Witcher` places all the HTTP variables 
    discoverd by `Reqr` into the fuzzing dictionary

### Evaluation 

- blackbox vs greybox: Outperforms `Blurp` in vulnerabilities found
- Covers more code than `BlackWidow` and `webFuzz`
    - they both specialize in XSS so we can't compare 

### Limitations 

- there are other web vulnerabilities 
    - XSS
    - path traversal
    - local file inclusion 
    - remote code evaluation 
- only detect reflected injection vulnerabilities
    - when user input flows directly to a sensitive sink 
    during a HTTP request
    - no detection of second-order vulnerabilities where there
    is a first step to store the injection in the webapp data
        - stored SQL injection
    - fault escalation would trigger but hard to investigate 
    the actual input that stored the malicious injection
- does not reason about the application state
    - fuzzes one URL at a time 
    - does not reason about multi-state actions
   
## [BlackWidow](https://www.cse.chalmers.se/~andrei/bw21.pdf)

Date: 2021
[BlackWidow Github](https://github.com/1N3/BlackWidow)
TODO

## [BackREST](https://arxiv.org/pdf/2108.08455.pdf)

Date: 2021
Blackbox fuzzing

## REST API fuzzing (TODO)

### Challenges

- it's hard to trigger long sequence valid requests to trigger hard-to reach states
- it's hard to forge high-quality requests that that pass the cloud service checking

### [Miner](https://www.usenix.org/system/files/sec23fall-prepub-129-lyu.pdf)
 
TODO

- uses data history to guide fuzzing
- uses AI attention model to produce _param-value_ list for each request
- uses request response checker to keep interesting testcase

### [RESTler](https://patricegodefroid.github.io/public_psfiles/icse2019.pdf)

[RESTler Github](https://github.com/microsoft/restler-fuzzer)

TODO

