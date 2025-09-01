# Issue #57: Remove Unused Dependencies

## Task Description
The recent change from a monolithic core to a containerized core introduced some unused crates that slow down the building process.

## Tasks
1. Check and remove all the unused dependencies
2. Make sure that everything is running properly after the changes

## Commands
- Build: `cargo build`
- Test: `cargo test`
- Check unused deps: `cargo machete` or manual analysis

## COMPLETATA.


## ISSUE [#134](https://github.com/CortexFlow/CortexBrain/issues/134)

Problem Description:

When pushing new updates, testing the build processing is a must to ensure that everything is working properly for every user.
Solution:

Create a CI/cd pipeline to build the core components triggered by new Pull Requests or a merge in the main branch. The project already has bash building scripts, so the task is only about creating a pipeline that triggers the scripts

Best practices, no unnecessary overengineering, no core changes.