# Axiom Package Management

## Commands

```bash
axiom pkg add     user/repo           # Install latest
axiom pkg add     user/repo@1.2.3     # Install pinned version
axiom pkg remove  user/repo           # Uninstall
axiom pkg upgrade user/repo           # Upgrade to latest
axiom pkg list                        # List installed
axiom pkg info    user/repo           # Package details
axiom pkg info    .                   # Current project info
```

## Using Packages

```axiom
load @user/repo                // loads lib.ax from ~/.axiomlibs/user/repo/src/
let result = repo.fn(42)

load @user/repo/submod         // loads submod.ax
```

## axiom.pkg — Manifest

```toml
[package]
name        = "my-lib"
version     = "1.0.0"
author      = "Alice"
license     = "MIT"

[dependencies]
http-utils = "2.1.0"
math-extra = "0.5.0"
```

## Package Directory Layout

```
~/.axiomlibs/
  user/
    repo/
      axiom.pkg
      src/
        lib.ax
        utils.ax
      examples/
      tests/
```

## Creating & Publishing

```bash
mkdir mylib && cd mylib
axiom pkg init              # scaffold axiom.pkg + src/lib.ax
# ... write src/lib.ax ...
axiom pkg publish           # publish to registry.axiom-lang.dev
```

## Version Lockfile

`axiom.lock` is auto-generated. Commit it to get reproducible builds.

```toml
[packages]
"user/repo" = { version = "1.2.3", checksum = "sha256:abc..." }
```

## Related Error Codes

| Code | Trigger | Fix |
|------|---------|-----|
| AXM_601 | Module not found | `axiom pkg add <n>` |
| AXM_602 | Version conflict | Pin in `axiom.pkg` |
| AXM_603 | Circular import | Extract shared module |
| AXM_604 | Module has errors | Fix the source |
