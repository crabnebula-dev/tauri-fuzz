# Security Policies

In this section we present the inner workings of `tauri-fuzz`.

## Security policies

We said in the previous section that we want to fuzz applications security boundaries.
Security boundaries define what an application should be allowed to interact with on the host system.
But each application have distinct security boundaries. An application A could be allowed interactions with the file system
but not the network, application B could be allowed interactions with the file system but only in a specific directory, application C
could be allowed interactions with the shell...

How can we fuzz applications effectively knowing that each application have different security boundaries?

Our proposal for this is to describe security boundaries through policies implemented in `tauri-fuzz-policies`.
You can implement and customize these policies to fuzz your Tauri application more effectively.

## How to provide a suitable policy to fuzz my Tauri application

We propose three ways to use a suitable policy for your application:

- create a policy manually
- derive a policy from Tauri configuration
- use the generic policy

### Create a policy tailored to your application

Policies that are provided to `tauri-fuzz` are defined in our crate `tauri-fuzz-policies`.
These policies can be implemented by the application developer and provided to the fuzzer.
A basic set of policies have already been implemented in `tauri-fuzz-policies` and can be reused
by the user.

A list of available policies are presented in the [user guide](../user_guide/available_policies.md).
A guide on how to create your own policy is also available [here](../user_guide/write_custom_policy.md).

### Create a policy based on the Tauri app configuration

> **[Disclaimer]** This is still ongoing work and has not been implemented in `tauri-fuzz`

Tauri applications have a [capability system](https://v2.tauri.app/security/capabilities/) which
describes what features the application frontend is allowed to use.
These capabilities can also be seen as the security boundaries of a Tauri application frontend.

The objective would be able to automatically derive a policy for fuzzing from the capability configuration of a Tauri app.
While this is not an exact mapping of the security boundaries of the whole application this is still an approximation.

### Generic policy: no error policy

> **[Disclaimer]** This is still ongoing work and is not complete yet

Since our goal was to make fuzzing Tauri applications as easy as possible we don't want to force users
to write their own policy to fuzz their application.
We came up with a policy `no_error_policy` that is relevant enough to fuzz most applications.

**The `no_error_policy` monitors all the interaction with the system resources and blocks only when an interaction returns a status error.**

![No error on return policy](../images/no_error_policy.drawio.svg "No error on return policy")

In the schema we can see the situation where the application interacts with the file system.
In one case the file system access went well and is let through by our fuzzer runtime.
In the other case an error occurred which results in a return error status.
The fuzzer runtime will block this return error and report the occurrence in the fuzzing report as a case to investigate.

The motivation behind the `no_error_policy` is that if an application enables errors to happen when accessing system resources
then a malicious attacker could potentially leverage the application to control the system resources to its advantage.
The `no_error_policy` aims to detect vulnerabilities that could be exploited via input manipulation by a malicious attacker.
Since fuzzing uses pseudo random data we expect that most of the time these vulnerabilities would appear as syntax errors.

This idea was inspired from the fuzzer [Witcher](https://github.com/sefcom/Witcher).
