use llvm_sys::orc::{LLVMOrcJITStackRef, LLVMOrcModuleHandle, LLVMOrcErrorCode, LLVMOrcTargetAddress, LLVMSharedModuleRef};
use llvm_sys::execution_engine::LLVMLinkInMCJIT;
use llvm_sys::target_machine::{LLVMTargetMachineRef, LLVMTargetHasJIT};
use targets::TargetMachine;
use module::Module;
use std::mem::{uninitialized, transmute, forget};
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::c_char;
use std::fmt;

// some Axiom-specific ORC functions
extern "C" {
    pub fn LLVMAxiomOrcCreateInstance(machine: LLVMTargetMachineRef) -> LLVMOrcJITStackRef;
    pub fn LLVMAxiomOrcAddBuiltin(jit_stack: LLVMOrcJITStackRef, name: *const ::libc::c_char, address: LLVMOrcTargetAddress);
    pub fn LLVMAxiomOrcAddModule(jit_stack: LLVMOrcJITStackRef, module: LLVMSharedModuleRef) -> LLVMOrcModuleHandle;
    pub fn LLVMAxiomOrcRemoveModule(jit_stack: LLVMOrcJITStackRef, handle: LLVMOrcModuleHandle);
    pub fn LLVMAxiomOrcGetSymbolAddress(jit_stack: LLVMOrcJITStackRef, name: *const ::libc::c_char) -> LLVMOrcTargetAddress;
    pub fn LLVMAxiomOrcDisposeInstance(jit: LLVMOrcJITStackRef);
}

#[derive(Debug)]
pub struct Orc {
    jit: LLVMOrcJITStackRef,
}

pub type OrcModuleKey = LLVMOrcModuleHandle;

impl Orc {
    pub fn link_in_jit() {
        unsafe {
            LLVMLinkInMCJIT();
        }
    }

    pub fn new(target: TargetMachine) -> Self {
        assert!(target.get_target().has_jit());

        // LLVMAxiomOrcCreateInstance takes ownership of `target`
        let target_ref = target.target_machine;
        forget(target);
        Orc {
            jit: unsafe { LLVMAxiomOrcCreateInstance(target_ref) }
        }
    }

    pub fn add_builtin(&self, name: &str, address: LLVMOrcTargetAddress) {
        let c_string = CString::new(name).unwrap();
        unsafe { LLVMAxiomOrcAddBuiltin(self.jit, c_string.as_ptr(), address) }
    }

    pub fn add_module(&self, module: &Module) -> OrcModuleKey {
        unsafe { LLVMAxiomOrcAddModule(self.jit, module.make_shared()) }
    }

    pub fn remove_module(&self, key: OrcModuleKey) {
        unsafe { LLVMAxiomOrcRemoveModule(self.jit, key) };
    }

    pub fn get_symbol_address(&self, symbol: &str) -> u64 {
        let c_string = CString::new(symbol).unwrap();
        unsafe { LLVMAxiomOrcGetSymbolAddress(self.jit, c_string.as_ptr()) }
    }
}

impl Drop for Orc {
    fn drop(&mut self) {
        unsafe {
            LLVMAxiomOrcDisposeInstance(self.jit);
        }
    }
}
