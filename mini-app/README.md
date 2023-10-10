# Mini-app

This is a minimal Tauri app that is used for testing Tauri fuzzing.

## `mini-app` vulnerabilities

We want the fuzzer to be able to test for:
- shell command injection
- file access and corruption
- sql injection

Commands containing these kind of vulnerabilities are implemented as Tauri `command` in `mini-app`

## Categories of vulnerabilities

We take [OWASP top 10 API security risks](https://owasp.org/API-Security/editions/2023/en/0x11-t10/) and try to create them in the app.

### Broken Object Level Authorization

Attacker is able to access objects that he should not be able to access.

- Accessing private crab photos

### Broken Authenticaiton

Attacker is able to authenticate as someone else.

- Bruteforcing password authentication

### Broken Object Property Level Authorization

Attacker is able to access specific object properties/fields he is not supposed to.

- Get path of crab photo

### Unrestricted Resource Consumption

Attacker is able to create a DOS or increase in operational cost by spamming or cleverly crafting requeqt.

### Broken Function Level Authorization

Attacker is able to access API endpoints he should not have access to.

- administrative endpoints

### Unrestricted Access to Sensitive Business Flows

Attacker is able to gain business advantage by exploiting loopholes in the business flow

- being able to buy the whole stock of a product

### Server Side Request Forgery

Attacker is able to control the application flow by providing/modifying an URL that is provided by the client

### Security Misconfiguration

Attacker is able to access ressources taking advantage of wrong security configuration.

- CSP policy
- tauri plugins security configuration

### Improper Inventory Management

Attacker is able to get unauthorized access through old API endpoints running or through a 3rd party who have unauthorized access.

- old endpoints not removed

### Unsafe Consumption of APIs

Attacker uses a third-party API to attack in which the application has more trust.
