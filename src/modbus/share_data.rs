use std::sync::{Arc, RwLock};
use tokio::sync::{Notify, watch};
use eframe::egui::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayFormat {
    Binary,
    Octal,
    #[default]
    Decimal,
    Hexadecimal,
}

impl DisplayFormat {
    pub fn format(
        &self,
        value: &DValue,
    ) -> String {
        match value {
            DValue::U16(v) => self.format_u64(*v as u64, 16),
            DValue::I16(v) => match self {
                DisplayFormat::Decimal => v.to_string(),
                _ => self.format_u64(*v as u16 as u64, 16),
            }

            DValue::U32(v) => self.format_u64(*v as u64, 32),
            DValue::I32(v) => match self {
                DisplayFormat::Decimal => v.to_string(),
                _ => self.format_u64(*v as u32 as u64, 32),
            }
            DValue::F32(v) => match self {
                DisplayFormat::Decimal => v.to_string(),
                _ => self.format_u64(v.to_bits() as u32 as u64, 32),
            }

            DValue::U64(v) => self.format_u64(*v, 64),
            DValue::I64(v) => match self {
                DisplayFormat::Decimal => v.to_string(),
                _ => self.format_u64(*v as u64, 64),
            }
            DValue::F64(v) => match self {
                DisplayFormat::Decimal => v.to_string(),
                _ => self.format_u64(v.to_bits(), 64),
            }
        }
    }

    fn format_u64(
        &self,
        value: u64,
        bits: usize,
    ) -> String {
        match self {
            DisplayFormat::Binary => {
                let s = format!("{value:b}");

                let width = bits.max(s.len());

                let padded = format!("{value:0width$b}");

                let groups: Vec<_> = padded
                    .as_bytes()
                    .chunks(4)
                    .map(|c| std::str::from_utf8(c).unwrap())
                    .collect();

                format!("0b_{}", groups.join("_"))
            }

            DisplayFormat::Octal => {
                format!("0o{value:o}")
            }

            DisplayFormat::Decimal => {
                value.to_string()
            }

            DisplayFormat::Hexadecimal => {
                let width = bits / 4;
                format!("0x{value:0width$X}")
            }
        }
    }

    pub fn parse(
        &self,
        text: &str,
        reg: &RegType,
    ) -> Option<DValue> {
        match reg {
            RegType::U16 => {
                Some(
                    DValue::U16(
                        self.parse_u64(text)? as u16
                    )
                )
            }
            RegType::I16 => {
                Some(
                    DValue::I16(
                        self.parse_i16(text)?
                    )
                )
            }
            RegType::U32 => {
                Some(
                    DValue::U32(
                        self.parse_u64(text)? as u32
                    )
                )
            }
            RegType::I32 => {
                Some(
                    DValue::I32(
                        self.parse_i32(text)?
                    )
                )
            }

            RegType::F32 => {
                Some(
                    DValue::F32(
                        self.parse_f32(text)?
                    )
                )
            }

            RegType::U64 => {
                Some(
                    DValue::U64(
                        self.parse_u64(text)?
                    )
                )
            }

            RegType::I64 => {
                Some(
                    DValue::I64(
                        self.parse_i64(text)?
                    )
                )
            }

            RegType::F64 => {
                Some(
                    DValue::F64(
                        self.parse_f64(text)?
                    )
                )
            }
           
        }
    }

    fn parse_i16(
        &self,
        text: &str,
    ) -> Option<i16> {
        match self {
            DisplayFormat::Decimal => {
                text.trim().parse::<i16>().ok()
            }
            _ => {
                Some(
                    self.parse_u64(text)?
                        as u16
                        as i16
                )
            }
        }
    }

    fn parse_i32(
        &self,
        text: &str,
    ) -> Option<i32> {
        match self {
            DisplayFormat::Decimal => {
                text.trim().parse::<i32>().ok()
            }
            _ => {
                Some(
                    self.parse_u64(text)?
                        as u32
                        as i32
                )
            }
        }
    }

    fn parse_f32(
        &self,
        text: &str,
    ) -> Option<f32> {
        match self {
            DisplayFormat::Decimal => {
                text.trim().parse::<f32>().ok()
            }
            _ => {
                Some(f32::from_bits(
                    self.parse_u64(text)? as u32
                ))
            }
        }
    }
    
    fn parse_i64(
        &self,
        text: &str,
    ) -> Option<i64> {
        match self {
            DisplayFormat::Decimal => {
                text.trim().parse::<i64>().ok()
            }
            _ => {
                Some(
                    self.parse_u64(text)?
                        as i64
                )
            }
        }
    }

    fn parse_u64(
        &self,
        text: &str,
    ) -> Option<u64> {
        let t = text
            .trim()
            .replace('_', "");

        match self {
            DisplayFormat::Binary => {
                let s = t
                    .strip_prefix("0b")
                    .unwrap_or(&t);

                u64::from_str_radix(s, 2).ok()
            }


            DisplayFormat::Octal => {
                let s = t
                    .strip_prefix("0o")
                    .unwrap_or(&t);

                u64::from_str_radix(s, 8).ok()
            }


            DisplayFormat::Decimal => {
                t.parse::<u64>().ok()
            }


            DisplayFormat::Hexadecimal => {
                let s = t
                    .strip_prefix("0x")
                    .unwrap_or(&t);

                u64::from_str_radix(s, 16).ok()
            }
        }
    }

    fn parse_f64(
        &self,
        text: &str,
    ) -> Option<f64> {
        match self {
            DisplayFormat::Decimal => {
                text.trim().parse::<f64>().ok()
            }
            _ => {
                Some(f64::from_bits(
                    self.parse_u64(text)?
                ))
            }
        }
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum RegType {
    U16,
    #[default]
    I16,
    U32,
    I32,
    F32,
    U64,
    I64,
    F64,
}

impl RegType {
    pub fn span(
        &self, 
    ) -> usize {
        match self {
            RegType::U16 | RegType::I16 => 1,
            RegType::U32 | RegType::I32 | RegType::F32 => 2,
            RegType::U64 | RegType::I64 | RegType::F64 => 4,
        }
    }
}

pub enum DValue {
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    U64(u64),
    I64(i64),
    F64(f64),
}

pub type ShareDataRef = Arc<RwLock<ShareData>>;
pub struct ShareData {
    pub ctx: Option<Context>,                        // Gui refresh
    pub is_running: bool,                            // Current running state
    pub change_conection_state: Arc<Notify>,         // Change connection state
    pub error_msg: watch::Sender<Option<String>>,    // Error message

    pub address: String,                              // Address
    pub port: u16,                                    // Port

    pub coil_count: u16,                              // Coil count
    pub coils_offset: u16,                            // Coils offset   
    pub coils: Vec<bool>,                               // Coils raw data    

    pub discrete_inputs_count: u16,                   // Discrete input count       
    pub discrete_inputs_offset: u16,                  // Discrete input offset        
    pub discrete_inputs: Vec<bool>,                   // Discrete input raw data    

    pub input_registers_count: u16,                   // Input register count
    pub input_registers_offset: u16,                  // Input register offset
    pub input_registers: Vec<u16>,                       // Input register raw data
    pub input_registers_config: Vec<Option<RegType>>,    // Input register type
    pub input_registers_format: Vec<Option<DisplayFormat>>,    // Input register display format

    pub holding_registers_count: u16,                 // Holding register count
    pub holding_registers_offset: u16,                // Holding register offset
    pub holding_registers: Vec<u16>,                     // Holding register raw data
    pub holding_registers_config: Vec<Option<RegType>>,  // Holding register type
    pub holding_registers_format: Vec<Option<DisplayFormat>>,  // Holding register display format
    pub word_swap: bool,                         // Swap words
    pub byte_swap: bool,                         // Swap bytes within word
}

impl ShareData {
    pub fn new(
    ) -> Self {
        let (error_tx, _error_rx) = watch::channel(None);
        Self {
            ctx: None,
            is_running: false,
            change_conection_state: Arc::new(Notify::new()),
            error_msg: error_tx,

            address: "127.0.0.1".to_string(),
            port: 502,

            coil_count: 16,
            coils_offset: 0,
            coils: vec![false; 16 as usize],

            discrete_inputs_count: 16,
            discrete_inputs_offset: 0,
            discrete_inputs: vec![false; 16 as usize],

            input_registers_count: 16,
            input_registers_offset: 0,
            input_registers: vec![0; 16 as usize],
            input_registers_config: vec![Some(RegType::default()); 16 as usize],
            input_registers_format: vec![Some(DisplayFormat::default()); 16 as usize],

            holding_registers_count: 16,
            holding_registers_offset: 0,
            holding_registers: vec![0; 16 as usize],
            holding_registers_config: vec![Some(RegType::default()); 16 as usize],
            holding_registers_format: vec![Some(DisplayFormat::default()); 16 as usize],
            word_swap: false,
            byte_swap: false,
        }
    }

    /// reset coils raw data
    pub fn reset_coils(
        &mut self,
    ) {
        self.coils.resize(self.coil_count as usize, Default::default());
    }

    /// reset discrete inputs raw data
    pub fn reset_discrete_inputs(
        &mut self,
    ) {
        self.discrete_inputs.resize(self.discrete_inputs_count as usize, Default::default());
    }

    /// reset input registers raw data
    pub fn reset_input_registers(
        &mut self,
    ) {
        self.input_registers.resize(self.input_registers_count as usize, Default::default());
        self.input_registers_config = vec![Some(RegType::default()); self.input_registers_count as usize];
        self.input_registers_format = vec![Some(DisplayFormat::default()); self.input_registers_count as usize];
    }

    /// reset holding registers raw data
    pub fn reset_holding_registers(
        &mut self,
    ) {
        self.holding_registers.resize(self.holding_registers_count as usize, Default::default());
        self.holding_registers_config = vec![Some(RegType::default()); self.input_registers_count as usize];
        self.holding_registers_format = vec![Some(DisplayFormat::default()); self.input_registers_count as usize];
    }

    pub fn refresh_ui(
        &self,
    ) {
        if let Some(ctx) = &self.ctx {
            ctx.request_repaint();
        }
    }
    pub fn send_error(
        &self,
        error: Option<String>,
    ) {
        let _ = self.error_msg.send(error);
        self.refresh_ui();
    }

    /// Read 16-bit raw data
    fn read_16_raw(
        &self,
        in_reg: bool,
        addr: usize,
    ) -> u16 {
        let a = if in_reg {
            self.input_registers[addr]
        } else {
            self.holding_registers[addr]
        };

        let a = if self.byte_swap {
            a.swap_bytes()
        } else {
            a
        };
        a
    }

    /// Read 32-bit raw data
    fn read_32_raw(
        &self,
        in_reg: bool,       
        addr: usize,
    ) -> u32 {
        let (a, b) = if in_reg {    
            (self.input_registers[addr], self.input_registers[addr + 1])  
        } else {
            (self.holding_registers[addr], self.holding_registers[addr + 1]) 
        };
        let (a, b) = if self.word_swap {    
            (b, a)
        } else {
            (a, b)
        };
        let (a, b) = if self.byte_swap {    
            (a.swap_bytes(), b.swap_bytes())
        } else {
            (a, b)
        };
        a as u32 | ((b as u32) << 16)
    }

    /// Read 64-bit raw data
    fn read_64_raw(
        &self,
        in_reg: bool,
        addr: usize,
    ) -> u64 {
        let (a, b, c, d) = if in_reg {    
            (self.input_registers[addr], self.input_registers[addr + 1] , self.input_registers[addr + 2], self.input_registers[addr + 3])
        } else {
            (self.holding_registers[addr], self.holding_registers[addr + 1], self.holding_registers[addr + 2], self.holding_registers[addr + 3])
        };
        let (a, b, c, d) = if self.word_swap {    
            (d, c, b, a)
        } else {
            (a, b, c, d)
        };
        let (a, b, c, d) = if self.byte_swap {    
            (a.swap_bytes(), b.swap_bytes(), c.swap_bytes(), d.swap_bytes())
        } else {
            (a, b, c, d)
        };
        a as u64 | ((b as u64) << 16) | ((c as u64) << 32) | ((d as u64) << 48)
    }

    /// Read registers
    pub fn read_register(
        &self,
        in_reg: bool,
        addr: usize,
    ) -> Option<String> {

        let (data, reg_type, fmt) = if in_reg {
            (
                &self.input_registers, 
                &self.input_registers_config,
                &self.input_registers_format,
            )
        } else {
            (
                &self.holding_registers, 
                &self.holding_registers_config,
                &self.holding_registers_format,
            )
        };
        
        let reg_type = reg_type.get(addr)?.as_ref()?;
        
        let fmt = fmt.get(addr)?.as_ref()?;

        if addr + reg_type.span() > data.len() {
            return None;
        }
        let dvalue = match reg_type {
            RegType::U16 => {
                DValue::U16(self.read_16_raw(in_reg, addr))
            }
            RegType::I16 => {
                DValue::I16(self.read_16_raw(in_reg, addr) as i16)
            }
            RegType::U32 => {
                DValue::U32(self.read_32_raw(in_reg, addr))
            }
            RegType::I32 => {
                DValue::I32(self.read_32_raw(in_reg, addr) as i32)
            }
            RegType::F32 => {
                DValue::F32(f32::from_bits(self.read_32_raw(in_reg, addr)))
            }
            RegType::U64 => {
                DValue::U64(self.read_64_raw(in_reg, addr))
            }
            RegType::I64 => {
                DValue::I64(self.read_64_raw(in_reg, addr) as i64)
            }
            RegType::F64 => {
                DValue::F64(f64::from_bits(self.read_64_raw(in_reg, addr)))
            }
        };
        Some(fmt.format(&dvalue))
    }

    /// Write 16-bit raw data
    fn write_16_raw(
        &mut self,
        in_reg: bool,
        addr: usize,
        value: u16,
    ) {
        let a = if self.byte_swap {
            value.swap_bytes()
        } else {
            value
        };
        if in_reg {
            self.input_registers[addr] = a;
        } else {
            self.holding_registers[addr] = a;
        }
    }
    /// Write 32-bit raw data
    fn write_32_raw(
        &mut self,
        in_reg: bool,
        addr: usize,
        value: u32,
    ) {
        let (a, b) = {
            (value as u16, (value >> 16) as u16)
        };
        let (a, b) = if self.byte_swap {        
            (a.swap_bytes(), b.swap_bytes())
        } else {
            (a, b)
        };
        let (a, b) = if self.word_swap {       
            (b, a)
        } else {
            (a, b)
        };

        if in_reg {
            self.input_registers[addr] = a;
            self.input_registers[addr + 1] = b;
            self.input_registers_config[addr + 1] = None;
        } else {
            self.holding_registers[addr] = a;
            self.holding_registers[addr + 1] = b;
            self.holding_registers_config[addr + 1] = None;
        }
    }
    /// Write 64-bit raw data
    fn write_64_raw(
        &mut self,
        in_reg: bool,
        addr: usize,
        value: u64,
    ) {
        let (a, b, c, d) = {
            (value as u16, (value >> 16) as u16, (value >> 32) as u16, (value >> 48) as u16)
        };

        let (a, b, c, d) = if self.byte_swap {              // 交换字中的字节
            (a.swap_bytes(), b.swap_bytes(), c.swap_bytes(), d.swap_bytes())
        } else {
            (a, b, c, d)
        };

        let (a, b, c, d) = if self.word_swap {
            (d, c, b, a)
        } else {
            (a, b, c, d)
        };

        if in_reg {
            self.input_registers[addr] = a;
            self.input_registers[addr + 1] = b;
            self.input_registers[addr + 2] = c;
            self.input_registers[addr + 3] = d;

            self.input_registers_config[addr + 1] = None;
            self.input_registers_config[addr + 2] = None;
            self.input_registers_config[addr + 3] = None;
        } else {
            self.holding_registers[addr] = a;
            self.holding_registers[addr + 1] = b;
            self.holding_registers[addr + 2] = c;
            self.holding_registers[addr + 3] = d;

            self.holding_registers_config[addr + 1] = None;
            self.holding_registers_config[addr + 2] = None;
            self.holding_registers_config[addr + 3] = None;
        }
    }

    /// Write registers
    pub fn write_register(
        &mut self,
        in_reg: bool,
        addr: usize,
        value: &String,
    ) -> bool {

        /*
        let offset_adrr = if in_reg {
            addr.checked_sub(self.input_registers_offset as usize)
        } else {
            addr.checked_sub(self.holding_registers_offset as usize)
        }.ok_or_else(|| return false).unwrap();

        let addr = addr - offset_adrr;
        */
        let (reg_type, format) = {
            let config = if in_reg {
                &self.input_registers_config
            } else {
                &self.holding_registers_config
            };
            let formats = if in_reg {
                &self.input_registers_format
            } else {
                &self.holding_registers_format
            };
            let Some(ty) = config
                .get(addr)
                .and_then(|v| v.as_ref())
            else {
                return false;
            };
            let Some(format) = formats
                .get(addr)
                .and_then(|v| v.as_ref())
            else {
                return false;
            };
            (*ty, *format)
        };

        // Check data range
        let len = if in_reg {
            self.input_registers.len()
        } else {
            self.holding_registers.len()
        };

        if addr + reg_type.span() > len {
            return false;
        }

        // Convert String to DValue
        let Some(value) = format.parse(
            value, 
            &reg_type
        ) else {
            return false;
        };

        match value {
            DValue::U16(v) => {
                self.write_16_raw(in_reg, addr, v);
            }
            DValue::I16(v) => {
                self.write_16_raw(in_reg, addr, v as u16);
            }
            DValue::U32(v) => {
                self.write_32_raw(in_reg, addr, v);
            }
            DValue::I32(v) => {
                self.write_32_raw(in_reg, addr, v as u32);
            }
            DValue::F32(v) => {
                self.write_32_raw(in_reg, addr, v.to_bits());
            }
            DValue::U64(v) => {
                self.write_64_raw(in_reg, addr, v);
            }
            DValue::I64(v) => {
                self.write_64_raw(in_reg, addr, v as u64);
            }
            DValue::F64(v) => {
                self.write_64_raw(in_reg, addr, v.to_bits());
            }
        }
        
        true
    }
    
    pub fn write_reg_type(
        &mut self,
        in_reg: bool,
        addr: usize,
        reg_type: RegType,
    ) -> bool {
        let config = if in_reg {
            &mut self.input_registers_config
        } else {
            &mut self.holding_registers_config
        };
        // Prevent out-of-bounds
        let span = reg_type.span();
        if addr + span > config.len() {
            return false;
        }
        // Get old configuration
        let old_cfg = match config.get(addr).unwrap() {
            Some(v) => {
                v.clone()
            }
            None => {
                return false;
            }
        };
        // Update configuration
        config[addr] = Some(reg_type);
        let old_span = old_cfg.span();
        // If the new span is smaller than the old one, restore the remaining to i16; if the new span is larger, clear the rest
        if span < old_span {
            for i in addr + span..addr + old_span {
                config[i] = Some(RegType::default());
            }
        } else if span > old_span {
            for i in addr + old_span..addr + span {
                config[i] = None;
            }
        }
        
        true
    }
}