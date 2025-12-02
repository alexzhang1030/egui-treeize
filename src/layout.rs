//! Layout algorithms for tree-like graphs.

use std::collections::HashMap;

use dagre_rs::layout::{DagreLayout, LayoutOptions, LayoutResult, RankDir};
use egui::{Context, Id, Pos2, Vec2, pos2};
use petgraph::{graph::DiGraph, visit::EdgeRef};

use crate::ui::TreeizeViewer;
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

/// Performs hierarchical layout on a tree-like graph.
///
/// This function arranges nodes in a top-to-bottom tree structure.
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
/// A map from node ID to its calculated position.
#[allow(clippy::implicit_hasher)]
pub fn layout_tree<T>(
  treeize: &Treeize<T>,
  config: LayoutConfig,
  mut has_output: impl FnMut(NodeId) -> bool,
  mut has_input: impl FnMut(NodeId) -> bool,
  node_sizes: Option<&HashMap<NodeId, Vec2>>,
) -> HashMap<NodeId, Pos2> {
  let mut positions = HashMap::new();

  // Build petgraph DiGraph
  let mut graph: DiGraph<NodeId, ()> = DiGraph::new();
  let mut node_index_map: HashMap<NodeId, petgraph::graph::NodeIndex> = HashMap::new();

  // Add all nodes to the graph
  for (node_id, _) in treeize.node_ids() {
    let idx = graph.add_node(node_id);
    node_index_map.insert(node_id, idx);
  }

  // Add edges from wires
  for (out_pin, in_pin) in treeize.wires() {
    let from_node = out_pin.node;
    let to_node = in_pin.node;

    if has_output(from_node)
      && has_input(to_node)
      && let (Some(&from_idx), Some(&to_idx)) =
        (node_index_map.get(&from_node), node_index_map.get(&to_node))
    {
      graph.add_edge(from_idx, to_idx, ());
    }
  }

  // Use dagre_rs to calculate hierarchical layout
  let layout_options = LayoutOptions {
    rank_dir: RankDir::TopToBottom,
    node_sep: config.horizontal_spacing,
    rank_sep: config.vertical_spacing,
    ..Default::default()
  };

  let dagre_layout = DagreLayout::with_options(layout_options);

  // Create a graph with node sizes for dagre
  let mut dagre_graph: DiGraph<(NodeId, f32, f32), ()> = DiGraph::new();
  let mut dagre_node_map: HashMap<NodeId, petgraph::graph::NodeIndex> = HashMap::new();

  // Add nodes to dagre graph with their sizes
  for (node_id, _) in treeize.node_ids() {
    let (width, height) = node_sizes
      .and_then(|sizes| sizes.get(&node_id))
      .map_or((config.horizontal_spacing, config.vertical_spacing), |size| (size.x, size.y));

    let dagre_idx = dagre_graph.add_node((node_id, width, height));
    dagre_node_map.insert(node_id, dagre_idx);
  }

  // Add edges to dagre graph
  for edge in graph.edge_references() {
    let from_node = graph[edge.source()];
    let to_node = graph[edge.target()];
    if let (Some(&from_idx), Some(&to_idx)) =
      (dagre_node_map.get(&from_node), dagre_node_map.get(&to_node))
    {
      dagre_graph.add_edge(from_idx, to_idx, ());
    }
  }

  // Calculate layout using dagre
  let layout_result: LayoutResult = dagre_layout.compute(&dagre_graph);

  // Use dagre's calculated positions and layers
  // Map dagre node positions to our NodeId positions
  for (node_idx, &(x, y)) in &layout_result.node_positions {
    if let Some((node_id, _, _)) = dagre_graph.node_weight(*node_idx) {
      // Apply offset from config.start_pos
      positions.insert(*node_id, pos2(config.start_pos.x + x, config.start_pos.y + y));
    }
  }

  // Handle nodes not in layout_result (disconnected nodes)
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
/// ```no_run
/// use egui_treeize::{Treeize, layout::{LayoutConfig, layout_and_apply}};
/// use std::collections::HashMap;
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
/// ```no_run
/// use egui_treeize::{Treeize, layout::{LayoutConfig, layout_with_viewer}};
/// use egui_treeize::ui::TreeizeViewer;
///
/// struct MyViewer;
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
    let node_state_id = treeize_id.with(("snarl-node", node_id));
    // NodeData is stored in Context with the node_state_id as key
    if let Some(node_data) = ctx.data(|d| d.get_temp::<crate::ui::state::NodeData>(node_state_id)) {
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
