//! Stratum's on-screen controls. Menus retain Horizon's normal pointer hit testing.
use super::*;
use horizon_client::lang;
use horizon_client_input::touch_controls::{Action, Changes};

impl Game<'_> {
    fn touch_is_playing(&self) -> bool {
        self.focused
            && self.player.is_some()
            && self.title.is_none()
            && self.screen == Screen::Playing
            && !self.screen_over_playing()
            && self.pending_save.is_none()
            && !self.scripted_run()
    }
    fn apply_touch(&mut self, changes: Changes) {
        for (action, down) in changes.edges {
            if action == Action::Customize {
                if down {
                    self.open_touch_settings();
                }
                continue;
            }
            let key = match action {
                Action::Slot(i) => Some(match i {
                    0 => KeyCode::Digit1,
                    1 => KeyCode::Digit2,
                    2 => KeyCode::Digit3,
                    3 => KeyCode::Digit4,
                    4 => KeyCode::Digit5,
                    5 => KeyCode::Digit6,
                    6 => KeyCode::Digit7,
                    7 => KeyCode::Digit8,
                    _ => KeyCode::Digit9,
                }),
                Action::Forward => Some(KeyCode::KeyW),
                Action::Backward => Some(KeyCode::KeyS),
                Action::Left => Some(KeyCode::KeyA),
                Action::Right => Some(KeyCode::KeyD),
                Action::Jump => Some(KeyCode::Space),
                Action::Sneak => Some(KeyCode::ShiftLeft),
                Action::Sprint => Some(KeyCode::ControlLeft),
                Action::Inventory => Some(KeyCode::KeyE),
                Action::Pause => Some(KeyCode::Escape),
                Action::Drop => Some(KeyCode::KeyQ),
                _ => None,
            };
            let result = if let Some(key) = key {
                self.mapped_key(key, down, false, key, Source::Touch)
            } else if matches!(action, Action::Attack | Action::Use) {
                self.mouse_mapping(
                    if action == Action::Attack {
                        MouseButton::Left
                    } else {
                        MouseButton::Right
                    },
                    down,
                    Source::Touch,
                )
            } else {
                if down {
                    self.hotbar_step(action == Action::Next);
                }
                Ok(())
            };
            if let Err(error) = result {
                log::warn!("Touch input: {error}");
            }
            // Keyboard sprint normally latches until forward is released. A touch
            // joystick must instead return to walking when pulled back inward.
            if action == Action::Sprint
                && !down
                && !self.toggle_held_elsewhere(1, KeyCode::ControlLeft, Source::Touch)
            {
                if let Some(player) = self.player.as_mut() {
                    if let Err(error) = player.set_sprinting(false) {
                        log::warn!("Touch sprint release: {error}");
                    }
                }
            }
        }
        if changes.look != [0.0; 2] && self.touch_is_playing() {
            self.mouse.on_move(
                0.0,
                0.0,
                changes.look[0] * f64::from(self.touch_ui.config.look),
                changes.look[1] * f64::from(self.touch_ui.config.look),
                true,
                true,
            );
            self.last_input = Instant::now();
        }
    }
    pub(super) fn cancel_touch(&mut self) {
        let changes = self.touch_controls.cancel();
        self.apply_touch(changes);
    }
    pub(super) fn touch_event(
        &mut self,
        event_loop: &dyn EventLoopCtl,
        id: WindowId,
        t: winit::event::Touch,
        origin: input_trace::InputOrigin,
    ) {
        self.load_touch_settings();
        if self.touch_ui.editing {
            if let Some(w) = &self.window {
                let s = w.inner_size();
                self.edit_touch(t, [s.width as f32, s.height as f32]);
            }
            return;
        }
        let playing = self.touch_is_playing();
        if playing != self.touch_playing {
            self.cancel_touch();
            for event in self.touch_pointer.cancel() {
                self.handle_window_event_as(event_loop, id, event, origin);
            }
            self.touch_playing = playing;
        }
        let Some(window) = &self.window else { return };
        let size = window.inner_size();
        let size = [size.width as f32, size.height as f32];
        let mut layout = self.touch_ui.config.layout(size, playing, false);
        if playing {
            let gui = HudBatch::new(
                [size[0] as u32, size[1] as u32],
                self.settings.gui_scale_inputs(),
            );
            let slot = 20. * gui.scale;
            for i in 0..9 {
                layout
                    .buttons
                    .push(horizon_client_input::touch_controls::Button {
                        action: Action::Slot(i),
                        label: "",
                        x: size[0] / 2. + (i as f32 - 4.) * slot,
                        y: size[1] - 11. * gui.scale,
                        radius: slot / 2.,
                    });
            }
        }
        // In-world screens need an accessible close action as well as normal tapping.
        let back = self.title.is_none()
            && self.player.is_some()
            && layout.buttons.iter().any(|b| {
                (t.location.x as f32 - b.x).abs() <= b.radius
                    && (t.location.y as f32 - b.y).abs() <= b.radius
            });
        if playing || back || self.touch_controls.held(Action::Pause) {
            let changes = self.touch_controls.touch_layout(t, size, playing, &layout);
            self.apply_touch(changes);
        } else {
            for event in self.touch_pointer.touch(t) {
                self.handle_window_event_as(event_loop, id, event, origin);
            }
        }
    }
    pub(super) fn draw_touch_controls(&mut self, batch: &mut HudBatch, size: [u32; 2]) {
        self.load_touch_settings();
        if let Some(renderer) = self.renderer.as_mut() {
            let _ = renderer.set_section_cull(!self.touch_ui.config.stable_terrain);
        }
        let playing = self.touch_is_playing();
        if !playing {
            self.cancel_touch();
        }
        if self.touch_ui.editing && self.screen != Screen::Paused {
            self.touch_ui.editing = false;
            self.touch_ui.config.save_on_exit();
        }
        if !playing && (self.title.is_some() || self.player.is_none()) {
            return;
        }
        let editing = self.touch_ui.editing;
        let size = [size[0] as f32, size[1] as f32];
        let scale = batch.scale;
        let atlas = &self.gui.atlas;
        if editing {
            batch.fill(atlas, 0., 0., size[0] / scale, size[1] / scale, 0xe0182028);
        }
        let layout = self.touch_ui.config.layout(size, playing, editing);
        let classic = self.touch_ui.config.classic;
        if playing || editing {
            let (x, y, r) = layout.stick;
            let alpha = (self.touch_ui.config.opacity("move", editing) * 255.) as u32;
            if self.touch_ui.config.joystick {
                let offset = self.touch_controls.stick_offset(&layout);
                if !touch_sprite(
                    batch,
                    atlas,
                    "stick",
                    [x / scale, y / scale],
                    r / scale,
                    alpha,
                ) {
                    batch.fill_disc(
                        atlas,
                        [x / scale, y / scale],
                        r / scale,
                        (alpha << 24) | 0x404040,
                    );
                }
                let knob = [
                    (x + offset[0] * r * 0.55) / scale,
                    (y + offset[1] * r * 0.55) / scale,
                ];
                if !touch_sprite(
                    batch,
                    atlas,
                    "stick_knob",
                    knob,
                    r * 0.5 / scale,
                    alpha.max(150),
                ) {
                    batch.fill_disc(
                        atlas,
                        knob,
                        r * 0.5 / scale,
                        (alpha.max(150) << 24) | 0xb0b0b0,
                    );
                }
                // The run chevrons light up when the outer forward zone is held.
                let running = self.touch_controls.held(Action::Sprint);
                let run_alpha = if running { 255 } else { 100 };
                let run_center = [x / scale, (y - r * 1.3) / scale];
                if !touch_sprite(
                    batch,
                    atlas,
                    "sprint",
                    run_center,
                    r * 0.22 / scale,
                    run_alpha,
                ) {
                    glyph(
                        batch,
                        atlas,
                        Action::Sprint,
                        run_center[0],
                        run_center[1],
                        r * 0.22 / scale,
                        (run_alpha << 24) | 0xffffff,
                    );
                }
            } else {
                for (name, dx, dy, a) in [
                    ("dpad_up", 0., -0.65, Action::Forward),
                    ("dpad_down", 0., 0.65, Action::Backward),
                    ("dpad_left", -0.65, 0., Action::Left),
                    ("dpad_right", 0.65, 0., Action::Right),
                ] {
                    let c = [(x + dx * r) / scale, (y + dy * r) / scale];
                    let name = if self.touch_controls.held(a) {
                        format!("{name}_pressed")
                    } else {
                        name.to_owned()
                    };
                    if !touch_sprite(batch, atlas, &name, c, r * 0.34 / scale, alpha) {
                        plate(
                            batch,
                            atlas,
                            c[0],
                            c[1],
                            r * 0.34 / scale,
                            alpha,
                            self.touch_controls.held(a),
                            self.touch_selected("move"),
                        );
                        glyph(
                            batch,
                            atlas,
                            a,
                            c[0],
                            c[1],
                            r * 0.35 / scale,
                            (alpha << 24) | 0xffffff,
                        );
                    }
                }
            }
        }
        for b in layout.buttons {
            let id = super::touch_settings::control_id(b.action);
            let alpha = (self.touch_ui.config.opacity(id, editing) * 255.) as u32;
            let (x, y, r) = (b.x / scale, b.y / scale, b.radius / scale);
            let name = match b.action {
                Action::Attack => "attack",
                Action::Use => "use",
                Action::Inventory => "inventory",
                Action::Customize => "custom_gear",
                Action::Sprint => "sprint",
                _ => id,
            };
            let sprite = if self.touch_controls.held(b.action) {
                format!("{name}_pressed")
            } else {
                name.to_owned()
            };
            let skinned = classic && touch_sprite(batch, atlas, &sprite, [x, y], r, alpha);
            if !skinned {
                if classic {
                    plate(
                        batch,
                        atlas,
                        x,
                        y,
                        r,
                        alpha,
                        self.touch_controls.held(b.action),
                        self.touch_selected(id),
                    );
                } else {
                    batch.fill_disc(atlas, [x, y], r, (alpha << 24) | 0x303030);
                }
                glyph(
                    batch,
                    atlas,
                    b.action,
                    x,
                    y,
                    r * 0.9,
                    (alpha << 24) | 0xffffff,
                );
            }
            if self.touch_selected(id) {
                batch.fill(atlas, x - r, y + r + 1., r * 2., 1., 0xffffff55);
            }
            if self.touch_ui.config.labels || editing {
                let text = if b.action == Action::Pause && !playing && !editing {
                    lang::get("stratum.touch.back")
                } else {
                    lang::get(&format!("stratum.touch.{id}"))
                };
                batch.text(
                    atlas,
                    &text,
                    x - HudBatch::text_width(atlas, &text) / 2.,
                    y + r + 2.,
                    0xffffffff,
                    true,
                );
            }
        }
        if editing {
            self.draw_touch_editor(batch, size);
        }
    }
}
fn plate(
    batch: &mut HudBatch,
    atlas: &horizon_client::hud::HudAtlas,
    x: f32,
    y: f32,
    r: f32,
    alpha: u32,
    pressed: bool,
    selected: bool,
) {
    let ink = if selected {
        0xffd86b
    } else if pressed {
        0xbac9d0
    } else {
        0x70787d
    };
    batch.fill(
        atlas,
        x - r,
        y - r,
        2. * r,
        2. * r,
        (alpha << 24) | 0x15191b,
    );
    batch.fill(
        atlas,
        x - r + 1.,
        y - r + 1.,
        2. * r - 2.,
        2. * r - 2.,
        (alpha << 24) | ink,
    );
    batch.fill(
        atlas,
        x - r + 1.,
        y - r + 1.,
        2. * r - 2.,
        1.,
        (alpha << 24) | 0xd0d5d8,
    );
    batch.fill(
        atlas,
        x - r + 1.,
        y + r - 2.,
        2. * r - 2.,
        1.,
        (alpha << 24) | 0x30373b,
    );
}
// Original geometric pixel glyphs; no Mojang textures are embedded in the APK.
fn glyph(
    batch: &mut HudBatch,
    atlas: &horizon_client::hud::HudAtlas,
    a: Action,
    x: f32,
    y: f32,
    size: f32,
    color: u32,
) {
    let rows: [u8; 8] = match a {
        Action::Jump | Action::Forward => [0x18, 0x3c, 0x7e, 0x18, 0x18, 0x18, 0x18, 0],
        Action::Sneak | Action::Backward => [0x18, 0x18, 0x18, 0x18, 0x7e, 0x3c, 0x18, 0],
        Action::Sprint => [0x18, 0x3c, 0x66, 0, 0x18, 0x3c, 0x66, 0],
        Action::Attack => [0x7e, 0x0c, 0x1a, 0x30, 0x60, 0xc0, 0, 0],
        Action::Use => [0x7e, 0x42, 0x5a, 0x7e, 0x5a, 0x42, 0x7e, 0],
        Action::Inventory => [0x3c, 0x24, 0x7e, 0x42, 0x7e, 0x42, 0x7e, 0],
        Action::Pause => [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0],
        Action::Drop => [0x18, 0x18, 0x18, 0x7e, 0x3c, 0x18, 0, 0x7e],
        Action::Previous | Action::Left => [0x08, 0x18, 0x38, 0x78, 0x38, 0x18, 0x08, 0],
        Action::Next | Action::Right => [0x10, 0x18, 0x1c, 0x1e, 0x1c, 0x18, 0x10, 0],
        _ => [0x66, 0x7e, 0x18, 0x5a, 0x18, 0x7e, 0x66, 0],
    };
    let pixel = size / 8.;
    for (row, bits) in rows.into_iter().enumerate() {
        for col in 0..8 {
            if bits & (1 << (7 - col)) != 0 {
                batch.fill(
                    atlas,
                    x - size / 2. + col as f32 * pixel,
                    y - size / 2. + row as f32 * pixel,
                    pixel,
                    pixel,
                    color,
                );
            }
        }
    }
}

fn touch_sprite(
    batch: &mut HudBatch,
    atlas: &horizon_client::hud::HudAtlas,
    name: &str,
    c: [f32; 2],
    r: f32,
    alpha: u32,
) -> bool {
    let key = format!("starling:gui/sprites/touch/{name}");
    if !atlas.sprites.contains_key(&key) {
        return false;
    }
    batch.sprite_scaled(
        atlas,
        &key,
        c[0] - r,
        c[1] - r,
        2. * r,
        2. * r,
        (alpha << 24) | 0xffffff,
    );
    true
}
