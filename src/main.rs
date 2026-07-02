#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::net::SocketAddr;
use eframe::NativeOptions;
use tokio::{net::TcpListener};
use tokio_modbus::{
    server::tcp::{Server, accept_tcp_connection}
};
use std::sync::{Arc, RwLock};
use clap::Parser;

mod modbus;
use crate::modbus::{modbustcp_server::ModbusTcpServer};
use crate::modbus::share_data::{ShareData, ShareDataRef};

mod ui;
use crate::ui::app_ui::AppUi;
/// Adjust the UI display scale
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// UI display scale factor (e.g., 1.0, 1.5, 2.0)
    #[arg(short = 's', long = "scale", default_value_t = 1.0)]
    scale: f32,
}



fn main() -> eframe::Result<()>{
    let args = Args::parse();
    let share_data = Arc::new(RwLock::new(ShareData::new()));
    {
        let share_data_clone = share_data.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                if let Err(e) = server_context(share_data_clone).await {
                    eprintln!("Modbus Server Error: {e}");
                }
            });
        });
    }
    let nativeoptions = eframe::NativeOptions{
        run_and_return: true,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(format!("ModbusTcp-Server  v{} | {}" , env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS")))

            .with_inner_size([800.0, 570.0]),
        ..NativeOptions::default()
    };
    // run gui
    eframe::run_native(
        "ModbusTcp-Server", 
        nativeoptions, 
        Box::new(|cc| {
            let fonts = load_fonts_from_dir("fonts");
            cc.egui_ctx.set_fonts(fonts);
            {
                cc.egui_ctx.set_theme(eframe::egui::Theme::Dark);
                let mut data = share_data.write().expect("'share_data': RwLock poisoned");
                data.ctx = Some(cc.egui_ctx.clone());
            }
            Ok(Box::new(AppUi::new(share_data.clone(), args.scale)))
        }),
    )
}

async fn server_context(
    share_data: ShareDataRef,
) -> anyhow::Result<()> {
    loop {
        // Wait for the startup signal to begin listening
        let notify = {
            let data = share_data.read().expect("'share_data': RwLock poisoned");
            data.change_conection_state.clone()
        };      // The lock on share_data must be released quickly after acquisition, so use this scope to handle it.
        notify.notified().await;
        
        let socket_addr = {
            let data = share_data.read().expect("'share_data': RwLock poisoned");
            let socket_addr = format!("{}:{}", data.address, data.port)
                .parse::<SocketAddr>()
                .unwrap();

            /*
            println!("socket_addr: {}, coil_count: {}, discrete_inputs_count: {}, input_registers_count: {}, holding_registers_count: {}",
                socket_addr,
                data.coil_count,
                data.discrete_inputs_count,
                data.input_registers_count,
                data.holding_registers_count,
            );
            */
            socket_addr
        };
        
        let listener = match TcpListener::bind(socket_addr).await {
            Ok(listener) => {
                let mut data = share_data.write().expect("'share_data': RwLock poisoned");
                data.is_running = true;
                data.send_error(None);
                listener
            }
            Err(e) => {
                let err_msg = format!("Error binding to {socket_addr}: {e}");
                eprintln!("{err_msg}");
                let data = share_data.read().expect("'share_data': RwLock poisoned");
                data.send_error(Some(err_msg));
                continue;
            }
        };
        let server = Server::new(listener);
        
        let new_service = |socket_addr| {
            println!("Client connected: {socket_addr}");
            Ok(Some(ModbusTcpServer::new(
                share_data.clone(), 
                socket_addr
            )))
        };

        let on_connected = |stream, socket_addr| async move {
            let result = accept_tcp_connection(stream, socket_addr, new_service);
            result
        };
        let on_process_error = |err| {
            eprintln!("{err}");
        };
        tokio::select! {
            result = server.serve(&on_connected, on_process_error) => {
                if let Err(err) = result {
                    let mut data = share_data.write().expect("'share_data': RwLock poisoned");
                    data.is_running = false;
                    let msg = format!("Server error: {err}");
                    eprintln!("{}", msg);
                    data.send_error(Some(msg));
                }
            }
            _ = async {
                let notify = {
                    let data = share_data.read().expect("'share_data': RwLock poisoned");
                    data.change_conection_state.clone()
                };
                notify.notified().await;
            } => {
                let mut data = share_data.write().expect("'share_data': RwLock poisoned");
                data.is_running = false;
                println!("Server stopped");
            }
        }
    }
}

fn load_fonts_from_dir(dir: &str) -> eframe::egui::FontDefinitions {
    let mut fonts = eframe::egui::FontDefinitions::default();
    let default_font = include_bytes!("../fonts/AlibabaPuHuiTi-3-55-Regular.ttf");
    let emoji_font = include_bytes!("../fonts/NotoEmoji-VariableFont_wght.ttf");
    fonts.font_data.insert("AlibabaPuHuiTi".to_owned(), Arc::new(eframe::egui::FontData::from_static(default_font)));
    fonts.font_data.insert("NotoEmoji".to_owned(), Arc::new(eframe::egui::FontData::from_static(emoji_font)));
    
    let mut family_order = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "ttf" || ext == "otf" {
                        let font_name = path.file_stem().unwrap().to_string_lossy().to_string();
                        if font_name == "AlibabaPuHuiTi-3-55-Regular" {
                            continue;
                        }
                        let data = std::fs::read(&path).unwrap();
                        fonts.font_data.insert(font_name.clone(), Arc::new(eframe::egui::FontData::from_owned(data)));
                        family_order.push(font_name);
                    }
                }
            }
        }
    }
    
    family_order.sort();
    
    let proportional = fonts.families.get_mut(&eframe::egui::FontFamily::Proportional).unwrap();
    proportional.insert(1, "NotoEmoji".to_owned());
    proportional.insert(0, "AlibabaPuHuiTi".to_owned());
    
    for name in family_order {
        proportional.push(name);
    }
    
    fonts
}