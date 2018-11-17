use llvm_sys::core::{LLVMGetParamTypes, LLVMIsFunctionVarArg, LLVMCountParamTypes, LLVMConstInlineAsm};
use llvm_sys::prelude::LLVMTypeRef;

use std::fmt;
use std::mem::forget;
use std::ffi::CString;

use context::ContextRef;
use support::LLVMString;
use types::traits::AsTypeRef;
use types::{Type, BasicTypeEnum};
use values::FunctionValue;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct FunctionType {
    fn_type: Type,
}

impl FunctionType {
    pub(crate) fn new(fn_type: LLVMTypeRef) -> FunctionType {
        assert!(!fn_type.is_null());

        FunctionType {
            fn_type: Type::new(fn_type)
        }
    }

    pub fn is_var_arg(&self) -> bool {
        unsafe {
            LLVMIsFunctionVarArg(self.as_type_ref()) != 0
        }
    }

    pub fn get_param_types(&self) -> Vec<BasicTypeEnum> {
        let count = self.count_param_types();
        let mut raw_vec: Vec<LLVMTypeRef> = Vec::with_capacity(count as usize);
        let ptr = raw_vec.as_mut_ptr();

        forget(raw_vec);

        let raw_vec = unsafe {
            LLVMGetParamTypes(self.as_type_ref(), ptr);

            Vec::from_raw_parts(ptr, count as usize, count as usize)
        };

        raw_vec.iter().map(|val| BasicTypeEnum::new(*val)).collect()
    }

    pub fn count_param_types(&self) -> u32 {
        unsafe {
            LLVMCountParamTypes(self.as_type_ref())
        }
    }

    // REVIEW: Always false -> const fn?
    pub fn is_sized(&self) -> bool {
        self.fn_type.is_sized()
    }

    pub fn get_context(&self) -> ContextRef {
        self.fn_type.get_context()
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.fn_type.print_to_string()
    }

    pub fn as_asm(self, asm_str: &str, constraints: &str, has_side_effects: bool, is_align_stack: bool) -> FunctionValue {
        let asm_c_string = CString::new(asm_str).expect("Conversion to CString failed unexpectedly");
        let constraints_c_string = CString::new(constraints).expect("Conversion to CString failed unexpectedly");

        let value = unsafe {
            LLVMConstInlineAsm(self.as_type_ref(), asm_c_string.as_ptr(), constraints_c_string.as_ptr(), has_side_effects as i32, is_align_stack as i32)
        };

        FunctionValue::new(value).unwrap()
    }

    // See Type::print_to_stderr note on 5.0+ status
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm5-0", feature = "llvm6-0")))]
    pub fn print_to_stderr(&self) {
        self.fn_type.print_to_stderr()
    }

    // REVIEW: Can you do undef for functions?
    // Seems to "work" - no UB or SF so far but fails
    // LLVMIsAFunction() check. Commenting out for further research
    // pub fn get_undef(&self) -> FunctionValue {
    //     FunctionValue::new(self.fn_type.get_undef()).expect("Should always get an undef value")
    // }
}

impl fmt::Debug for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_type = self.print_to_string();

        f.debug_struct("FunctionType")
            .field("address", &self.as_type_ref())
            .field("llvm_type", &llvm_type)
            .finish()
    }
}

impl AsTypeRef for FunctionType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.fn_type.type_
    }
}
