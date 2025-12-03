//! Layout algorithms for tree-like graphs.

use std::collections::HashMap;

use egui::{Context, Id, Pos2, Vec2, pos2};

use crate::ui::TreeizeViewer;
use crate::ui::state::NodeState;
use crate::{NodeId, Treeize};

/// Configuration for tree layout algorithm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutConfig {
  /// Horizontal spacing between nodes at the same level.
  pub horizontal_spacing: f32,

  /// Vertical spacing between levels.
  pub vertical_spacing: f32,

  /// Starting position for the layout.
  pub start_pos: Pos2,
}

impl Default for LayoutConfig {
  fn default() -> Self {
    LayoutConfig { horizontal_spacing: 200.0, vertical_spacing: 150.0, start_pos: Pos2::ZERO }
  }
}

/// Node dimension provider for the layout algorithm.
/// This trait allows the layout algorithm to query node sizes and children.
trait NodeDimensionProvider {
  /// Returns `(width, height)` for a node.
  fn get_size(&self, node_id: NodeId) -> (f32, f32);

  /// Returns the list of child node IDs for a given node.
  fn get_children(&self, node_id: NodeId) -> Vec<NodeId>;
}

/// Adapter that implements `NodeDimensionProvider` for `Treeize`.
struct TreeizeAdapter<'a, T> {
  node_sizes: Option<&'a HashMap<NodeId, Vec2>>,
  children_map: HashMap<NodeId, Vec<NodeId>>,
  config: LayoutConfig,
  _phantom: std::marker::PhantomData<T>,
}

impl<'a, T> TreeizeAdapter<'a, T> {
  fn new(
    treeize: &Treeize<T>,
    node_sizes: Option<&'a HashMap<NodeId, Vec2>>,
    has_output: &mut impl FnMut(NodeId) -> bool,
    has_input: &mut impl FnMut(NodeId) -> bool,
    config: LayoutConfig,
  ) -> Self {
    // Build children map from wires
    let mut children_map: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for (out_pin, in_pin) in treeize.wires() {
      let from_node = out_pin.node;
      let to_node = in_pin.node;

      if has_output(from_node) && has_input(to_node) {
        children_map.entry(from_node).or_default().push(to_node);
      }
    }

    TreeizeAdapter { node_sizes, children_map, config, _phantom: std::marker::PhantomData }
  }
}

impl<T> NodeDimensionProvider for TreeizeAdapter<'_, T> {
  fn get_size(&self, node_id: NodeId) -> (f32, f32) {
    if let Some(sizes) = self.node_sizes
      && let Some(size) = sizes.get(&node_id)
    {
      return (size.x, size.y);
    }
    // Default size if not provided
    (self.config.horizontal_spacing, self.config.vertical_spacing)
  }

  fn get_children(&self, node_id: NodeId) -> Vec<NodeId> {
    self.children_map.get(&node_id).cloned().unwrap_or_default()
  }
}

/// Internal layout node structure.
struct LayoutNode {
  /// X offset relative to parent node's center.
  offset_x: f32,
  /// Contour: list of `(left_edge, right_edge)` for each depth level.
  /// Depth 0 is the node itself, depth 1 is its children, etc.
  contour: Vec<(f32, f32)>,
  width: f32,
  height: f32,
}

/// Layout state that stores intermediate calculations.
struct LayoutState {
  nodes: HashMap<NodeId, LayoutNode>,
}

impl LayoutState {
  fn new() -> Self {
    Self { nodes: HashMap::new() }
  }
}

/// First walk: bottom-up traversal to calculate contours and offsets.
fn first_walk<P: NodeDimensionProvider>(
  node_id: NodeId,
  provider: &P,
  config: &LayoutConfig,
  state: &mut LayoutState,
) {
  let (w, h) = provider.get_size(node_id);
  let children = provider.get_children(node_id);

  if children.is_empty() {
    // Leaf node: contour is just the node itself
    state.nodes.insert(
      node_id,
      LayoutNode { offset_x: 0.0, contour: vec![(-w / 2.0, w / 2.0)], width: w, height: h },
    );
    return;
  }

  // 1. Recursively process all children
  for child_id in &children {
    first_walk(*child_id, provider, config, state);
  }

  // 2. Merge children contours (Contour Merge - the core of Mr.Tree algorithm)
  let mut merged_contour: Vec<(f32, f32)> = Vec::new();
  let mut child_offsets: Vec<f32> = Vec::new();
  let mut current_offset = 0.0;

  for (i, child_id) in children.iter().enumerate() {
    let child_node = state.nodes.get(child_id).unwrap();

    if i == 0 {
      // First subtree: use as-is
      merged_contour.clone_from(&child_node.contour);
      child_offsets.push(0.0);
    } else {
      // Subsequent subtrees: calculate minimum shift to avoid overlap
      let mut shift = 0.0f32;

      // Check all overlapping depth levels
      for ((_l1, r1), (l2, _r2)) in merged_contour.iter().zip(child_node.contour.iter()) {
        // Calculate required distance: r1 + gap - l2
        let dist = (r1 + config.horizontal_spacing) - l2;
        if dist > shift {
          shift = dist;
        }
      }

      child_offsets.push(shift);
      current_offset = shift;

      // Merge the new subtree's contour into the merged contour
      for (depth, (l, r)) in child_node.contour.iter().enumerate() {
        let shifted_l = l + shift;
        let shifted_r = r + shift;

        if depth < merged_contour.len() {
          // Extend existing level
          let (exist_l, exist_r) = merged_contour[depth];
          merged_contour[depth] = (exist_l.min(shifted_l), exist_r.max(shifted_r));
        } else {
          // Add new level
          merged_contour.push((shifted_l, shifted_r));
        }
      }
    }
  }

  // 3. Calculate parent node position
  // Parent should be centered above its children
  // The center of children group is at current_offset / 2.0
  let children_center = current_offset / 2.0;

  // 4. Generate final contour for this node
  // Layer 0: the node itself
  // Layer 1+: children group, shifted so that children center aligns with parent center (0.0)
  let shift_children_left = -children_center;
  let mut final_contour = Vec::new();
  final_contour.push((-w / 2.0, w / 2.0)); // Layer 0: parent node

  // Layer 1+: children
  for (l, r) in merged_contour {
    final_contour.push((l + shift_children_left, r + shift_children_left));
  }

  // 5. Update child offsets in LayoutState
  for (i, child_id) in children.iter().enumerate() {
    if let Some(node) = state.nodes.get_mut(child_id) {
      // Final X offset = offset within children group + group's offset relative to parent
      node.offset_x = child_offsets[i] + shift_children_left;
    }
  }

  // 6. Store this node's layout info
  state.nodes.insert(
    node_id,
    LayoutNode {
      offset_x: 0.0, // Root nodes have 0 offset (will be set in second_walk)
      contour: final_contour,
      width: w,
      height: h,
    },
  );
}

/// Second walk: top-down traversal to calculate absolute positions.
fn second_walk<P: NodeDimensionProvider>(
  node_id: NodeId,
  absolute_x: f32,
  absolute_y: f32,
  state: &LayoutState,
  positions: &mut HashMap<NodeId, Pos2>,
  provider: &P,
  config: &LayoutConfig,
) {
  let layout_node = state.nodes.get(&node_id).unwrap();

  // Calculate absolute position (top-left corner)
  // absolute_x is the parent's center X
  // layout_node.offset_x is the offset from parent's center
  let center_x = absolute_x + layout_node.offset_x;
  let top_left_x = center_x - layout_node.width / 2.0;

  positions.insert(node_id, pos2(top_left_x, absolute_y));

  let children = provider.get_children(node_id);
  let next_y = absolute_y + layout_node.height + config.vertical_spacing;

  for child_id in children {
    second_walk(child_id, center_x, next_y, state, positions, provider, config);
  }
}

/// Performs hierarchical layout on a tree-like graph.
///
/// This function arranges nodes in a top-to-bottom tree structure using
/// a compact contour-based layout algorithm (similar to ELK Mr.Tree).
/// Nodes are organized into levels based on their distance from root nodes.
///
/// # Arguments
///
/// * `treeize` - The tree graph to layout
/// * `config` - Layout configuration parameters
/// * `has_output` - Function to check if a node has output pins
/// * `has_input` - Function to check if a node has input pins
/// * `node_sizes` - Optional map from node ID to node size (width, height).
///   If provided, the layout will consider actual node sizes to avoid overlaps.
///
/// # Returns
///
/// A map from node ID to its calculated position (top-left corner).
///
/// # Panics
///
/// This function does not panic, but may produce unexpected layouts if the graph
/// contains cycles or if node sizes are invalid.
#[allow(clippy::implicit_hasher)]
pub fn layout_tree<T>(
  treeize: &Treeize<T>,
  config: LayoutConfig,
  mut has_output: impl FnMut(NodeId) -> bool,
  mut has_input: impl FnMut(NodeId) -> bool,
  node_sizes: Option<&HashMap<NodeId, Vec2>>,
) -> HashMap<NodeId, Pos2> {
  let mut positions = HashMap::new();

  // Build adapter
  let adapter = TreeizeAdapter::new(treeize, node_sizes, &mut has_output, &mut has_input, config);

  // Find root nodes (nodes with no incoming edges)
  let mut has_incoming: HashMap<NodeId, bool> = HashMap::new();
  for (out_pin, in_pin) in treeize.wires() {
    let to_node = in_pin.node;
    if has_output(out_pin.node) && has_input(to_node) {
      has_incoming.insert(to_node, true);
    }
  }

  let root_nodes: Vec<NodeId> = treeize
    .node_ids()
    .map(|(node_id, _)| node_id)
    .filter(|node_id| !has_incoming.get(node_id).copied().unwrap_or(false))
    .collect();

  if root_nodes.is_empty() {
    // No root nodes found, treat all nodes as disconnected
    for (node_id, _) in treeize.node_ids() {
      positions.insert(node_id, config.start_pos);
    }
    return positions;
  }

  // Calculate layout for each root node's tree
  let mut layout_state = LayoutState::new();
  let mut root_x_offset = 0.0;

  for root_id in &root_nodes {
    // First walk: calculate contours and offsets
    first_walk(*root_id, &adapter, &config, &mut layout_state);

    // Calculate the width of this root's tree
    let root_node = layout_state.nodes.get(root_id).unwrap();
    let tree_width = root_node.contour.iter().map(|(l, r)| r - l).fold(0.0f32, f32::max);

    // Second walk: calculate absolute positions
    // Start at config.start_pos.x + root_x_offset (centered)
    let root_center_x = config.start_pos.x + root_x_offset + tree_width / 2.0;
    second_walk(
      *root_id,
      root_center_x,
      config.start_pos.y,
      &layout_state,
      &mut positions,
      &adapter,
      &config,
    );

    // Update offset for next root
    root_x_offset += tree_width + config.horizontal_spacing;
  }

  // Handle disconnected nodes (nodes not reachable from any root)
  for (node_id, _) in treeize.node_ids() {
    positions.entry(node_id).or_insert(config.start_pos);
  }

  positions
}

/// Applies the calculated layout positions to the treeize.
///
/// # Arguments
///
/// * `treeize` - The tree graph to update
/// * `positions` - Map from node ID to position
pub fn apply_layout<T, H>(treeize: &mut Treeize<T>, positions: &HashMap<NodeId, Pos2, H>)
where
  H: std::hash::BuildHasher,
{
  for (node_id, pos) in positions {
    if let Some(node) = treeize.get_node_info_mut(*node_id) {
      node.pos = *pos;
    }
  }
}

/// Convenience function that performs layout and applies it in one step.
///
/// # Arguments
///
/// * `treeize` - The tree graph to layout and update
/// * `config` - Layout configuration parameters
/// * `has_output` - Function to check if a node has output pins
/// * `has_input` - Function to check if a node has input pins
/// * `node_sizes` - Optional map from node ID to node size (width, height).
///   If provided, the layout will consider actual node sizes to avoid overlaps.
///
/// # Example
///
/// ```rs
/// use egui_treeize::{Treeize, layout::{LayoutConfig, layout_and_apply}};
/// use std::collections::HashMap;
///
/// struct MyNode;
///
/// let mut treeize = Treeize::<MyNode>::new();
/// let config = LayoutConfig {
///     horizontal_spacing: 200.0,
///     vertical_spacing: 150.0,
///     start_pos: egui::pos2(0.0, 0.0),
/// };
///
/// // Optional: provide node sizes
/// let mut node_sizes = HashMap::new();
/// // node_sizes.insert(node_id, egui::vec2(100.0, 50.0));
///
/// layout_and_apply(
///     &mut treeize,
///     config,
///     |node_id| {
///         // Check if node has output pins
///         let node = &treeize[node_id];
///         // Your logic here
///         true
///     },
///     |node_id| {
///         // Check if node has input pins
///         let node = &treeize[node_id];
///         // Your logic here
///         true
///     },
///     Some(&node_sizes),  // or None to ignore sizes
/// );
/// ```
#[allow(clippy::implicit_hasher)]
pub fn layout_and_apply<T>(
  treeize: &mut Treeize<T>,
  config: LayoutConfig,
  has_output: impl FnMut(NodeId) -> bool,
  has_input: impl FnMut(NodeId) -> bool,
  node_sizes: Option<&HashMap<NodeId, Vec2>>,
) {
  let positions = layout_tree(treeize, config, has_output, has_input, node_sizes);
  apply_layout(treeize, &positions);
}

/// Convenience function that performs layout using a `TreeizeViewer`.
///
/// This function automatically uses the viewer's `has_input` and `has_output` methods
/// to determine node connectivity.
///
/// # Arguments
///
/// * `treeize` - The tree graph to layout and update
/// * `viewer` - The [`TreeizeViewer`] implementation
/// * `config` - Layout configuration parameters
/// * `ctx` - The egui [`Context`] to read node sizes from [`NodeState`]
/// * `treeize_id` - The ID of the treeize widget (used to construct node IDs)
///
/// # Example
///
/// ```rs
/// use egui_treeize::{Treeize, layout::{LayoutConfig, layout_with_viewer}};
/// use egui_treeize::ui::TreeizeViewer;
///
/// struct MyViewer;
/// struct MyNode;
///
/// impl TreeizeViewer<MyNode> for MyViewer {
///     // ... implement required methods
/// }
///
/// let mut treeize = Treeize::<MyNode>::new();
/// let mut viewer = MyViewer;
/// let config = LayoutConfig::default();
/// let ctx = ui.ctx();  // Get from your UI
/// let treeize_id = treeize_widget.get_id(ui.id());  // Get from your TreeizeWidget
///
/// layout_with_viewer(&mut treeize, &mut viewer, config, ctx, treeize_id);
/// ```
pub fn layout_with_viewer<T, V>(
  treeize: &mut Treeize<T>,
  viewer: &mut V,
  config: LayoutConfig,
  ctx: &Context,
  treeize_id: Id,
) where
  V: TreeizeViewer<T>,
{
  // Read node sizes from NodeState stored in Context
  let mut node_sizes_map: HashMap<NodeId, Vec2> = HashMap::new();

  for (node_id, _) in treeize.node_ids() {
    let node_state_id = treeize_id.with(("treeize-node", node_id));
    if let Some(node_data) = NodeState::pick_data(ctx, node_state_id) {
      node_sizes_map.insert(node_id, node_data.size);
    }
  }

  let node_sizes = if node_sizes_map.is_empty() { None } else { Some(&node_sizes_map) };

  // First, collect all node data and compute has_input/has_output to avoid borrowing issues
  let node_info: Vec<(NodeId, bool, bool)> = treeize
    .node_ids()
    .map(|(node_id, data)| {
      let has_out = viewer.has_output(data);
      let has_in = viewer.has_input(data);
      (node_id, has_out, has_in)
    })
    .collect();

  // Create a lookup map
  let mut has_output_map: HashMap<NodeId, bool> = HashMap::new();
  let mut has_input_map: HashMap<NodeId, bool> = HashMap::new();

  for (node_id, has_out, has_in) in &node_info {
    has_output_map.insert(*node_id, *has_out);
    has_input_map.insert(*node_id, *has_in);
  }

  // Create closures that use the maps
  let has_output_fn =
    |node_id: NodeId| -> bool { has_output_map.get(&node_id).copied().unwrap_or(false) };

  let has_input_fn =
    |node_id: NodeId| -> bool { has_input_map.get(&node_id).copied().unwrap_or(false) };

  layout_and_apply(treeize, config, has_output_fn, has_input_fn, node_sizes);
}
