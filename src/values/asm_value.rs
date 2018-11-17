use llvm_sys::prelude::LLVMValueRef;

use support::LLVMString;
use values::Value;
use values::traits::AsValueRef;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AsmValue {
    asm_value: Value,
}

impl AsmValue {
    pub(crate) fn new(value: LLVMValueRef) -> Self {
        assert!(!value.is_null());

        AsmValue {
            asm_value: Value::new(value),
        }
    }

    pub fn is_null(&self) -> bool {
        self.asm_value.is_null()
    }

    pub fn is_undef(&self) -> bool {
        self.asm_value.is_undef()
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.asm_value.print_to_string()
    }

    pub fn print_to_stderr(&self) {
        self.asm_value.print_to_stderr()
    }
}

impl AsValueRef for AsmValue {
    fn as_value_ref(&self) -> LLVMValueRef { self.asm_value.value }
}
