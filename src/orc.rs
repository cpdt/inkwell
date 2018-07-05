use llvm_sys::orc::{LLVMOrcJITStackRef, LLVMOrcModuleHandle, LLVMOrcErrorCode, LLVMOrcMakeSharedModule, LLVMOrcDisposeSharedModuleRef, LLVMOrcCreateInstance, LLVMOrcGetErrorMsg, LLVMOrcGetMangledSymbol, LLVMOrcDisposeMangledSymbol, LLVMOrcCreateLazyCompileCallback, LLVMOrcSetIndirectStubPointer, LLVMOrcAddEagerlyCompiledIR, LLVMOrcAddLazilyCompiledIR, LLVMOrcAddObjectFile, LLVMOrcRemoveModule, LLVMOrcGetSymbolAddress, LLVMOrcDisposeInstance};
use llvm_sys::execution_engine::LLVMLinkInMCJIT;
use llvm_sys::target_machine::LLVMTargetHasJIT;
use targets::TargetMachine;
use module::Module;
use std::mem::{uninitialized, transmute};
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::c_char;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrcError<'a> {
    message: &'a str
}

pub struct MangledSymbol {
    ptr: *mut c_char
}

impl MangledSymbol {
    pub(crate) fn new(ptr: *mut c_char) -> Self {
        MangledSymbol {
            ptr
        }
    }

    pub fn to_string(&self) -> String {
        (*self).to_string_lossy().into_owned()
    }
}

impl Deref for MangledSymbol {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe {
            CStr::from_ptr(self.ptr)
        }
    }
}

impl fmt::Debug for MangledSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.deref())
    }
}

impl fmt::Display for MangledSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.deref())
    }
}

impl PartialEq for MangledSymbol {
    fn eq(&self, other: &MangledSymbol) -> bool {
        **self == **other
    }
}

impl Drop for MangledSymbol {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeMangledSymbol(self.ptr)
        }
    }
}

#[derive(Debug)]
pub struct Orc {
    jit: LLVMOrcJITStackRef,
}

pub type OrcModuleKey = LLVMOrcModuleHandle;

extern "C" fn orc_resolve_symbol(name: *const ::libc::c_char, ctx: *mut ::libc::c_void) -> u64 {
    let orc: &Orc = unsafe { transmute(ctx) };

    let mut address = unsafe { uninitialized() };
    let error_code = unsafe {
        LLVMOrcGetSymbolAddress(orc.jit, &mut address, name)
    };
    assert_eq!(error_code, LLVMOrcErrorCode::LLVMOrcErrSuccess);
    address
}

impl Orc {
    pub fn link_in_jit() {
        unsafe {
            LLVMLinkInMCJIT();
        }
    }

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
                LLVMOrcAddEagerlyCompiledIR(self.jit, &mut module_handle, shared_ref, Some(orc_resolve_symbol), transmute(&self))
            }
        } else {
            unsafe {
                LLVMOrcAddLazilyCompiledIR(self.jit, &mut module_handle, shared_ref, Some(orc_resolve_symbol), transmute(&self))
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

    pub fn mangle_symbol(&self, symbol: &str) -> MangledSymbol {
        let c_string = CString::new(symbol).unwrap();
        let mut mangled_ptr = unsafe { uninitialized() };

        unsafe {
            LLVMOrcGetMangledSymbol(self.jit, &mut mangled_ptr, c_string.as_ptr());
        }

        MangledSymbol::new(mangled_ptr)
    }

    pub fn get_symbol_address(&self, symbol: &str) -> Result<u64, OrcError> {
        let c_string = CString::new(symbol).unwrap();
        let mut address = unsafe { uninitialized() };

        let error_code = unsafe {
            LLVMOrcGetSymbolAddress(self.jit, &mut address, c_string.as_ptr())
        };

        if error_code == LLVMOrcErrorCode::LLVMOrcErrSuccess {
            Ok(address)
        } else {
            Err(self.get_last_error())
        }
    }

    fn get_last_error<'a>(&'a self) -> OrcError<'a> {
        let message = unsafe {
            CStr::from_ptr(LLVMOrcGetErrorMsg(self.jit))
        }.to_str().unwrap();
        OrcError { message }
    }
}

impl Drop for Orc {
    fn drop(&mut self) {
        unsafe {
            LLVMOrcDisposeInstance(self.jit);
        }
    }
}
