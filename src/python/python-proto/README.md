# Debugging

It can be hard to reason about the generated python protobuf types; To debug
them, you can use `./pants export-codegen ::`, which will write all codegen to
dist/codegen.
