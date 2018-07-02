use llvm_sys::core::{LLVMGetPointerAddressSpace, LLVMConstNull, LLVMGetElementType, LLVMConstArray};
use llvm_sys::prelude::LLVMTypeRef;

use AddressSpace;
use context::ContextRef;
use support::LLVMString;
use types::traits::AsTypeRef;
use types::{Type, BasicType, ArrayType, FunctionType, VectorType, BasicTypeEnum};
use values::{PointerValue, IntValue, ArrayValue};
use values::AsValueRef;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PointerType {
    ptr_type: Type,
}

impl PointerType {
    pub(crate) fn new(ptr_type: LLVMTypeRef) -> Self {
        assert!(!ptr_type.is_null());

        PointerType {
            ptr_type: Type::new(ptr_type),
        }
    }

    pub fn is_sized(&self) -> bool {
        self.ptr_type.is_sized()
    }

    pub fn size_of(&self) -> IntValue {
        self.ptr_type.size_of()
    }

    pub fn ptr_type(&self, address_space: AddressSpace) -> PointerType {
        self.ptr_type.ptr_type(address_space)
    }

    pub fn get_context(&self) -> ContextRef {
        self.ptr_type.get_context()
    }

    pub fn fn_type(&self, param_types: &[&BasicType], is_var_args: bool) -> FunctionType {
        self.ptr_type.fn_type(param_types, is_var_args)
    }

    pub fn array_type(&self, size: u32) -> ArrayType {
        self.ptr_type.array_type(size)
    }

    pub fn get_address_space(&self) -> AddressSpace {
        unsafe {
            LLVMGetPointerAddressSpace(self.as_type_ref()).into()
        }
    }

    pub fn element_type(&self) -> BasicTypeEnum {
        let element_type_ref = unsafe {
            LLVMGetElementType(self.as_type_ref())
        };
        BasicTypeEnum::new(element_type_ref)
    }

    pub fn print_to_string(&self) -> LLVMString {
        self.ptr_type.print_to_string()
    }

    // See Type::print_to_stderr note on 5.0+ status
    #[cfg(not(any(feature = "llvm3-6", feature = "llvm5-0", feature = "llvm6-0")))]
    pub fn print_to_stderr(&self) {
        self.ptr_type.print_to_stderr()
    }

    pub fn const_null_ptr(&self) -> PointerValue {
        self.ptr_type.const_null_ptr()
    }

    pub fn const_null(&self) -> PointerValue {
        let null = unsafe {
            LLVMConstNull(self.as_type_ref())
        };

        PointerValue::new(null)
    }

    pub fn const_array(&self, values: &[PointerValue]) -> ArrayValue {
        let mut values: Vec<_> = values.iter().map(|val| val.as_value_ref()).collect();

        let value = unsafe {
            LLVMConstArray(self.as_type_ref(), values.as_mut_ptr(), values.len() as u32)
        };

        ArrayValue::new(value)
    }

    pub fn get_undef(&self) -> PointerValue {
        PointerValue::new(self.ptr_type.get_undef())
    }

    pub fn vec_type(&self, size: u32) -> VectorType {
        self.ptr_type.vec_type(size)
    }
}

impl AsTypeRef for PointerType {
    fn as_type_ref(&self) -> LLVMTypeRef {
        self.ptr_type.type_
    }
}
