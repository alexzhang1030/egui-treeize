# egui-treeize

> The original repo is [egui-snarl](https://github.com/zakarumych/egui-snarl).

---

> Status: Highly-Risky, not stable, PLEASE USE WITH CAUTION.

This project is a fork of [egui-snarl](https://github.com/zakarumych/egui-snarl) for tree-like graph visualization.

Most of the apis are the same as in [egui-snarl](https://github.com/zakarumych/egui-snarl), but the differences are:

- `Treeize` only supports Top-To-Bottom layout, so the wires and links are only allowed to go from top to bottom.
- Supports `readonly` and `editable` modes. Defaults is `readonly`.
- No draggable nodes for readonly mode, no deletable wires for dual modes.
- No multiple input/output pins for a node. We connect all wires to the same input/output pin.
- No footer.
