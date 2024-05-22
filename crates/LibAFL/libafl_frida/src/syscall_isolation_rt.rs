//! Functionality for [`frida`].
//! With it, a fuzzer can detect any access to a system call

#[allow(unused_imports)]
use std::{
    fmt::{Debug, Formatter},
    rc::Rc,
    sync::{Arc, Mutex},
};

use capstone::Capstone;
use frida_gum::{
    interceptor::{Interceptor, InvocationContext, InvocationListener},
    ExportDetails, Gum, Module, ModuleDetails, ModuleDetailsOwned, ModuleMap, NativePointer,
    SymbolDetails,
};
#[cfg(unix)]
use frida_gum_sys::Insn as FridaInsn;
use libafl::{
    inputs::{HasTargetBytes, Input},
    Error,
};
use policies::engine::{Context, FunctionPolicy, FuzzPolicy};
use rangemap::RangeMap;

use crate::helper::FridaRuntime;
#[cfg(unix)]
use crate::utils::frida_to_cs;

/// `Frida`-based binary-only instrumentation that intercepts calls to system calls
pub struct SyscallIsolationRuntime {
    /// If the runtime has been initialized yet
    initialized: Arc<Mutex<bool>>,
    /// A listener to the harness function we are fuzzing
    harness_listener: HarnessListener,
    /// Listeners to all the libc functions that are being monitored
    function_listeners: Vec<FunctionListener>,
    /// Flag to indicate when the runtime is activated
    activated: Arc<Mutex<bool>>,
}

/// A listener to the Tauri command we are fuzzing
// The harness listener is used to indicate the fuzzer if the
// code being executed is the fuzz target or the fuzzer code
#[derive(Clone)]
struct HarnessListener {
    /// Pointer to the function
    function_pointer: NativePointer,
    /// Flag to indicate when we have started the harness
    in_the_harness: Arc<Mutex<bool>>,
}

impl InvocationListener for HarnessListener {
    /// When entering the fuzzed code set the flag to true
    fn on_enter(&mut self, _context: InvocationContext) {
        *self.in_the_harness.lock().unwrap() = true;
    }

    /// When leaving the fuzzed code set the flag to true
    fn on_leave(&mut self, _context: InvocationContext) {
        *self.in_the_harness.lock().unwrap() = false;
    }
}

/// The listener to one of the function which access is targeted
struct FunctionListener {
    /// Name of the function targeted
    function_name: String,
    /// Pointer to the function
    function_pointer: NativePointer,
    /// Policy applied to this function
    policy: FunctionPolicy,
    /// A flag to only trigger analysis when the targeted function is called from fuzzed code
    /// Otherwise we would also trigger the analysis for calls from the fuzzer code which we want
    /// to avoid.
    in_the_harness: Arc<Mutex<bool>>,
}

use frida_gum::interceptor::PointCut;
impl FunctionListener {
    fn policy_context_from_invoc_context(&self, invoc_context: &InvocationContext) -> Context {
        match invoc_context.point_cut() {
            PointCut::Enter => {
                let mut parameters = vec![];
                for i in 0..self.policy.nb_parameters {
                    parameters.push(invoc_context.arg(i));
                }
                Context::EntryContext(parameters)
            }
            PointCut::Leave => Context::LeaveContext(invoc_context.return_value()),
        }
    }

    fn is_policy_respected(&self, invoc_context: &InvocationContext) -> bool {
        let policy_context = self.policy_context_from_invoc_context(invoc_context);
        self.policy.is_respected(&policy_context)
    }
}

impl InvocationListener for FunctionListener {
    fn on_enter(&mut self, context: InvocationContext) {
        let flag = self.in_the_harness.lock().unwrap();
        if *flag {
            // Check the deny rules of the function
            log::info!("#{} Entering: {:?}", context.thread_id(), self);
            // We drop the flag before function that may panic.
            // The fuzzer panic_hook will need to access it.
            // Otherwise we'd have a deadlock
            drop(flag);
            if !self.is_policy_respected(&context) {
                panic!(
                    "Intercepting call to [{}].\n{}",
                    self.function_name,
                    self.policy.policy_infringement_message(
                        &self.policy_context_from_invoc_context(&context)
                    )
                );
            }
        }
    }

    fn on_leave(&mut self, context: InvocationContext) {
        let flag = self.in_the_harness.lock().unwrap();

        if *flag {
            // Check the deny rules of the function
            log::info!("#{} Leaving: {:?}", context.thread_id(), self);
            // We drop the flag before function that may panic.
            // The fuzzer panic_hook will need to access it.
            // Otherwise we'd have a deadlock
            drop(flag);
            if !self.is_policy_respected(&context) {
                panic!(
                    "Intercepting returning function [{}].\n{}",
                    self.function_name,
                    self.policy.policy_infringement_message(
                        &self.policy_context_from_invoc_context(&context)
                    )
                );
            }
        }
    }
}

impl FridaRuntime for SyscallIsolationRuntime {
    /// Check that libc and the instrumented tauri app are in the instrumented module binary
    fn init(
        &mut self,
        gum: &Gum,
        _ranges: &RangeMap<usize, (u16, String)>,
        _module_map: &Rc<ModuleMap>,
    ) {
        // // Used if using the stalker
        // let is_libc_instrumented = module_map
        //     .find(self.libc.base_address.try_into().unwrap())
        //     .is_some();
        // if !is_libc_instrumented {
        //     panic!("{} not instrumented", self.libc.name)
        // }
        // let is_tauri_app_instrumented = module_map
        //     .find(self.tauri_app.base_address.try_into().unwrap())
        //     .is_some();
        //
        // if !is_tauri_app_instrumented {
        //     panic!("{} not instrumented", self.tauri_app.name)
        // }
        if *self.initialized.lock().expect("Poisoned initialized mutex") {
            return;
        }
        *self.initialized.lock().unwrap() = true;

        log::error!("Initiating the SyscallIsolationRuntime");
        let mut interceptor = Interceptor::obtain(gum);
        interceptor.attach(
            self.harness_listener.function_pointer,
            &mut self.harness_listener,
        );
        for listener in self.function_listeners.iter_mut() {
            interceptor.attach(listener.function_pointer, listener);
        }

        // Activate the function listeners
        let flag = Arc::clone(&self.activated);

        // NOTE this is not the ideal way but seems to work
        // We modify the panic hook so that the `SyscallIsolationRuntime`
        // is deactivated when crashing and does not interfere with the
        // fuzzer code
        let old_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            *flag.lock().unwrap() = false;
            old_hook(panic_info);
        }));
    }

    fn pre_exec<I: Input + HasTargetBytes>(&mut self, _input: &I) -> Result<(), Error> {
        Ok(())
    }

    fn post_exec<I: Input + HasTargetBytes>(&mut self, _input: &I) -> Result<(), Error> {
        *self.activated.lock().unwrap() = false;
        Ok(())
    }
}

use capstone::arch::{x86::X86Operand, ArchOperand};

impl SyscallIsolationRuntime {
    /// Creates a [`SyscallIsolationRuntime`]
    /// Setup listeners for the monitored libc functions provided.
    /// Setup listener for the tauri command being fuzzed.
    #[must_use]
    pub fn new(fuzz_policy: FuzzPolicy, harness_address: usize) -> Result<Self, Error> {
        log::debug!("{:#?}", modules_info());

        // println!("{:#?}", modules_info());
        // let lib = Module::enumerate_modules()
        //     .into_iter()
        //     .find(|m| m.name.contains("KERNEL32"))
        //     .ok_or(Error::unknown(format!(
        //         "lib {} not found in modules",
        //         "kernel"
        //     )))?;
        // println!("{:#?}", symbols_in_module(&lib.name));
        // println!("{:#?}", exports_in_module(&lib.name));

        let mut listeners: Vec<FunctionListener> = vec![];
        let flag = Arc::new(Mutex::new(false));

        // Create function listeners from the fuzz policy received
        for function_policy in fuzz_policy.into_iter() {
            // Get the function lib
            let lib = Module::enumerate_modules()
                .into_iter()
                .find(|m| m.name.contains(&function_policy.lib))
                .ok_or(Error::unknown(format!(
                    "lib {} not found in modules",
                    function_policy.lib
                )))?;

            // Get the function pointer
            let func_ptr = Module::find_export_by_name(Some(&lib.name), &function_policy.name)
                .unwrap_or_else(|| {
                    panic!(
                        "Failed to find init interceptor for function {}",
                        &function_policy.name
                    )
                });
            if func_ptr.is_null() {
                panic!(
                    "Function {} in lib {} is null pointer",
                    &function_policy.name, function_policy.lib
                );
            }

            // Create listener
            let listener = FunctionListener {
                function_name: function_policy.name.clone(),
                policy: function_policy,
                function_pointer: func_ptr,
                in_the_harness: Arc::clone(&flag),
            };
            log::info!("listener: {:?}", listener);
            listeners.push(listener);
        }

        let harness_listener = Self::create_harness_listener(harness_address, Arc::clone(&flag))?;

        let res = SyscallIsolationRuntime {
            initialized: Arc::new(Mutex::new(false)),
            harness_listener,
            function_listeners: listeners,
            activated: flag,
        };

        Ok(res)
    }

    /// Create a listener for the fuzz harness
    /// It's used to signal to the fuzzer if we are currently executing harness code.
    fn create_harness_listener(
        harness_address: usize,
        flag: Arc<Mutex<bool>>,
    ) -> Result<HarnessListener, Error> {
        let harness_ptr = NativePointer(harness_address as *mut core::ffi::c_void);

        let harness_listener = HarnessListener {
            function_pointer: harness_ptr,
            in_the_harness: Arc::clone(&flag),
        };

        log::info!("{:?}", harness_listener);
        Ok(harness_listener)
    }

    /// Check if current instruction is relevant for the [`SyscallIsolationRuntime`]
    #[cfg(unix)]
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

#[cfg(unix)]
fn is_syscall_instruction(cs_instr: &capstone::Insn) -> bool {
    match cs_instr.mnemonic().unwrap() {
        "syscall" => true,
        _ => false,
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn module_details_to_string(module: &ModuleDetails) -> String {
    let name = module.name();
    let range = module.range();
    let base = range.base_address().0 as usize;
    let size = range.size();

    format!(
        "ModuleDetails {}: [{:#018x}, {:#018x}]",
        name,
        base,
        base + size
    )
}

#[allow(dead_code)]
fn module_details_owned_to_string(module: &ModuleDetailsOwned) -> String {
    let _name = &module.name;
    let base = module.base_address;
    let size = module.size;
    let path = &module.path;

    format!(
        "ModuleDetailsOwned {}: [{:#018x}, {:#018x}]",
        path,
        base,
        base + size
    )
}

// Functions for debugging

#[allow(dead_code)]
fn modules_info() -> Vec<String> {
    Module::enumerate_modules()
        .iter()
        .map(|module| module_details_owned_to_string(module))
        .collect::<Vec<String>>()
}

#[allow(dead_code)]
fn symbols_in_module(module_name: &str) -> Vec<String> {
    Module::enumerate_symbols(module_name)
        .iter()
        .map(|module| symbol_details_to_string(module))
        .collect::<Vec<String>>()
}

#[allow(dead_code)]
fn exports_in_module(module_name: &str) -> Vec<String> {
    Module::enumerate_exports(module_name)
        .iter()
        .map(|module| export_details_to_string(module))
        .collect::<Vec<String>>()
}

#[allow(dead_code)]
fn symbol_details_to_string(s: &SymbolDetails) -> String {
    format!(
        "SymbolDetails {}: [{:#018x}, {:#018x}]",
        s.name,
        s.address,
        s.address + s.size
    )
}

#[allow(dead_code)]
fn export_details_to_string(e: &ExportDetails) -> String {
    format!(
        "SymbolDetails {}: {:#018x}, typ: {}]",
        e.name, e.address, e.typ
    )
}

impl Debug for SyscallIsolationRuntime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dbg_me = f.debug_struct("SyscallIsolationRuntime");
        dbg_me.field("harness_listener", &self.harness_listener);
        dbg_me.field(
            "function_listeners",
            &self
                .function_listeners
                .iter()
                .map(|l| l.function_name.clone())
                .collect::<Vec<String>>(),
        );
        dbg_me.finish()
    }
}

impl Debug for HarnessListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dbg_me = f.debug_struct("HarnessListener");
        dbg_me.field("in_the_harness", &self.in_the_harness);
        dbg_me.finish_non_exhaustive()
    }
}

impl Debug for FunctionListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dbg_me = f.debug_struct("LibCListener");
        dbg_me.field("function_name", &self.function_name);
        // dbg_me.field("in_the_harness", &self.in_the_harness);
        dbg_me.finish_non_exhaustive()
    }
}
