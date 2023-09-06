use std::env;
use libafl_qemu::{
    elf::EasyElf,
    emu::Emulator,
    GuestAddr,
    MmapPerms,
    Regs,
};
use libafl::bolts::AsSlice;
use crate::utils::*;
use libafl::inputs::HasTargetBytes;
use libafl::prelude::{BytesInput, ExitKind};
use crate::fuzzer::MAX_INPUT_SIZE;
use std::path::PathBuf;

pub type QemuHarness = dyn Fn(&Emulator, &BytesInput, GuestAddr, GuestAddr, GuestAddr) -> ExitKind;
pub(crate) const TAURI_CMD_1: &str = "tauri_cmd_1";
pub(crate) const TAURI_CMD_2: &str = "tauri_cmd_2";
const CRASH_INPUT_1: &[u8] = b"abc";
const CRASH_INPUT_2: u32 = 100;


pub(crate) fn setup_qemu_emulator(fuzzed_binary_path: PathBuf, fuzzed_func: &str) -> (Emulator, GuestAddr, GuestAddr, GuestAddr) {
    // Initialize QEMU
    env::remove_var("LD_LIBRARY_PATH");
    let mut args: Vec<String> = env::args().collect();
    args.push(fuzzed_binary_path.into_os_string().into_string().unwrap());
    let env: Vec<(String, String)> = env::vars().collect();
    let emu = Emulator::new(&args, &env).unwrap();

    let mut elf_buffer = Vec::new();
    let elf = EasyElf::from_file(emu.binary_path(), &mut elf_buffer).unwrap();

    // Save a memory slot to store dynamically sized input
    let heap_mem = emu
        .map_private(0, MAX_INPUT_SIZE, MmapPerms::ReadWrite)
        .unwrap();
    println!("[fuzzer] Memory for dynamically sized input at {heap_mem:#x}");

    // Get the address of the fuzzed function
    let fuzzed_func_addr = elf
        .resolve_symbol(fuzzed_func, emu.load_addr())
        .unwrap_or_else(|| panic!("Symbol \"{}\" not found", fuzzed_func));
    println!("[fuzzer] {} @ {fuzzed_func_addr:#x}", fuzzed_func);

    // We run the program until _mini-app_ calls the fuzzed function
    emu.set_breakpoint(fuzzed_func_addr);
    unsafe {
        emu.run();
    };

    println!("[fuzzer] Break at {:#x}", emu.read_reg::<_, u64>(Regs::Rip).unwrap());

    // Get the return address
    let stack_ptr: u64 = emu.read_reg(Regs::Rsp).unwrap();
    let mut ret_addr = [0; 8];
    unsafe { emu.read_mem(stack_ptr, &mut ret_addr) };
    let ret_addr = u64::from_le_bytes(ret_addr);

    println!("[fuzzer] Stack pointer = {stack_ptr:#x}");
    println!("[fuzzer] Return address = {ret_addr:#x}");

    emu.remove_breakpoint(fuzzed_func_addr);
    emu.set_breakpoint(ret_addr);

    (emu, fuzzed_func_addr, stack_ptr, heap_mem)
}


// Harness that calls the tauri_cmd_1 in mini-app
pub(crate) fn tauri_cmd_1_harness(
    emu: &Emulator,
    bytes_input: &BytesInput,
    fuzzed_func_addr: GuestAddr,
    stack_ptr: GuestAddr,
    heap_mem: GuestAddr
) -> ExitKind {
    let target = bytes_input.target_bytes();
    let mut buf: &[u8] = target.as_slice();
    let mut len = buf.len();
    if len > MAX_INPUT_SIZE {
        buf = &buf[0..MAX_INPUT_SIZE];
        len = MAX_INPUT_SIZE;
    }
    println!("[fuzzer] buf: {:?}", buf);
    println!("[fuzzer] len: {:?}", len);

    unsafe {
        emu.write_mem(heap_mem, buf);
        emu.write_reg(Regs::Rdi, heap_mem).unwrap();
        emu.write_reg(Regs::Rsi, len).unwrap();
        emu.write_reg(Regs::Rip, fuzzed_func_addr).unwrap();
        emu.write_reg(Regs::Rsp, stack_ptr).unwrap();
        emu.run();
    }

    ExitKind::Ok
}


// Harness that calls the tauri_cmd_2 in mini-app
pub(crate) fn tauri_cmd_2_harness(
    emu: &Emulator,
    bytes_input: &BytesInput,
    fuzzed_func_addr: GuestAddr,
    stack_ptr: GuestAddr,
    _heap_mem: GuestAddr
) -> ExitKind {
    let input: u32 = bytes_input_to_u32(bytes_input);
    
    let input_addr = emu
        .map_private(0, MAX_INPUT_SIZE, MmapPerms::ReadWrite)
        .unwrap();

    unsafe {
    // NOTE: argument have to be placed at Rsi register rather than usual Rdi
    // And we need an empty argument at Rsi, don't know why yet but it works
        emu.write_reg(Regs::Rdi, input_addr).unwrap();
        emu.write_reg(Regs::Rsi, input).unwrap();
        emu.write_reg(Regs::Rip, fuzzed_func_addr).unwrap();
        emu.write_reg(Regs::Rsp, stack_ptr).unwrap();
        emu.run();
    }

    ExitKind::Ok

}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_tauri_cmd_1_harness() {
        let (emu, fuzzed_func_addr, stack_ptr, heap_mem) = setup_qemu_emulator(mini_app_path(), TAURI_CMD_2);

        // Should works fine
        let input = BytesInput::from(vec![0, 0, 0, 5]);
        let res = tauri_cmd_2_harness(&emu, &input, fuzzed_func_addr, stack_ptr, heap_mem);
        assert_eq!(res, ExitKind::Ok);

        // // TODO we cannot detect when the emulated program crash 
        // let u32_as_bytes: [u8; 4] = CRASH_INPUT_2.to_be_bytes();
        // let input = BytesInput::from(&u32_as_bytes[..]);
        // let res = tauri_cmd_2_harness(&emu, &input, fuzzed_func_addr, stack_ptr, heap_mem);
        // assert_eq!(res, ExitKind::Ok);
    }

    #[test]
    fn test_tauri_cmd_2_harness() {
        let (emu, fuzzed_func_addr, stack_ptr, heap_mem) = setup_qemu_emulator(mini_app_path(), TAURI_CMD_2);

        // Should works fine
        let input = BytesInput::from(vec![0, 0, 0, 5]);
        let res = tauri_cmd_2_harness(&emu, &input, fuzzed_func_addr, stack_ptr, heap_mem);
        assert_eq!(res, ExitKind::Ok);

        // // TODO we cannot detect when the emulated program crash 
        // let u32_as_bytes: [u8; 4] = CRASH_INPUT_2.to_be_bytes();
        // let input = BytesInput::from(&u32_as_bytes[..]);
        // let res = tauri_cmd_2_harness(&emu, &input, fuzzed_func_addr, stack_ptr, heap_mem);
        // assert_eq!(res, ExitKind::Ok);
    }
}

