//! Translate types from LLVM IR to Cretonne IL.

use cretonne::ir;
use std::ptr;
use llvm_sys::prelude::*;
use llvm_sys::core::*;
use llvm_sys::target::*;
use llvm_sys::LLVMTypeKind::*;

/// Return a Cretonne integer type with the given bit width.
pub fn translate_integer_type(bitwidth: usize) -> ir::Type {
    match bitwidth {
        1 => ir::types::B1,
        8 => ir::types::I8,
        16 => ir::types::I16,
        32 => ir::types::I32,
        64 => ir::types::I64,
        width => panic!("unimplemented integer bit width {}", width),
    }
}

/// Return the Cretonne integer type for a pointer.
pub fn translate_pointer_type(dl: LLVMTargetDataRef) -> ir::Type {
    translate_integer_type(unsafe { LLVMPointerSize(dl) * 8 } as usize)
}

/// Translate an LLVM first-class type into a Cretonne type.
pub fn translate_type(llvm_ty: LLVMTypeRef, dl: LLVMTargetDataRef) -> ir::Type {
    match unsafe { LLVMGetTypeKind(llvm_ty) } {
        LLVMVoidTypeKind => ir::types::VOID,
        LLVMHalfTypeKind => panic!("unimplemented: f16 type"),
        LLVMFloatTypeKind => ir::types::F32,
        LLVMDoubleTypeKind => ir::types::F64,
        LLVMX86_FP80TypeKind => panic!("unimplemented: x86_fp80 type"),
        LLVMFP128TypeKind => panic!("unimplemented: f128 type"),
        LLVMPPC_FP128TypeKind => panic!("unimplemented: double double type"),
        LLVMLabelTypeKind => panic!("unimplemented: label types"),
        LLVMIntegerTypeKind => translate_integer_type(
            unsafe { LLVMGetIntTypeWidth(llvm_ty) } as usize,
        ),
        LLVMFunctionTypeKind => panic!("use translate_sig to translate function types"),
        LLVMStructTypeKind => panic!("unimplemented: first-class struct types"),
        LLVMArrayTypeKind => panic!("unimplemented: first-class array types"),
        LLVMPointerTypeKind => {
            if unsafe { LLVMGetPointerAddressSpace(llvm_ty) } != 0 {
                panic!("unimplemented: non-default address spaces");
            }
            translate_pointer_type(dl)
        }
        LLVMVectorTypeKind => panic!("unimplemented: vector types"),
        LLVMMetadataTypeKind => panic!("attempted to translate a metadata type"),
        LLVMX86_MMXTypeKind => panic!("unimplemented: MMX type"),
        LLVMTokenTypeKind => panic!("unimplemented: token types"),
    }
}

/// Translate an LLVM function type into a Cretonne signature.
pub fn translate_sig(llvm_ty: LLVMTypeRef, dl: LLVMTargetDataRef) -> ir::Signature {
    debug_assert_eq!(unsafe { LLVMGetTypeKind(llvm_ty) }, LLVMFunctionTypeKind);

    let mut sig = ir::Signature::new(ir::CallConv::Native);

    let num_llvm_params = unsafe { LLVMCountParamTypes(llvm_ty) } as usize;
    let mut llvm_params: Vec<LLVMTypeRef> = Vec::with_capacity(num_llvm_params);
    llvm_params.resize(num_llvm_params, ptr::null_mut());

    // TODO: First-class aggregate params and return values.

    unsafe { LLVMGetParamTypes(llvm_ty, llvm_params.as_mut_ptr()) };
    let mut params: Vec<ir::AbiParam> = Vec::with_capacity(num_llvm_params);
    for llvm_param in &llvm_params {
        params.push(ir::AbiParam::new(translate_type(*llvm_param, dl)));
    }
    sig.params = params;

    let mut returns: Vec<ir::AbiParam> = Vec::with_capacity(1);
    match translate_type(unsafe { LLVMGetReturnType(llvm_ty) }, dl) {
        ir::types::VOID => {}
        ty => returns.push(ir::AbiParam::new(ty)),
    }
    sig.returns = returns;

    sig
}
