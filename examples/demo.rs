#![allow(clippy::use_self)]

use eframe::{App, CreationContext};
use egui::{Id, Ui};
use egui_treeize::{
  InPin, OutPin, Treeize,
  ui::{NodeLayout, PinPlacement, TreeizePin, TreeizeStyle, TreeizeViewer, TreeizeWidget},
};

pub struct DemoNode;

pub struct DemoApp {
  treeize: Treeize<DemoNode>,
  style: TreeizeStyle,
}

const fn default_style() -> TreeizeStyle {
  TreeizeStyle {
    node_layout: Some(NodeLayout::compact()),
    pin_placement: Some(PinPlacement::Edge),
    pin_size: Some(7.0),
    node_frame: Some(egui::Frame {
      inner_margin: egui::Margin::same(8),
      outer_margin: egui::Margin { left: 0, right: 0, top: 0, bottom: 4 },
      corner_radius: egui::CornerRadius::same(8),
      fill: egui::Color32::from_gray(30),
      stroke: egui::Stroke::NONE,
      shadow: egui::Shadow::NONE,
    }),
    bg_frame: Some(egui::Frame {
      inner_margin: egui::Margin::ZERO,
      outer_margin: egui::Margin::same(2),
      corner_radius: egui::CornerRadius::ZERO,
      fill: egui::Color32::from_gray(40),
      stroke: egui::Stroke::NONE,
      shadow: egui::Shadow::NONE,
    }),
    ..TreeizeStyle::new()
  }
}

impl DemoApp {
  pub fn new(cx: &CreationContext) -> Self {
    egui_extras::install_image_loaders(&cx.egui_ctx);

    cx.egui_ctx.style_mut(|style| style.animation_time *= 10.0);

    let treeize = cx.storage.map_or_else(Treeize::new, |storage| {
      storage
        .get_string("treeize")
        .and_then(|treeize| serde_json::from_str(&treeize).ok())
        .unwrap_or_default()
    });
    // let treeize = Treeize::new();

    let style = cx.storage.map_or_else(default_style, |storage| {
      storage
        .get_string("style")
        .and_then(|style| serde_json::from_str(&style).ok())
        .unwrap_or_else(default_style)
    });
    // let style = TreeizeStyle::new();

    DemoApp { treeize, style }
  }
}

struct DemoViewer;

impl TreeizeViewer<DemoNode> for DemoViewer {
  fn title(&mut self, node: &DemoNode) -> String {
    "DemoNode".to_string()
  }
  fn has_input(&mut self, node: &DemoNode) -> bool {
    false
  }
  fn has_output(&mut self, node: &DemoNode) -> bool {
    false
  }
  fn show_input(
    &mut self,
    pin: &InPin,
    ui: &mut Ui,
    treeize: &mut Treeize<DemoNode>,
  ) -> impl TreeizePin + 'static {
    todo!()
  }
  fn show_output(
    &mut self,
    pin: &OutPin,
    ui: &mut Ui,
    treeize: &mut Treeize<DemoNode>,
  ) -> impl TreeizePin + 'static {
    todo!()
  }
}

impl App for DemoApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      TreeizeWidget::new().id(Id::new("snarl-demo")).style(self.style).show(
        &mut self.treeize,
        &mut DemoViewer,
        ui,
        None,
        None,
      );
    });
  }

  fn save(&mut self, storage: &mut dyn eframe::Storage) {
    let treeize = serde_json::to_string(&self.treeize).unwrap();
    storage.set_string("treeize", treeize);

    let style = serde_json::to_string(&self.style).unwrap();
    storage.set_string("style", style);
  }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
  let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_inner_size([400.0, 300.0])
      .with_min_inner_size([300.0, 220.0]),
    ..Default::default()
  };

  eframe::run_native(
    "egui-treeize demo",
    native_options,
    Box::new(|cx| Ok(Box::new(DemoApp::new(cx)))),
  )
}

#[cfg(target_arch = "wasm32")]
fn get_canvas_element() -> Option<web_sys::HtmlCanvasElement> {
  use eframe::wasm_bindgen::JsCast;

  let document = web_sys::window()?.document()?;
  let canvas = document.get_element_by_id("egui_treeize_demo")?;
  canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
  let canvas = get_canvas_element().expect("Failed to find canvas with id 'egui_treeize_demo'");

  let web_options = eframe::WebOptions::default();

  wasm_bindgen_futures::spawn_local(async {
    eframe::WebRunner::new()
      .start(canvas, web_options, Box::new(|cx| Ok(Box::new(DemoApp::new(cx)))))
      .await
      .expect("failed to start eframe");
  });
}

fn format_float(v: f64) -> String {
  let v = (v * 1000.0).round() / 1000.0;
  format!("{v}")
}
