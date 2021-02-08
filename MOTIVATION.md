# Motivation

In operations automation, a 100 step process may fail at step 50 due to issues such as invalid input, infrastructure flakiness, and so on. Restarting from the beginning can be costly for the following reasons:

* Duration to reach that step can be long.
* Cleaning up resources that were created in the initial execution is wasteful, if the next execution would recreate near identical resources.
* Having multiple commands that run different parts of the execution requires additional learning and remembering, which is an opportunity for further error.

A design that mitigates the above issues would:

* Re-use existing resources if possible.
* Converge existing resources to the required state, if a subsequent execution alters the resource.
* Automatically replace resources if they cannot be re-used.
* Work all of the above out from the same command.

Much like a programming language compiler, if the desired artifact (or state) is already existent, then little or no work is done.

**Choo Choo** is an attempt to bring the simplicity of a modern build tool to the operations world.
