#[cfg(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8"))]
use llvm_sys::target::LLVMAddTargetData;
use llvm_sys::target::{LLVMTargetDataRef, LLVMCopyStringRepOfTargetData, LLVMSizeOfTypeInBits, LLVMCreateTargetData, LLVMByteOrder, LLVMPointerSize, LLVMByteOrdering, LLVMStoreSizeOfType, LLVMABISizeOfType, LLVMABIAlignmentOfType, LLVMCallFrameAlignmentOfType, LLVMPreferredAlignmentOfType, LLVMPreferredAlignmentOfGlobal, LLVMElementAtOffset, LLVMOffsetOfElement, LLVMDisposeTargetData, LLVMPointerSizeForAS, LLVMIntPtrType, LLVMIntPtrTypeForAS, LLVMIntPtrTypeInContext, LLVMIntPtrTypeForASInContext};
use llvm_sys::target_machine::{LLVMGetFirstTarget, LLVMTargetRef, LLVMGetNextTarget, LLVMGetTargetFromName, LLVMGetTargetFromTriple, LLVMGetTargetName, LLVMGetTargetDescription, LLVMTargetHasJIT, LLVMTargetHasTargetMachine, LLVMTargetHasAsmBackend, LLVMTargetMachineRef, LLVMDisposeTargetMachine, LLVMGetTargetMachineTarget, LLVMGetTargetMachineTriple, LLVMSetTargetMachineAsmVerbosity, LLVMCreateTargetMachine, LLVMGetTargetMachineCPU, LLVMGetTargetMachineFeatureString, LLVMGetDefaultTargetTriple, LLVMAddAnalysisPasses, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMRelocMode, LLVMCodeGenFileType, LLVMTargetMachineEmitToMemoryBuffer, LLVMTargetMachineEmitToFile};

use OptimizationLevel;
use context::Context;
use data_layout::DataLayout;
use memory_buffer::MemoryBuffer;
use module::Module;
use passes::PassManager;
use support::LLVMString;
use types::{AnyType, AsTypeRef, StructType, PointerType};
use values::{AsValueRef, GlobalValue};

use std::default::Default;
use std::ffi::{CStr, CString};
use std::mem::zeroed;
use std::path::Path;
use std::ptr;

#[derive(Debug, PartialEq, Eq)]
pub enum CodeModel {
    Default,
    JITDefault,
    Small,
    Kernel,
    Medium,
    Large,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RelocMode {
    Default,
    Static,
    PIC,
    DynamicNoPic,
}


#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    Assembly,
    Object,
}

impl FileType {
    fn as_llvm_file_type(&self) -> LLVMCodeGenFileType {
        match *self {
            FileType::Assembly => LLVMCodeGenFileType::LLVMAssemblyFile,
            FileType::Object => LLVMCodeGenFileType::LLVMObjectFile,
        }
    }
}

// TODO: Doc: Base gets you TargetMachine support, machine_code gets you asm_backend
#[derive(Debug, PartialEq, Eq)]
pub struct InitializationConfig {
    pub asm_parser: bool,
    pub asm_printer: bool,
    pub base: bool,
    pub disassembler: bool,
    pub info: bool,
    pub machine_code: bool,
}

impl Default for InitializationConfig {
    fn default() -> Self {
        InitializationConfig {
            asm_parser: true,
            asm_printer: true,
            base: true,
            disassembler: true,
            info: true,
            machine_code: true,
        }
    }
}

// NOTE: Versions verified as target-complete: 3.6, 3.7, 3.8, 3.9, 4.0
#[derive(Debug)]
pub struct Target {
    target: LLVMTargetRef,
}

impl Target {
    fn new(target: LLVMTargetRef) -> Self {
        assert!(!target.is_null());

        Target {
            target,
        }
    }

    // REVIEW: Should this just initialize all? Is opt into each a good idea?
    pub fn initialize_x86(config: &InitializationConfig) {
        use llvm_sys::target::{LLVMInitializeX86Target, LLVMInitializeX86TargetInfo, LLVMInitializeX86TargetMC, LLVMInitializeX86Disassembler, LLVMInitializeX86AsmPrinter, LLVMInitializeX86AsmParser};

        unsafe {
            if config.base {
                LLVMInitializeX86Target()
            }

            if config.info {
                LLVMInitializeX86TargetInfo()
            }

            if config.asm_printer {
                LLVMInitializeX86AsmPrinter()
            }

            if config.asm_parser {
                LLVMInitializeX86AsmParser()
            }

            if config.disassembler {
                LLVMInitializeX86Disassembler()
            }

            if config.machine_code {
                LLVMInitializeX86TargetMC()
            }
        }
    }

    pub fn initialize_native(config: &InitializationConfig) -> Result<(), String> {
        use llvm_sys::target::{LLVM_InitializeNativeTarget, LLVM_InitializeNativeAsmParser, LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeDisassembler};

        if config.base {
            let code = unsafe {
                LLVM_InitializeNativeTarget()
            };

            if code == 1 {
                return Err("Unknown error in initializing native target".into());
            }
        }

        if config.asm_printer {
            let code = unsafe {
                LLVM_InitializeNativeAsmPrinter()
            };

            if code == 1 {
                return Err("Unknown error in initializing native asm printer".into());
            }
        }

        if config.asm_parser {
            let code = unsafe {
                LLVM_InitializeNativeAsmParser()
            };

            if code == 1 { // REVIEW: Does parser need to go before printer?
                return Err("Unknown error in initializing native asm parser".into());
            }
        }

        if config.disassembler {
            let code = unsafe {
                LLVM_InitializeNativeDisassembler()
            };

            if code == 1 {
                return Err("Unknown error in initializing native disassembler".into());
            }
        }

        Ok(())
    }

    pub fn initialize_all(config: &InitializationConfig) {
        use llvm_sys::target::{LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargets, LLVM_InitializeAllTargetMCs, LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllAsmParsers, LLVM_InitializeAllDisassemblers};

        unsafe {
            if config.base {
                LLVM_InitializeAllTargets()
            }

            if config.info {
                LLVM_InitializeAllTargetInfos()
            }

            if config.asm_parser {
                LLVM_InitializeAllAsmParsers()
            }

            if config.asm_printer {
                LLVM_InitializeAllAsmPrinters()
            }

            if config.disassembler {
                LLVM_InitializeAllDisassemblers()
            }

            if config.machine_code {
                LLVM_InitializeAllTargetMCs()
            }
        }
    }

    pub fn create_target_machine(&self, triple: &str, cpu: &str, features: &str, level: OptimizationLevel, reloc_mode: RelocMode, code_model: CodeModel) -> Option<TargetMachine> {
        let triple = CString::new(triple).expect("Conversion to CString failed unexpectedly");
        let cpu = CString::new(cpu).expect("Conversion to CString failed unexpectedly");
        let features = CString::new(features).expect("Conversion to CString failed unexpectedly");
        let level = match level {
            OptimizationLevel::None => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            OptimizationLevel::Less => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
            OptimizationLevel::Default => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            OptimizationLevel::Aggressive => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
        };
        let code_model = match code_model {
            CodeModel::Default => LLVMCodeModel::LLVMCodeModelDefault,
            CodeModel::JITDefault => LLVMCodeModel::LLVMCodeModelJITDefault,
            CodeModel::Small => LLVMCodeModel::LLVMCodeModelSmall,
            CodeModel::Kernel => LLVMCodeModel::LLVMCodeModelKernel,
            CodeModel::Medium => LLVMCodeModel::LLVMCodeModelMedium,
            CodeModel::Large => LLVMCodeModel::LLVMCodeModelLarge,
        };
        let reloc_mode = match reloc_mode {
            RelocMode::Default => LLVMRelocMode::LLVMRelocDefault,
            RelocMode::Static => LLVMRelocMode::LLVMRelocStatic,
            RelocMode::PIC => LLVMRelocMode::LLVMRelocPIC,
            RelocMode::DynamicNoPic => LLVMRelocMode::LLVMRelocDynamicNoPic,
        };
        let target_machine = unsafe {
            LLVMCreateTargetMachine(self.target, triple.as_ptr(), cpu.as_ptr(), features.as_ptr(), level, reloc_mode, code_model)
        };

        if target_machine.is_null() {
            return None;
        }

        Some(TargetMachine::new(target_machine))
    }

    pub fn get_first() -> Option<Self> {
        let target = unsafe {
            LLVMGetFirstTarget()
        };

        if target.is_null() {
            return None;
        }

        Some(Target::new(target))
    }

    pub fn get_next(&self) -> Option<Target> {
        let target = unsafe {
            LLVMGetNextTarget(self.target)
        };

        if target.is_null() {
            return None;
        }

        Some(Target::new(target))
    }

    pub fn get_name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetName(self.target))
        }
    }

    pub fn get_description(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetDescription(self.target))
        }
    }

    pub fn from_name(name: &str) -> Target { // REVIEW: Option?
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let target = unsafe {
            LLVMGetTargetFromName(c_string.as_ptr())
        };

        Target::new(target)
    }

    pub fn from_triple(triple: &str) -> Result<Target, LLVMString> {
        let c_string = CString::new(triple).expect("Conversion to CString failed unexpectedly");
        let mut target = ptr::null_mut();
        let mut err_string = unsafe { zeroed() };

        let code = unsafe {
            LLVMGetTargetFromTriple(c_string.as_ptr(), &mut target, &mut err_string)
        };

        if code == 1 { // REVIEW: 1 is error value
            return Err(LLVMString::new(err_string));
        }

        Ok(Target::new(target))
    }

    pub fn has_jit(&self) -> bool {
        unsafe {
            LLVMTargetHasJIT(self.target) == 1
        }
    }

    pub fn has_target_machine(&self) -> bool {
        unsafe {
            LLVMTargetHasTargetMachine(self.target) == 1
        }
    }

    pub fn has_asm_backend(&self) -> bool {
        unsafe {
            LLVMTargetHasAsmBackend(self.target) == 1
        }
    }
}

#[derive(Debug)]
pub struct TargetMachine {
    target_machine: LLVMTargetMachineRef,
}

impl TargetMachine {
    fn new(target_machine: LLVMTargetMachineRef) -> Self {
        assert!(!target_machine.is_null());

        TargetMachine {
            target_machine,
        }
    }

    pub fn get_target(&self)-> Target {
        let target = unsafe {
            LLVMGetTargetMachineTarget(self.target_machine)
        };

        Target::new(target)
    }

    pub fn get_triple(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetMachineTriple(self.target_machine))
        }
    }

    pub fn get_default_triple() -> &'static CStr { // FIXME: Probably not static?
        unsafe {
            CStr::from_ptr(LLVMGetDefaultTargetTriple())
        }
    }

    pub fn get_cpu(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetMachineCPU(self.target_machine))
        }
    }

    pub fn get_feature_string(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(LLVMGetTargetMachineFeatureString(self.target_machine))
        }
    }

    pub fn set_asm_verbosity(&self, verbosity: bool) {
        unsafe {
            LLVMSetTargetMachineAsmVerbosity(self.target_machine, verbosity as i32)
        }
    }

    // TODO: Move to PassManager?
    pub fn add_analysis_passes(&self, pass_manager: &PassManager) {
        unsafe {
            LLVMAddAnalysisPasses(self.target_machine, pass_manager.pass_manager)
        }
    }

    pub fn write_to_memory_buffer(&self, module: &Module, file_type: FileType) -> Result<MemoryBuffer, LLVMString> {
        let mut memory_buffer = ptr::null_mut();
        let mut err_string = unsafe { zeroed() };
        let return_code = unsafe {
            LLVMTargetMachineEmitToMemoryBuffer(self.target_machine, module.module.get(), file_type.as_llvm_file_type(), &mut err_string, &mut memory_buffer)
        };

        // TODO: Verify 1 is error code (LLVM can be inconsistent)
        if return_code == 1 {
            return Err(LLVMString::new(err_string));
        }

        Ok(MemoryBuffer::new(memory_buffer))
    }

    pub fn write_to_file(&self, module: &Module, file_type: FileType, path: &Path) -> Result<(), LLVMString> {
        let path = path.to_str().expect("Did not find a valid Unicode path string");
        let mut err_string = unsafe { zeroed() };
        let return_code = unsafe {
            // REVIEW: Why does LLVM need a mutable reference to path...?
            LLVMTargetMachineEmitToFile(self.target_machine, module.module.get(), path.as_ptr() as *mut i8, file_type.as_llvm_file_type(), &mut err_string)
        };

        // TODO: Verify 1 is error code (LLVM can be inconsistent)
        if return_code == 1 {
            return Err(LLVMString::new(err_string));
        }

        Ok(())
    }
}

impl Drop for TargetMachine {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeTargetMachine(self.target_machine)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ByteOrdering {
    BigEndian,
    LittleEndian,
}

#[derive(PartialEq, Eq, Debug)]
pub struct TargetData {
    pub(crate) target_data: LLVMTargetDataRef,
}

impl TargetData {
    pub(crate) fn new(target_data: LLVMTargetDataRef) -> TargetData {
        assert!(!target_data.is_null());

        TargetData {
            target_data: target_data
        }
    }

    pub fn int_ptr_type(&self) -> PointerType {
        let ptr_type = unsafe {
            LLVMIntPtrType(self.target_data)
        };

        PointerType::new(ptr_type)
    }

    pub fn int_ptr_type_for_as(&self, as_: u32) -> PointerType {
        let ptr_type = unsafe {
            LLVMIntPtrTypeForAS(self.target_data, as_)
        };

        PointerType::new(ptr_type)
    }

    pub fn int_ptr_type_in_context(&self, context: &Context) -> PointerType {
        let ptr_type = unsafe {
            LLVMIntPtrTypeInContext(*context.context, self.target_data)
        };

        PointerType::new(ptr_type)
    }

    pub fn int_ptr_type_for_as_in_context(&self, context: &Context, as_: u32) -> PointerType {
        let ptr_type = unsafe {
            LLVMIntPtrTypeForASInContext(*context.context, self.target_data, as_)
        };

        PointerType::new(ptr_type)
    }

    pub fn get_data_layout(&self) -> DataLayout {
        let data_layout = unsafe {
            LLVMCopyStringRepOfTargetData(self.target_data)
        };

        DataLayout::new_owned(data_layout)
    }

    // REVIEW: Does this only work if Sized?
    pub fn get_bit_size(&self, type_: &AnyType) -> u64 {
        unsafe {
            LLVMSizeOfTypeInBits(self.target_data, type_.as_type_ref())
        }
    }

    pub fn create(str_repr: &str) -> TargetData {
        let c_string = CString::new(str_repr).expect("Conversion to CString failed unexpectedly");

        let target_data = unsafe {
            LLVMCreateTargetData(c_string.as_ptr())
        };

        TargetData::new(target_data)
    }

    #[cfg(any(feature = "llvm3-6", feature = "llvm3-7", feature = "llvm3-8"))]
    pub fn add_target_data(&self, pass_manager: &PassManager) {
        unsafe {
            LLVMAddTargetData(self.target_data, pass_manager.pass_manager)
        }
    }

    pub fn get_byte_ordering(&self) -> ByteOrdering {
        let byte_ordering = unsafe {
            LLVMByteOrder(self.target_data)
        };

        match byte_ordering {
            LLVMByteOrdering::LLVMBigEndian => ByteOrdering::BigEndian,
            LLVMByteOrdering::LLVMLittleEndian => ByteOrdering::LittleEndian,
        }
    }

    pub fn get_pointer_byte_size(&self) -> u32 {
        unsafe {
            LLVMPointerSize(self.target_data)
        }
    }

    pub fn get_pointer_byte_size_for_as(&self, as_: u32) -> u32 {
        unsafe {
            LLVMPointerSizeForAS(self.target_data, as_)
        }
    }

    pub fn get_store_size(&self, type_: &AnyType) -> u64 {
        unsafe {
            LLVMStoreSizeOfType(self.target_data, type_.as_type_ref())
        }
    }

    pub fn get_abi_size(&self, type_: &AnyType) -> u64 {
        unsafe {
            LLVMABISizeOfType(self.target_data, type_.as_type_ref())
        }
    }

    pub fn get_abi_alignment(&self, type_: &AnyType) -> u32 {
        unsafe {
            LLVMABIAlignmentOfType(self.target_data, type_.as_type_ref())
        }
    }

    pub fn get_call_frame_alignment(&self, type_: &AnyType) -> u32 {
        unsafe {
            LLVMCallFrameAlignmentOfType(self.target_data, type_.as_type_ref())
        }
    }

    pub fn get_preferred_alignment(&self, type_: &AnyType) -> u32 {
        unsafe {
            LLVMPreferredAlignmentOfType(self.target_data, type_.as_type_ref())
        }
    }

    pub fn get_preferred_alignment_of_global(&self, value: &GlobalValue) -> u32 {
        unsafe {
            LLVMPreferredAlignmentOfGlobal(self.target_data, value.as_value_ref())
        }
    }

    pub fn element_at_offset(&self, struct_type: &StructType, offset: u64) -> u32 {
        unsafe {
            LLVMElementAtOffset(self.target_data, struct_type.as_type_ref(), offset)
        }
    }

    pub fn offset_of_element(&self, struct_type: &StructType, element: u32) -> Option<u64> {
        if element > struct_type.count_fields() - 1 {
            return None;
        }

        unsafe {
            Some(LLVMOffsetOfElement(self.target_data, struct_type.as_type_ref(), element))
        }
    }
}

// FIXME: Make sure this doesn't SegFault:
impl Drop for TargetData {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeTargetData(self.target_data)
        }
    }
}
