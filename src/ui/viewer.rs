use egui::{Painter, Pos2, Rect, Style, Ui, emath::TSTransform};

use crate::{InPin, InPinId, NodeId, OutPin, OutPinId, Treeize};

use super::{
  BackgroundPattern, NodeLayout, TreeizeStyle,
  pin::{AnyPins, TreeizePin},
};

/// `TreeizeViewer` is a trait for viewing a Treeize.
///
/// It can extract necessary data from the nodes and controls their
/// response to certain events.
pub trait TreeizeViewer<T> {
  /// Returns title of the node.
  fn title(&mut self, node: &T) -> String;

  /// Returns the node's frame.
  /// All node's elements will be rendered inside this frame.
  /// Except for pins if they are configured to be rendered outside of the frame.
  ///
  /// Returns `default` by default.
  /// `default` frame is taken from the [`TreeizeStyle::node_frame`] or constructed if it's `None`.
  ///
  /// Override this method to customize the frame for specific nodes.
  fn node_frame(
    &mut self,
    default: egui::Frame,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    treeize: &Treeize<T>,
  ) -> egui::Frame {
    let _ = (node, inputs, outputs, treeize);
    default
  }

  /// Returns the node's header frame.
  ///
  /// This frame would be placed on top of the node's frame.
  /// And header UI (see [`show_header`]) will be placed inside this frame.
  ///
  /// Returns `default` by default.
  /// `default` frame is taken from the [`TreeizeStyle::header_frame`],
  /// or [`TreeizeStyle::node_frame`] with removed shadow if `None`,
  /// or constructed if both are `None`.
  fn header_frame(
    &mut self,
    default: egui::Frame,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    treeize: &Treeize<T>,
  ) -> egui::Frame {
    let _ = (node, inputs, outputs, treeize);
    default
  }
  /// Checks if node has a custom egui style.
  #[inline]
  fn has_node_style(
    &mut self,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    treeize: &Treeize<T>,
  ) -> bool {
    let _ = (node, inputs, outputs, treeize);
    false
  }

  /// Modifies the node's egui style
  fn apply_node_style(
    &mut self,
    style: &mut Style,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    treeize: &Treeize<T>,
  ) {
    let _ = (style, node, inputs, outputs, treeize);
  }

  /// Returns elements layout for the node.
  ///
  /// Node consists of 5 parts: header, body, footer, input pins and output pins.
  /// See [`NodeLayout`] for available placements.
  ///
  /// Returns `default` by default.
  /// `default` layout is taken from the [`TreeizeStyle::node_layout`] or constructed if it's `None`.
  /// Override this method to customize the layout for specific nodes.
  #[inline]
  fn node_layout(
    &mut self,
    default: NodeLayout,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    treeize: &Treeize<T>,
  ) -> NodeLayout {
    let _ = (node, inputs, outputs, treeize);
    default
  }

  /// Renders elements inside the node's header frame.
  ///
  /// This is the good place to show the node's title and controls related to the whole node.
  ///
  /// By default it shows the node's title.
  #[inline]
  fn show_header(
    &mut self,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    ui: &mut Ui,
    treeize: &mut Treeize<T>,
  ) {
    let _ = (inputs, outputs);
    ui.label(self.title(&treeize[node]));
  }

  /// Returns number of input pins of the node.
  ///
  /// [`TreeizeViewer::show_input`] will be called for each input in range `0..inputs()`.
  fn inputs(&mut self, node: &T) -> usize;

  /// Renders one specified node's input element and returns drawer for the corresponding pin.
  fn show_input(
    &mut self,
    pin: &InPin,
    ui: &mut Ui,
    treeize: &mut Treeize<T>,
  ) -> impl TreeizePin + 'static;

  /// Returns number of output pins of the node.
  ///
  /// [`TreeizeViewer::show_output`] will be called for each output in range `0..outputs()`.
  fn outputs(&mut self, node: &T) -> usize;

  /// Renders the node's output.
  fn show_output(
    &mut self,
    pin: &OutPin,
    ui: &mut Ui,
    treeize: &mut Treeize<T>,
  ) -> impl TreeizePin + 'static;

  /// Checks if node has something to show in body - between input and output pins.
  #[inline]
  fn has_body(&mut self, node: &T) -> bool {
    let _ = node;
    false
  }

  /// Renders the node's body.
  #[inline]
  fn show_body(
    &mut self,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    ui: &mut Ui,
    treeize: &mut Treeize<T>,
  ) {
    let _ = (node, inputs, outputs, ui, treeize);
  }

  /// Checks if node has something to show in footer - below pins and body.
  #[inline]
  fn has_footer(&mut self, node: &T) -> bool {
    let _ = node;
    false
  }

  /// Renders the node's footer.
  #[inline]
  fn show_footer(
    &mut self,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    ui: &mut Ui,
    treeize: &mut Treeize<T>,
  ) {
    let _ = (node, inputs, outputs, ui, treeize);
  }

  /// Reports the final node's rect after rendering.
  ///
  /// It aimed to be used for custom positioning of nodes that requires node dimensions for calculations.
  /// Node's position can be modified directly in this method.
  #[inline]
  fn final_node_rect(&mut self, node: NodeId, rect: Rect, ui: &mut Ui, treeize: &mut Treeize<T>) {
    let _ = (node, rect, ui, treeize);
  }

  /// Checks if node has something to show in on-hover popup.
  #[inline]
  fn has_on_hover_popup(&mut self, node: &T) -> bool {
    let _ = node;
    false
  }

  /// Renders the node's on-hover popup.
  #[inline]
  fn show_on_hover_popup(
    &mut self,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    ui: &mut Ui,
    treeize: &mut Treeize<T>,
  ) {
    let _ = (node, inputs, outputs, ui, treeize);
  }

  /// Checks if wire has something to show in widget.
  /// This may not be called if wire is invisible.
  #[inline]
  fn has_wire_widget(&mut self, from: &OutPinId, to: &InPinId, treeize: &Treeize<T>) -> bool {
    let _ = (from, to, treeize);
    false
  }

  /// Renders the wire's widget.
  /// This may not be called if wire is invisible.
  #[inline]
  fn show_wire_widget(&mut self, from: &OutPin, to: &InPin, ui: &mut Ui, treeize: &mut Treeize<T>) {
    let _ = (from, to, ui, treeize);
  }

  /// Checks if the treeize has something to show in context menu if right-clicked or long-touched on empty space at `pos`.
  #[inline]
  fn has_graph_menu(&mut self, pos: Pos2, treeize: &mut Treeize<T>) -> bool {
    let _ = (pos, treeize);
    false
  }

  /// Show context menu for the treeize.
  ///
  /// This can be used to implement menu for adding new nodes.
  #[inline]
  fn show_graph_menu(&mut self, pos: Pos2, ui: &mut Ui, treeize: &mut Treeize<T>) {
    let _ = (pos, ui, treeize);
  }

  /// Checks if the treeize has something to show in context menu if wire drag is stopped at `pos`.
  #[inline]
  fn has_dropped_wire_menu(&mut self, src_pins: AnyPins, treeize: &mut Treeize<T>) -> bool {
    let _ = (src_pins, treeize);
    false
  }

  /// Show context menu for the treeize. This menu is opened when releasing a pin to empty
  /// space. It can be used to implement menu for adding new node, and directly
  /// connecting it to the released wire.
  #[inline]
  fn show_dropped_wire_menu(
    &mut self,
    pos: Pos2,
    ui: &mut Ui,
    src_pins: AnyPins,
    treeize: &mut Treeize<T>,
  ) {
    let _ = (pos, ui, src_pins, treeize);
  }

  /// Checks if the node has something to show in context menu if right-clicked or long-touched on the node.
  #[inline]
  fn has_node_menu(&mut self, node: &T) -> bool {
    let _ = node;
    false
  }

  /// Show context menu for the treeize.
  ///
  /// This can be used to implement menu for adding new nodes.
  #[inline]
  fn show_node_menu(
    &mut self,
    node: NodeId,
    inputs: &[InPin],
    outputs: &[OutPin],
    ui: &mut Ui,
    treeize: &mut Treeize<T>,
  ) {
    let _ = (node, inputs, outputs, ui, treeize);
  }

  /// Asks the viewer to connect two pins.
  ///
  /// This is usually happens when user drags a wire from one node's output pin to another node's input pin or vice versa.
  /// By default this method connects the pins and returns `Ok(())`.
  #[inline]
  fn connect(&mut self, from: &OutPin, to: &InPin, treeize: &mut Treeize<T>) {
    treeize.connect(from.id, to.id);
  }

  /// Asks the viewer to disconnect two pins.
  #[inline]
  fn disconnect(&mut self, from: &OutPin, to: &InPin, treeize: &mut Treeize<T>) {
    treeize.disconnect(from.id, to.id);
  }

  /// Asks the viewer to disconnect all wires from the output pin.
  ///
  /// This is usually happens when right-clicking on an output pin.
  /// By default this method disconnects the pins and returns `Ok(())`.
  #[inline]
  fn drop_outputs(&mut self, pin: &OutPin, treeize: &mut Treeize<T>) {
    treeize.drop_outputs(pin.id);
  }

  /// Asks the viewer to disconnect all wires from the input pin.
  ///
  /// This is usually happens when right-clicking on an input pin.
  /// By default this method disconnects the pins and returns `Ok(())`.
  #[inline]
  fn drop_inputs(&mut self, pin: &InPin, treeize: &mut Treeize<T>) {
    treeize.drop_inputs(pin.id);
  }

  /// Draws background of the treeize view.
  ///
  /// By default it draws the background pattern using [`BackgroundPattern::draw`].
  ///
  /// If you want to draw the background yourself, you can override this method.
  #[inline]
  fn draw_background(
    &mut self,
    background: Option<&BackgroundPattern>,
    viewport: &Rect,
    treeize_style: &TreeizeStyle,
    style: &Style,
    painter: &Painter,
    treeize: &Treeize<T>,
  ) {
    let _ = treeize;

    if let Some(background) = background {
      background.draw(viewport, treeize_style, style, painter);
    }
  }

  /// Informs the viewer what is the current transform of the treeize view
  /// and allows viewer to override it.
  ///
  /// This method is called in the beginning of the graph rendering.
  ///
  /// By default it does nothing.
  #[inline]
  fn current_transform(&mut self, to_global: &mut TSTransform, treeize: &mut Treeize<T>) {
    let _ = (to_global, treeize);
  }
}
