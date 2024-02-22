# Available policies

These list the policy that are currently available in our fuzzing.

## Generic Policies

### No policy

`fuzzer::policies::no_policy()`

No functions are monitored and this will not provoke crashes.
Used if your fuzz target can inherently crash and you just want to investigate those.

## File System Policies

### No file access

`fuzzer::policies::file_policy::no_file_access()`

Any access to file system will provoke a crash.

### Read only access

`fuzzer::policies::file_policy::read_only_access()`

Any access to file system with write access will provoke a crash.

### No access to _filenames_

`fuzzer::policies::file_policy::no_access_to_filenames()`

Any access to the files given as parameter will provoke a crash.

## Rule helper

### Block on entry

`fuzzer::policies::block_on_entry()`

The function monitored with this rule will just automatically crash when called.

