# HCL2 Type Reflection

## What is this?

This is a small python library for reflection of hcl2 types. This is built atop
of the python-hcl2 library. The python-hcl2 is what wraps the types in `${}`

## Why

This was built to fix a type mismatch in Grapl's pulumi code during pulumi
preview. The issue is that

1. In our pulumi code we generate objects via pulumi (such as DBs) whose
   attributes are then passed into Nomad
2. During `pulumi preview`, the attributes are stored in an object that's
   essentially a future without the await attribute
3. We then attempt to pass in a Pulumi object to a typed input. Nomad then blows
   up because a Pulumi object isn't the type it expects.

With this library we're dynamically parsing the types during runtime and then
mocking them out with the right type structure.

## TODO/Future Work

Add the rest of the supported hcl2 types. So far, we've only added types that
are in use within our codebase.
