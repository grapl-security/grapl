# Debugging

It can be hard to reason about the generated python protobuf types; To debug
them, you can use `./pants export-codegen ::`, which will write all codegen to
dist/codegen.

Alternatively, you can explore it with a repl, with 
`./pants --no-pantsd repl --shell=ipython src/python/python-proto/python_proto`
