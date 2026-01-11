# Adding a block

To create a block called `my_example`, do the following:

## 1. Required types

Create struct `MyExample` somewhere in `content::block`. Add an `impl BlockInstance for MyExample`
section. It will be filled out in the next steps.

Create struct `MyExampleKind` next to `MyExample`. Add an `impl BlockKindInstance for MyExampleKind`
section. It will be filled out in the next steps.

## 2. Properties

If the block has some per-block properties, add them as fields in the struct, otherwise leave it
empty. The struct will be duplicated in memory for each instance of the block in the level, so it
should be very lightweight.

Implement `BlockInstance::from` for `MyExample`.

## 3. View

### Simple opaque cube
If the block should appear as a simple opaque cube, add `type View = FullCube` to the impl section
for `MyExample`. Add a `FullCube` member to `MyExampleKind`, initialize it in
`BlockKindInstance::new` and return a `.clone()` of it in `BlockInstance::view`. See `Stone` for an
example.

### Custom look and feel
Otherwise, create struct `MyExampleView` next to `MyExample` and use it as `type View`. Implement
render in `impl Drawable for MyExampleView`, caching as many assets as possible in `MyExampleKind`.
For example, if the 3D model contains two moving parts, generate or load the parts in
`MyExampleKind` and store `Rc` references to them in `MyExampleView`, rather than processing them
for every `BlockInstance::view` call.

## Registration

Add a line like `my_example: MyExample` to the `all_blocks!` section in `src/content/block.rs`. Add
`mod` and `use` statements as necessary.
