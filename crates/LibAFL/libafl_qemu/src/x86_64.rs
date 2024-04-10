use std::{mem::size_of, sync::OnceLock};

use capstone::arch::BuildsCapstone;
use enum_map::{enum_map, EnumMap};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(feature = "python")]
use pyo3::prelude::*;
pub use strum_macros::EnumIter;
pub use syscall_numbers::x86_64::*;

use crate::{sync_backdoor::SyncBackdoorArgs, CallingConvention};

#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone, Copy, EnumIter)]
#[repr(i32)]
pub enum Regs {
    Rax = 0,
    Rbx = 1,
    Rcx = 2,
    Rdx = 3,
    Rsi = 4,
    Rdi = 5,
    Rbp = 6,
    Rsp = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
    Rip = 16,
    Rflags = 17,
}

static SYNC_BACKDOOR_ARCH_REGS: OnceLock<EnumMap<SyncBackdoorArgs, Regs>> = OnceLock::new();

pub fn get_sync_backdoor_arch_regs() -> &'static EnumMap<SyncBackdoorArgs, Regs> {
    SYNC_BACKDOOR_ARCH_REGS.get_or_init(|| {
        enum_map! {
            SyncBackdoorArgs::Ret  => Regs::Rax,
            SyncBackdoorArgs::Cmd  => Regs::Rax,
            SyncBackdoorArgs::Arg1 => Regs::Rdi,
            SyncBackdoorArgs::Arg2 => Regs::Rsi,
            SyncBackdoorArgs::Arg3 => Regs::Rdx,
            SyncBackdoorArgs::Arg4 => Regs::R10,
            SyncBackdoorArgs::Arg5 => Regs::R8,
            SyncBackdoorArgs::Arg6 => Regs::R9,
        }
    })
}

/// alias registers
#[allow(non_upper_case_globals)]
impl Regs {
    pub const Sp: Regs = Regs::Rsp;
    pub const Pc: Regs = Regs::Rip;
}

#[cfg(feature = "python")]
impl IntoPy<PyObject> for Regs {
    fn into_py(self, py: Python) -> PyObject {
        let n: i32 = self.into();
        n.into_py(py)
    }
}

/// Return an X86 `ArchCapstoneBuilder`
#[must_use]
pub fn capstone() -> capstone::arch::x86::ArchCapstoneBuilder {
    capstone::Capstone::new()
        .x86()
        .mode(capstone::arch::x86::ArchMode::Mode64)
}

pub type GuestReg = u64;

impl crate::ArchExtras for crate::CPU {
    fn read_return_address<T>(&self) -> Result<T, String>
    where
        T: From<GuestReg>,
    {
        let stack_ptr: GuestReg = self.read_reg(Regs::Rsp)?;
        let mut ret_addr = [0; size_of::<GuestReg>()];
        unsafe { self.read_mem(stack_ptr, &mut ret_addr) };
        Ok(GuestReg::from_le_bytes(ret_addr).into())
    }

    fn write_return_address<T>(&self, val: T) -> Result<(), String>
    where
        T: Into<GuestReg>,
    {
        let stack_ptr: GuestReg = self.read_reg(Regs::Rsp)?;
        let val: GuestReg = val.into();
        let ret_addr = val.to_le_bytes();
        unsafe { self.write_mem(stack_ptr, &ret_addr) };
        Ok(())
    }

    fn read_function_argument<T>(&self, conv: CallingConvention, idx: i32) -> Result<T, String>
    where
        T: From<GuestReg>,
    {
        if conv != CallingConvention::Cdecl {
            return Err(format!("Unsupported calling convention: {conv:#?}"));
        }

        match idx {
            0 => self.read_reg(Regs::Rdi),
            1 => self.read_reg(Regs::Rsi),
            _ => Err(format!("Unsupported argument: {idx:}")),
        }
    }

    fn write_function_argument<T>(
        &self,
        conv: CallingConvention,
        idx: i32,
        val: T,
    ) -> Result<(), String>
    where
        T: Into<GuestReg>,
    {
        if conv != CallingConvention::Cdecl {
            return Err(format!("Unsupported calling convention: {conv:#?}"));
        }

        let val: GuestReg = val.into();
        match idx {
            0 => self.write_reg(Regs::Rdi, val),
            1 => self.write_reg(Regs::Rsi, val),
            _ => Err(format!("Unsupported argument: {idx:}")),
        }
    }
}