use llvm_sys::orc::{LLVMOrcJITStackRef, LLVMOrcModuleHandle, LLVMOrcErrorCode, LLVMOrcMakeSharedModule, LLVMOrcDisposeSharedModuleRef, LLVMOrcCreateInstance, LLVMOrcGetErrorMsg, LLVMOrcGetMangledSymbol, LLVMOrcDisposeMangledSymbol, LLVMOrcCreateLazyCompileCallback, LLVMOrcSetIndirectStubPointer, LLVMOrcAddEagerlyCompiledIR, LLVMOrcAddLazilyCompiledIR, LLVMOrcAddObjectFile, LLVMOrcRemoveModule, LLVMOrcGetSymbolAddress, LLVMOrcDisposeInstance};
use llvm_sys::target_machine::LLVMTargetHasJIT;
use targets::TargetMachine;
use module::Module;
use std::mem::uninitialized;
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::c_char;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrcError<'a> {
    message: &'a str
}

#[derive(Debug, PartialEq, Eq)]
pub struct MangledSymbol {
    content: *mut c_char
}

impl Deref for MangledSymbol {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { CStr::from_ptr(self.content) }.to_str().unwrap()
    }
}

impl Drop for MangledSymbol {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeMangledSymbol(self.content)
        }
    }
}

#[derive(Debug)]
pub struct Orc {
    jit: LLVMOrcJITStackRef,
}

pub type OrcModuleKey = LLVMOrcModuleHandle;

impl Orc {
    pub fn new(target: TargetMachine) -> Self {
        assert!(target.get_target().has_jit());
        Orc {
            jit: unsafe { LLVMOrcCreateInstance(target.release()) }
        }
    }

    pub fn add_module(&self, module: &Module, eager: bool) -> Result<OrcModuleKey, OrcError> {
        let shared_ref = module.make_shared();
        let mut module_handle: OrcModuleKey = unsafe { uninitialized() };
        let error_code = if eager {
            unsafe {
                LLVMOrcAddEagerlyCompiledIR(self.jit, &mut module_handle, shared_ref, unimplemented!(), unimplemented!())
            }
        } else {
            unsafe {
                LLVMOrcAddLazilyCompiledIR(self.jit, &mut module_handle, shared_ref, unimplemented!(), unimplemented!())
            }
        };

        if error_code == LLVMOrcErrorCode::LLVMOrcErrSuccess {
            Ok(module_handle)
        } else {
            Err(self.get_last_error())
        }
    }

    pub fn remove_module(&self, key: OrcModuleKey) -> Option<OrcError> {
        let error_code = unsafe { LLVMOrcRemoveModule(self.jit, key) };

        if error_code == LLVMOrcErrorCode::LLVMOrcErrSuccess {
            None
        } else {
            Some(self.get_last_error())
        }
    }

    fn get_last_error<'a>(&'a self) -> OrcError<'a> {
        let message = unsafe {
            CStr::from_ptr(LLVMOrcGetErrorMsg(self.jit))
        }.to_str().unwrap();
        OrcError { message }
    }

    fn mangle_symbol(&self, symbol: &str) -> MangledSymbol {
        let c_string = CString::new(symbol).unwrap();
        let mut mangled_ptr = unsafe { uninitialized() };

        unsafe {
            LLVMOrcGetMangledSymbol(self.jit, &mut mangled_ptr, c_string.as_ptr());
        }

        MangledSymbol { content: mangled_ptr }
    }
}

impl Drop for Orc {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeInstance(self.jit);
        }
    }
}
