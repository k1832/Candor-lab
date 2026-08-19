# Candor diagnostics — the error-code taxonomy

Every code mined from `compiler/src/`. Format: **code — meaning — the fix a model
should make.** Grouped by family. (This doubles as the P4 diagnostic taxonomy.)
`P0xxx` = parse errors; `E01xx`–`E11xx` = check errors.

## P0xxx — parse (shape is wrong; ≤2-token, no symbol table)
| code | meaning | fix |
|------|---------|-----|
| P0001 | expected token X, found Y | fix the syntax at the span; check delimiters/keywords |
| P0002 | `copy` may only precede `struct`/`enum` | move/remove the stray `copy` |
| P0003 | `unsafe` requires a justification string literal | write `unsafe "why" { ... }` |
| P0004 | `ptr_null[T]()` takes no arguments | drop the args: `ptr_null[T]()` |
| P0005 | this intrinsic requires exactly one argument | give exactly one arg |
| P0006 | comparison operators are non-associative | parenthesize: `(a < b) && (b < c)` |
| P0007 | (parser) malformed construct | correct per grammar.md |
| P0009 | (parser) malformed construct | correct per grammar.md |

## E01xx — items, names, types, resolution
| code | meaning | fix |
|------|---------|-----|
| E0101 | duplicate item `name` | rename or remove the duplicate |
| E0102 | unknown type `T` | declare/import the type; check spelling |
| E0103 | unknown name (variable/callee) | bind or import it before use |
| E0105 | duplicate field | remove the repeated field |
| E0106 | duplicate variant | remove the repeated variant |
| E0107 | type has no field `f` | use a real field; check the struct def |
| E0108 | enum has no variant `V` | use a declared variant |
| E0109 | enum marks more than one `ok` variant | keep exactly one `ok` marker |

## E02xx — field / type well-formedness
| code | meaning | fix |
|------|---------|-----|
| E0201 | struct field may not have a borrow type | store `Box`/value/index/`rawptr`, not a borrow (spec 04 §8) |
| E0202 | a `copy` struct may not have a `drop` hook | drop `copy` or drop the hook |
| E0203 | mode on a borrow-kind type is ill-formed | a `read T`/`[T]` param takes no `read`/`write` mode; pass by value |

## E03xx — move, init, drop
| code | meaning | fix |
|------|---------|-----|
| E0301 | use of moved value | don't read after move; `clone`, reborrow, or restructure |
| E0302 | inconsistent move state at a join | make all arms/paths agree — `return` from every consuming arm |
| E0303 | partial move out of a `drop`-hooked type | move the whole value, or don't give it a hook |
| E0304 | use of possibly-uninitialized value | initialize on all paths before use |
| E0305 | `out` param not assigned on this return path | assign it before every return |
| E0306 | `out` param read before first assignment | assign before reading |
| E0307 | out-mode arg must carry the `out` marker | write `f(out place)` |
| E0308 | `out` marker only valid on an out-mode arg | remove the stray `out` |
| E0309 | init state differs across paths at a drop point | make the place init-uniform (init on all paths, or moved on all) |
| E0310 | non-copy move through a `deref`/index path | use `unbox` for a Box pointee; don't move through a borrow/index |
| E0311 | write/`write`-borrow/`out` of an immutable `static` | statics are read-only; use a local or pass a handle |

## E04xx — effects (the `alloc` partition)
| code | meaning | fix |
|------|---------|-----|
| E0401 | non-`alloc` function performs allocation | mark the fn `alloc`, or don't box/unbox/drop-a-Box/call-alloc here |
| E0402 | `alloc` fn assigned to a non-`alloc` fn-pointer | make the pointer type `alloc` (understating is forbidden) |

## E05xx — unsafe & pointers (the valve)
| code | meaning | fix |
|------|---------|-----|
| E0501 | raw-pointer op needs an `unsafe` block | wrap it: `unsafe "why" { ... }` |
| E0502 | `unsafe` justification must be a non-empty string | supply a real justification |
| E0510 | `field_ptr` target has no such field | use a static field of the `rawptr StructT` |

## E06xx — conv, match, patterns, control flow
| code | meaning | fix |
|------|---------|-----|
| E0601 | non-exhaustive match | add the missing variant arm(s) or `_` |
| E0602 | overlapping pattern — the arm is unreachable | remove or reorder the shadowed arm |
| E0603 | match scrutinee is not an enum | match only enums |
| E0604 | pattern names a different enum than the scrutinee | use the scrutinee's enum |
| E0605 | wrong variant payload arity | match the declared payload count |
| E0606 | pattern kind does not fit the scrutinee (literal vs enum, variant vs integer, wrong literal) | write a pattern of the scrutinee's kind |
| E0701 | `conv` operand must be an integer | convert only integer scalars |
| E0702 | `result` only inside an `ensures` clause | remove/relocate `result` |
| E0703 | type mismatch: expected `T`, found `U` | supply a value of the expected type |
| E0704 | value is not callable | call a fn / fn-pointer only |
| E0705 | `out` argument must be a place | pass a place, not a temporary |
| E0706 | wrong argument count | match the declared arity |
| E0707 | `break`/`continue` outside a loop | move it inside a loop |
| E0708 | `?` (or a non-read) inside a contract clause | contract clauses are read-only, no `?` |
| E0709 | integer literal out of range for its type | use a fitting literal/suffix |
| E0710 | constant known to lose value in `conv` | widen the target, or use `wrapping`/`saturating` |
| E0711 | `?` applied to a non-enum | `?` only on result-shaped enums |
| E0712 | `?` needs the fn to return the same result type / a `From` impl | make return types match, or (multi-module) widen explicitly |
| E0713 | a write through a borrow needs an explicit `.*` | write `p.*.f = e`, not `p.f = e` |
| E0714 | illegal `bitcast`: not a same-width float<->integer pair (or literal doesn't fit the bit pattern) | use f64<->{i64,u64,isize,usize} / f32<->{i32,u32}; suffix a high-bit literal |
| E0715 | range pattern lower bound exceeds upper bound | order the bounds `lo ..= hi` |
| E0716 | array repeat requires a `copy` element type (spec 03 §8.1) | use a `copy` element, or build the array element-by-element |
| E0717 | array repeat length must be a compile-time constant (spec 03 §8.1) | write an integer literal or a named constant |

## E08xx — borrows, aliasing, regions, all-paths-return
| code | meaning | fix |
|------|---------|-----|
| E0801 | conflicting borrow (exclusive excludes all others) | don't overlap loans; shorten a borrow's live range |
| E0802 | cannot move out of a place while it is borrowed | end the loan before moving |
| E0803 | cannot write to a place while it is borrowed | end the loan before writing |
| E0804 | cannot read a place while it is exclusively borrowed | read through the exclusive borrow, or end it |
| E0805 | args borrow overlapping places incompatibly (no two-phase) | split into separate statements |
| E0806 | returned borrow may not outlive the body | return an owned value or a borrow of an input |
| E0807 | return region not declared in the signature | declare the `region` variable |
| E0808 | return region tags no borrow parameter | tag the source parameter with the region |
| E0809 | write through a shared borrow | take a `write` borrow for the write path |
| E0810 | control may reach the end of a non-unit fn | return a value on every path |

## E09xx — modules (design 0008)
| code | meaning | fix |
|------|---------|-----|
| E0900 | cannot read the module tree | fix the file/path layout |
| E0901 | unresolved import: no such module | create/spell the module correctly |
| E0902 | import: module has no such item | export (`pub`) or fix the item name |
| E0903 | private item is not `pub` | mark it `pub` to export |
| E0904 | import cycle | break the dependency cycle (DAG only) |
| E0905 | no root `main.cnr` defining `fn main` | add the entry module |
| E0906 | missing package entry file / re-check source absent | restore the package's `src/` entry file |

## E091x–E093x — build cache & packages (design 0017)
| code | meaning | fix |
|------|---------|-----|
| E0910 | cannot create the build cache dir | fix filesystem permissions/paths |
| E0911 | cannot serialize a build artifact | environment error; retry or clear the cache |
| E0912 | cannot write a build artifact | fix filesystem permissions/paths |
| E0913 | cannot create the codegen cache dir | fix filesystem permissions/paths |
| E0914 | cannot write a codegen cache entry | fix filesystem permissions/paths |
| E0920 | internal: resolve on a manifest-less directory | add a `candor.toml` (design 0017 §1) |
| E0921 | dependency directory has no `candor.toml` | point the dep at a real package |
| E0922 | dependency `package = "..."` alias mismatches the dep's manifest name | align the alias with the dep's `package.name` |
| E0923 | dependency conflict: one package required from two different sources | unify the two requirements |
| E0925 | cannot resolve a dependency path | fix the path in the manifest |
| E0926 | cannot read a source file | fix the file/permissions |
| E0927 | package dependency cycle | break the cycle (packages form a DAG, design 0017 §8) |
| E0928 | cannot serialize `candor.lock` | environment error; retry |
| E0929 | cannot write `candor.lock` | fix filesystem permissions |
| E0930 | dependency name collides with a local top-level module | rename the module or alias the dep |
| E0931 | cannot run `git` for a git dependency | install git / fix PATH |
| E0932 | git clone failed | check the URL and network |
| E0933 | git rev/tag/branch does not resolve (or checkout failed) | pin an existing rev |
| E0934 | cannot create/publish the git cache | fix filesystem permissions |
| E0935 | freestanding composition imports the `std` package | drop the `std` dependency in freestanding builds |
| E0936 | `candor.lock` update grows the dependency trust surface | review the delta; re-run with `CANDOR_ACCEPT_TRUST_DELTA=1` to accept |

## E10xx–E11xx — generics, interfaces, impls (designs 0007/0009)
| code | meaning | fix |
|------|---------|-----|
| E1001 | not a generic function | remove the type-arg spelling |
| E1002 | cannot infer a type parameter from the args | annotate at a binding/return (no call-site turbofish) |
| E1003 | unknown interface | declare/import the interface |
| E1004 | not a generic type | remove the `[...]` type args |
| E1005 | wrong number of type arguments | match the declared arity |
| E1006 | a borrow type is not a legal type argument | pass an owned/`rawptr` type, never `read U`/`[U]` |
| E1007 | type arg does not satisfy `copy` | supply a `copy` type, or drop the `copy` bound |
| E1008 | type arg does not implement the interface | provide an `impl`, or relax the bound |
| E1009 | overlapping impl | keep one impl per `(interface-instance, type)` |
| E1012 | impl target must be a nominal type | impl only on a named struct/enum |
| E1013 | orphan impl | put the impl in the type's or interface's module |
| E1014 | method not declared by the interface | remove the extra impl method (one interface, one shape) |
| E1015 | impl is missing an interface method | implement every method |
| E1016 | impl type param doesn't appear in the target type | use `T` in the target (`impl[T] I for List[T]`) |
| E1017 | assoc-type projection has no bounded base | project only off a bounded parameter (`T::Item`, `T: Iter`) |
| E1018 | impl binds the wrong associated type name | bind the interface's declared `type` name |
| E1020 | polymorphic recursion with a growing type argument | remove the self-growing instantiation |
| E1021 | impl method `self`-receiver mismatch | match the interface's receiver (presence/mode) |
| E1022 | impl method parameter-count mismatch | match the interface arity |
| E1023 | impl method parameter mode mismatch | match the declared mode |
| E1024 | impl method parameter type mismatch | match the declared type |
| E1025 | impl method return-type mismatch | match the declared return |
| E1026 | impl method effect-marker mismatch | match the interface's `alloc`/absence exactly |
| E1027 | array type argument has an unresolved length | defensive monomorphization refusal; unreachable from checked source (sizes parse as literal/name, runtime repeat lengths are E0717) — report if seen |
| E1099 | monomorphization exceeded the depth limit | break unbounded instantiation (resource limit, not a type error) |
