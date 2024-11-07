// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! A fuzzing runtime that monitor calls to specific functions depending on the policy provided

#[allow(unused_imports)]
use std::{
    fmt::{Debug, Formatter},
    rc::Rc,
    sync::{Arc, Mutex},
};

use frida_gum::{
    interceptor::{Interceptor, InvocationContext, InvocationListener},
    ExportDetails, Gum, Module, ModuleDetails, ModuleDetailsOwned, ModuleMap, NativePointer,
    SymbolDetails,
};
use libafl::{
    inputs::{HasTargetBytes, Input},
    Error,
};
use libafl_frida::helper::FridaRuntime;

use rangemap::RangeMap;
use tauri_fuzz_policies::engine::{Context, FunctionPolicy, FuzzPolicy};

/// `Frida`-based binary-only instrumentation that intercepts calls to system calls
pub struct FunctionListenerRuntime {
    /// A listener to the harness function we are fuzzing
    // harness_listener: HarnessListener,
    /// Listeners to all the libc functions that are being monitored
    function_listeners: Vec<FunctionListener>,
    /// Flag to indicate when the runtime is activated
    switch: Arc<Mutex<InterceptionSwitch>>,
    /// Pointer to the harness code
    harness_pointer: NativePointer,
    /// Flag to avoid initializing twice
    is_init: Arc<Mutex<bool>>,
}

#[derive(Debug)]
struct InterceptionSwitch {
    flag: bool,
}

impl InterceptionSwitch {
    fn activate(&mut self) {
        self.flag = true;
    }

    fn deactivate(&mut self) {
        self.flag = false;
    }

    fn is_active(&self) -> bool {
        self.flag
    }
}

impl InvocationListener for FunctionListenerRuntime {
    /// When entering the fuzzed code set the flag to true
    fn on_enter(&mut self, _context: InvocationContext) {
        self.switch.lock().unwrap().activate()
    }

    /// When leaving the fuzzed code set the flag to true
    fn on_leave(&mut self, _context: InvocationContext) {
        self.switch.lock().unwrap().deactivate()
    }
}

/// The listener to one of the function which access is targeted
#[derive(Debug)]
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
    interception_switch: Arc<Mutex<InterceptionSwitch>>,
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

    fn policy_should_block(&mut self, invoc_context: &InvocationContext) -> bool {
        let policy_context = self.policy_context_from_invoc_context(invoc_context);
        self.policy.should_block(&policy_context)
    }
}

impl InvocationListener for FunctionListener {
    fn on_enter(&mut self, context: InvocationContext) {
        if self.interception_switch.lock().unwrap().is_active() {
            // Check the deny rules of the function
            log::info!("#{} Entering: {:?}", context.thread_id(), self);
            // We drop the flag before function that may panic.
            // The fuzzer panic_hook will need to access it.
            // Otherwise we'd have a deadlock
            // drop(flag);

            if self.policy_should_block(&context) {
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
        if self.interception_switch.lock().unwrap().is_active() {
            // Check the deny rules of the function
            log::info!("#{} Leaving: {:?}", context.thread_id(), self);
            // We drop the flag before function that may panic.
            // The fuzzer panic_hook will need to access it.
            // Otherwise we'd have a deadlock
            // drop(flag);
            if self.policy_should_block(&context) {
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

impl FridaRuntime for FunctionListenerRuntime {
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

        // If the runtime is already initialized skip
        if *self.is_init.lock().unwrap() {
            log::trace!("SyscallIsolationRuntime already initialized, skipping");
            return;
        }

        log::trace!("Initiating the SyscallIsolationRuntime");

        *self.is_init.lock().unwrap() = true;
        let mut interceptor = Interceptor::obtain(gum);
        // Create interceptor for the harness
        interceptor.attach(self.harness_pointer, self);
        // Create interceptors for the functions we are monitoring
        for listener in self.function_listeners.iter_mut() {
            interceptor.attach(listener.function_pointer, listener);
        }

        // NOTE this is not the ideal way but seems to work
        // We modify the panic hook so that the `SyscallIsolationRuntime`
        // is deactivated when crashing and does not continue monitoring
        // the fuzzer code
        let old_hook = std::panic::take_hook();
        let switch = self.switch.clone();
        std::panic::set_hook(Box::new(move |panic_info| {
            switch.lock().unwrap().deactivate();
            old_hook(panic_info);
        }));
    }

    fn pre_exec<I: Input + HasTargetBytes>(&mut self, _input: &I) -> Result<(), Error> {
        Ok(())
    }

    fn deinit(&mut self, _gum: &Gum) {}

    fn post_exec<I: Input + HasTargetBytes>(&mut self, _input: &I) -> Result<(), Error> {
        self.switch.lock().unwrap().deactivate();
        Ok(())
    }
}

impl FunctionListenerRuntime {
    /// Creates a [`SyscallIsolationRuntime`]
    /// Setup listeners for the monitored libc functions provided.
    /// Setup listener for the tauri command being fuzzed.
    pub fn new(fuzz_policy: FuzzPolicy, harness_address: usize) -> Result<Self, Error> {
        log::debug!("{:#?}", modules_info());

        // println!("{:#?}", modules_info());
        // let libs = Module::enumerate_modules();
        // let current_bin = libs.first().unwrap();
        // println!("{:#?}", symbols_in_module(&current_bin.name));
        // println!("{:#?}", exports_in_module(&current_bin.name));
        //
        //
        // let lib = Module::enumerate_modules()
        //     .into_iter()
        //     .find(|m| m.name.contains("KERNEL32.DLL"))
        //     .ok_or(Error::unknown(format!(
        //         "lib {} not found in modules",
        //         "kernel"
        //     )))?;
        // println!("{:#?}", symbols_in_module(&lib.name));
        // println!("{:#?}", exports_in_module(&lib.name));

        let mut listeners: Vec<FunctionListener> = vec![];
        let switch = Arc::new(Mutex::new(InterceptionSwitch { flag: false }));

        // Create function listeners from the fuzz policy received
        for function_policy in fuzz_policy.into_iter() {
            // Get the function lib
            let func_ptr = find_symbol_in_modules(&function_policy);

            if let Some(func_ptr) = func_ptr {
                // Create listener
                let listener = FunctionListener {
                    function_name: function_policy.name.clone(),
                    policy: function_policy,
                    function_pointer: func_ptr,
                    interception_switch: switch.clone(),
                };

                log::info!("listener: {:?}", listener);
                listeners.push(listener);
            }
        }

        let res = FunctionListenerRuntime {
            harness_pointer: NativePointer(harness_address as *mut core::ffi::c_void),
            function_listeners: listeners,
            switch,
            is_init: Arc::new(Mutex::new(false)),
        };

        Ok(res)
    }
}

fn find_symbol_in_modules(policy: &FunctionPolicy) -> Option<NativePointer> {
    let lib = Module::enumerate_modules()
        .into_iter()
        .find(|m| m.path.contains(&policy.lib))
        .unwrap_or_else(|| panic!("Failed to find library for policy {:#?}", policy));

    let function_name = if policy.is_rust_function {
        // If the function is a Rust we have to find it among mangled names
        let parsed_tokens = policy.name.split("::");

        let mut symbols: Vec<String> = Module::enumerate_symbols(&lib.name)
            .into_iter()
            .map(|symbol| symbol.name)
            .collect();
        // Add the export symbols to look for
        let export_names = Module::enumerate_exports(&lib.name)
            .into_iter()
            .map(|export| export.name);
        symbols.extend(export_names);

        for token in parsed_tokens {
            symbols.retain(|symbol| symbol.contains(token));
        }
        // Remove the $got version of the function we are searching
        symbols.retain(|symbol| !symbol.contains("$got"));
        // Remove the $GT version of the function we are searching
        symbols.retain(|symbol| !symbol.contains("$GT"));

        match symbols.len() {
            // It's possible that certain functions are required by policy
            // but not loaded in binary because they were not used
            0 => {
                return None;
            }

            // We have found exactly one matching symbol
            1 => symbols.first().unwrap().clone(),

            // Multiple symbols have been found that should not be possible
            _ => {
                panic!("Multiple symbol found for {}: {:?}", policy.name, symbols);
            }
        }
    } else {
        policy.name.clone()
    };

    let func_ptr =
        // Get the function pointer
        // Search in the exports first
        Module::find_export_by_name(Some(&lib.name), &function_name).unwrap_or_else(|| {
            // Else search in the symbols
            Module::find_symbol_by_name(&lib.name, &function_name).unwrap_or_else(|| {
                panic!(
                    "Failed to find init interceptor for function {}",
                    policy.name
                )
            })
        }) ;

    if func_ptr.is_null() {
        panic!(
            "Function {} in lib {} is null pointer",
            policy.name, policy.lib
        );
    }
    Some(func_ptr)
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
        .map(module_details_owned_to_string)
        .collect::<Vec<String>>()
}

#[allow(dead_code)]
fn symbols_in_module(module_name: &str) -> Vec<String> {
    Module::enumerate_symbols(module_name)
        .iter()
        .map(symbol_details_to_string)
        .collect::<Vec<String>>()
}

#[allow(dead_code)]
fn exports_in_module(module_name: &str) -> Vec<String> {
    Module::enumerate_exports(module_name)
        .iter()
        .map(export_details_to_string)
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
        "ExportDetails {}: {:#018x}, typ: {}]",
        e.name, e.address, e.typ
    )
}

impl Debug for FunctionListenerRuntime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dbg_me = f.debug_struct("SyscallIsolationRuntime");
        dbg_me.field("activated", &self.switch.lock().unwrap().is_active());
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
