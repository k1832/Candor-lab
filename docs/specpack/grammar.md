# Candor grammar — the real surface by example

Source: spec 01 (lexis) + 02 (grammar) + design 0006. Parses with **≤2-token
lookahead, no symbol table** (NN#13). Example-first; every construct shown in
canonical form (the formatter's output is the only conforming form).

## Items
```
copy struct Point { x: i64, y: i64 }              // copy: opt-in, all fields copy, no drop hook
struct Node { v: i64, next: Box Node }            // owning; moves by default
struct R { id: i64 } drop(write self) { trace(self.id); }   // at most one drop hook; receiver EXACTLY `write self`
enum Opt[T] { ok Some(T), None }                  // `ok` marks the ONE success variant (result-shaped)
enum Res[T, E] { ok Ok(T), Err(E) }               // payloads positional; 0..n per variant
static MAX: i64 = 100;                            // program-lifetime, IMMUTABLE, initialized by an Expr
pub fn f(...) ... { ... }                         // `pub` exports across modules (design 0008)
```
- `copy` may precede only `struct`/`enum`. A `copy` type may not have a `drop` hook.
- Generic items: `struct List[T]`, `enum Res[T, E]`, `fn id[T](x: T)`, `interface Iter { ... }`.
  Bounds: `[T: Ord + copy]` (`+` conjoins). Region+type mixed list: `fn choose[region r, T](...)`.
- `interface I { type Item; fn m(...) ...; }` — at most one `type` member; methods carry full sigs.
  `impl[T] I for List[T] { type Item = T; fn m(...) {...} }` (orphan rule: impl in T's or I's module).

## Types
```
i8 i16 i32 i64 isize   u8 u16 u32 u64 usize   bool   unit    // scalars; isize/usize target-width
rawptr T     Box T     BoxResult T                            // pointer / heap-own / compiler-known box result
read T       write T                                          // shared / exclusive borrow (keywords, never &)
[T]          write [T]      read[r] [T]     write[r] [T]      // shared slice / excl slice / region-tagged slices
[N]T                                                          // owned fixed array, N a const (INT or const name)
fn(a: read T, n: usize) alloc -> rawptr u8                    // fn-pointer: param MODES + effect marker in the type
Name         Name[Args]     T::Item                           // named type / generic application / assoc-type projection
```
- Borrows wear keywords `read`/`write`; `&` is ONLY bitwise-and (frees the `&` sigil).
- A struct/enum field may NOT be a borrow/slice type (E0201). Fields are owned, `Box`, or `rawptr`.
- `copy` types: all integers, `bool`, `unit`, every `rawptr T`, every `read T`, `[T]`, `[N]T` iff T copy,
  and a `struct`/`enum` marked `copy` with all-copy payloads and no drop hook. Everything else MOVES.

## Modes (parameters) — take-by-omission
```
fn g(a: T, b: read T, c: write T, d: out T) -> T { ... }
```
| mode | meaning | caller writes |
|------|---------|---------------|
| `take` (omit) | move in (copy if copy); callee owns | `g(x)` (bare; borrow-typed arg passed by value) |
| `read`  | shared borrow; caller keeps ownership | `g(read x)` on owned; `g(b)` reborrow from a borrow |
| `write` | exclusive borrow; caller can't touch during call | `g(write x)` on owned; `g(b)` reborrow from excl |
| `out`   | caller passes an owned slot the callee initializes | `g(out place)` (the `out` marker is mandatory) |
- Return `-> [read[r]|write] T`: omitted mode = by-value move-out. RVO is a move, never a hidden copy.
- SigTail after `)`: any order of `alloc`, `requires(Expr)`, `ensures(Expr)`. `ensures` may name `result`.
- `alloc` is the sole tracked effect this edition (contextual keyword, effect-marker slot only).

Contract clauses sit between the parameter list and the effect/return tail, in
this order: `fn f(x: i64) requires(x > 0) ensures(result >= x) alloc -> i64`.


## Expressions — precedence (tightest first; NORMATIVE)
```
1  postfix .* , field . , index a[i] , call f(...)      (left; .* binds tightest: s.*.f = (s.*).f)
2  postfix ?
3  prefix - ! ~ ; borrow read/write ; conv T            (prefix)
4  * / %      5  + -      6  << >>      7  &      8  ^      9  |
10 == != < > <= >=   (NON-associative: `a < b < c` is a parse error)
11 &&      12 ||      13 = (assignment; STATEMENT position only, not an operator)
```
```
b.*                       // deref a Box/borrow to its pointee value
s.base    s.mem[i]        // READ path: auto-derefs through a borrow (no .* needed)
p.*.f = e   p.*.mem[i]=e  // WRITE path: EVERY deref explicit (E0713 if omitted)
conv usize i              // conv <ScalarKw> <prefix-operand>; parens only if needed: conv i64 (a + b)
expr?                     // unwrap ok-variant of a result-shaped enum, else return it (same enum type only)
-9223372036854775808i64   // neg-literal fold: `- INT` is one range-checked constant; `-(...)` is not
```
Primaries: `INT` `STRING` `true` `false` `self` `result` `break` `continue`
`return [Expr]` `assert(Expr)` `panic(Expr)` `(Expr)` array-literals
`[a, b, c]` / `[v; n]` / `[]`, block-like exprs, intrinsics, `E::V(args)`
(enum ctor), `S { f: e, g: e }` (struct literal), bare `IDENT`.

Intrinsics (keyword-led, bracket carries the type argument):
```
sizeof(T)  alignof(T)  min_of(T)  max_of(T)  offsetof(T, field)  field_ptr(p, field)
cast_ptr[U](p)  addr_to_ptr[T](a)  ptr_null[T]()
```
Unsafe-gated pointer ops (only inside `unsafe`): `addr_of(pl)` `addr_of_mut(pl)`
`ptr_read(p)` `ptr_write(p, v)` `ptr_offset(p, n)` `ptr_to_addr(p)`. Safe queries:
`is_null(p)`, `field_ptr`, `sizeof`/`alignof`/`offsetof`. Compiler-known box ops:
`box(a, v) -> BoxResult T`, `unbox(b) -> T`, `clone place`.

## Statements
```
let x: T = e;      let mut y = e;      let z: T;          // type optional if inferable; `mut` for reassignment
place = expr;                                              // assignment (LHS is a place); drops old value first
f(x);                                                      // expression statement
if c { ... } else { ... }   while c { ... }   loop { ... } match s { ... }
wrapping { ... }   saturating { ... }   unsafe "why" { ... }   // block-like: NO trailing `;`, following `(` = new stmt
```

## match / patterns / control
```
match scrutinee {                 // scrutinee is ExprNoStruct (bare `Ident {` is a block, not a struct literal)
    Opt::Some(v) => { ... }        // binding modes fixed by how scrutinee is held (owned=move, borrowed=borrow)
    Opt::None    => { ... }        // arms comma-separated; comma OPTIONAL after a block-bodied arm
}
```
Patterns: `_` (wildcard), `IDENT` (binding), `E::V(pats)` (variant, nested ok).
`match` must be exhaustive (E0601). `if`/`while`/`match` heads are `ExprNoStruct`.

## Iteration (design 0009)
```
for x in read coll { ... }    // Indexed protocol: borrows coll, copies each Item out; non-alloc (ground floor)
for x in coll { ... }         // Iter protocol: MOVES coll; next(take self) consumes; alloc-marked
```
`for`/`in`/`type` are contextual keywords. Operand is `ExprNoStruct`.

## Contextual vs hard keywords
- Contextual (identifiers elsewhere): `alloc`, `ok`, `for`, `in`, `type`.
- Hard (reserved everywhere, incl. `out`): the item/mode/control/contract/regime
  words, scalar+`bool`+`unit`, `rawptr`/`Box`/`BoxResult`, and the intrinsic
  keywords. `::` is ONLY the enum-variant / module path; `->` only the return
  arrow; `?` only postfix propagation; `>>` always shift (no `<>` generics).

## Canonical form (formatter is the only form)
4-space indent, K&R braces, mandatory blocks (no brace-less bodies), one stmt per
line, simple stmts end `;` and block-like take none, spaces around binary ops /
`=>` / after `,` / after each mode keyword and none for `.*`/`?`/`.`/`[]`/`()`,
trailing commas in multi-line lists. `snake_case` fns/locals/fields, `PascalCase`
types/variants, `SCREAMING_SNAKE` statics. Formatter also: collapses reborrows
`f(write b.*)`→`f(b)`, drops read-path `.*`, removes redundant parens.

## Divergences (spec/design ahead of the prototype — generate the SPACE form)
- **`Box[T]` / `BoxResult[T]{ ok Boxed, OutOfMemory }`** (design 0007 §6.3) is
  ahead. The prototype + corpus use `Box T` (space form) and the variants
  `BoxResult::boxed` / `BoxResult::oom` (lowercase). **Write `Box T`.**
- **`min_of`/`max_of`** are **reserved but not implemented** (01 §2.3 downgraded,
  02 §6.8; chapter 99 OBL-MINMAX-INTRINSICS) and not in the corpus. Do not use
  them — the negative-literal fold covers the programmatic bound.
- **Spec ch03/04 spellings — RESOLVED (2026-07-09):** those chapters now use the
  real spellings (`read T`/`write T`/`[T]`/`write [T]`/`.*`), matching this file,
  ch01/02, and the corpus (formerly lagged in throwaway `borrow T`/`slice T`/
  `deref b`).
