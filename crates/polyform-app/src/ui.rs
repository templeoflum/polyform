use polyform_core::animation::PipelineKind;
use polyform_core::frame_state::FrameState;
use polyform_core::palette::PALETTE_KEYS;
use polyform_core::preset::Preset;
use polyform_core::registry::Registry;
use polyform_core::types::{LatticeType, Param, ProjectionMode, QuaternionMode, ShapeFunction, ToggleParam};

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
    LoadPreset,
    SetViewportSize(u32, u32),
    SetLatticeType(LatticeType),
    SetLatticeSpokeCount(u32),
    SetLatticeInnerRadius(f64),
    SetLatticeFocusDistance(f64),
    SetLatticeTauCount(u32),
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
// Main UI builder
// =============================================================================

/// Overlay toggle state, owned by the caller (App).
pub struct OverlayState {
    pub show_signal_flow: bool,
    pub show_dim_count: bool,
    pub show_active_layers: bool,
    pub show_fps: bool,
    pub show_caustic: bool,
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
            viewport_width: 1920,
            viewport_height: 1080,
            #[cfg(target_os = "windows")]
            spout_enabled: false,
            #[cfg(target_os = "windows")]
            spout_include_overlays: false,
        }
    }
}

pub fn build_ui(
    ctx: &egui::Context,
    preset: &Preset,
    registry: &Registry,
    frame_state: Option<&FrameState>,
    overlay: &mut OverlayState,
) -> Vec<UiAction> {
    let mut actions = Vec::new();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Polyform Controls");
        ui.separator();

        // Sync viewport from preset
        overlay.viewport_width = preset.viewport.width;
        overlay.viewport_height = preset.viewport.height;

        // Top section: 3 columns — [preset/output] [lattice] [tempo]
        ui.columns(3, |cols| {
            // Column 0: Preset identity + output
            cols[0].push_id("top_preset", |ui| {
                header_section(ui, preset, &mut actions);

                ui.horizontal(|ui| {
                    ui.label("Overlays:");
                    ui.checkbox(&mut overlay.show_signal_flow, "Signal");
                    ui.checkbox(&mut overlay.show_dim_count, "Dim");
                    ui.checkbox(&mut overlay.show_active_layers, "Layers");
                    ui.checkbox(&mut overlay.show_fps, "FPS");
                    ui.checkbox(&mut overlay.show_caustic, "Caustic");
                });

                viewport_section(ui, overlay, &mut actions);

                #[cfg(target_os = "windows")]
                {
                    ui.horizontal(|ui| {
                        ui.label("Spout:");
                        if ui.checkbox(&mut overlay.spout_enabled, "Send").changed() {
                            actions.push(UiAction::SetSpoutEnabled(overlay.spout_enabled));
                        }
                        if overlay.spout_enabled {
                            if ui.checkbox(&mut overlay.spout_include_overlays, "Include overlays").changed() {
                                actions.push(UiAction::SetSpoutOverlays(overlay.spout_include_overlays));
                            }
                        }
                    });
                }
            });

            // Column 1: Lattice / sampling
            cols[1].push_id("top_lattice", |ui| {
                sampling_section(ui, preset, frame_state, &mut actions);
            });

            // Column 2: Tempo
            cols[2].push_id("top_tempo", |ui| {
                tempo_section(ui, preset, &mut actions);
            });
        });

        ui.separator();

        // Multi-column layout: each column scrolls independently
        let available = ui.available_size();
        let col_count = if available.x > 800.0 { 3 } else if available.x > 500.0 { 2 } else { 1 };
        let col_width = available.x / col_count as f32 - 8.0;

        ui.columns(col_count, |columns| {
            // Column 0: Operations + Lift
            columns[0].push_id("col_ops", |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("scroll_ops")
                    .max_height(available.y)
                    .show(ui, |ui| {
                        ui.set_min_width(col_width);
                        pipeline_section(ui, "Operations", PipelineKind::Operations,
                            &preset.pipeline.operations, registry, frame_state, &mut actions);
                        ui.separator();
                        pipeline_section(ui, "Lift", PipelineKind::Lift,
                            &preset.pipeline.lift, registry, frame_state, &mut actions);
                    });
            });

            if col_count >= 2 {
                // Column 1: Camera
                columns[1].push_id("col_cam", |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("scroll_cam")
                        .max_height(available.y)
                        .show(ui, |ui| {
                            ui.set_min_width(col_width);
                            camera_position_section(ui, preset, frame_state, &mut actions);
                            ui.separator();
                            camera_lens_section(ui, preset, frame_state, &mut actions);
                            ui.separator();
                            camera_distortion_section(ui, preset, frame_state, &mut actions);
                            ui.separator();
                            phosphor_section(ui, preset, frame_state, &mut actions);
                        });
                });
            }

            if col_count >= 3 {
                // Column 2: Quaternion
                columns[2].push_id("col_quat", |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("scroll_quat")
                        .max_height(available.y)
                        .show(ui, |ui| {
                            ui.set_min_width(col_width);
                            quaternion_section(ui, preset, frame_state, &mut actions);
                        });
                });
            }

            // Fallback: if single column, put camera + quat below pipelines
            if col_count == 1 {
                columns[0].push_id("col_rest", |ui| {
                    ui.separator();
                    camera_position_section(ui, preset, frame_state, &mut actions);
                    ui.separator();
                    camera_lens_section(ui, preset, frame_state, &mut actions);
                    ui.separator();
                    phosphor_section(ui, preset, frame_state, &mut actions);
                    ui.separator();
                    quaternion_section(ui, preset, frame_state, &mut actions);
                });
            }
        });
    });

    actions
}

// =============================================================================
// Header section
// =============================================================================

fn header_section(ui: &mut egui::Ui, preset: &Preset, actions: &mut Vec<UiAction>) {
    ui.horizontal(|ui| {
        ui.label("Name:");
        let mut name = preset.metadata.name.clone();
        if ui.text_edit_singleline(&mut name).changed() {
            actions.push(UiAction::SetPresetName(name));
        }
    });

    ui.horizontal(|ui| {
        ui.label("Palette:");
        let current = &preset.palette;
        egui::ComboBox::from_id_salt("palette_combo")
            .selected_text(current.as_str())
            .show_ui(ui, |ui| {
                for &key in PALETTE_KEYS {
                    if ui.selectable_label(current == key, key).clicked() {
                        actions.push(UiAction::SetPalette(key.to_string()));
                    }
                }
            });
    });

    ui.horizontal(|ui| {
        if ui.button("Save Preset").clicked() {
            actions.push(UiAction::SavePreset);
        }
        if ui.button("Load Preset").clicked() {
            actions.push(UiAction::LoadPreset);
        }
    });
}

// =============================================================================
// Tempo section (BPM display with preset buttons)
// =============================================================================

fn tempo_section(ui: &mut egui::Ui, preset: &Preset, actions: &mut Vec<UiAction>) {
    ui.label("Tempo");

    let bpm = preset.tempo.value * 60.0;

    // Manual BPM entry — accepts any value from 0.01 to 1,000,000
    ui.horizontal(|ui| {
        let mut bpm_entry = bpm;
        if ui.add(
            egui::DragValue::new(&mut bpm_entry)
                .speed(1.0)
                .range(0.01..=1_000_000.0)
                .suffix(" bpm")
        ).changed() {
            actions.push(UiAction::SetParam(ParamField::Tempo, bpm_entry / 60.0));
        }
    });

    // Logarithmic slider for quick exploration
    let mut bpm_val = bpm.clamp(0.01, 1_000_000.0);
    if ui.add(egui::Slider::new(&mut bpm_val, 0.01..=1_000_000.0)
        .logarithmic(true)
        .show_value(false))
        .changed()
    {
        actions.push(UiAction::SetParam(ParamField::Tempo, bpm_val / 60.0));
    }

    // BPM preset buttons — grouped in rows
    for group in BPM_GROUPS {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(group.label).weak().size(10.0));
            for &preset_bpm in group.values {
                let is_active = (bpm.round() as u32) == preset_bpm;
                if ui.selectable_label(is_active, format!("{}", preset_bpm)).clicked() {
                    actions.push(UiAction::SetParam(ParamField::Tempo, preset_bpm as f64 / 60.0));
                }
            }
        });
    }
}

// =============================================================================
// Pipeline sections (operations / lift)
// =============================================================================

fn pipeline_section(
    ui: &mut egui::Ui,
    label: &str,
    kind: PipelineKind,
    layers: &[polyform_core::layer::Layer],
    registry: &Registry,
    frame_state: Option<&FrameState>,
    actions: &mut Vec<UiAction>,
) {
    let id = ui.make_persistent_id(label);
    egui::CollapsingHeader::new(label)
        .id_salt(id)
        .default_open(true)
        .show(ui, |ui| {
            for (idx, layer) in layers.iter().enumerate() {
                let def_label = match kind {
                    PipelineKind::Operations => registry.operations.get(layer.type_key.as_str())
                        .map(|d| d.label),
                    PipelineKind::Lift => registry.lift.get(layer.type_key.as_str())
                        .map(|d| d.label),
                };
                let display_label = def_label.unwrap_or(&layer.type_key);

                // Look up the flat layer state for this index to get swept values
                let flat_layer = frame_state.and_then(|fs| {
                    let flat_layers = match kind {
                        PipelineKind::Operations => &fs.operations,
                        PipelineKind::Lift => &fs.lifts,
                    };
                    flat_layers.get(idx)
                });

                let layer_id = ui.make_persistent_id(format!("{label}_{}", layer.id));
                egui::CollapsingHeader::new(format!("{}. {}", idx + 1, display_label))
                    .id_salt(layer_id)
                    .default_open(false)
                    .show(ui, |ui| {
                        // Enable toggle
                        let mut enabled = layer.enabled;
                        if ui.checkbox(&mut enabled, "Enabled").changed() {
                            actions.push(UiAction::SetLayerEnabled(kind, layer.id.clone(), enabled));
                        }

                        // Type combo box
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
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt(format!("type_{}_{}", label, layer.id))
                                .selected_text(display_label)
                                .show_ui(ui, |ui| {
                                    for (key, lbl) in &type_entries {
                                        if ui.selectable_label(*key == layer.type_key.as_str(), *lbl).clicked() {
                                            actions.push(UiAction::ChangeLayerType(kind, idx, key.to_string()));
                                        }
                                    }
                                });
                        });

                        // Blend — full param widget (animatable like any other param)
                        let swept_blend = if layer.blend.anim {
                            flat_layer.map(|fl| fl.blend)
                        } else {
                            flat_layer.and_then(|fl| {
                                // Show transition-driven blend even when not animating
                                let diff = (fl.blend - layer.blend.value).abs();
                                if diff > 0.001 { Some(fl.blend) } else { None }
                            })
                        };
                        param_widget(
                            ui,
                            "blend",
                            &layer.blend,
                            ParamField::LayerBlend { kind, layer_id: layer.id.clone() },
                            swept_blend,
                            actions,
                        );

                        // Param sliders
                        // Get param order from registry
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
                                param_widget(
                                    ui,
                                    key,
                                    param,
                                    ParamField::LayerParam {
                                        kind,
                                        layer_id: layer.id.clone(),
                                        key: key.to_string(),
                                    },
                                    swept_value,
                                    actions,
                                );
                            }
                        }

                        // Move / Remove buttons
                        ui.horizontal(|ui| {
                            if idx > 0 && ui.button("\u{2191} Up").clicked() {
                                actions.push(UiAction::MoveLayer(kind, idx, -1));
                            }
                            if idx < layers.len() - 1 && ui.button("\u{2193} Down").clicked() {
                                actions.push(UiAction::MoveLayer(kind, idx, 1));
                            }
                            if ui.button("Remove").clicked() {
                                actions.push(UiAction::RemoveLayer(kind, idx));
                            }
                        });
                    });
            }

            // Add layer button
            add_layer_button(ui, label, kind, registry, actions);
        });
}

fn add_layer_button(
    ui: &mut egui::Ui,
    section_label: &str,
    kind: PipelineKind,
    registry: &Registry,
    actions: &mut Vec<UiAction>,
) {
    // Group entries by category
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
// Ratio picker widget
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
// Modulator ratio widget
// =============================================================================

/// Ratio widget that emits SetModulatorRatio instead of SetParamRatio.
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
// Min/max editing
// =============================================================================

fn min_max_widget(
    ui: &mut egui::Ui,
    param: &Param,
    field: &ParamField,
    actions: &mut Vec<UiAction>,
) {
    ui.horizontal(|ui| {
        ui.add_space(16.0);
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
// Param widget
// =============================================================================

fn param_widget(
    ui: &mut egui::Ui,
    label: &str,
    param: &Param,
    field: ParamField,
    swept_value: Option<f64>,
    actions: &mut Vec<UiAction>,
) {
    // Show swept value on slider when animating, but drag still sets param.value
    let display_value = swept_value.unwrap_or(param.value);
    let mut value = display_value;
    let slider_label = if swept_value.is_some() {
        format!("{} ~", label)
    } else {
        label.to_string()
    };
    let response = ui.add(egui::Slider::new(&mut value, param.min..=param.max).text(&slider_label));
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

    // Anim controls on a second row (compact)
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        let mut anim = param.anim;
        if ui.checkbox(&mut anim, "anim").changed() {
            actions.push(UiAction::SetParamAnim(field.clone(), anim));
        }
        if param.anim {
            // Shape selector: sawtooth vs cosine
            let is_sawtooth = param.shape == ShapeFunction::Sawtooth;
            if ui.selectable_label(is_sawtooth, "saw").clicked() && !is_sawtooth {
                actions.push(UiAction::SetParamShape(field.clone(), ShapeFunction::Sawtooth));
            }
            if ui.selectable_label(!is_sawtooth, "cos").clicked() && is_sawtooth {
                actions.push(UiAction::SetParamShape(field.clone(), ShapeFunction::Cosine));
            }
            ratio_widget(ui, param.ratio, &field, &format!("ratio_{}", label), actions);
        }
    });

    // Min/max, offset, and modulator when animated
    if param.anim {
        min_max_widget(ui, param, &field, actions);
        offset_and_modulator_controls(ui, param, &field, label, actions);
    }
}

// =============================================================================
// Toggle param widget (camera / sampling params)
// =============================================================================

fn toggle_param_widget(
    ui: &mut egui::Ui,
    label: &str,
    tp: &ToggleParam,
    view_key: &str,
    frame_state: Option<&FrameState>,
    actions: &mut Vec<UiAction>,
) {
    // Look up swept value from frame state if param is on and animating
    let swept_value = if tp.on && tp.param.anim {
        frame_state.and_then(|fs| frame_state_view_value(fs, view_key))
    } else {
        None
    };

    ui.horizontal(|ui| {
        let mut on = tp.on;
        if ui.checkbox(&mut on, "").changed() {
            actions.push(UiAction::SetToggleParamOn(view_key.to_string(), on));
        }

        let display_value = swept_value.unwrap_or(tp.param.value);
        let mut value = display_value;
        let slider_label = if swept_value.is_some() {
            format!("{} ~", label)
        } else {
            label.to_string()
        };
        let slider = egui::Slider::new(&mut value, tp.param.min..=tp.param.max)
            .text(&slider_label)
            .clamping(egui::SliderClamping::Never);
        let response = ui.add(slider);
        if response.changed() {
            actions.push(UiAction::SetParam(
                ParamField::View(view_key.to_string()),
                value,
            ));
        }
        if tp.param.anim {
            let grab_field = ParamField::View(view_key.to_string());
            if response.drag_started() {
                actions.push(UiAction::ParamGrabStart(grab_field.clone()));
            }
            if response.drag_stopped() {
                actions.push(UiAction::ParamGrabEnd(grab_field));
            }
        }
    });

    let field = ParamField::View(view_key.to_string());

    ui.horizontal(|ui| {
        ui.add_space(32.0);
        let mut anim = tp.param.anim;
        if ui.checkbox(&mut anim, "anim").changed() {
            actions.push(UiAction::SetParamAnim(field.clone(), anim));
        }
        if tp.param.anim {
            // Shape selector: sawtooth vs cosine
            let is_sawtooth = tp.param.shape == ShapeFunction::Sawtooth;
            if ui.selectable_label(is_sawtooth, "saw").clicked() && !is_sawtooth {
                actions.push(UiAction::SetParamShape(field.clone(), ShapeFunction::Sawtooth));
            }
            if ui.selectable_label(!is_sawtooth, "cos").clicked() && is_sawtooth {
                actions.push(UiAction::SetParamShape(field.clone(), ShapeFunction::Cosine));
            }
            ratio_widget(ui, tp.param.ratio, &field, &format!("ratio_{}", view_key), actions);
        }
    });

    // Min/max, offset, and modulator when animated
    if tp.param.anim {
        min_max_widget(ui, &tp.param, &field, actions);
        offset_and_modulator_controls(ui, &tp.param, &field, view_key, actions);
    }
}

/// Offset and modulator controls — shared between param_widget and toggle_param_widget.
fn offset_and_modulator_controls(
    ui: &mut egui::Ui,
    param: &Param,
    field: &ParamField,
    id_hint: &str,
    actions: &mut Vec<UiAction>,
) {
    // Phase offset slider
    ui.horizontal(|ui| {
        ui.add_space(16.0);
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
        ui.add_space(16.0);
        ui.label("FM depth:");
        let mut depth = mod_depth;
        if ui.add(egui::Slider::new(&mut depth, 0.0..=1.0).show_value(true)).changed() {
            actions.push(UiAction::SetModulatorDepth(field.clone(), depth));
        }
    });

    if mod_depth.abs() > 1e-6 {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            // Modulator shape selector
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

// =============================================================================
// Camera sections
// =============================================================================

fn camera_position_section(ui: &mut egui::Ui, preset: &Preset, frame_state: Option<&FrameState>, actions: &mut Vec<UiAction>) {
    let pos = &preset.camera.position;
    let all_keys: &[&str] = &[
        "camera.position.orbitX", "camera.position.orbitY",
        "camera.position.distance", "camera.position.targetX",
        "camera.position.targetY", "camera.position.roll",
    ];
    let any_on = pos.orbit_x.on || pos.orbit_y.on || pos.distance.on
        || pos.target_x.on || pos.target_y.on || pos.roll.on;

    ui.horizontal(|ui| {
        let mut group_on = any_on;
        if ui.checkbox(&mut group_on, "").changed() {
            for key in all_keys {
                actions.push(UiAction::SetToggleParamOn(key.to_string(), group_on));
            }
        }
    });

    egui::CollapsingHeader::new("Camera > Position")
        .default_open(false)
        .show(ui, |ui| {
            toggle_param_widget(ui, "orbit x", &pos.orbit_x, "camera.position.orbitX", frame_state, actions);
            toggle_param_widget(ui, "orbit y", &pos.orbit_y, "camera.position.orbitY", frame_state, actions);
            toggle_param_widget(ui, "distance", &pos.distance, "camera.position.distance", frame_state, actions);
            toggle_param_widget(ui, "target x", &pos.target_x, "camera.position.targetX", frame_state, actions);
            toggle_param_widget(ui, "target y", &pos.target_y, "camera.position.targetY", frame_state, actions);
            toggle_param_widget(ui, "roll", &pos.roll, "camera.position.roll", frame_state, actions);
        });
}

fn camera_lens_section(ui: &mut egui::Ui, preset: &Preset, frame_state: Option<&FrameState>, actions: &mut Vec<UiAction>) {
    let lens = &preset.camera.lens;
    let all_keys: &[&str] = &[
        "camera.lens.focalLength", "camera.lens.sensorSize",
        "camera.lens.orthoZoom", "camera.lens.anamorphic",
        "camera.lens.shiftX", "camera.lens.shiftY",
    ];
    let any_on = lens.focal_length.on || lens.sensor_size.on || lens.ortho_zoom.on
        || lens.anamorphic.on || lens.shift_x.on || lens.shift_y.on;

    ui.horizontal(|ui| {
        let mut group_on = any_on;
        if ui.checkbox(&mut group_on, "").changed() {
            for key in all_keys {
                actions.push(UiAction::SetToggleParamOn(key.to_string(), group_on));
            }
        }
    });

    egui::CollapsingHeader::new("Camera > Lens")
        .default_open(false)
        .show(ui, |ui| {
            // Projection mode
            ui.horizontal(|ui| {
                ui.label("Projection:");
                let mut proj = preset.camera.lens.projection;
                if ui.selectable_label(proj == ProjectionMode::Perspective, "Perspective").clicked() {
                    proj = ProjectionMode::Perspective;
                    actions.push(UiAction::SetProjection(proj));
                }
                if ui.selectable_label(proj == ProjectionMode::Orthographic, "Orthographic").clicked() {
                    proj = ProjectionMode::Orthographic;
                    actions.push(UiAction::SetProjection(proj));
                }
            });

            toggle_param_widget(ui, "focal length", &lens.focal_length, "camera.lens.focalLength", frame_state, actions);
            toggle_param_widget(ui, "sensor size", &lens.sensor_size, "camera.lens.sensorSize", frame_state, actions);
            toggle_param_widget(ui, "ortho zoom", &lens.ortho_zoom, "camera.lens.orthoZoom", frame_state, actions);
            toggle_param_widget(ui, "anamorphic", &lens.anamorphic, "camera.lens.anamorphic", frame_state, actions);
            toggle_param_widget(ui, "shift x", &lens.shift_x, "camera.lens.shiftX", frame_state, actions);
            toggle_param_widget(ui, "shift y", &lens.shift_y, "camera.lens.shiftY", frame_state, actions);
        });
}

fn camera_distortion_section(ui: &mut egui::Ui, preset: &Preset, frame_state: Option<&FrameState>, actions: &mut Vec<UiAction>) {
    let dist = &preset.camera.distortion;
    let all_keys: &[&str] = &[
        "camera.distortion.k1", "camera.distortion.k2",
        "camera.distortion.k3", "camera.distortion.k4",
        "camera.distortion.p1", "camera.distortion.p2",
        "camera.distortion.centerX", "camera.distortion.centerY",
        "camera.distortion.squeeze",
    ];
    let any_on = dist.k1.on || dist.k2.on || dist.k3.on || dist.k4.on
        || dist.p1.on || dist.p2.on || dist.center_x.on || dist.center_y.on || dist.squeeze.on;

    // Header with group toggle visible even when collapsed
    ui.horizontal(|ui| {
        let mut group_on = any_on;
        if ui.checkbox(&mut group_on, "").changed() {
            for key in all_keys {
                actions.push(UiAction::SetToggleParamOn(key.to_string(), group_on));
            }
        }
    });

    egui::CollapsingHeader::new("Camera > Distortion")
        .default_open(false)
        .show(ui, |ui| {
            toggle_param_widget(ui, "k1", &dist.k1, "camera.distortion.k1", frame_state, actions);
            toggle_param_widget(ui, "k2", &dist.k2, "camera.distortion.k2", frame_state, actions);
            toggle_param_widget(ui, "k3", &dist.k3, "camera.distortion.k3", frame_state, actions);
            toggle_param_widget(ui, "k4", &dist.k4, "camera.distortion.k4", frame_state, actions);
            toggle_param_widget(ui, "p1", &dist.p1, "camera.distortion.p1", frame_state, actions);
            toggle_param_widget(ui, "p2", &dist.p2, "camera.distortion.p2", frame_state, actions);
            toggle_param_widget(ui, "center x", &dist.center_x, "camera.distortion.centerX", frame_state, actions);
            toggle_param_widget(ui, "center y", &dist.center_y, "camera.distortion.centerY", frame_state, actions);
            toggle_param_widget(ui, "squeeze", &dist.squeeze, "camera.distortion.squeeze", frame_state, actions);
        });
}

// =============================================================================
// Quaternion section
// =============================================================================

fn quaternion_section(ui: &mut egui::Ui, preset: &Preset, frame_state: Option<&FrameState>, actions: &mut Vec<UiAction>) {
    egui::CollapsingHeader::new("Quaternion")
        .default_open(false)
        .show(ui, |ui| {
            let q = &preset.quaternion;

            let mut on = q.on;
            if ui.checkbox(&mut on, "Enable quaternion").changed() {
                actions.push(UiAction::SetQuatOn(on));
            }

            if q.on {
                // Mode
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    if ui.selectable_label(q.mode == QuaternionMode::Fixed, "Fixed").clicked() {
                        actions.push(UiAction::SetQuatMode(QuaternionMode::Fixed));
                    }
                    if ui.selectable_label(q.mode == QuaternionMode::Drift, "Drift").clicked() {
                        actions.push(UiAction::SetQuatMode(QuaternionMode::Drift));
                    }
                });

                // Angle — full param widget
                let swept_angle = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.angle"));
                param_widget(
                    ui,
                    "angle",
                    &q.angle,
                    ParamField::View("quaternion.angle".to_string()),
                    swept_angle,
                    actions,
                );

                // Axes — full param widgets (only in Fixed mode)
                if q.mode == QuaternionMode::Fixed {
                    let swept_ax = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.ax"));
                    param_widget(
                        ui,
                        "ax",
                        &q.ax,
                        ParamField::View("quaternion.ax".to_string()),
                        swept_ax,
                        actions,
                    );
                    let swept_ay = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.ay"));
                    param_widget(
                        ui,
                        "ay",
                        &q.ay,
                        ParamField::View("quaternion.ay".to_string()),
                        swept_ay,
                        actions,
                    );
                    let swept_az = frame_state.and_then(|fs| frame_state_view_value(fs, "quaternion.az"));
                    param_widget(
                        ui,
                        "az",
                        &q.az,
                        ParamField::View("quaternion.az".to_string()),
                        swept_az,
                        actions,
                    );
                }

                if q.mode == QuaternionMode::Drift {
                    let mut drift = q.drift_speed;
                    if ui.add(egui::Slider::new(&mut drift, 0.01..=2.0).text("drift speed")).changed() {
                        actions.push(UiAction::SetQuatDriftSpeed(drift));
                    }
                }
            }
        });
}

// =============================================================================
// Sampling section
// =============================================================================

fn sampling_section(ui: &mut egui::Ui, preset: &Preset, frame_state: Option<&FrameState>, actions: &mut Vec<UiAction>) {
    toggle_param_widget(ui, "lattice scale", &preset.sampling.scale, "sampling.scale", frame_state, actions);
    toggle_param_widget(ui, "lattice density", &preset.sampling.density, "sampling.density", frame_state, actions);

    // Lattice type selector
    ui.horizontal(|ui| {
        ui.label("Lattice:");
        let lt = preset.sampling.lattice_type;
        for &(label, ty) in &[
            ("Cartesian", LatticeType::Cartesian),
            ("Polar", LatticeType::Polar),
            ("Hex", LatticeType::Hex),
            ("LogPolar", LatticeType::LogPolar),
            ("Bipolar", LatticeType::Bipolar),
        ] {
            if ui.selectable_label(lt == ty, label).clicked() && lt != ty {
                actions.push(UiAction::SetLatticeType(ty));
            }
        }
    });

    // Per-type controls
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
            });
            ui.horizontal(|ui| {
                ui.label("inner radius:");
                let mut v = lp.inner_radius;
                if ui.add(egui::DragValue::new(&mut v).speed(0.005).range(0.01..=0.5)).changed() {
                    actions.push(UiAction::SetLatticeInnerRadius(v));
                }
            });
        }
        LatticeType::Bipolar => {
            ui.horizontal(|ui| {
                ui.label("focus distance:");
                let mut v = lp.focus_distance;
                if ui.add(egui::DragValue::new(&mut v).speed(0.01).range(0.1..=3.0)).changed() {
                    actions.push(UiAction::SetLatticeFocusDistance(v));
                }
            });
            ui.horizontal(|ui| {
                ui.label("tau curves:");
                let mut v = lp.tau_count as i32;
                if ui.add(egui::DragValue::new(&mut v).speed(1).range(4..=80)).changed() {
                    actions.push(UiAction::SetLatticeTauCount(v as u32));
                }
            });
        }
        LatticeType::Cartesian | LatticeType::Hex => {}
    }
}

// =============================================================================
// Phosphor section
// =============================================================================

fn phosphor_section(ui: &mut egui::Ui, preset: &Preset, frame_state: Option<&FrameState>, actions: &mut Vec<UiAction>) {
    let ph = &preset.phosphor;
    let all_keys: &[&str] = &[
        "phosphor.persistence", "phosphor.beamWidth", "phosphor.beamIntensity",
        "phosphor.gain", "phosphor.blendMix", "phosphor.saturation",
        "phosphor.bloomIntensity", "phosphor.bloomRadius", "phosphor.bloomThreshold",
        "phosphor.gamma",
    ];
    let any_on = ph.persistence.on || ph.beam_width.on || ph.beam_intensity.on
        || ph.gain.on || ph.blend_mix.on || ph.saturation.on
        || ph.bloom_intensity.on || ph.bloom_radius.on || ph.bloom_threshold.on
        || ph.gamma.on;

    // Header with group toggle
    ui.horizontal(|ui| {
        let mut group_on = any_on;
        if ui.checkbox(&mut group_on, "").changed() {
            for key in all_keys {
                actions.push(UiAction::SetToggleParamOn(key.to_string(), group_on));
            }
        }
    });

    egui::CollapsingHeader::new("Phosphor")
        .default_open(false)
        .show(ui, |ui| {
            toggle_param_widget(ui, "persistence", &ph.persistence, "phosphor.persistence", frame_state, actions);
            toggle_param_widget(ui, "beam width", &ph.beam_width, "phosphor.beamWidth", frame_state, actions);
            toggle_param_widget(ui, "beam drive", &ph.beam_intensity, "phosphor.beamIntensity", frame_state, actions);
            toggle_param_widget(ui, "gain", &ph.gain, "phosphor.gain", frame_state, actions);
            toggle_param_widget(ui, "blend mix", &ph.blend_mix, "phosphor.blendMix", frame_state, actions);
            toggle_param_widget(ui, "saturation", &ph.saturation, "phosphor.saturation", frame_state, actions);
            toggle_param_widget(ui, "bloom", &ph.bloom_intensity, "phosphor.bloomIntensity", frame_state, actions);
            toggle_param_widget(ui, "bloom radius", &ph.bloom_radius, "phosphor.bloomRadius", frame_state, actions);
            toggle_param_widget(ui, "bloom thresh", &ph.bloom_threshold, "phosphor.bloomThreshold", frame_state, actions);
            toggle_param_widget(ui, "gamma", &ph.gamma, "phosphor.gamma", frame_state, actions);
        });
}

// =============================================================================
// Viewport section
// =============================================================================

fn viewport_section(ui: &mut egui::Ui, overlay: &mut OverlayState, actions: &mut Vec<UiAction>) {
    ui.horizontal(|ui| {
        ui.label("Viewport:");

        let mut w = overlay.viewport_width as i32;
        let mut h = overlay.viewport_height as i32;

        let w_changed = ui.add(
            egui::DragValue::new(&mut w)
                .speed(1)
                .range(320..=7680),
        ).changed();

        ui.label("\u{00d7}");

        let h_changed = ui.add(
            egui::DragValue::new(&mut h)
                .speed(1)
                .range(320..=7680),
        ).changed();

        if w_changed || h_changed {
            overlay.viewport_width = w as u32;
            overlay.viewport_height = h as u32;
            actions.push(UiAction::SetViewportSize(w as u32, h as u32));
        }

        if ui.small_button("HD").clicked() {
            overlay.viewport_width = 1920;
            overlay.viewport_height = 1080;
            actions.push(UiAction::SetViewportSize(1920, 1080));
        }
        if ui.small_button("2K").clicked() {
            overlay.viewport_width = 2560;
            overlay.viewport_height = 1440;
            actions.push(UiAction::SetViewportSize(2560, 1440));
        }
        if ui.small_button("4K").clicked() {
            overlay.viewport_width = 3840;
            overlay.viewport_height = 2160;
            actions.push(UiAction::SetViewportSize(3840, 2160));
        }
    });
}
