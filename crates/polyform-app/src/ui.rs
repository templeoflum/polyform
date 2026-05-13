use std::collections::HashSet;

use polyform_core::animation::PipelineKind;
use polyform_core::frame_state::FrameState;
use polyform_core::palette::PALETTE_KEYS;
use polyform_core::preset::Preset;
use polyform_core::registry::Registry;
use polyform_core::preset_io::PresetKind;
use polyform_core::types::{LatticeType, Param, ProjectionMode, QuaternionMode, ShapeFunction, ToggleParam};

// =============================================================================
// Popup state (stored in egui's type-map)
// =============================================================================

#[derive(Clone, Default)]
struct OpenPopups(HashSet<String>);

fn is_popup_open(ctx: &egui::Context, id: &str) -> bool {
    ctx.data(|d| {
        d.get_temp::<OpenPopups>(egui::Id::NULL)
            .map_or(false, |p| p.0.contains(id))
    })
}

fn toggle_popup(ctx: &egui::Context, id: &str) {
    ctx.data_mut(|d| {
        let mut popups = d.get_temp::<OpenPopups>(egui::Id::NULL).unwrap_or_default();
        if !popups.0.remove(id) {
            popups.0.insert(id.to_string());
        }
        d.insert_temp(egui::Id::NULL, popups);
    });
}

fn close_popup(ctx: &egui::Context, id: &str) {
    ctx.data_mut(|d| {
        let mut popups = d.get_temp::<OpenPopups>(egui::Id::NULL).unwrap_or_default();
        popups.0.remove(id);
        d.insert_temp(egui::Id::NULL, popups);
    });
}

/// Small save/load preset buttons for a section header.
fn section_preset_buttons(
    ui: &mut egui::Ui,
    kind: PresetKind,
    actions: &mut Vec<UiAction>,
    section_id: &str,
) {
    if ui.small_button("\u{1f4be}").on_hover_text("Save section preset").clicked() {
        actions.push(UiAction::SaveSectionPreset(kind));
    }
    let combo_id = format!("section_load_{}", section_id);
    egui::ComboBox::from_id_salt(&combo_id)
        .selected_text("\u{1f4c2}")
        .width(30.0)
        .show_ui(ui, |ui| {
            let entries = crate::preset_library::list_presets(kind);
            if entries.is_empty() {
                ui.label("(no presets)");
            } else {
                for entry in &entries {
                    if ui.selectable_label(false, &entry.name).clicked() {
                        actions.push(UiAction::LoadSectionPreset(kind, entry.path.clone()));
                    }
                }
            }
        });
}

/// Collapsible section with a group-mute checkbox visible on the header line.
/// Returns true if the body is open (caller renders body content after this).
/// When `preset_kind` is provided, save/load preset buttons are rendered on the header.
fn section_with_toggle(
    ui: &mut egui::Ui,
    id_str: &str,
    label: &str,
    default_open: bool,
    any_on: bool,
    keys: &[&str],
    actions: &mut Vec<UiAction>,
    preset_kind: Option<PresetKind>,
) -> bool {
    let id = egui::Id::new(id_str);
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(), id, default_open,
    );
    let is_open = state.is_open();

    ui.horizontal(|ui| {
        let mut group_on = any_on;
        if ui.checkbox(&mut group_on, "").changed() {
            for key in keys {
                actions.push(UiAction::SetToggleParamOn(key.to_string(), group_on));
            }
        }
        let icon = if is_open { "\u{25BC}" } else { "\u{25B6}" };
        if ui.small_button(icon).clicked() {
            state.toggle(ui);
        }
        ui.strong(label);
        if let Some(kind) = preset_kind {
            section_preset_buttons(ui, kind, actions, id_str);
        }
    });

    let open = state.is_open();
    state.store(ui.ctx());
    open
}

// =============================================================================
// Constants
// =============================================================================

const PHI: f64 = 1.618033988749895; // (1 + sqrt(5)) / 2
const SILVER: f64 = 2.414213562373095; // 1 + sqrt(2)
const BRONZE: f64 = 3.302775637731995; // (3 + sqrt(13)) / 2
const PLASTIC: f64 = 1.32471795724474602596;

const SQRT2: f64 = std::f64::consts::SQRT_2;
const SQRT3: f64 = 1.7320508075688772;
const SQRT5: f64 = 2.23606797749979;

struct RatioPreset {
    label: &'static str,
    value: f64,
}

struct RatioGroup {
    label: &'static str,
    presets: &'static [RatioPreset],
}

const RATIO_GROUPS: &[RatioGroup] = &[
    RatioGroup {
        label: "integer",
        presets: &[
            RatioPreset { label: "1", value: 1.0 },
            RatioPreset { label: "2", value: 2.0 },
            RatioPreset { label: "3", value: 3.0 },
            RatioPreset { label: "4", value: 4.0 },
            RatioPreset { label: "5", value: 5.0 },
            RatioPreset { label: "6", value: 6.0 },
            RatioPreset { label: "7", value: 7.0 },
            RatioPreset { label: "8", value: 8.0 },
            RatioPreset { label: "12", value: 12.0 },
            RatioPreset { label: "16", value: 16.0 },
            RatioPreset { label: "24", value: 24.0 },
            RatioPreset { label: "32", value: 32.0 },
            RatioPreset { label: "48", value: 48.0 },
            RatioPreset { label: "64", value: 64.0 },
        ],
    },
    RatioGroup {
        label: "1/n",
        presets: &[
            RatioPreset { label: "1/2", value: 0.5 },
            RatioPreset { label: "1/3", value: 1.0 / 3.0 },
            RatioPreset { label: "1/4", value: 0.25 },
            RatioPreset { label: "1/5", value: 0.2 },
            RatioPreset { label: "1/6", value: 1.0 / 6.0 },
            RatioPreset { label: "1/7", value: 1.0 / 7.0 },
            RatioPreset { label: "1/8", value: 0.125 },
            RatioPreset { label: "1/10", value: 0.1 },
            RatioPreset { label: "1/12", value: 1.0 / 12.0 },
            RatioPreset { label: "1/16", value: 0.0625 },
            RatioPreset { label: "1/24", value: 1.0 / 24.0 },
            RatioPreset { label: "1/32", value: 1.0 / 32.0 },
            RatioPreset { label: "1/48", value: 1.0 / 48.0 },
            RatioPreset { label: "1/64", value: 1.0 / 64.0 },
        ],
    },
    RatioGroup {
        label: "harmonic",
        presets: &[
            RatioPreset { label: "2/3", value: 2.0 / 3.0 },
            RatioPreset { label: "3/4", value: 3.0 / 4.0 },
            RatioPreset { label: "4/5", value: 4.0 / 5.0 },
            RatioPreset { label: "5/6", value: 5.0 / 6.0 },
            RatioPreset { label: "3/2", value: 3.0 / 2.0 },
            RatioPreset { label: "4/3", value: 4.0 / 3.0 },
            RatioPreset { label: "5/4", value: 5.0 / 4.0 },
            RatioPreset { label: "6/5", value: 6.0 / 5.0 },
            RatioPreset { label: "5/3", value: 5.0 / 3.0 },
            RatioPreset { label: "7/4", value: 7.0 / 4.0 },
            RatioPreset { label: "7/5", value: 7.0 / 5.0 },
            RatioPreset { label: "9/8", value: 9.0 / 8.0 },
            RatioPreset { label: "8/7", value: 8.0 / 7.0 },
            RatioPreset { label: "7/6", value: 7.0 / 6.0 },
        ],
    },
    RatioGroup {
        label: "just intonation",
        presets: &[
            RatioPreset { label: "16/15", value: 16.0 / 15.0 },
            RatioPreset { label: "9/8", value: 9.0 / 8.0 },
            RatioPreset { label: "6/5", value: 6.0 / 5.0 },
            RatioPreset { label: "5/4", value: 5.0 / 4.0 },
            RatioPreset { label: "4/3", value: 4.0 / 3.0 },
            RatioPreset { label: "3/2", value: 3.0 / 2.0 },
            RatioPreset { label: "8/5", value: 8.0 / 5.0 },
            RatioPreset { label: "5/3", value: 5.0 / 3.0 },
            RatioPreset { label: "15/8", value: 15.0 / 8.0 },
            RatioPreset { label: "7/4", value: 7.0 / 4.0 },
            RatioPreset { label: "2", value: 2.0 },
        ],
    },
    RatioGroup {
        label: "fibonacci",
        presets: &[
            RatioPreset { label: "1/8", value: 0.125 },
            RatioPreset { label: "1/5", value: 0.2 },
            RatioPreset { label: "2/5", value: 0.4 },
            RatioPreset { label: "3/5", value: 0.6 },
            RatioPreset { label: "5/8", value: 0.625 },
            RatioPreset { label: "8/13", value: 8.0 / 13.0 },
            RatioPreset { label: "13/21", value: 13.0 / 21.0 },
            RatioPreset { label: "21/34", value: 21.0 / 34.0 },
            RatioPreset { label: "5/3", value: 5.0 / 3.0 },
            RatioPreset { label: "8/5", value: 8.0 / 5.0 },
            RatioPreset { label: "13/8", value: 13.0 / 8.0 },
            RatioPreset { label: "21/13", value: 21.0 / 13.0 },
            RatioPreset { label: "34/21", value: 34.0 / 21.0 },
        ],
    },
    RatioGroup {
        label: "metallic",
        presets: &[
            RatioPreset { label: "1/phi^2", value: 1.0 / (PHI * PHI) },
            RatioPreset { label: "1/phi", value: 1.0 / PHI },
            RatioPreset { label: "phi", value: PHI },
            RatioPreset { label: "phi^2", value: PHI * PHI },
            RatioPreset { label: "delta_s", value: SILVER },
            RatioPreset { label: "1/delta_s", value: 1.0 / SILVER },
            RatioPreset { label: "delta_b", value: BRONZE },
            RatioPreset { label: "1/delta_b", value: 1.0 / BRONZE },
            RatioPreset { label: "rho", value: PLASTIC },
            RatioPreset { label: "1/rho", value: 1.0 / PLASTIC },
        ],
    },
    RatioGroup {
        label: "irrational",
        presets: &[
            RatioPreset { label: "1/pi", value: 1.0 / std::f64::consts::PI },
            RatioPreset { label: "1/e", value: 1.0 / std::f64::consts::E },
            RatioPreset { label: "sqrt2/2", value: SQRT2 / 2.0 },
            RatioPreset { label: "sqrt3/2", value: SQRT3 / 2.0 },
            RatioPreset { label: "sqrt2", value: SQRT2 },
            RatioPreset { label: "sqrt3", value: SQRT3 },
            RatioPreset { label: "sqrt5", value: SQRT5 },
            RatioPreset { label: "e", value: std::f64::consts::E },
            RatioPreset { label: "pi", value: std::f64::consts::PI },
            RatioPreset { label: "pi/2", value: std::f64::consts::FRAC_PI_2 },
            RatioPreset { label: "2pi", value: std::f64::consts::TAU },
            RatioPreset { label: "e/pi", value: std::f64::consts::E / std::f64::consts::PI },
            RatioPreset { label: "pi/e", value: std::f64::consts::PI / std::f64::consts::E },
        ],
    },
    RatioGroup {
        label: "powers of 2",
        presets: &[
            RatioPreset { label: "1/128", value: 1.0 / 128.0 },
            RatioPreset { label: "1/64", value: 1.0 / 64.0 },
            RatioPreset { label: "1/32", value: 1.0 / 32.0 },
            RatioPreset { label: "1/16", value: 0.0625 },
            RatioPreset { label: "1/8", value: 0.125 },
            RatioPreset { label: "1/4", value: 0.25 },
            RatioPreset { label: "1/2", value: 0.5 },
            RatioPreset { label: "2", value: 2.0 },
            RatioPreset { label: "4", value: 4.0 },
            RatioPreset { label: "8", value: 8.0 },
            RatioPreset { label: "16", value: 16.0 },
            RatioPreset { label: "32", value: 32.0 },
            RatioPreset { label: "64", value: 64.0 },
            RatioPreset { label: "128", value: 128.0 },
        ],
    },
];

/// BPM preset groups — each row has a label and values with intentional meaning.
struct BpmGroup {
    label: &'static str,
    values: &'static [u32],
}

const BPM_GROUPS: &[BpmGroup] = &[
    BpmGroup {
        label: "slow",
        values: &[1, 5, 10, 20, 30, 40, 50, 60],
    },
    BpmGroup {
        label: "musical",
        values: &[72, 80, 90, 100, 110, 120, 130, 140, 160, 180, 200],
    },
    BpmGroup {
        label: "fast",
        values: &[240, 300, 360, 480, 600, 720, 960],
    },
    BpmGroup {
        label: "raster",
        values: &[1200, 1800, 2400, 3600, 7200, 14400, 30000, 60000],
    },
];

// =============================================================================
// UiAction — mutations the UI wants to perform
// =============================================================================

pub enum UiAction {
    SetParam(ParamField, f64),
    SetParamAnim(ParamField, bool),
    SetParamShape(ParamField, ShapeFunction),
    SetParamRatio(ParamField, f64),
    SetParamMin(ParamField, f64),
    SetParamMax(ParamField, f64),
    SetParamOffset(ParamField, f64),
    SetModulatorDepth(ParamField, f64),
    SetModulatorRatio(ParamField, f64),
    SetModulatorShape(ParamField, ShapeFunction),
    AddLayer(PipelineKind, String),    // type_key
    RemoveLayer(PipelineKind, usize),
    MoveLayer(PipelineKind, usize, i32), // index, direction (-1 up, +1 down)
    ChangeLayerType(PipelineKind, usize, String),
    SetLayerEnabled(PipelineKind, String, bool),
    SetAllLayersEnabled(PipelineKind, bool),
    SetPalette(String),
    SetProjection(ProjectionMode),
    SetQuatOn(bool),
    SetQuatMode(QuaternionMode),
    SetQuatDriftSpeed(f64),
    SetToggleParamOn(String, bool),    // view_key
    ParamGrabStart(ParamField),        // user started dragging an animated slider
    ParamGrabEnd(ParamField),          // user released an animated slider
    SetPresetName(String),
    SavePreset,
    SetViewportSize(u32, u32),
    SetLatticeType(LatticeType),
    SetLatticeSpokeCount(u32),
    SetLatticeInnerRadius(f64),
    SetLatticeFocusDistance(f64),
    SetLatticeTauCount(u32),
    SaveSectionPreset(PresetKind),
    LoadSectionPreset(PresetKind, std::path::PathBuf),
    SetBlendTime(f64),
    CancelMorph,
    #[cfg(target_os = "windows")]
    SetSpoutEnabled(bool),
    #[cfg(target_os = "windows")]
    SetSpoutOverlays(bool),
}

/// Identifies a specific Param for direct mutation.
#[derive(Clone)]
pub enum ParamField {
    LayerParam {
        kind: PipelineKind,
        layer_id: String,
        key: String,
    },
    LayerBlend {
        kind: PipelineKind,
        layer_id: String,
    },
    Tempo,
    View(String), // view_key like "camera.position.orbitX"
}

// =============================================================================
// Overlay state
// =============================================================================

/// Overlay toggle state, owned by the caller (App).
pub struct OverlayState {
    pub show_signal_flow: bool,
    pub show_dim_count: bool,
    pub show_active_layers: bool,
    pub show_fps: bool,
    pub show_caustic: bool,
    pub show_domain_coloring: bool,
    pub show_curl: bool,
    pub show_conformal: bool,
    pub show_normal_shading: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    #[cfg(target_os = "windows")]
    pub spout_enabled: bool,
    #[cfg(target_os = "windows")]
    pub spout_include_overlays: bool,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            show_signal_flow: true,
            show_dim_count: true,
            show_active_layers: true,
            show_fps: false,
            show_caustic: false,
            show_domain_coloring: false,
            show_curl: false,
            show_conformal: false,
            show_normal_shading: false,
            viewport_width: 1920,
            viewport_height: 1080,
            #[cfg(target_os = "windows")]
            spout_enabled: false,
            #[cfg(target_os = "windows")]
            spout_include_overlays: false,
        }
    }
}

// =============================================================================
// Main UI builder — top strip + responsive columns
// =============================================================================

pub fn build_ui(
    ctx: &egui::Context,
    preset: &Preset,
    registry: &Registry,
    frame_state: Option<&FrameState>,
    overlay: &mut OverlayState,
    tempo: &Param,
    palette: &str,
    blend_time: f64,
    viewport_width: u32,
    viewport_height: u32,
) -> Vec<UiAction> {
    let mut actions = Vec::new();

    // Sync viewport from session state
    overlay.viewport_width = viewport_width;
    overlay.viewport_height = viewport_height;

    egui::CentralPanel::default().show(ctx, |ui| {
        // ── Top strip: global controls ──
        top_strip(ctx, ui, preset, overlay, &mut actions, tempo, palette, blend_time);
        ui.separator();

        // ── Column body ──
        let available = ui.available_size();
        let min_col = 500.0_f32;
        let col_count = ((available.x / min_col).floor() as usize).clamp(3, 5);
        let col_width = available.x / col_count as f32 - 8.0;

        ui.columns(col_count, |columns| {
            match col_count {
                5 => {
                    // [Ops] [Lift] [Lattice+Phosphor+Vis] [Camera] [Camera cont.]
                    columns[0].push_id("col_ops", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_ops").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            operations_column(ctx, ui, preset, registry, frame_state, &mut actions);
                        });
                    });
                    columns[1].push_id("col_lift", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_lift").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            lift_column(ctx, ui, preset, registry, frame_state, &mut actions);
                        });
                    });
                    columns[2].push_id("col_lattice", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_lattice").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            lattice_rendering_column(ctx, ui, preset, frame_state, overlay, &mut actions);
                        });
                    });
                    columns[3].push_id("col_cam1", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_cam1").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            camera_position_lens_section(ctx, ui, preset, frame_state, &mut actions);
                        });
                    });
                    columns[4].push_id("col_cam2", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_cam2").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            camera_distortion_quat_section(ctx, ui, preset, frame_state, &mut actions);
                        });
                    });
                }
                4 => {
                    // [Ops] [Lift] [Lattice+Phosphor+Vis] [Camera]
                    columns[0].push_id("col_ops", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_ops").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            operations_column(ctx, ui, preset, registry, frame_state, &mut actions);
                        });
                    });
                    columns[1].push_id("col_lift", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_lift").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            lift_column(ctx, ui, preset, registry, frame_state, &mut actions);
                        });
                    });
                    columns[2].push_id("col_lattice", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_lattice").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            lattice_rendering_column(ctx, ui, preset, frame_state, overlay, &mut actions);
                        });
                    });
                    columns[3].push_id("col_cam", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_cam").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            camera_column(ctx, ui, preset, frame_state, &mut actions);
                        });
                    });
                }
                _ => {
                    // 3 cols: [Ops+Lift] [Lattice+Phosphor+Vis] [Camera]
                    columns[0].push_id("col_ops_lift", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_ops_lift").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            operations_column(ctx, ui, preset, registry, frame_state, &mut actions);
                            ui.separator();
                            lift_column(ctx, ui, preset, registry, frame_state, &mut actions);
                        });
                    });
                    columns[1].push_id("col_lattice", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_lattice").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            lattice_rendering_column(ctx, ui, preset, frame_state, overlay, &mut actions);
                        });
                    });
                    columns[2].push_id("col_cam", |ui| {
                        egui::ScrollArea::vertical().id_salt("scroll_cam").show(ui, |ui| {
                            ui.set_min_width(col_width);
                            camera_column(ctx, ui, preset, frame_state, &mut actions);
                        });
                    });
                }
            }
        });
    });

    actions
}

// =============================================================================
// Top strip: Tempo, Preset, Palette, Viewport, Spout, Overlays
// =============================================================================

fn top_strip(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    preset: &Preset,
    overlay: &mut OverlayState,
    actions: &mut Vec<UiAction>,
    tempo: &Param,
    palette: &str,
    blend_time: f64,
) {
    // Row 1: Tempo + Preset name + Palette + Viewport + Spout
    ui.horizontal(|ui| {
        // Tempo: drag value + log slider + popup for presets
        // BPM / 240 = Hz: ratio=1 means one oscillation per bar (4 beats in 4/4)
        let bpm = tempo.value * 240.0;
        let mut bpm_entry = bpm;
        if ui.add(
            egui::DragValue::new(&mut bpm_entry)
                .speed(1.0)
                .range(0.01..=1_000_000.0)
                .suffix(" bpm")
        ).changed() {
            actions.push(UiAction::SetParam(ParamField::Tempo, bpm_entry / 240.0));
        }
        let mut bpm_val = bpm.clamp(0.01, 1_000_000.0);
        if ui.add(
            egui::Slider::new(&mut bpm_val, 0.01..=1_000_000.0)
                .logarithmic(true)
                .show_value(false)
                .text("")
        ).changed() {
            actions.push(UiAction::SetParam(ParamField::Tempo, bpm_val / 240.0));
        }
        // Popup for BPM presets
        let tempo_popup_id = "popup_tempo_presets";
        if ui.small_button("...").clicked() {
            toggle_popup(ctx, tempo_popup_id);
        }
        if is_popup_open(ctx, tempo_popup_id) {
            let mut open = true;
            egui::Window::new("BPM Presets")
                .id(egui::Id::new(tempo_popup_id))
                .open(&mut open)
                .resizable(false)
                .default_width(400.0)
                .show(ctx, |ui| {
                    for group in BPM_GROUPS {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new(group.label).weak().size(10.0));
                            for &preset_bpm in group.values {
                                let is_active = (bpm.round() as u32) == preset_bpm;
                                if ui.selectable_label(is_active, format!("{}", preset_bpm)).clicked() {
                                    actions.push(UiAction::SetParam(ParamField::Tempo, preset_bpm as f64 / 240.0));
                                }
                            }
                        });
                    }
                });
            if !open {
                close_popup(ctx, tempo_popup_id);
            }
        }

        ui.separator();

        // Preset name
        let mut name = preset.metadata.name.clone();
        if ui.add(egui::TextEdit::singleline(&mut name).desired_width(120.0)).changed() {
            actions.push(UiAction::SetPresetName(name));
        }
        if ui.small_button("Save").clicked() {
            actions.push(UiAction::SavePreset);
        }
        egui::ComboBox::from_id_salt("preset_load_combo")
            .selected_text("Load")
            .width(60.0)
            .show_ui(ui, |ui| {
                let entries = crate::preset_library::list_presets(PresetKind::Full);
                if entries.is_empty() {
                    ui.label("(no presets)");
                } else {
                    for entry in &entries {
                        if ui.selectable_label(false, &entry.name).clicked() {
                            actions.push(UiAction::LoadSectionPreset(PresetKind::Full, entry.path.clone()));
                        }
                    }
                }
            });

        ui.separator();

        // Palette
        let current = palette;
        egui::ComboBox::from_id_salt("palette_combo")
            .selected_text(current)
            .width(100.0)
            .show_ui(ui, |ui| {
                for &key in PALETTE_KEYS {
                    if ui.selectable_label(current == key, key).clicked() {
                        actions.push(UiAction::SetPalette(key.to_string()));
                    }
                }
            });

        ui.separator();

        // Morph duration
        let mut morph_s = blend_time;
        ui.label(egui::RichText::new("morph").weak());
        if ui.add(
            egui::DragValue::new(&mut morph_s)
                .speed(0.1)
                .range(0.1..=30.0)
                .suffix("s")
        ).changed() {
            actions.push(UiAction::SetBlendTime(morph_s));
        }

        ui.separator();

        // Viewport
        let mut w = overlay.viewport_width as i32;
        let mut h = overlay.viewport_height as i32;
        let w_changed = ui.add(egui::DragValue::new(&mut w).speed(1).range(320..=7680)).changed();
        ui.label("\u{00d7}");
        let h_changed = ui.add(egui::DragValue::new(&mut h).speed(1).range(320..=7680)).changed();
        if w_changed || h_changed {
            overlay.viewport_width = w as u32;
            overlay.viewport_height = h as u32;
            actions.push(UiAction::SetViewportSize(w as u32, h as u32));
        }
        if ui.small_button("HD").clicked() {
            overlay.viewport_width = 1920; overlay.viewport_height = 1080;
            actions.push(UiAction::SetViewportSize(1920, 1080));
        }
        if ui.small_button("4K").clicked() {
            overlay.viewport_width = 3840; overlay.viewport_height = 2160;
            actions.push(UiAction::SetViewportSize(3840, 2160));
        }

        #[cfg(target_os = "windows")]
        {
            ui.separator();
            if ui.checkbox(&mut overlay.spout_enabled, "Spout").changed() {
                actions.push(UiAction::SetSpoutEnabled(overlay.spout_enabled));
            }
            if overlay.spout_enabled {
                if ui.checkbox(&mut overlay.spout_include_overlays, "w/ overlays").changed() {
                    actions.push(UiAction::SetSpoutOverlays(overlay.spout_include_overlays));
                }
            }
        }
    });

    // Row 2: Overlays
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Overlays:").weak());
        ui.checkbox(&mut overlay.show_signal_flow, "Signal");
        ui.checkbox(&mut overlay.show_dim_count, "Dim");
        ui.checkbox(&mut overlay.show_active_layers, "Layers");
        ui.checkbox(&mut overlay.show_fps, "FPS");
        ui.separator();
        ui.label(egui::RichText::new("Vis:").weak());
        ui.checkbox(&mut overlay.show_caustic, "Caustic");
        ui.checkbox(&mut overlay.show_domain_coloring, "Domain");
        ui.checkbox(&mut overlay.show_curl, "Curl");
        ui.checkbox(&mut overlay.show_conformal, "Conformal");
        ui.checkbox(&mut overlay.show_normal_shading, "Normals");
    });
}

// =============================================================================
// Camera split helpers for 5-column layout
// =============================================================================

fn camera_position_lens_section(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    preset: &Preset,
    frame_state: Option<&FrameState>,
    actions: &mut Vec<UiAction>,
) {
    ui.horizontal(|ui| {
        ui.strong("Camera");
        section_preset_buttons(ui, PresetKind::Camera, actions, "sec_camera");
    });

    // Position
    let pos = &preset.camera.position;
    let pos_keys: &[&str] = &[
        "camera.position.orbitX", "camera.position.orbitY",
        "camera.position.distance", "camera.position.targetX",
        "camera.position.targetY", "camera.position.roll",
    ];
    let pos_any_on = pos.orbit_x.on || pos.orbit_y.on || pos.distance.on
        || pos.target_x.on || pos.target_y.on || pos.roll.on;

    if section_with_toggle(ui, "sec_position", "Position", true, pos_any_on, pos_keys, actions, Some(PresetKind::Camera)) {
        compact_toggle_param_widget(ctx, ui, "orbit x", &pos.orbit_x, "camera.position.orbitX", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "orbit y", &pos.orbit_y, "camera.position.orbitY", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "distance", &pos.distance, "camera.position.distance", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "target x", &pos.target_x, "camera.position.targetX", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "target y", &pos.target_y, "camera.position.targetY", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "roll", &pos.roll, "camera.position.roll", frame_state, actions);
    }

    // Lens
    let lens = &preset.camera.lens;
    let lens_keys: &[&str] = &[
        "camera.lens.focalLength", "camera.lens.sensorSize",
        "camera.lens.orthoZoom", "camera.lens.anamorphic",
        "camera.lens.shiftX", "camera.lens.shiftY",
    ];
    let lens_any_on = lens.focal_length.on || lens.sensor_size.on || lens.ortho_zoom.on
        || lens.anamorphic.on || lens.shift_x.on || lens.shift_y.on;

    if section_with_toggle(ui, "sec_lens", "Lens", true, lens_any_on, lens_keys, actions, Some(PresetKind::Camera)) {
        ui.horizontal(|ui| {
            let proj = preset.camera.lens.projection;
            if ui.selectable_label(proj == ProjectionMode::Perspective, "Persp").clicked() && proj != ProjectionMode::Perspective {
                actions.push(UiAction::SetProjection(ProjectionMode::Perspective));
            }
            if ui.selectable_label(proj == ProjectionMode::Orthographic, "Ortho").clicked() && proj != ProjectionMode::Orthographic {
                actions.push(UiAction::SetProjection(ProjectionMode::Orthographic));
            }
        });
        compact_toggle_param_widget(ctx, ui, "focal", &lens.focal_length, "camera.lens.focalLength", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "sensor", &lens.sensor_size, "camera.lens.sensorSize", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "ortho z", &lens.ortho_zoom, "camera.lens.orthoZoom", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "anamor", &lens.anamorphic, "camera.lens.anamorphic", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "shift x", &lens.shift_x, "camera.lens.shiftX", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "shift y", &lens.shift_y, "camera.lens.shiftY", frame_state, actions);
    }
}

fn camera_distortion_quat_section(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    preset: &Preset,
    frame_state: Option<&FrameState>,
    actions: &mut Vec<UiAction>,
) {
    // Distortion
    let dist = &preset.camera.distortion;
    let dist_keys: &[&str] = &[
        "camera.distortion.k1", "camera.distortion.k2",
        "camera.distortion.k3", "camera.distortion.k4",
        "camera.distortion.p1", "camera.distortion.p2",
        "camera.distortion.centerX", "camera.distortion.centerY",
        "camera.distortion.squeeze",
    ];
    let dist_any_on = dist.k1.on || dist.k2.on || dist.k3.on || dist.k4.on
        || dist.p1.on || dist.p2.on || dist.center_x.on || dist.center_y.on || dist.squeeze.on;

    if section_with_toggle(ui, "sec_distortion", "Distortion", false, dist_any_on, dist_keys, actions, Some(PresetKind::Camera)) {
        compact_toggle_param_widget(ctx, ui, "k1", &dist.k1, "camera.distortion.k1", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "k2", &dist.k2, "camera.distortion.k2", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "k3", &dist.k3, "camera.distortion.k3", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "k4", &dist.k4, "camera.distortion.k4", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "p1", &dist.p1, "camera.distortion.p1", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "p2", &dist.p2, "camera.distortion.p2", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "ctr x", &dist.center_x, "camera.distortion.centerX", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "ctr y", &dist.center_y, "camera.distortion.centerY", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "squeeze", &dist.squeeze, "camera.distortion.squeeze", frame_state, actions);
    }

    // Quaternion (uses enable checkbox, not group toggle)
    let q = &preset.quaternion;
    {
        let id = egui::Id::new("sec_quat");
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), id, false,
        );
        let is_open = state.is_open();
        ui.horizontal(|ui| {
            let mut on = q.on;
            if ui.checkbox(&mut on, "").changed() {
                actions.push(UiAction::SetQuatOn(on));
            }
            let icon = if is_open { "\u{25BC}" } else { "\u{25B6}" };
            if ui.small_button(icon).clicked() {
                state.toggle(ui);
            }
            ui.strong("Quaternion");
        });
        if state.is_open() && q.on {
            ui.horizontal(|ui| {
                if ui.selectable_label(q.mode == QuaternionMode::Fixed, "Fixed").clicked() {
                    actions.push(UiAction::SetQuatMode(QuaternionMode::Fixed));
                }
                if ui.selectable_label(q.mode == QuaternionMode::Drift, "Drift").clicked() {
                    actions.push(UiAction::SetQuatMode(QuaternionMode::Drift));
                }
            });
            let swept_angle = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.angle"));
            compact_param_widget(ctx, ui, "angle", &q.angle, ParamField::View("quaternion.angle".to_string()), swept_angle, actions);
            if q.mode == QuaternionMode::Fixed {
                let swept_ax = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.ax"));
                compact_param_widget(ctx, ui, "ax", &q.ax, ParamField::View("quaternion.ax".to_string()), swept_ax, actions);
                let swept_ay = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.ay"));
                compact_param_widget(ctx, ui, "ay", &q.ay, ParamField::View("quaternion.ay".to_string()), swept_ay, actions);
                let swept_az = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.az"));
                compact_param_widget(ctx, ui, "az", &q.az, ParamField::View("quaternion.az".to_string()), swept_az, actions);
            }
            if q.mode == QuaternionMode::Drift {
                ui.horizontal(|ui| {
                    ui.label("drift:");
                    let mut drift = q.drift_speed;
                    if ui.add(egui::DragValue::new(&mut drift).speed(0.01).range(0.01..=2.0)).changed() {
                        actions.push(UiAction::SetQuatDriftSpeed(drift));
                    }
                });
            }
        }
        state.store(ui.ctx());
    }
}

// =============================================================================
// Column 1: Operations
// =============================================================================

fn operations_column(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    preset: &Preset,
    registry: &Registry,
    frame_state: Option<&FrameState>,
    actions: &mut Vec<UiAction>,
) {
    let layers = &preset.pipeline.operations;
    let kind = PipelineKind::Operations;
    let any_enabled = layers.iter().any(|l| l.enabled);

    ui.horizontal(|ui| {
        let mut group_on = any_enabled;
        if ui.checkbox(&mut group_on, "").changed() {
            actions.push(UiAction::SetAllLayersEnabled(kind, group_on));
        }
        ui.strong("Operations");
        section_preset_buttons(ui, PresetKind::Operations, actions, "sec_operations");
    });

    for (idx, layer) in layers.iter().enumerate() {
        compact_layer_row(ctx, ui, layer, idx, kind, layers.len(), registry, frame_state, actions);
    }

    add_layer_button(ui, "Operations", kind, registry, actions);
}

// =============================================================================
// Column 2: Lift
// =============================================================================

fn lift_column(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    preset: &Preset,
    registry: &Registry,
    frame_state: Option<&FrameState>,
    actions: &mut Vec<UiAction>,
) {
    let layers = &preset.pipeline.lift;
    let kind = PipelineKind::Lift;
    let any_enabled = layers.iter().any(|l| l.enabled);

    ui.horizontal(|ui| {
        let mut group_on = any_enabled;
        if ui.checkbox(&mut group_on, "").changed() {
            actions.push(UiAction::SetAllLayersEnabled(kind, group_on));
        }
        ui.strong("Lift");
        section_preset_buttons(ui, PresetKind::Lift, actions, "sec_lift");
    });

    for (idx, layer) in layers.iter().enumerate() {
        compact_layer_row(ctx, ui, layer, idx, kind, layers.len(), registry, frame_state, actions);
    }

    add_layer_button(ui, "Lift", kind, registry, actions);
}

// =============================================================================
// Column 3: Lattice + Rendering
// =============================================================================

fn lattice_rendering_column(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    preset: &Preset,
    frame_state: Option<&FrameState>,
    _overlay: &mut OverlayState,
    actions: &mut Vec<UiAction>,
) {
    // --- Lattice (no group toggle — scale/density have their own on checkboxes) ---
    {
        let id = egui::Id::new("sec_lattice");
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), id, true,
        );
        let is_open = state.is_open();
        ui.horizontal(|ui| {
            let icon = if is_open { "\u{25BC}" } else { "\u{25B6}" };
            if ui.small_button(icon).clicked() {
                state.toggle(ui);
            }
            ui.strong("Lattice");
            section_preset_buttons(ui, PresetKind::Sampling, actions, "sec_lattice");
        });
        if state.is_open() {
            compact_toggle_param_widget(ctx, ui, "scale", &preset.sampling.scale, "sampling.scale", frame_state, actions);
            compact_toggle_param_widget(ctx, ui, "density", &preset.sampling.density, "sampling.density", frame_state, actions);

            ui.horizontal(|ui| {
                let lt = preset.sampling.lattice_type;
                for &(label, ty) in &[
                    ("Cart", LatticeType::Cartesian),
                    ("Polar", LatticeType::Polar),
                    ("Hex", LatticeType::Hex),
                    ("LogP", LatticeType::LogPolar),
                    ("Bipol", LatticeType::Bipolar),
                ] {
                    if ui.selectable_label(lt == ty, label).clicked() && lt != ty {
                        actions.push(UiAction::SetLatticeType(ty));
                    }
                }
            });

            let lp = &preset.sampling.lattice_params;
            match preset.sampling.lattice_type {
                LatticeType::Polar => {
                    ui.horizontal(|ui| {
                        ui.label("spokes:");
                        let mut v = lp.spoke_count as i32;
                        if ui.add(egui::DragValue::new(&mut v).speed(1).range(4..=80)).changed() {
                            actions.push(UiAction::SetLatticeSpokeCount(v as u32));
                        }
                    });
                }
                LatticeType::LogPolar => {
                    ui.horizontal(|ui| {
                        ui.label("spokes:");
                        let mut v = lp.spoke_count as i32;
                        if ui.add(egui::DragValue::new(&mut v).speed(1).range(4..=80)).changed() {
                            actions.push(UiAction::SetLatticeSpokeCount(v as u32));
                        }
                        ui.label("inner r:");
                        let mut ir = lp.inner_radius;
                        if ui.add(egui::DragValue::new(&mut ir).speed(0.005).range(0.01..=0.5)).changed() {
                            actions.push(UiAction::SetLatticeInnerRadius(ir));
                        }
                    });
                }
                LatticeType::Bipolar => {
                    ui.horizontal(|ui| {
                        ui.label("focus:");
                        let mut v = lp.focus_distance;
                        if ui.add(egui::DragValue::new(&mut v).speed(0.01).range(0.1..=3.0)).changed() {
                            actions.push(UiAction::SetLatticeFocusDistance(v));
                        }
                        ui.label("tau:");
                        let mut tc = lp.tau_count as i32;
                        if ui.add(egui::DragValue::new(&mut tc).speed(1).range(4..=80)).changed() {
                            actions.push(UiAction::SetLatticeTauCount(tc as u32));
                        }
                    });
                }
                LatticeType::Cartesian | LatticeType::Hex => {}
            }
        }
        state.store(ui.ctx());
    }

    // --- Phosphor ---
    let ph = &preset.phosphor;
    let ph_keys: &[&str] = &[
        "phosphor.persistence", "phosphor.beamWidth", "phosphor.beamIntensity",
        "phosphor.gain", "phosphor.blendMix", "phosphor.saturation",
        "phosphor.bloomIntensity", "phosphor.bloomRadius", "phosphor.bloomThreshold",
        "phosphor.gamma",
    ];
    let ph_any_on = ph.persistence.on || ph.beam_width.on || ph.beam_intensity.on
        || ph.gain.on || ph.blend_mix.on || ph.saturation.on
        || ph.bloom_intensity.on || ph.bloom_radius.on || ph.bloom_threshold.on
        || ph.gamma.on;

    if section_with_toggle(ui, "sec_phosphor", "Phosphor", true, ph_any_on, ph_keys, actions, Some(PresetKind::Phosphor)) {
        compact_toggle_param_widget(ctx, ui, "persistence", &ph.persistence, "phosphor.persistence", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "beam width", &ph.beam_width, "phosphor.beamWidth", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "beam drive", &ph.beam_intensity, "phosphor.beamIntensity", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "gain", &ph.gain, "phosphor.gain", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "blend mix", &ph.blend_mix, "phosphor.blendMix", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "saturation", &ph.saturation, "phosphor.saturation", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "bloom", &ph.bloom_intensity, "phosphor.bloomIntensity", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "bloom rad", &ph.bloom_radius, "phosphor.bloomRadius", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "bloom thr", &ph.bloom_threshold, "phosphor.bloomThreshold", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "gamma", &ph.gamma, "phosphor.gamma", frame_state, actions);
    }
}

// =============================================================================
// Column 4: Camera
// =============================================================================

fn camera_column(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    preset: &Preset,
    frame_state: Option<&FrameState>,
    actions: &mut Vec<UiAction>,
) {
    // --- Position ---
    let pos = &preset.camera.position;
    let pos_keys: &[&str] = &[
        "camera.position.orbitX", "camera.position.orbitY",
        "camera.position.distance", "camera.position.targetX",
        "camera.position.targetY", "camera.position.roll",
    ];
    let pos_any_on = pos.orbit_x.on || pos.orbit_y.on || pos.distance.on
        || pos.target_x.on || pos.target_y.on || pos.roll.on;

    if section_with_toggle(ui, "sec_position", "Position", true, pos_any_on, pos_keys, actions, Some(PresetKind::Camera)) {
        compact_toggle_param_widget(ctx, ui, "orbit x", &pos.orbit_x, "camera.position.orbitX", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "orbit y", &pos.orbit_y, "camera.position.orbitY", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "distance", &pos.distance, "camera.position.distance", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "target x", &pos.target_x, "camera.position.targetX", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "target y", &pos.target_y, "camera.position.targetY", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "roll", &pos.roll, "camera.position.roll", frame_state, actions);
    }

    // --- Lens ---
    let lens = &preset.camera.lens;
    let lens_keys: &[&str] = &[
        "camera.lens.focalLength", "camera.lens.sensorSize",
        "camera.lens.orthoZoom", "camera.lens.anamorphic",
        "camera.lens.shiftX", "camera.lens.shiftY",
    ];
    let lens_any_on = lens.focal_length.on || lens.sensor_size.on || lens.ortho_zoom.on
        || lens.anamorphic.on || lens.shift_x.on || lens.shift_y.on;

    if section_with_toggle(ui, "sec_lens", "Lens", true, lens_any_on, lens_keys, actions, Some(PresetKind::Camera)) {
        ui.horizontal(|ui| {
            let proj = preset.camera.lens.projection;
            if ui.selectable_label(proj == ProjectionMode::Perspective, "Persp").clicked() && proj != ProjectionMode::Perspective {
                actions.push(UiAction::SetProjection(ProjectionMode::Perspective));
            }
            if ui.selectable_label(proj == ProjectionMode::Orthographic, "Ortho").clicked() && proj != ProjectionMode::Orthographic {
                actions.push(UiAction::SetProjection(ProjectionMode::Orthographic));
            }
        });
        compact_toggle_param_widget(ctx, ui, "focal", &lens.focal_length, "camera.lens.focalLength", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "sensor", &lens.sensor_size, "camera.lens.sensorSize", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "ortho z", &lens.ortho_zoom, "camera.lens.orthoZoom", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "anamor", &lens.anamorphic, "camera.lens.anamorphic", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "shift x", &lens.shift_x, "camera.lens.shiftX", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "shift y", &lens.shift_y, "camera.lens.shiftY", frame_state, actions);
    }

    // --- Distortion ---
    let dist = &preset.camera.distortion;
    let dist_keys: &[&str] = &[
        "camera.distortion.k1", "camera.distortion.k2",
        "camera.distortion.k3", "camera.distortion.k4",
        "camera.distortion.p1", "camera.distortion.p2",
        "camera.distortion.centerX", "camera.distortion.centerY",
        "camera.distortion.squeeze",
    ];
    let dist_any_on = dist.k1.on || dist.k2.on || dist.k3.on || dist.k4.on
        || dist.p1.on || dist.p2.on || dist.center_x.on || dist.center_y.on || dist.squeeze.on;

    if section_with_toggle(ui, "sec_distortion", "Distortion", false, dist_any_on, dist_keys, actions, Some(PresetKind::Camera)) {
        compact_toggle_param_widget(ctx, ui, "k1", &dist.k1, "camera.distortion.k1", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "k2", &dist.k2, "camera.distortion.k2", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "k3", &dist.k3, "camera.distortion.k3", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "k4", &dist.k4, "camera.distortion.k4", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "p1", &dist.p1, "camera.distortion.p1", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "p2", &dist.p2, "camera.distortion.p2", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "ctr x", &dist.center_x, "camera.distortion.centerX", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "ctr y", &dist.center_y, "camera.distortion.centerY", frame_state, actions);
        compact_toggle_param_widget(ctx, ui, "squeeze", &dist.squeeze, "camera.distortion.squeeze", frame_state, actions);
    }

    // --- Quaternion (uses enable checkbox, not group toggle) ---
    let q = &preset.quaternion;
    {
        let id = egui::Id::new("sec_quat");
        let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(), id, false,
        );
        let is_open = state.is_open();
        ui.horizontal(|ui| {
            let mut on = q.on;
            if ui.checkbox(&mut on, "").changed() {
                actions.push(UiAction::SetQuatOn(on));
            }
            let icon = if is_open { "\u{25BC}" } else { "\u{25B6}" };
            if ui.small_button(icon).clicked() {
                state.toggle(ui);
            }
            ui.strong("Quaternion");
        });
        if state.is_open() && q.on {
            ui.horizontal(|ui| {
                if ui.selectable_label(q.mode == QuaternionMode::Fixed, "Fixed").clicked() {
                    actions.push(UiAction::SetQuatMode(QuaternionMode::Fixed));
                }
                if ui.selectable_label(q.mode == QuaternionMode::Drift, "Drift").clicked() {
                    actions.push(UiAction::SetQuatMode(QuaternionMode::Drift));
                }
            });
            let swept_angle = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.angle"));
            compact_param_widget(ctx, ui, "angle", &q.angle, ParamField::View("quaternion.angle".to_string()), swept_angle, actions);
            if q.mode == QuaternionMode::Fixed {
                let swept_ax = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.ax"));
                compact_param_widget(ctx, ui, "ax", &q.ax, ParamField::View("quaternion.ax".to_string()), swept_ax, actions);
                let swept_ay = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.ay"));
                compact_param_widget(ctx, ui, "ay", &q.ay, ParamField::View("quaternion.ay".to_string()), swept_ay, actions);
                let swept_az = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.az"));
                compact_param_widget(ctx, ui, "az", &q.az, ParamField::View("quaternion.az".to_string()), swept_az, actions);
            }
            if q.mode == QuaternionMode::Drift {
                ui.horizontal(|ui| {
                    ui.label("drift:");
                    let mut drift = q.drift_speed;
                    if ui.add(egui::DragValue::new(&mut drift).speed(0.01).range(0.01..=2.0)).changed() {
                        actions.push(UiAction::SetQuatDriftSpeed(drift));
                    }
                });
            }
        }
        state.store(ui.ctx());
    }
}


// =============================================================================
// Compact layer row (collapsing)
// =============================================================================

fn compact_layer_row(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    layer: &polyform_core::layer::Layer,
    idx: usize,
    kind: PipelineKind,
    layer_count: usize,
    registry: &Registry,
    frame_state: Option<&FrameState>,
    actions: &mut Vec<UiAction>,
) {
    let def_label = match kind {
        PipelineKind::Operations => registry.operations.get(layer.type_key.as_str()).map(|d| d.label),
        PipelineKind::Lift => registry.lift.get(layer.type_key.as_str()).map(|d| d.label),
    };
    let display_label = def_label.unwrap_or(&layer.type_key);

    // Get flat layer state for swept values
    let flat_layer = frame_state.and_then(|fs| {
        let flat_layers = match kind {
            PipelineKind::Operations => &fs.operations,
            PipelineKind::Lift => &fs.lifts,
        };
        flat_layers.get(idx)
    });

    // Header line: [enabled] > Name  [blend slider thin]
    let collapsing_id = egui::Id::new(format!("layer_{:?}_{}", kind, layer.id));
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        collapsing_id,
        false,
    );

    ui.horizontal(|ui| {
        // Enable checkbox
        let mut enabled = layer.enabled;
        if ui.checkbox(&mut enabled, "").changed() {
            actions.push(UiAction::SetLayerEnabled(kind, layer.id.clone(), enabled));
        }

        // Expand toggle
        let icon = if state.is_open() { "\u{25BC}" } else { "\u{25B6}" };
        if ui.small_button(icon).clicked() {
            state.toggle(ui);
        }

        ui.label(display_label);

        // Thin blend slider (no value, no label)
        let visual_blend = flat_layer.map(|fl| fl.blend).unwrap_or(layer.blend.value);
        let mut blend_val = visual_blend;
        let response = ui.add(
            egui::Slider::new(&mut blend_val, 0.0..=1.0)
                .show_value(false)
                .text("")
        );
        if response.changed() {
            actions.push(UiAction::SetParam(
                ParamField::LayerBlend { kind, layer_id: layer.id.clone() },
                blend_val,
            ));
        }
        // Grab events for blend slider
        if layer.blend.anim {
            if response.drag_started() {
                actions.push(UiAction::ParamGrabStart(
                    ParamField::LayerBlend { kind, layer_id: layer.id.clone() },
                ));
            }
            if response.drag_stopped() {
                actions.push(UiAction::ParamGrabEnd(
                    ParamField::LayerBlend { kind, layer_id: layer.id.clone() },
                ));
            }
        }
    });

    // Expanded content
    state.show_body_unindented(ui, |ui| {
        // Type combo
        let type_entries: Vec<(&str, &str)> = match kind {
            PipelineKind::Operations => {
                let mut entries: Vec<_> = registry.operations.values()
                    .map(|d| (d.key, d.label))
                    .collect();
                entries.sort_by_key(|e| e.0);
                entries
            }
            PipelineKind::Lift => {
                let mut entries: Vec<_> = registry.lift.values()
                    .map(|d| (d.key, d.label))
                    .collect();
                entries.sort_by_key(|e| e.0);
                entries
            }
        };

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.label("Type:");
            egui::ComboBox::from_id_salt(format!("type_{:?}_{}", kind, layer.id))
                .selected_text(display_label)
                .show_ui(ui, |ui| {
                    for (key, lbl) in &type_entries {
                        if ui.selectable_label(*key == layer.type_key.as_str(), *lbl).clicked() {
                            actions.push(UiAction::ChangeLayerType(kind, idx, key.to_string()));
                        }
                    }
                });
        });

        // Full blend param widget
        let swept_blend = if layer.blend.anim {
            flat_layer.map(|fl| fl.blend)
        } else {
            flat_layer.and_then(|fl| {
                let diff = (fl.blend - layer.blend.value).abs();
                if diff > 0.001 { Some(fl.blend) } else { None }
            })
        };
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.scope(|ui| {
                compact_param_widget(ctx, ui, "blend", &layer.blend,
                    ParamField::LayerBlend { kind, layer_id: layer.id.clone() },
                    swept_blend, actions);
            });
        });

        // Per-param widgets
        let param_order: Vec<&str> = match kind {
            PipelineKind::Operations => registry.operations
                .get(layer.type_key.as_str())
                .map(|d| d.params.iter().map(|p| p.key).collect())
                .unwrap_or_default(),
            PipelineKind::Lift => registry.lift
                .get(layer.type_key.as_str())
                .map(|d| d.params.iter().map(|p| p.key).collect())
                .unwrap_or_default(),
        };

        for key in param_order {
            if let Some(param) = layer.params.get(key) {
                let swept_value = if param.anim {
                    flat_layer.and_then(|fl| fl.params.get(key).copied())
                } else {
                    None
                };
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.scope(|ui| {
                        compact_param_widget(ctx, ui, key, param,
                            ParamField::LayerParam {
                                kind,
                                layer_id: layer.id.clone(),
                                key: key.to_string(),
                            },
                            swept_value, actions);
                    });
                });
            }
        }

        // Move / Remove buttons
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            if idx > 0 && ui.small_button("\u{2191}").clicked() {
                actions.push(UiAction::MoveLayer(kind, idx, -1));
            }
            if idx < layer_count - 1 && ui.small_button("\u{2193}").clicked() {
                actions.push(UiAction::MoveLayer(kind, idx, 1));
            }
            if ui.small_button("Remove").clicked() {
                actions.push(UiAction::RemoveLayer(kind, idx));
            }
        });
    });

    // Store state
    state.store(ui.ctx());
}

// =============================================================================
// Compact param widget (one line + popup)
// =============================================================================

fn compact_param_widget(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    label: &str,
    param: &Param,
    field: ParamField,
    swept_value: Option<f64>,
    actions: &mut Vec<UiAction>,
) {
    let display_value = swept_value.unwrap_or(param.value);
    let mut value = display_value;
    let slider_label = if swept_value.is_some() {
        format!("{}~", label)
    } else {
        label.to_string()
    };

    ui.horizontal(|ui| {
        let response = ui.add(
            egui::Slider::new(&mut value, param.min..=param.max)
                .text(&slider_label)
                .clamping(egui::SliderClamping::Never)
        );
        if response.changed() {
            actions.push(UiAction::SetParam(field.clone(), value));
        }
        if param.anim {
            if response.drag_started() {
                actions.push(UiAction::ParamGrabStart(field.clone()));
            }
            if response.drag_stopped() {
                actions.push(UiAction::ParamGrabEnd(field.clone()));
            }
        }

        // Anim checkbox (unlabeled)
        let mut anim = param.anim;
        if ui.checkbox(&mut anim, "").changed() {
            actions.push(UiAction::SetParamAnim(field.clone(), anim));
        }

        // Ratio badge when animating
        if param.anim {
            ui.label(egui::RichText::new(format!("r:{:.2}", param.ratio)).weak().size(10.0));
        }

        // Popup button
        let popup_id = format!("popup_{}_{:?}", label, field_id_hint(&field));
        if ui.small_button("...").clicked() {
            toggle_popup(ctx, &popup_id);
        }

        // Render popup window
        if is_popup_open(ctx, &popup_id) {
            render_param_popup(ctx, &popup_id, label, param, &field, actions);
        }
    });
}

// =============================================================================
// Compact toggle param widget (one line + popup)
// =============================================================================

fn compact_toggle_param_widget(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    label: &str,
    tp: &ToggleParam,
    view_key: &str,
    frame_state: Option<&FrameState>,
    actions: &mut Vec<UiAction>,
) {
    let swept_value = if tp.on && tp.param.anim {
        frame_state.and_then(|fs| frame_state_view_value(fs, view_key))
    } else {
        None
    };

    let display_value = swept_value.unwrap_or(tp.param.value);
    let mut value = display_value;
    let slider_label = if swept_value.is_some() {
        format!("{}~", label)
    } else {
        label.to_string()
    };

    let field = ParamField::View(view_key.to_string());

    ui.horizontal(|ui| {
        // On checkbox
        let mut on = tp.on;
        if ui.checkbox(&mut on, "").changed() {
            actions.push(UiAction::SetToggleParamOn(view_key.to_string(), on));
        }

        let response = ui.add(
            egui::Slider::new(&mut value, tp.param.min..=tp.param.max)
                .text(&slider_label)
                .clamping(egui::SliderClamping::Never)
        );
        if response.changed() {
            actions.push(UiAction::SetParam(field.clone(), value));
        }
        if tp.param.anim {
            if response.drag_started() {
                actions.push(UiAction::ParamGrabStart(field.clone()));
            }
            if response.drag_stopped() {
                actions.push(UiAction::ParamGrabEnd(field.clone()));
            }
        }

        // Anim checkbox (unlabeled)
        let mut anim = tp.param.anim;
        if ui.checkbox(&mut anim, "").changed() {
            actions.push(UiAction::SetParamAnim(field.clone(), anim));
        }

        // Ratio badge
        if tp.param.anim {
            ui.label(egui::RichText::new(format!("r:{:.2}", tp.param.ratio)).weak().size(10.0));
        }

        // Popup button
        let popup_id = format!("popup_{}", view_key);
        if ui.small_button("...").clicked() {
            toggle_popup(ctx, &popup_id);
        }

        if is_popup_open(ctx, &popup_id) {
            render_param_popup(ctx, &popup_id, label, &tp.param, &field, actions);
        }
    });
}

// =============================================================================
// Param popup window (animation detail)
// =============================================================================

fn render_param_popup(
    ctx: &egui::Context,
    popup_id: &str,
    label: &str,
    param: &Param,
    field: &ParamField,
    actions: &mut Vec<UiAction>,
) {
    let mut open = true;
    egui::Window::new(format!("{} detail", label))
        .id(egui::Id::new(popup_id))
        .open(&mut open)
        .resizable(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            // Shape selector
            ui.horizontal(|ui| {
                ui.label("Shape:");
                let is_sawtooth = param.shape == ShapeFunction::Sawtooth;
                if ui.selectable_label(is_sawtooth, "saw").clicked() && !is_sawtooth {
                    actions.push(UiAction::SetParamShape(field.clone(), ShapeFunction::Sawtooth));
                }
                if ui.selectable_label(!is_sawtooth, "cos").clicked() && is_sawtooth {
                    actions.push(UiAction::SetParamShape(field.clone(), ShapeFunction::Cosine));
                }
            });

            // Ratio
            ui.horizontal(|ui| {
                ui.label("Ratio:");
                ratio_widget(ui, param.ratio, field, &format!("popup_ratio_{}", popup_id), actions);
            });

            // Min/max
            min_max_widget(ui, param, field, actions);

            // Offset + modulator
            offset_and_modulator_controls(ui, param, field, popup_id, actions);
        });

    if !open {
        close_popup(ctx, popup_id);
    }
}

// =============================================================================
// Helper: field identity hint for popup IDs
// =============================================================================

fn field_id_hint(field: &ParamField) -> String {
    match field {
        ParamField::LayerParam { kind, layer_id, key } => format!("{:?}_{}_{}",kind, layer_id, key),
        ParamField::LayerBlend { kind, layer_id } => format!("{:?}_{}_blend", kind, layer_id),
        ParamField::Tempo => "tempo".to_string(),
        ParamField::View(vk) => vk.clone(),
    }
}



// =============================================================================
// Add layer button (categorized menu)
// =============================================================================

fn add_layer_button(
    ui: &mut egui::Ui,
    section_label: &str,
    kind: PipelineKind,
    registry: &Registry,
    actions: &mut Vec<UiAction>,
) {
    let entries: Vec<(&str, &str, &str)> = match kind {
        PipelineKind::Operations => {
            let mut v: Vec<_> = registry.operations.values()
                .map(|d| (d.key, d.label, d.category))
                .collect();
            v.sort_by_key(|e| (e.2, e.0));
            v
        }
        PipelineKind::Lift => {
            let mut v: Vec<_> = registry.lift.values()
                .map(|d| (d.key, d.label, d.category))
                .collect();
            v.sort_by_key(|e| (e.2, e.0));
            v
        }
    };

    ui.menu_button(format!("+ Add {}", section_label), |ui| {
        let mut current_cat = "";
        for (key, label, category) in &entries {
            if *category != current_cat {
                if !current_cat.is_empty() {
                    ui.separator();
                }
                ui.label(egui::RichText::new(*category).strong());
                current_cat = category;
            }
            if ui.button(*label).clicked() {
                actions.push(UiAction::AddLayer(kind, key.to_string()));
                ui.close_menu();
            }
        }
    });
}

// =============================================================================
// Ratio picker widget (reused in popup)
// =============================================================================

fn ratio_widget(
    ui: &mut egui::Ui,
    ratio: f64,
    field: &ParamField,
    combo_id: &str,
    actions: &mut Vec<UiAction>,
) {
    let mut current_ratio = ratio;
    if ui.add(
        egui::DragValue::new(&mut current_ratio)
            .speed(0.01)
            .range(0.001..=128.0)
            .prefix("r:")
    ).changed() {
        actions.push(UiAction::SetParamRatio(field.clone(), current_ratio));
    }

    egui::ComboBox::from_id_salt(combo_id)
        .selected_text("")
        .width(20.0)
        .show_ui(ui, |ui| {
            for group in RATIO_GROUPS {
                ui.label(egui::RichText::new(group.label).strong().size(11.0));
                ui.horizontal_wrapped(|ui| {
                    for preset in group.presets {
                        let is_active = (ratio - preset.value).abs() < 1e-6;
                        if ui.selectable_label(is_active, preset.label).clicked() {
                            actions.push(UiAction::SetParamRatio(field.clone(), preset.value));
                        }
                    }
                });
                ui.separator();
            }
        });
}

// =============================================================================
// Modulator ratio widget (reused in popup)
// =============================================================================

fn modulator_ratio_widget(
    ui: &mut egui::Ui,
    ratio: f64,
    field: &ParamField,
    combo_id: &str,
    actions: &mut Vec<UiAction>,
) {
    let mut current_ratio = ratio;
    if ui.add(
        egui::DragValue::new(&mut current_ratio)
            .speed(0.01)
            .range(0.001..=128.0)
            .prefix("r:")
    ).changed() {
        actions.push(UiAction::SetModulatorRatio(field.clone(), current_ratio));
    }

    egui::ComboBox::from_id_salt(combo_id)
        .selected_text("")
        .width(20.0)
        .show_ui(ui, |ui| {
            for group in RATIO_GROUPS {
                ui.label(egui::RichText::new(group.label).strong().size(11.0));
                ui.horizontal_wrapped(|ui| {
                    for preset in group.presets {
                        let is_active = (ratio - preset.value).abs() < 1e-6;
                        if ui.selectable_label(is_active, preset.label).clicked() {
                            actions.push(UiAction::SetModulatorRatio(field.clone(), preset.value));
                        }
                    }
                });
                ui.separator();
            }
        });
}

// =============================================================================
// Min/max editing (reused in popup)
// =============================================================================

fn min_max_widget(
    ui: &mut egui::Ui,
    param: &Param,
    field: &ParamField,
    actions: &mut Vec<UiAction>,
) {
    ui.horizontal(|ui| {
        ui.label("min:");
        let mut min_val = param.min;
        if ui.add(egui::DragValue::new(&mut min_val).speed(0.01)).changed() {
            actions.push(UiAction::SetParamMin(field.clone(), min_val));
        }
        ui.label("max:");
        let mut max_val = param.max;
        if ui.add(egui::DragValue::new(&mut max_val).speed(0.01)).changed() {
            actions.push(UiAction::SetParamMax(field.clone(), max_val));
        }
    });
}

// =============================================================================
// Offset and modulator controls (reused in popup)
// =============================================================================

fn offset_and_modulator_controls(
    ui: &mut egui::Ui,
    param: &Param,
    field: &ParamField,
    id_hint: &str,
    actions: &mut Vec<UiAction>,
) {
    // Phase offset slider
    ui.horizontal(|ui| {
        ui.label("offset:");
        let mut val = param.offset;
        if ui.add(egui::Slider::new(&mut val, 0.0..=1.0).show_value(true)).changed() {
            actions.push(UiAction::SetParamOffset(field.clone(), val));
        }
    });

    // FM modulator controls
    let mod_depth = param.modulator.as_ref().map_or(0.0, |m| m.depth);
    let mod_ratio = param.modulator.as_ref().map_or(1.0, |m| m.ratio);
    let mod_shape = param.modulator.as_ref().map_or(ShapeFunction::Cosine, |m| m.shape);

    ui.horizontal(|ui| {
        ui.label("FM depth:");
        let mut depth = mod_depth;
        if ui.add(egui::Slider::new(&mut depth, 0.0..=1.0).show_value(true)).changed() {
            actions.push(UiAction::SetModulatorDepth(field.clone(), depth));
        }
    });

    if mod_depth.abs() > 1e-6 {
        ui.horizontal(|ui| {
            let is_sawtooth = mod_shape == ShapeFunction::Sawtooth;
            ui.label("FM:");
            if ui.selectable_label(is_sawtooth, "saw").clicked() && !is_sawtooth {
                actions.push(UiAction::SetModulatorShape(field.clone(), ShapeFunction::Sawtooth));
            }
            if ui.selectable_label(!is_sawtooth, "cos").clicked() && is_sawtooth {
                actions.push(UiAction::SetModulatorShape(field.clone(), ShapeFunction::Cosine));
            }
            modulator_ratio_widget(ui, mod_ratio, field, &format!("mod_ratio_{}", id_hint), actions);
        });
    }
}

// =============================================================================
// Frame state view value lookup
// =============================================================================

/// Map view keys to FrameState fields for swept value display.
pub fn frame_state_view_value(fs: &FrameState, view_key: &str) -> Option<f64> {
    match view_key {
        "camera.position.orbitX" => Some(fs.orbit_x),
        "camera.position.orbitY" => Some(fs.orbit_y),
        "camera.position.distance" => Some(fs.distance),
        "camera.position.targetX" => Some(fs.target_x),
        "camera.position.targetY" => Some(fs.target_y),
        "camera.position.roll" => Some(fs.roll),
        "camera.lens.focalLength" => Some(fs.focal_length),
        "camera.lens.sensorSize" => Some(fs.sensor_size),
        "camera.lens.orthoZoom" => Some(fs.ortho_zoom),
        "camera.lens.anamorphic" => Some(fs.anamorphic),
        "camera.lens.shiftX" => Some(fs.shift_x),
        "camera.lens.shiftY" => Some(fs.shift_y),
        "camera.distortion.k1" => Some(fs.k1),
        "camera.distortion.k2" => Some(fs.k2),
        "camera.distortion.k3" => Some(fs.k3),
        "camera.distortion.k4" => Some(fs.k4),
        "camera.distortion.p1" => Some(fs.p1),
        "camera.distortion.p2" => Some(fs.p2),
        "camera.distortion.centerX" => Some(fs.distortion_cx),
        "camera.distortion.centerY" => Some(fs.distortion_cy),
        "camera.distortion.squeeze" => Some(fs.squeeze),
        "sampling.scale" => Some(fs.scale),
        "sampling.density" => Some(fs.density),
        "quaternion.angle" => Some(fs.quat_angle),
        "quaternion.ax" => Some(fs.quat_axis[0]),
        "quaternion.ay" => Some(fs.quat_axis[1]),
        "quaternion.az" => Some(fs.quat_axis[2]),
        "phosphor.persistence" => Some(fs.persistence),
        "phosphor.beamWidth" => Some(fs.beam_width),
        "phosphor.beamIntensity" => Some(fs.beam_intensity),
        "phosphor.gain" => Some(fs.gain),
        "phosphor.blendMix" => Some(fs.blend_mix),
        "phosphor.saturation" => Some(fs.saturation),
        "phosphor.bloomIntensity" => Some(fs.bloom_intensity),
        "phosphor.bloomRadius" => Some(fs.bloom_radius),
        "phosphor.bloomThreshold" => Some(fs.bloom_threshold),
        "phosphor.gamma" => Some(fs.gamma),
        _ => None,
    }
}

