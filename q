[1mdiff --cc src/rust/Cargo.lock[m
[1mindex 5a924f979,76ffb92b5..000000000[m
[1m--- a/src/rust/Cargo.lock[m
[1m+++ b/src/rust/Cargo.lock[m
[36m@@@ -827,44 -853,8 +827,49 @@@[m [msource = "registry+https://github.com/r[m
  checksum = "a0610544180c38b88101fecf2dd634b174a62eef6946f84dfc6a7127512b381c"[m
  dependencies = [[m
   "bitflags",[m
[32m++<<<<<<< HEAD[m
[32m + "textwrap 0.11.0",[m
[32m + "unicode-width",[m
[32m +][m
[32m +[m
[32m +[[package]][m
[32m +name = "clap"[m
[32m +version = "3.1.17"[m
[32m +source = "registry+https://github.com/rust-lang/crates.io-index"[m
[32m +checksum = "47582c09be7c8b32c0ab3a6181825ababb713fde6fff20fc573a3870dd45c6a0"[m
[32m +dependencies = [[m
[32m + "bitflags",[m
[32m + "clap_derive",[m
[32m + "clap_lex",[m
[32m + "indexmap",[m
[32m + "lazy_static",[m
[32m + "textwrap 0.15.0",[m
[32m +][m
[32m +[m
[32m +[[package]][m
[32m +name = "clap_derive"[m
[32m +version = "3.1.7"[m
[32m +source = "registry+https://github.com/rust-lang/crates.io-index"[m
[32m +checksum = "a3aab4734e083b809aaf5794e14e756d1c798d2c69c7f7de7a09a2f5214993c1"[m
[32m +dependencies = [[m
[32m + "heck 0.4.0",[m
[32m + "proc-macro-error",[m
[32m + "proc-macro2",[m
[32m + "quote",[m
[32m + "syn",[m
[32m +][m
[32m +[m
[32m +[[package]][m
[32m +name = "clap_lex"[m
[32m +version = "0.2.0"[m
[32m +source = "registry+https://github.com/rust-lang/crates.io-index"[m
[32m +checksum = "a37c35f1112dad5e6e0b1adaff798507497a18fceeb30cceb3bae7d1427b9213"[m
[32m +dependencies = [[m
[32m + "os_str_bytes",[m
[32m++=======[m
[32m+  "textwrap",[m
[32m+  "unicode-width",[m
[32m++>>>>>>> ba20a6359 (Update lock)[m
  ][m
  [m
  [[package]][m
