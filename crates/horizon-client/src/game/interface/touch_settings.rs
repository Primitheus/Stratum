//! Stratum touch preferences and an editor that uses the same layout as hit testing.
use super::*;
use horizon_client::lang;
use horizon_client_input::touch_controls::{Action, Button, Layout};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone)]
pub(super) struct Place {
    x: f32,
    y: f32,
    size: f32,
    opacity: f32,
    show: bool,
}
#[derive(Clone)]
pub(super) struct TouchSettings {
    pub classic: bool,
    pub joystick: bool,
    pub labels: bool,
    pub look: f32,
    pub stable_terrain: bool,
    places: BTreeMap<String, Place>,
}
impl Default for TouchSettings {
    fn default() -> Self {
        Self {
            classic: true,
            labels: false,
            joystick: true,
            look: 1.0,
            stable_terrain: true,
            places: BTreeMap::new(),
        }
    }
}
#[derive(Default)]
pub(super) struct TouchUi {
    pub config: TouchSettings,
    pub editing: bool,
    pub loaded: bool,
    selected: Option<String>,
    pub drag: Option<(DeviceId, u64, f32, f32)>,
    pub save_failed: bool,
}
pub(super) fn control_id(a: Action) -> &'static str {
    match a {
        Action::Jump => "jump",
        Action::Sneak => "sneak",
        Action::Sprint => "sprint",
        Action::Attack => "mine",
        Action::Use => "use",
        Action::Inventory => "inventory",
        Action::Pause => "pause",
        Action::Drop => "drop",
        Action::Previous => "previous",
        Action::Next => "next",
        Action::Customize => "hud",
        _ => "move",
    }
}
fn settings_path() -> Option<std::path::PathBuf> {
    std::env::var_os("STRATUM_CONTROLS_FILE").map(Into::into)
}
impl TouchSettings {
    fn load() -> Self {
        let mut config = Self::default();
        let Some(value) = settings_path()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        else {
            return config;
        };
        config.stable_terrain = value["stable_terrain"].as_bool().unwrap_or(true);
        config.classic = value["classic"].as_bool().unwrap_or(true);
        config.labels = value["version"].as_u64().is_some_and(|v| v >= 2)
            && value["labels"].as_bool().unwrap_or(false);
        config.joystick = value["joystick"].as_bool().unwrap_or(true);
        config.look = value["look"].as_f64().unwrap_or(1.0).clamp(0.25, 2.0) as f32;
        if let Some(places) = value["controls"].as_object() {
            for (id, p) in places {
                config.places.insert(
                    id.clone(),
                    Place {
                        x: p["x"].as_f64().unwrap_or(0.5).clamp(0.0, 1.0) as f32,
                        y: p["y"].as_f64().unwrap_or(0.5).clamp(0.0, 1.0) as f32,
                        size: p["size"].as_f64().unwrap_or(1.0).clamp(0.5, 1.75) as f32,
                        opacity: p["opacity"].as_f64().unwrap_or(0.65).clamp(0.15, 1.0) as f32,
                        show: p["show"].as_bool().unwrap_or(true),
                    },
                );
            }
        }
        config
    }
    fn save(&self) -> std::io::Result<()> {
        let Some(path) = settings_path() else {
            return Err(std::io::Error::other("controls path unavailable"));
        };
        let controls: BTreeMap<_, _> = self
            .places
            .iter()
            .map(|(k, p)| {
                (
                    k,
                    json!({"x":p.x,"y":p.y,"size":p.size,"opacity":p.opacity,"show":p.show}),
                )
            })
            .collect();
        let data = json!({"version":2,"joystick":self.joystick,"classic":self.classic,"labels":self.labels,"look":self.look,"stable_terrain":self.stable_terrain,"controls":controls});
        let temp = path.with_extension("tmp");
        std::fs::write(&temp, data.to_string())?;
        std::fs::rename(temp, path)
    }
    pub(super) fn save_on_exit(&self) {
        if let Err(e) = self.save() {
            eprintln!("Touch settings could not be saved: {e}");
        }
    }
    pub fn layout(&self, size: [f32; 2], playing: bool, editing: bool) -> Layout {
        let [w, h] = size;
        let s = (w / 1000.).min(h / 500.);
        let mut layout = Layout::new(size, playing || editing);
        layout.auto_sprint = self.joystick;
        // Clears the 126px landscape cutout on the A059 at its 2.16 layout scale.
        let edge = 64. * s;
        for b in &mut layout.buttons {
            b.radius = 32. * s;
            match b.action {
                Action::Pause => {
                    b.x = w / 2. + 26. * s;
                    b.y = 40. * s;
                    b.radius = 20. * s;
                }
                Action::Inventory => {
                    // Keep clear of the hotbar at independently selected GUI scales.
                    b.x = w - 110. * s;
                    b.y = h - 45. * s;
                }
                Action::Drop => {
                    b.x = w / 2. + 185. * s;
                    b.y = h - 32. * s;
                }
                Action::Jump => {
                    b.x = w - 110. * s;
                    b.y = h - 125. * s;
                }
                Action::Sneak => {
                    b.x = w - 190. * s;
                    b.y = h - 125. * s;
                }
                Action::Attack => {
                    b.x = w - 190. * s;
                    b.y = h - 210. * s;
                }
                Action::Use => {
                    b.x = w - 110. * s;
                    b.y = h - 210. * s;
                }
                _ => {}
            }
        }
        layout.stick.2 = 62. * s;
        layout.buttons.push(Button {
            action: Action::Customize,
            label: "HUD",
            x: w / 2. - 26. * s,
            y: 40. * s,
            radius: 20. * s,
        });
        for b in &mut layout.buttons {
            if let Some(p) = self.places.get(control_id(b.action)) {
                b.x = p.x * w;
                b.y = p.y * h;
                b.radius *= p.size;
            }
            b.x = b.x.clamp(b.radius + edge, w - b.radius - edge);
            b.y = b.y.clamp(b.radius + 30. * s, h - b.radius - 12. * s);
        }
        // Pause and the editor are always reachable, even in a heavily edited layout.
        layout.buttons.retain(|b| {
            editing
                || matches!(b.action, Action::Pause | Action::Customize)
                || self.places.get(control_id(b.action)).map_or(
                    !matches!(b.action, Action::Drop | Action::Previous | Action::Next)
                        && !(self.joystick && b.action == Action::Sprint),
                    |p| p.show,
                )
        });
        if let Some(p) = self.places.get("move") {
            layout.stick = (p.x * w, p.y * h, layout.stick.2 * p.size);
        }
        let (x, y, r) = &mut layout.stick;
        *x = x.clamp(*r + edge, w - *r - edge);
        *y = y.clamp(*r + 30. * s, h - *r - 12. * s);
        layout
    }
    pub fn opacity(&self, id: &str, editing: bool) -> f32 {
        self.places
            .get(id)
            .map_or(0.65, |p| if editing && !p.show { 0.2 } else { p.opacity })
    }
    fn place(&mut self, id: &str, size: [f32; 2]) -> &mut Place {
        let layout = self.layout(size, true, true);
        let (x, y) = if id == "move" {
            (layout.stick.0, layout.stick.1)
        } else {
            layout
                .buttons
                .iter()
                .find(|b| control_id(b.action) == id)
                .map_or((size[0] / 2., size[1] / 2.), |b| (b.x, b.y))
        };
        self.places.entry(id.to_owned()).or_insert(Place {
            x: x / size[0],
            y: y / size[1],
            size: 1.,
            opacity: 0.65,
            show: true,
        })
    }
}
// Centre palette leaves the controls around the edges available for dragging.
fn tools(size: [f32; 2]) -> Vec<(&'static str, [f32; 4])> {
    let [w, h] = size;
    let s = (w / 1000.).min(h / 500.);
    [
        "style", "labels", "movement", "smaller", "larger", "fainter", "stronger", "hide",
        "select", "mirror", "reset", "slower", "faster", "terrain", "widgets", "done",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, id)| {
        (
            id,
            [
                w / 2. - 170. * s + (i % 2) as f32 * 175. * s,
                h / 2. - 100. * s + (i / 2) as f32 * 30. * s,
                165. * s,
                26. * s,
            ],
        )
    })
    .collect()
}
fn contains(r: [f32; 4], x: f32, y: f32) -> bool {
    x >= r[0] && y >= r[1] && x < r[0] + r[2] && y < r[1] + r[3]
}
impl Game<'_> {
    pub(super) fn load_touch_settings(&mut self) {
        if !self.touch_ui.loaded {
            self.touch_ui.config = TouchSettings::load();
            self.touch_ui.loaded = true;
        }
    }
    pub(super) fn open_touch_settings(&mut self) {
        if self.player.is_none() || self.title.is_some() {
            return;
        }
        self.cancel_touch();
        self.keys.clear();
        self.set_screen(Screen::Paused);
        self.touch_ui.editing = true;
        self.touch_ui.selected = None;
        self.touch_ui.drag = None;
    }
    pub(super) fn edit_touch(&mut self, t: winit::event::Touch, size: [f32; 2]) {
        use winit::event::TouchPhase;
        let (x, y) = (t.location.x as f32, t.location.y as f32);
        if t.phase == TouchPhase::Started && self.touch_ui.drag.is_none() {
            if let Some((tool, _)) = tools(size).into_iter().find(|(_, r)| contains(*r, x, y)) {
                let selected = self.touch_ui.selected.clone();
                match tool {
                    "done" | "widgets" => {
                        self.touch_ui.save_failed = self.touch_ui.config.save().is_err();
                        if self.touch_ui.save_failed {
                            return;
                        }
                        self.touch_ui.editing = false;
                        if tool == "widgets" {
                            self.open_hud_editor(false);
                        } else {
                            self.set_screen(Screen::Playing);
                            self.capture(true);
                        }
                    }
                    "terrain" => {
                        self.touch_ui.config.stable_terrain = !self.touch_ui.config.stable_terrain
                    }
                    "movement" => self.touch_ui.config.joystick = !self.touch_ui.config.joystick,
                    "style" => self.touch_ui.config.classic = !self.touch_ui.config.classic,
                    "labels" => self.touch_ui.config.labels = !self.touch_ui.config.labels,
                    "reset" => {
                        self.touch_ui.config = TouchSettings::default();
                        self.touch_ui.selected = None;
                    }
                    "select" => {
                        let layout = self.touch_ui.config.layout(size, true, true);
                        let ids: Vec<_> = std::iter::once("move")
                            .chain(layout.buttons.iter().map(|b| control_id(b.action)))
                            .collect();
                        let next = selected
                            .as_deref()
                            .and_then(|s| ids.iter().position(|id| *id == s))
                            .map_or(0, |i| (i + 1) % ids.len());
                        self.touch_ui.selected = Some(ids[next].to_owned());
                    }
                    "mirror" => {
                        let layout = self.touch_ui.config.layout(size, true, true);
                        for id in std::iter::once("move")
                            .chain(layout.buttons.iter().map(|b| control_id(b.action)))
                        {
                            let p = self.touch_ui.config.place(id, size);
                            p.x = 1. - p.x;
                        }
                    }
                    "slower" => {
                        self.touch_ui.config.look = (self.touch_ui.config.look - 0.1).max(0.25)
                    }
                    "faster" => {
                        self.touch_ui.config.look = (self.touch_ui.config.look + 0.1).min(2.)
                    }
                    _ => {
                        if let Some(id) = selected {
                            let p = self.touch_ui.config.place(&id, size);
                            match tool {
                                "smaller" => p.size = (p.size - 0.1).max(0.5),
                                "larger" => p.size = (p.size + 0.1).min(1.75),
                                "fainter" => p.opacity = (p.opacity - 0.1).max(0.15),
                                "stronger" => p.opacity = (p.opacity + 0.1).min(1.),
                                "hide" if !["move", "pause", "hud"].contains(&id.as_str()) => {
                                    p.show = !p.show
                                }
                                _ => {}
                            }
                        }
                    }
                }
                return;
            }
            let layout = self.touch_ui.config.layout(size, true, true);
            let hit = layout
                .buttons
                .iter()
                .rev()
                .find(|b| (x - b.x).abs() <= b.radius && (y - b.y).abs() <= b.radius)
                .map(|b| control_id(b.action))
                .or_else(|| {
                    let (sx, sy, r) = layout.stick;
                    ((x - sx).abs() < r && (y - sy).abs() < r).then_some("move")
                });
            if let Some(id) = hit {
                self.touch_ui.selected = Some(id.to_owned());
            }
            if let Some(id) = self.touch_ui.selected.clone() {
                let p = self.touch_ui.config.place(&id, size);
                self.touch_ui.drag =
                    Some((t.device_id, t.id, x - p.x * size[0], y - p.y * size[1]));
            }
        } else if let Some((device, id, dx, dy)) = self.touch_ui.drag {
            if (device, id) != (t.device_id, t.id) {
                return;
            }
            if t.phase == TouchPhase::Moved {
                if let Some(id) = self.touch_ui.selected.clone() {
                    let p = self.touch_ui.config.place(&id, size);
                    p.x = ((x - dx) / size[0]).clamp(0.02, 0.98);
                    p.y = ((y - dy) / size[1]).clamp(0.08, 0.94);
                }
            }
            if matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                self.touch_ui.drag = None;
            }
        }
    }
    pub(super) fn draw_touch_editor(&self, batch: &mut HudBatch, size: [f32; 2]) {
        let scale = batch.scale;
        let atlas = &self.gui.atlas;
        let title = lang::get("stratum.touch.editor");
        let hint = lang::get("stratum.touch.hint");
        for (text, y) in [(&title, size[1] * 0.19), (&hint, size[1] * 0.24)] {
            batch.text(
                atlas,
                text,
                (size[0] / scale - HudBatch::text_width(atlas, text)) / 2.,
                y / scale,
                0xffffffff,
                true,
            );
        }
        for (id, r) in tools(size) {
            let mut label = lang::get(&format!("stratum.touch.{id}"));
            if id == "terrain" {
                label = lang::get(if self.touch_ui.config.stable_terrain {
                    "stratum.touch.terrain_stable"
                } else {
                    "stratum.touch.terrain_fast"
                });
            }
            if id == "movement" {
                label = lang::get(if self.touch_ui.config.joystick {
                    "stratum.touch.joystick"
                } else {
                    "stratum.touch.dpad"
                });
            }
            if id == "style" {
                label = lang::get(if self.touch_ui.config.classic {
                    "stratum.touch.classic"
                } else {
                    "stratum.touch.minimal"
                });
            }
            batch.fill(
                atlas,
                r[0] / scale,
                r[1] / scale,
                r[2] / scale,
                r[3] / scale,
                0xee30343c,
            );
            batch.text(
                atlas,
                &label,
                (r[0] + r[2] / 2.) / scale - HudBatch::text_width(atlas, &label) / 2.,
                (r[1] + r[3] / 2.) / scale - 4.,
                0xffffffff,
                true,
            );
        }
        let selected = self.touch_ui.selected.as_deref().unwrap_or("none");
        let name = lang::get(&format!("stratum.touch.{selected}"));
        let text = if self.touch_ui.save_failed {
            lang::get("stratum.touch.save_failed")
        } else {
            format!(
                "{}   {}: {:.1}x",
                name,
                lang::get("stratum.touch.look"),
                self.touch_ui.config.look
            )
        };
        batch.text(
            atlas,
            &text,
            (size[0] / scale - HudBatch::text_width(atlas, &text)) / 2.,
            size[1] * 0.84 / scale,
            0xffffff99,
            true,
        );
    }
    pub(super) fn touch_selected(&self, id: &str) -> bool {
        self.touch_ui.editing && self.touch_ui.selected.as_deref() == Some(id)
    }
}
