#![allow(clippy::use_self)]

use eframe::{App, CreationContext};
use egui::Id;
use egui_treeize::{
  Treeize,
  ui::{PinInfo, TreeizeStyle, TreeizeViewer, TreeizeWidget},
};

pub struct DemoNode;

pub struct DemoApp {
  treeize: Treeize<DemoNode>,
  style: TreeizeStyle,
}

impl DemoApp {
  pub fn new(cx: &CreationContext) -> Self {
    egui_extras::install_image_loaders(&cx.egui_ctx);

    cx.egui_ctx.style_mut(|style| style.animation_time *= 10.0);

    let treeize = Treeize::new();

    let style = TreeizeStyle::new();

    DemoApp { treeize, style }
  }
}

struct DemoViewer;

impl TreeizeViewer<DemoNode> for DemoViewer {
  fn title(&mut self, _node: &DemoNode) -> String {
    "DemoNode".to_string()
  }
  fn has_input(&mut self, _node: &DemoNode) -> bool {
    false
  }
  fn has_output(&mut self, _node: &DemoNode) -> bool {
    false
  }
  #[allow(refining_impl_trait)]
  fn show_input(
    &mut self,
    _pin: &egui_treeize::InPin,
    _ui: &mut egui::Ui,
    _treeize: &mut Treeize<DemoNode>,
  ) -> PinInfo {
    PinInfo::circle()
  }
  #[allow(refining_impl_trait)]
  fn show_output(
    &mut self,
    _pin: &egui_treeize::OutPin,
    _ui: &mut egui::Ui,
    _treeize: &mut Treeize<DemoNode>,
  ) -> PinInfo {
    PinInfo::circle()
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
