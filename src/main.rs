#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Calculadora gráfica em Rust, feita com [eframe](https://github.com/emilk/egui).
//!
//! A interface tem dois modos (básico e científico), memória, teclado e um
//! tema escuro próprio. Toda a lógica de cálculo vive em `rustcalc::eval`.

use eframe::egui;
use egui::{Align, Color32, CornerRadius, FontId, Key, RichText};
use rustcalc::eval::{self, AngleMode, Options};

// ----- Paleta -------------------------------------------------------------

const ACCENT: Color32 = Color32::from_rgb(252, 166, 62); // operadores / =
const KEY_BG: Color32 = Color32::from_rgb(44, 47, 55); // dígitos
const FN_BG: Color32 = Color32::from_rgb(33, 36, 44); // funções científicas
const MEM_BG: Color32 = Color32::from_rgb(40, 43, 51); // memória
const DANGER: Color32 = Color32::from_rgb(196, 66, 74); // AC / apagar
const TEXT: Color32 = Color32::from_rgb(238, 240, 244);
const SUBTEXT: Color32 = Color32::from_gray(150);

// ----- Layout das teclas ---------------------------------------------------

/// Espaço entre teclas adjacentes.
const GAP: f32 = 8.0;
/// Espaço entre o teclado básico e o painel científico.
const BETWEEN: f32 = 16.0;
const MEM_W: f32 = 60.0;
const MEM_H: f32 = 36.0;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 670.0])
            .with_min_inner_size([480.0, 580.0])
            .with_title("RustCalc")
            .with_app_id("br.dev.rustcalc"),
        ..Default::default()
    };
    eframe::run_native(
        "RustCalc",
        native_options,
        Box::new(|cc| Ok(Box::new(CalcApp::new(cc)))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Basic,
    Scientific,
}

struct CalcApp {
    mode: Mode,
    display: String,
    ans: f64,
    /// `true` logo após `=`: o display mostra o resultado, então o próximo
    /// dígito digitado começa um novo número.
    just_evaled: bool,
    radians: bool,
    memory: Option<f64>,
    error: Option<String>,
    show_about: bool,
}

impl CalcApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(theme());
        Self {
            mode: Mode::Basic,
            display: "0".to_string(),
            ans: 0.0,
            just_evaled: false,
            radians: true,
            memory: None,
            error: None,
            show_about: false,
        }
    }

    fn eval_opts(&self) -> Options {
        Options {
            angle: if self.radians {
                AngleMode::Radians
            } else {
                AngleMode::Degrees
            },
            ans: self.ans,
        }
    }

    fn insert_text(&mut self, s: &str) {
        let fresh_start = (self.just_evaled || self.display == "0" || self.display.is_empty())
            && s.chars().all(|c| c.is_ascii_digit());
        if fresh_start {
            self.display = s.to_string();
        } else {
            self.display.push_str(s);
        }
        self.just_evaled = false;
        self.error = None;
    }

    fn operator(&mut self, op: &str) {
        if matches!(
            self.display.chars().last(),
            Some('+') | Some('−') | Some('×') | Some('÷') | Some('^')
        ) {
            self.display.pop();
        }
        self.display.push_str(op);
        self.just_evaled = false;
        self.error = None;
    }

    fn equals(&mut self) {
        match eval::evaluate(&self.display, self.eval_opts()) {
            Ok(v) => {
                self.ans = v;
                self.display = eval::format_value(v);
                self.just_evaled = true;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.just_evaled = false;
            }
        }
    }

    fn clear(&mut self) {
        self.display = "0".to_string();
        self.just_evaled = false;
        self.error = None;
    }

    fn backspace(&mut self) {
        self.display.pop();
        if self.display.is_empty() {
            self.display.push('0');
        }
        self.just_evaled = false;
        self.error = None;
    }

    fn negate(&mut self) {
        if let Ok(v) = self.display.parse::<f64>() {
            self.display = eval::format_value(-v);
            self.just_evaled = true;
        }
    }

    fn memory_add(&mut self, sign: f64) {
        if let Ok(v) = eval::evaluate(&self.display, self.eval_opts()) {
            self.memory = Some(self.memory.unwrap_or(0.0) + sign * v);
            self.error = None;
        }
    }

    fn memory_recall(&mut self) {
        if let Some(m) = self.memory {
            self.display = eval::format_value(m);
            self.just_evaled = true;
            self.error = None;
        }
    }

    fn memory_clear(&mut self) {
        self.memory = None;
        self.error = None;
    }

    // ----- UI -------------------------------------------------------------

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .label(RichText::new("RustCalc").strong().size(15.0).color(ACCENT))
                .on_hover_text("Sobre")
                .clicked()
            {
                self.show_about = !self.show_about;
            }
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                ui.selectable_value(&mut self.mode, Mode::Scientific, "Científica");
                ui.selectable_value(&mut self.mode, Mode::Basic, "Básica");
            });
        });
        ui.add_space(6.0);
    }

    fn status_line(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let mem = if self.memory.is_some() { "M  " } else { "" };
            let ans = if self.just_evaled {
                "".to_string()
            } else {
                format!("ans = {}", eval::format_value(self.ans))
            };
            ui.label(
                RichText::new(format!("{mem}{ans}"))
                    .size(13.0)
                    .color(SUBTEXT),
            );
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if let Some(e) = &self.error {
                    ui.label(
                        RichText::new(e)
                            .size(13.0)
                            .color(Color32::from_rgb(235, 110, 110)),
                    );
                }
            });
        });
    }

    fn display_line(&mut self, ui: &mut egui::Ui) {
        let resp = ui.add(
            egui::TextEdit::singleline(&mut self.display)
                .font(FontId::monospace(30.0))
                .desired_width(f32::INFINITY)
                .horizontal_align(Align::RIGHT),
        );
        if resp.changed() {
            self.just_evaled = false;
            self.error = None;
        }
        if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            self.equals();
        }
    }

    fn memory_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let mem_on = self.memory.is_some();
            if key(ui, "MC", MEM_W, MEM_H, MEM_BG, 15.0).clicked() {
                self.memory_clear();
            }
            if key(ui, "MR", MEM_W, MEM_H, MEM_BG, 15.0).clicked() {
                self.memory_recall();
            }
            if key(ui, "M+", MEM_W, MEM_H, MEM_BG, 15.0).clicked() {
                self.memory_add(1.0);
            }
            if key(ui, "M−", MEM_W, MEM_H, MEM_BG, 15.0).clicked() {
                self.memory_add(-1.0);
            }
            if self.mode == Mode::Scientific {
                let label = if self.radians { "RAD" } else { "DEG" };
                let fill = if self.radians { ACCENT } else { MEM_BG };
                if key(ui, label, MEM_W, MEM_H, fill, 15.0).clicked() {
                    self.radians = !self.radians;
                }
            }
            // Indicador de memória ativa
            if mem_on {
                ui.add_space(4.0);
                ui.label(RichText::new("memória ocupada").size(12.0).color(SUBTEXT));
            }
        });
        ui.add_space(4.0);
    }

    /// Teclado básico (5 linhas) como uma coluna própria. `w`/`h` são o
    /// tamanho de cada tecla e `ts` o tamanho do texto (proporcional a `h`).
    fn basic_keypad(&mut self, ui: &mut egui::Ui, w: f32, h: f32) {
        ui.vertical(|ui| {
            let ts = (h * 0.44).clamp(15.0, 24.0);
            let mut row = |ui: &mut egui::Ui, labels: [&str; 4]| {
                ui.horizontal(|ui| {
                    for (i, label) in labels.iter().enumerate() {
                        let fill = if i == 3 {
                            ACCENT
                        } else if *label == "AC" {
                            DANGER
                        } else {
                            KEY_BG
                        };
                        if key(ui, label, w, h, fill, ts).clicked() {
                            self.press(label);
                        }
                    }
                });
            };

            row(ui, ["AC", "±", "%", "÷"]);
            row(ui, ["7", "8", "9", "×"]);
            row(ui, ["4", "5", "6", "−"]);
            row(ui, ["1", "2", "3", "+"]);
            ui.horizontal(|ui| {
                if key(ui, "0", w * 2.0 + GAP, h, KEY_BG, ts).clicked() {
                    self.insert_text("0");
                }
                if key(ui, ".", w, h, KEY_BG, ts).clicked() {
                    self.insert_text(".");
                }
                if key(ui, "=", w, h, ACCENT, ts).clicked() {
                    self.equals();
                }
            });
        });
    }

    /// Painel científico (8 linhas × 3 colunas) como uma coluna própria,
    /// empilhado ao lado do teclado básico com a mesma altura de tecla.
    fn scientific_pad(&mut self, ui: &mut egui::Ui, w: f32, h: f32) {
        ui.vertical(|ui| {
            let ts = (h * 0.40).clamp(13.0, 20.0);
            let mut frow = |ui: &mut egui::Ui, labels: [&str; 3]| {
                ui.horizontal(|ui| {
                    for label in labels {
                        if key(ui, label, w, h, FN_BG, ts).clicked() {
                            self.press(label);
                        }
                    }
                });
            };

            frow(ui, ["sin(", "cos(", "tan("]);
            frow(ui, ["asin(", "acos(", "atan("]);
            frow(ui, ["ln(", "log(", "log2("]);
            frow(ui, ["√(", "∛(", "exp("]);
            frow(ui, ["x^y", "10^x", "x!"]);
            frow(ui, ["|x|", "x²", "x³"]);
            frow(ui, ["π", "e", "ans"]);
            frow(ui, ["(", ")", "Del"]);
        });
    }

    /// Roteia qualquer tecla (básica ou científica) para uma ação.
    fn press(&mut self, label: &str) {
        if label.len() == 1
            && label
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit() || c == '.')
        {
            return self.insert_text(label);
        }
        match label {
            "AC" => self.clear(),
            "±" => self.negate(),
            "%" => self.insert_text("%"),
            "÷" => self.operator("÷"),
            "×" => self.operator("×"),
            "−" => self.operator("−"),
            "+" => self.operator("+"),
            "=" => self.equals(),
            "Del" => self.backspace(),
            "sin(" | "cos(" | "tan(" | "asin(" | "acos(" | "atan(" | "ln(" | "log(" | "log2("
            | "√(" | "∛(" | "exp(" => self.insert_text(match label {
                "√(" => "sqrt(",
                "∛(" => "cbrt(",
                _ => label,
            }),
            "π" => self.insert_text("π"),
            "e" => self.insert_text("e"),
            "ans" => self.insert_text("ans"),
            "(" => self.insert_text("("),
            ")" => self.insert_text(")"),
            "x^y" => self.operator("^"),
            "x!" => self.insert_text("!"),
            "10^x" => self.insert_text("10^("),
            "|x|" => self.insert_text("abs("),
            "x²" => self.insert_text("^2"),
            "x³" => self.insert_text("^3"),
            _ => {}
        }
    }

    fn about_window(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Sobre")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.heading("RustCalc");
                ui.label(format!("Versão {}", env!("CARGO_PKG_VERSION")));
                ui.add_space(8.0);
                ui.label("Calculadora gráfica em Rust com modos básico e científico.");
                ui.add_space(8.0);
                ui.label("Autor: João Lyma");
                ui.add_space(4.0);
                ui.hyperlink("https://github.com/lyma/rustcalc");
                ui.add_space(8.0);
                ui.label("Licença: MIT");
            });
    }
}

impl eframe::App for CalcApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
            self.top_bar(ui);
            self.status_line(ui);
            self.display_line(ui);
            ui.add_space(2.0);
            self.memory_row(ui);

            match self.mode {
                Mode::Basic => {
                    ui.allocate_space(egui::vec2(0.0, 6.0));
                    let w = ((ui.available_width() - 3.0 * GAP) / 4.0).clamp(48.0, 110.0);
                    let h = ((ui.available_height() - 4.0 * GAP) / 5.0).clamp(40.0, 64.0);
                    ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                        self.basic_keypad(ui, w, h);
                    });
                }
                Mode::Scientific => {
                    ui.add_space(6.0);
                    let w = ((ui.available_width() - 6.0 * GAP - BETWEEN) / 7.0).clamp(42.0, 84.0);
                    let h = ((ui.available_height() - 7.0 * GAP) / 8.0).clamp(30.0, 48.0);
                    ui.horizontal(|ui| {
                        self.basic_keypad(ui, w, h);
                        ui.add_space(BETWEEN);
                        self.scientific_pad(ui, w, h);
                    });
                }
            }
        });

        if self.show_about {
            self.about_window(ui);
        }
    }
}

// ----- Widgets auxiliares --------------------------------------------------

/// Um botão quadrado com preenchimento e cantos arredondados próprios.
fn key(
    ui: &mut egui::Ui,
    label: &str,
    w: f32,
    h: f32,
    fill: Color32,
    text_size: f32,
) -> egui::Response {
    let btn = egui::Button::new(RichText::new(label).size(text_size).color(TEXT))
        .fill(fill)
        .corner_radius(CornerRadius::same(12));
    ui.add_sized(egui::vec2(w, h), btn)
}

/// Fonte com cobertura de símbolos matemáticos (π, √, ∛, ×, ÷, −, ², ³…).
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "dejavu".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/DejaVuSans.ttf"
        ))),
    );
    fonts.font_data.insert(
        "dejavu_mono".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/DejaVuSansMono.ttf"
        ))),
    );
    for (family, name) in [
        (egui::FontFamily::Proportional, "dejavu"),
        (egui::FontFamily::Monospace, "dejavu_mono"),
    ] {
        fonts
            .families
            .get_mut(&family)
            .unwrap()
            .insert(0, name.to_owned());
    }
    ctx.set_fonts(fonts);
}

fn theme() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.panel_fill = Color32::from_rgb(24, 26, 31);
    v.window_fill = v.panel_fill;
    v.extreme_bg_color = Color32::from_rgb(30, 32, 38);
    v.faint_bg_color = Color32::from_rgb(30, 32, 38);
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = ACCENT;
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
    ] {
        w.corner_radius = CornerRadius::same(12);
    }
    v.widgets.inactive.bg_fill = Color32::from_rgb(34, 37, 44);
    v.widgets.hovered.bg_fill = Color32::from_rgb(58, 62, 72);
    v.widgets.active.bg_fill = Color32::from_rgb(52, 66, 70);
    v
}
