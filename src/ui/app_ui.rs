use core::fmt;
use eframe::egui;
use tokio::sync::watch;
use egui_extras::{TableBuilder, Column};

use crate::modbus::share_data::{ShareDataRef, RegType, DisplayFormat};



#[derive(Debug, Clone, Copy)]
enum DialogTarget {
    InputRegister(usize),
    HoldingRegister(usize),
}

impl fmt::Display for DialogTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DialogTarget::InputRegister(i) => write!(f, "InputRegister({})", i),
            DialogTarget::HoldingRegister(i) => write!(f, "HoldingRegister({})", i),
        }
    }
}

/// Open the display format modification dialog
struct DialogFormat {
    target: DialogTarget,
    temp_reg_type: RegType,
    temp_display_format: DisplayFormat,
    open_pos: egui::Pos2,
}

/// Edit value dialog
struct DialogValueEdit {
    target: DialogTarget,
    temp_value: String,
    open_pos: egui::Pos2,
    is_first_open: bool,          
}

pub struct AppUi {
    scale_factor: f32,          // Scale factor
    skin_dark: bool,            // Dark skin
    share_data: ShareDataRef,   // Shared data
    error_rx: watch::Receiver<Option<String>>,  // Receive error message
    max_len: usize,             // Max data size
    last_table_width : f32,     // Previous table width, used to check if a reset is needed
    
    dialog_format: Option<DialogFormat>,    // Modify display format dialog
    dialog_value_edit: Option<DialogValueEdit>,   // Modify value dialog

    
}

impl AppUi {
    pub fn new(
        share_data: ShareDataRef,
        scale_factor: f32,
    ) -> Self {
        let error_rx = share_data.read().unwrap().error_msg.subscribe();
        let share_data_clone = share_data.clone();
        let data = share_data_clone.read().unwrap();
        let max_len = (data.coil_count as usize)
            .max(data.discrete_inputs_count as usize)
            .max(data.input_registers_count as usize)
            .max(data.holding_registers_count as usize);
        Self {
            scale_factor: scale_factor,
            skin_dark: true,
            share_data: share_data,
            error_rx: error_rx,
            max_len: max_len,
            last_table_width: 0.0,
            dialog_format: None,
            dialog_value_edit: None,
        }
    }

    fn apply_dialog(&mut self, dialog: &DialogFormat) {
        let mut data = self.share_data.write().unwrap();
        match dialog.target {
            DialogTarget::InputRegister(i) => {
                data.write_reg_type(true, i, dialog.temp_reg_type);
                data.input_registers_format[i] = Some(dialog.temp_display_format);
            }
            DialogTarget::HoldingRegister(i) => {
                data.write_reg_type(false, i, dialog.temp_reg_type);
                data.holding_registers_format[i] = Some(dialog.temp_display_format);
            }
        }
    }
}

impl eframe::App for AppUi {
    fn ui(
        &mut self, 
        ui: &mut eframe::egui::Ui, 
        _frame: &mut eframe::Frame
    ) {
        
        egui::Panel::bottom("status_bar")
            .resizable(false)
            .show_separator_line(true)
            .max_size(24.0)
            .show(ui, |ui| {
                let mut data = self.share_data.write().unwrap();
                let is_running = data.is_running;
                let status_text = if is_running { "Disconnect" } else { "Connect" };
                let dot_color = if is_running {
                    egui::Color32::from_rgb(0, 180, 0)   // Dark green
                } else {
                    egui::Color32::from_rgb(180, 0, 0)   // Dark red
                };
                
                ui.horizontal(|ui| {
                    ui.colored_label(dot_color, "●");
                    if ui.add(
                        egui::Button::new(status_text).fill(egui::Color32::TRANSPARENT)
                    ).clicked() {
                        data.change_conection_state.notify_one();
                    }
                    ui.checkbox(&mut data.word_swap, "word-swap");
                    ui.checkbox(&mut data.byte_swap, "byte-swap");
                    
                    ui.add(egui::Separator::default().vertical());
                    if self.error_rx.has_changed().unwrap_or(false) {
                        if let Some(err) = self.error_rx.borrow().clone() {
                            ui.colored_label(egui::Color32::RED, err);
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let theme_text = if self.skin_dark { "🌙" } else { "☀️" };
                        if ui.button(theme_text).clicked() {
                            self.skin_dark = !self.skin_dark; 
                            if self.skin_dark {
                                ui.ctx().set_theme(eframe::egui::Theme::Dark);
                            } else {
                                ui.ctx().set_theme(eframe::egui::Theme::Light);
                            }
                        }
                        let fps = ui.ctx().input(|i| i.unstable_dt) * 1000.0;
                        ui.label(format!("{:.1}ms", fps));
                    });
                });
        });
        
        egui::CentralPanel::default().show(ui, |ui| {
            let mut data = self.share_data.write().unwrap();
            ui.vertical_centered(|ui| {
                ui.heading("Modbus TCP Server");
            });
            ui.add_space(10.0);
            egui::ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("IP:");
                    egui::ComboBox::from_id_salt("server_address")
                        .selected_text(&data.address)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(data.address == "127.0.0.1", "127.0.0.1").clicked() {
                                data.address = "127.0.0.1".to_string();
                            }
                            if ui.selectable_label(data.address == "0.0.0.0", "0.0.0.0").clicked() {
                                data.address = "0.0.0.0".to_string();
                            }
                        });

                    ui.add(
                        egui::DragValue::new(&mut data.port)
                            .range(1..=65535)
                            .speed(0)
                            .prefix("Port: ")
                            
                    );
                });
                ui.add_space(10.0);
                let current_width = ui.available_width();
                let table = TableBuilder::new(ui)
                    .id_salt("table")
                    .auto_shrink([true; 2])
                    .column(Column::exact(100.0))
                    .column(Column::exact(150.0))
                    .column(Column::remainder().at_least(120.0).resizable(true))
                    .column(Column::remainder().at_least(150.0));
                if (current_width - self.last_table_width).abs() > 0.01 {
                    table.reset();
                }
                self.last_table_width = current_width;
                table.header(50.0, |mut row| {
                    row.col(|ui| { 
                        let mut tmp = data.coil_count;
                        let response = ui.add(
                            egui::DragValue::new(&mut tmp)
                                .range(0..=65535)
                                .speed(0)
                                .prefix("Coils: ")
                        );
                        if response.changed() && response.lost_focus() {
                            data.coil_count = tmp;
                            data.reset_coils();
                            self.max_len = (data.coil_count as usize)
                                .max(data.discrete_inputs_count as usize)
                                .max(data.input_registers_count as usize)
                                .max(data.holding_registers_count as usize);
                        }
                        tmp = data.coils_offset;
                        let response = ui.add(
                            egui::DragValue::new(&mut tmp)
                                .range(0..=65535)
                                .speed(0)
                                .prefix("Offsets: ")
                        );
                        if response.changed() && response.lost_focus() {
                            data.coils_offset = tmp;
                        }
                    });
                    row.col(|ui| { 
                        let mut tmp = data.discrete_inputs_count;
                        let response = ui.add(
                            egui::DragValue::new(&mut tmp)
                                .range(0..=65535)
                                .speed(0)
                                .prefix("Discrete Inputs: ")
                        );
                        if response.changed() && response.lost_focus() {
                            data.discrete_inputs_count = tmp;
                            data.reset_discrete_inputs();
                            self.max_len = (data.coil_count as usize)
                                .max(data.discrete_inputs_count as usize)
                                .max(data.input_registers_count as usize)
                                .max(data.holding_registers_count as usize);
                        }
                        tmp = data.discrete_inputs_offset;
                        let response = ui.add(
                            egui::DragValue::new(&mut tmp)
                                .range(0..=65535)
                                .speed(0)
                                .prefix("Offsets: ")
                        );
                        if response.changed() && response.lost_focus() {
                            data.discrete_inputs_offset = tmp;
                        }
                    });
                    row.col(|ui| { 
                        let mut tmp = data.input_registers_count;
                        let response = ui.add(
                            egui::DragValue::new(&mut tmp)
                                .range(0..=65535)
                                .speed(0)
                                .prefix("Input Registers Count: ")
                        );
                        if response.changed() && response.lost_focus() {
                            data.input_registers_count = tmp;
                            data.reset_input_registers();
                            self.max_len = (data.coil_count as usize)
                                .max(data.discrete_inputs_count as usize)
                                .max(data.input_registers_count as usize)
                                .max(data.holding_registers_count as usize);
                        }
                        tmp = data.input_registers_offset;
                        let response = ui.add(
                            egui::DragValue::new(&mut tmp)
                                .range(0..=65535)
                                .speed(0)
                                .prefix("Offsets: ")
                        );
                        if response.changed() && response.lost_focus() {
                            data.input_registers_offset = tmp;
                        }
                    });
                    row.col(|ui| { 
                        let mut tmp = data.holding_registers_count;
                        let response = ui.add(
                            egui::DragValue::new(&mut tmp)
                                .range(0..=65535)
                                .speed(0)
                                .prefix("Holding Registers: ")
                        );
                        if response.changed() && response.lost_focus() {
                            data.holding_registers_count = tmp;
                            data.reset_holding_registers();
                            self.max_len = (data.coil_count as usize)
                                .max(data.discrete_inputs_count as usize)
                                .max(data.input_registers_count as usize)
                                .max(data.holding_registers_count as usize);
                        }
                        tmp = data.holding_registers_offset;
                        let response = ui.add(
                            egui::DragValue::new(&mut tmp)
                                .range(0..=65535)
                                .speed(0)
                                .prefix("Offsets: ")
                        );
                        if response.changed() && response.lost_focus() {
                            data.holding_registers_offset = tmp;
                        }
                    });
                })
                .body(|body| {
                    
                    body.rows(20.0, self.max_len, |mut row| {
                        let i = row.index();
                        row.col(|ui| {
                            let coils_index = i + data.coils_offset as usize;
                            if let Some(val) = data.coils.get_mut(i) {
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                                    ui.checkbox(val, egui::RichText::new(format!("{}", coils_index)).monospace());
                                });
                            }
                        });
                        row.col(|ui| {
                            let discrete_inputs_index = i + data.discrete_inputs_offset as usize;
                            if let Some(val) = data.discrete_inputs.get_mut(i) {
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                                    ui.checkbox(val, egui::RichText::new(format!("{}", discrete_inputs_index)).monospace());
                                });
                            }
                        });
                        
                        row.col(|ui| {
                            let reg_config = data.input_registers_config
                                .get(i)
                                .and_then(|v| *v);
                            let fmt = data.input_registers_format
                                .get(i)
                                .and_then(|v| *v);

                            let input_register_index = i + data.input_registers_offset as usize;
                            if let Some(text) = data.read_register(true, i) {
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                                    ui.label(egui::RichText::new(format!("{:3}", input_register_index)).monospace());

                                    let resp = ui.add_sized(
                                        [ui.available_width(), ui.spacing().interact_size.y],
                                        egui::Button::new(&text),
                                    );
                                    if resp.clicked() {
                                        self.dialog_value_edit = Some(
                                            DialogValueEdit {
                                                target: DialogTarget::InputRegister(i),
                                                temp_value: text,
                                                open_pos: resp.rect.center(),
                                                is_first_open: true,
                                            }
                                        )
                                    }
                                    if resp.secondary_clicked() {
                                        let pos = self.dialog_format
                                            .as_ref()
                                            .map(|d| d.open_pos)
                                            .unwrap_or(resp.rect.center());
                                        self.dialog_format = Some(
                                            DialogFormat {
                                                target: DialogTarget::InputRegister(i),
                                                temp_reg_type: reg_config.unwrap_or(RegType::I16),
                                                temp_display_format: fmt.unwrap(),
                                                open_pos: pos,
                                            }
                                        )
                                    }
                                });
                            }
                        });
                        
                        row.col(|ui| {
                            let reg_config = data.holding_registers_config
                                .get(i)
                                .and_then(|v| *v);
                            let fmt = data.holding_registers_format
                                .get(i)
                                .and_then(|v| *v);

                            let holding_register_index = i + data.holding_registers_offset as usize;
                            if let Some(text) = data.read_register(false, i) {
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                                    ui.label(egui::RichText::new(format!("{:3}", holding_register_index)).monospace());

                                    let resp = ui.add_sized(
                                        [ui.available_width(), ui.spacing().interact_size.y],
                                        egui::Button::new(&text),
                                    );
                                    
                                    if resp.clicked() {
                                        self.dialog_value_edit = Some(
                                            DialogValueEdit {
                                                target: DialogTarget::HoldingRegister(i),
                                                temp_value: text,
                                                open_pos: resp.rect.center(),
                                                is_first_open: true,
                                            }
                                        )
                                    }
                                    if resp.secondary_clicked() {
                                        let pos = self.dialog_format
                                            .as_ref()
                                            .map(|d| d.open_pos)
                                            .unwrap_or(resp.rect.center());
                                        self.dialog_format = Some(
                                            DialogFormat {
                                                target: DialogTarget::HoldingRegister(i),
                                                temp_reg_type: reg_config.unwrap_or(RegType::I16),
                                                temp_display_format: fmt.unwrap(),
                                                open_pos: pos,
                                            }
                                        )
                                    }
                                });
                            } 
                        });
                    });
                });
            });
        });
        if let Some(mut dialog) = self.dialog_format.take() {

            let mut should_close = false;
            let mut should_apply = false;
            // Fullscreen overlay
            let screen_rect = ui.ctx().content_rect();

            let bg_response = egui::Area::new(egui::Id::new("dialog_mask"))
                .order(egui::Order::Background)
                .fixed_pos(screen_rect.min)
                .show(ui.ctx(), |ui| {
                
                    let rect = egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        screen_rect.size(),
                    );
                
                    // Semi-transparent background (optional)
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        egui::Color32::from_black_alpha(64),
                    );
                
                    ui.allocate_rect(
                        rect,
                        egui::Sense::click(),
                    )
                })
                .inner;
            
            // Window
            
            let mut window_rect = None;
            let window_title = format!("{} Settings", dialog.target);
            let response = egui::Window::new(window_title)
                .id(egui::Id::new("reg_dialog"))
                .resizable(false)
                .movable(true)
                .default_pos(dialog.open_pos)
                .show(ui.ctx(), |ui| {
            
                    ui.label("Data Type:");
                
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut dialog.temp_reg_type, RegType::U16, "U16");
                        ui.selectable_value(&mut dialog.temp_reg_type, RegType::I16, "I16");
                        ui.selectable_value(&mut dialog.temp_reg_type, RegType::U32, "U32");
                        ui.selectable_value(&mut dialog.temp_reg_type, RegType::I32, "I32");
                        ui.selectable_value(&mut dialog.temp_reg_type, RegType::F32, "F32");
                        ui.selectable_value(&mut dialog.temp_reg_type, RegType::U64, "U64");
                        ui.selectable_value(&mut dialog.temp_reg_type, RegType::I64, "I64");
                        ui.selectable_value(&mut dialog.temp_reg_type, RegType::F64, "F64");
                    });
                
                    ui.separator();
                
                    // ================= Format =================
                
                    ui.label("Display Format:");
                
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut dialog.temp_display_format,
                            DisplayFormat::Decimal,
                            "DEC",
                        );
                    
                        ui.selectable_value(
                            &mut dialog.temp_display_format,
                            DisplayFormat::Hexadecimal,
                            "HEX",
                        );
                    
                        ui.selectable_value(
                            &mut dialog.temp_display_format,
                            DisplayFormat::Binary,
                            "BIN",
                        );
                    
                        ui.selectable_value(
                            &mut dialog.temp_display_format,
                            DisplayFormat::Octal,
                            "OCT",
                        );
                    });
                
                    ui.separator();
                
                    ui.horizontal(|ui| {
                    
                        if ui.button("OK").clicked() {
                            should_apply = true;
                            should_close = true;
                        }
                    
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                    });
                    // Press Enter to click OK
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        should_apply = true;
                        should_close = true;
                    }
                    // Press ESC to click Cancel
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        should_close = true;
                    }
                });
            
            // Window position
            
            if let Some(resp) = response {
                window_rect = Some(resp.response.rect);
            
                // Remember position after dragging
                dialog.open_pos = resp.response.rect.min;
            }
        
            // Close on overlay click
        
            if bg_response.clicked() {
            
                let mouse_pos =
                    ui.ctx().input(|i| i.pointer.interact_pos());
            
                if let (Some(rect), Some(pos)) =
                    (window_rect, mouse_pos)
                {
                    if !rect.contains(pos) {
                        should_close = true;
                    }
                }
            }
        
            // Apply
            if should_apply {
                self.apply_dialog(&dialog);
            }
        
            // Save current state
            if !should_close {
                self.dialog_format = Some(dialog);
            }
        }

        // Modify value
        if let Some(mut value_dialog) = self.dialog_value_edit.take() {
            
            
            let mut should_close = false;
            let mut should_apply =false;

            let screen_rect = ui.ctx().content_rect();

            let bg_response = egui::Area::new(egui::Id::new("dialog_mask"))
                .order(egui::Order::Background)
                .fixed_pos(screen_rect.min)
                .show(ui.ctx(), |ui| {
                
                    let rect = egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        screen_rect.size(),
                    );
                
                    
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        egui::Color32::from_black_alpha(64),
                    );
                
                    ui.allocate_rect(
                        rect,
                        egui::Sense::click(),
                    )
                })
                .inner;
            
            let mut window_rect = None;

            let window_title = format!("Edit {}", value_dialog.target);

            let response = egui::Window::new(window_title)
                .id(egui::Id::new("value_edit_dialog"))
                .resizable(false)
                .default_pos(value_dialog.open_pos)
                .show(ui.ctx(), |ui| {
                    ui.label("New Value:");
                    let edit_text = ui.text_edit_singleline(&mut value_dialog.temp_value);
                    if value_dialog.is_first_open {
                        value_dialog.is_first_open = false;
                        edit_text.request_focus();

                    }

                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        should_apply = true;
                        should_close = true;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        should_close = true;
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            should_apply = true;
                            should_close = true;
                        }
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                    });

                    
                    
                });
                
            if let Some(resp) = response {
                window_rect = Some(resp.response.rect);
            
                
                value_dialog.open_pos = resp.response.rect.min;
            }
        
           
        
            if bg_response.clicked() {
            
                let mouse_pos =
                    ui.ctx().input(|i| i.pointer.interact_pos());
            
                if let (Some(rect), Some(pos)) =
                    (window_rect, mouse_pos)
                {
                    if !rect.contains(pos) {
                        should_close = true;
                    }
                }
            }


            if should_apply {
                let mut data = self.share_data.write().unwrap();
                match value_dialog.target {
                    DialogTarget::InputRegister(index) => {
                        data.write_register(true, index, &value_dialog.temp_value);
                    }
                    DialogTarget::HoldingRegister(index) => {
                        data.write_register(false, index, &value_dialog.temp_value);
                    
                    }
                }
            }
            if !should_close {
                self.dialog_value_edit = Some(value_dialog);
            }
        }
    }
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let target_scale = ctx.pixels_per_point() * self.scale_factor;
        if (ctx.pixels_per_point() - target_scale).abs() > 0.01 {
            ctx.set_pixels_per_point(self.scale_factor);
        }
    }
    fn on_exit(&mut self) {
        let data = self.share_data.write().unwrap();
        data.change_conection_state.notify_one();
    }
}