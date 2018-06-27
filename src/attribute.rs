/// An `Attribute` is a piece of information that can be attached to functions and call sites.

use llvm_sys::prelude::LLVMAttributeRef;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AttrKind {
    Alignment,
    AllocSize,
    AlwaysInline,
    ArgMemOnly,
    Builtin,
    ByVal,
    Cold,
    Convergent,
    Dereferenceable,
    DereferenceableOrNull,
    InAlloca,
    InReg,
    InaccessibleMemOnly,
    InaccessibleMemOrArgMemOnly,
    InlineHint,
    JumpTable,
    MinSize,
    Naked,
    Nest,
    NoAlias,
    NoBuiltin,
    NoCapture,
    NoDuplicate,
    NoImplicitFloat,
    NoInline,
    NoRecurse,
    NoRedZone,
    NoReturn,
    NoUnwind,
    NonLazyBind,
    NonNull,
    OptimizeForSize,
    OptimizeNone,
    ReadNone,
    ReadOnly,
    Returned,
    ReturnsTwice,
    SExt,
    SafeStack,
    SanitizeAddress,
    SanitizeHWAddress,
    SanitizeMemory,
    SanitizeThread,
    Speculatable,
    StackAlignment,
    StackProtect,
    StackProtectReq,
    StackProtectStrong,
    StrictFP,
    StructRet,
    SwiftError,
    SwiftSelf,
    UWTable,
    WriteOnly,
    ZExt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribute {
    pub(crate) attr: LLVMAttributeRef,
}

impl Attribute {
    pub(crate) fn new(attr: LLVMAttributeRef) -> Self {
        Attribute {
            attr,
        }
    }
}
