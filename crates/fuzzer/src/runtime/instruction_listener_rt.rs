//! This is deprecated and not maintained anymore

//! Runtime that will detect any calls to the assembly instruction `syscall`
use capstone::arch::{x86::X86Operand, ArchOperand};
use capstone::Capstone;
use frida_gum_sys::Insn as FridaInsn;

struct InstrListenerRuntime {}

impl InstrListenerRuntime {
    /// Check if current instruction is relevant for the [`SyscallIsolationRuntime`]
    pub fn is_interesting_instruction(&self, capstone: &Capstone, _addr: u64, instr: &FridaInsn) {
        // We need to re-decode frida-internal capstone values to upstream capstone
        let cs_block = frida_to_cs(capstone, instr);

        let _cs_instr = cs_block.first().unwrap();
        // log::warn!("instr: {}", cs_instr);
        for cs_instr in cs_block.as_ref() {
            if is_syscall_instruction(cs_instr) {
                panic!(
                    "Found syscall: {:#?}\nSyscall details:\n{}",
                    cs_instr,
                    cs_instr_details(capstone, cs_instr)
                );
            }
        }
    }
}

fn is_syscall_instruction(cs_instr: &capstone::Insn) -> bool {
    match cs_instr.mnemonic().unwrap() {
        "syscall" => true,
        _ => false,
    }
}

fn cs_instr_details(capstone: &Capstone, cs_instr: &capstone::Insn) -> String {
    let insn_detail = capstone.insn_detail(cs_instr).unwrap();

    let operands = insn_detail.arch_detail().operands();

    let operands: Vec<X86Operand> = operands
        .into_iter()
        .map(|op| match op {
            ArchOperand::X86Operand(op) => op,
            _ => unimplemented!(),
        })
        .collect();

    format!(
        "instr: {}\noperands: {:#?}\ninsn_detail: {:#?}",
        cs_instr, operands, insn_detail
    )
}

/// Function block copied from LibAFL
/// Translates a frida instruction to a capstone instruction.
/// Returns a [`capstone::Instructions`] with a single [`capstone::Insn`] inside.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub(crate) fn frida_to_cs<'a>(
    capstone: &'a Capstone,
    frida_insn: &frida_gum_sys::Insn,
) -> capstone::Instructions<'a> {
    capstone
        .disasm_count(frida_insn.bytes(), frida_insn.address(), 1)
        .unwrap()
}
