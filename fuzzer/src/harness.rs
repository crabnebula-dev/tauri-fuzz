use libafl_qemu::{
    //asan::QemuAsanHelper,
    edges::{edges_map_mut_slice, QemuEdgeCoverageHelper, MAX_EDGES_NUM},
    elf::EasyElf,
    emu::Emulator,
    GuestAddr,
    //snapshot::QemuSnapshotHelper,
    MmapPerms,
    QemuExecutor,
    QemuHooks,
    Regs,
};

use libafl::prelude::{BytesInput, ExitKind};
use crate::fuzzer::{MAX_INPUT_SIZE, CRASH_INPUT_2, CRASH_INPUT_1};
use crate::utils::bytes_input_to_u32;

pub(crate) fn test_harness(bytes_input: &BytesInput) -> ExitKind {
    let input: u32 = bytes_input_to_u32(bytes_input);
    
    if input == 4096 {
        println!("[harness] input: {}, bytes_input: {:?}", input, bytes_input);
        ExitKind::Crash
    } else {
        ExitKind::Ok
    }
}


fn test_tauri_cmd_2_harness(emu: &Emulator, fuzzed_func_addr: GuestAddr, stack_ptr: GuestAddr) {
    let mut count: u32 = 0;
    while count < 5 {
        count = count + 1;
        let input: BytesInput;
        if count != 3 {
            // Just give it a random byte
            input = BytesInput::from(vec![0, 0, 0, 5]);
        } else {
            //Make the app crash with dangerous input
            let u32_as_bytes: [u8; 4] = CRASH_INPUT_2.to_be_bytes();
            input = BytesInput::from(&u32_as_bytes[..]);
        }
        tauri_cmd_2_harness(&emu, &input, fuzzed_func_addr, stack_ptr);
    }
}

pub(crate) fn tauri_cmd_2_harness(
    emu: &Emulator,
    bytes_input: &BytesInput,
    fuzzed_func_addr: GuestAddr,
    stack_ptr: GuestAddr,
) -> ExitKind {
    let input: u32 = bytes_input_to_u32(bytes_input);
    
    let input_addr = emu
        .map_private(0, MAX_INPUT_SIZE, MmapPerms::ReadWrite)
        .unwrap();

    // NOTE: argument have to be placed at Rsi register rather than usual Rdi
    unsafe {
        // emu.write_mem(input_addr, buf);
        emu.write_reg(Regs::Rdi, input_addr).unwrap();
        // emu.write_reg(Regs::Rdi, target).unwrap();
        emu.write_reg(Regs::Rsi, input).unwrap();
        emu.write_reg(Regs::Rip, fuzzed_func_addr).unwrap();
        emu.write_reg(Regs::Rsp, stack_ptr).unwrap();
        // emu.add_gdb_cmd()
        emu.run();
    }

    ExitKind::Ok
}
