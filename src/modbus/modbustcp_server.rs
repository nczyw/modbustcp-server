use std::future;
use tokio_modbus::{
    prelude::*,
    server::Service,
};

use std::net::SocketAddr;

use crate::modbus::share_data::ShareDataRef;
pub struct ModbusTcpServer {
    share_data: ShareDataRef,
    socket_addr: SocketAddr,        
}
impl Drop for ModbusTcpServer {
    fn drop(&mut self) {
        println!("Client disconected :{}", self.socket_addr);
    }

}

impl ModbusTcpServer {
    pub fn new(
        share_data: ShareDataRef,
        socket_addr: SocketAddr,
    ) -> Self {
        Self {
            share_data: share_data,
            socket_addr: socket_addr,
        }
    }

    /// Read coils
    fn read_coil(
        &self,
        addr: u16,
        cnt: u16,
    ) -> Result<Vec<bool>, ExceptionCode> {
        
        let share_data = self.share_data.read().map_err(|_|ExceptionCode::ServerDeviceFailure)?;
        if addr < share_data.coils_offset {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        let addr = addr - share_data.coils_offset;
        if addr + cnt > share_data.coils.len() as u16  {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        Ok(share_data.coils[addr as usize..(addr + cnt) as usize].to_vec())
    }
    /// Write single coil
    fn write_single_coil(
        &self,
        addr: u16,
        value: bool,
    ) -> Result<(), ExceptionCode> {
        let mut share_data = self.share_data.write().map_err(|_|ExceptionCode::ServerDeviceFailure)?;
        if addr < share_data.coils_offset {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        let addr = addr  - share_data.coils_offset;
        if addr >= share_data.coils.len() as u16 {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        share_data.coils[addr as usize] = value;
        share_data.refresh_ui();
        Ok(())
    }

    /// Write multiple coils
    fn write_multiple_coils(
        &self,
        addr: u16,
        values: &[bool],
    ) -> Result<(), ExceptionCode> {
        let mut share_data = self.share_data.write().map_err(|_|ExceptionCode::ServerDeviceFailure)?;
        if addr < share_data.coils_offset {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        let addr = addr  - share_data.coils_offset;
        let start = addr as usize;
        let end = start
            .checked_add(values.len())
            .ok_or(ExceptionCode::IllegalDataAddress)?;
        if start >= share_data.coils.len() || end > share_data.coils.len() {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        
        share_data.coils[start..end].copy_from_slice(&values);
        share_data.refresh_ui();
        Ok(())
    }

    /// Read discrete inputs
    fn read_discrete_inputs(
        &self,
        addr: u16,
        cnt: u16,
    ) -> Result<Vec<bool>, ExceptionCode> {
        let share_data = self.share_data.read().map_err(|_|ExceptionCode::ServerDeviceFailure)?;
        if addr < share_data.discrete_inputs_offset {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        let addr = addr  - share_data.discrete_inputs_offset;
        let start = addr as usize;
        let end = start + cnt as usize;
        if end > share_data.discrete_inputs.len() {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        Ok(share_data.discrete_inputs[start..end].to_vec())
    }

    /// Read input registers
    fn read_input_registers(
        &self,
        addr: u16,
        cnt: u16,
    ) -> Result<Vec<u16>, ExceptionCode> {
        let share_data = self.share_data.read().map_err(|_|ExceptionCode::ServerDeviceFailure)?;
        if addr < share_data.input_registers_offset {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        let addr = addr  - share_data.input_registers_offset;
        let start = addr as usize;
        let end = start + cnt as usize;
        if end > share_data.input_registers.len() {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        Ok(share_data.input_registers[start..end].to_vec())
    }
    /// Read holding registers
    fn read_holding_registers(
        &self,
        addr: u16,
        cnt: u16,
    ) -> Result<Vec<u16>, ExceptionCode> {
        let share_data = self.share_data.read().map_err(|_|ExceptionCode::ServerDeviceFailure)?;
        if addr < share_data.holding_registers_offset {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        let addr = addr  - share_data.holding_registers_offset;
        let start = addr as usize;
        let end = start + cnt as usize;
        if end > share_data.holding_registers.len() {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        Ok(share_data.holding_registers[start..end].to_vec())
    }
    /// Write single holding register
    fn write_single_holding_register(
        &self,
        addr: u16,
        value: u16,
    ) -> Result<(), ExceptionCode> {
        let mut share_data = self.share_data.write().map_err(|_|ExceptionCode::ServerDeviceFailure)?;
        if addr < share_data.holding_registers_offset {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        let addr = addr  - share_data.holding_registers_offset;
        let index = addr as usize;
        if index >= share_data.holding_registers.len() {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        share_data.holding_registers[index] = value;
        share_data.refresh_ui();
        Ok(())
    }

    /// Write multiple holding registers
    fn write_multiple_holding_registers(
        &self,
        addr: u16,
        values: &[u16],
    ) -> Result<(), ExceptionCode> {
        let mut share_data = self.share_data.write().map_err(|_|ExceptionCode::ServerDeviceFailure)?;
        if addr < share_data.holding_registers_offset {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        let addr = addr  - share_data.holding_registers_offset;
        let start = addr as usize;
        let end = start
            .checked_add(values.len())
            .ok_or(ExceptionCode::IllegalDataAddress)?;
        if start >= share_data.holding_registers.len() || end > share_data.holding_registers.len() {
            return Err(ExceptionCode::IllegalDataAddress);
        }
        share_data.holding_registers[start..end].copy_from_slice(values);
        share_data.refresh_ui();
        Ok(())
    }
}

impl Service for ModbusTcpServer {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = future::Ready<Result<Self::Response, Self::Exception>>;
    fn call(&self, req: Self::Request) -> Self::Future {
        let res = match req {
            Request::ReadCoils(addr, cnt ) => {
                self.read_coil(addr, cnt)
                    .map(Response::ReadCoils)
            }
            Request::WriteSingleCoil(addr, value) => {
                self.write_single_coil(addr, value)
                    .map(|_| Response::WriteSingleCoil(addr, value))
            }
            Request::WriteMultipleCoils(addr, values) => {
                self.write_multiple_coils(addr, values.as_ref())
                    .map(|_| Response::WriteMultipleCoils(addr, values.len() as u16))
            }
            Request::ReadDiscreteInputs(addr, cnt) => {
                self.read_discrete_inputs(addr, cnt)
                    .map(Response::ReadDiscreteInputs)
            }
            Request::ReadInputRegisters(addr, cnt) => {
                self.read_input_registers(addr, cnt)
                    .map(Response::ReadInputRegisters)
            }
            Request::ReadHoldingRegisters(addr, cnt) => {
                self.read_holding_registers(addr, cnt)
                    .map(Response::ReadHoldingRegisters)
            },
            Request::WriteSingleRegister(addr, value) => {
                self.write_single_holding_register(addr, value)
                    .map(|_| Response::WriteSingleRegister(addr, value))
            },
            Request::WriteMultipleRegisters(addr, values) => {
                self.write_multiple_holding_registers(addr, values.as_ref())
                    .map(|_| Response::WriteMultipleRegisters(addr, values.len() as u16))
            },
            _ => {
                println!(
                    "SERVER: Exception::IllegalFunction - Unimplemented function code in request: {req:?}"
                );
                Err(ExceptionCode::IllegalFunction)
            }
        };
        future::ready(res)
    }
}