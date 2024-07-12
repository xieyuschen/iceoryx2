// Copyright (c) 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(non_camel_case_types)]

use crate::{iox2_semantic_string_error_e, IntoCInt, IOX2_OK};

use iceoryx2::prelude::*;
use iceoryx2_bb_elementary::static_assert::*;

use core::ffi::{c_char, c_int};
use core::mem::{align_of, size_of, MaybeUninit};
use core::{slice, str};
use std::alloc::{alloc, dealloc, Layout};

// BEGIN type definition

#[repr(C)]
#[repr(align(8))] // alignment of Option<NodeName>
pub struct iox2_node_name_storage_t {
    internal: [u8; 24], // magic number obtained with size_of::<Option<NodeName>>()
}

impl iox2_node_name_storage_t {
    const fn assert_storage_layout() {
        static_assert_ge::<
            { align_of::<iox2_node_name_storage_t>() },
            { align_of::<Option<NodeName>>() },
        >();
        static_assert_ge::<
            { size_of::<iox2_node_name_storage_t>() },
            { size_of::<Option<NodeName>>() },
        >();
    }

    fn init(&mut self, node_name: NodeName) {
        iox2_node_name_storage_t::assert_storage_layout();

        unsafe { &mut *(self as *mut Self).cast::<MaybeUninit<Option<NodeName>>>() }
            .write(Some(node_name));
    }

    unsafe fn assume_init_mut(&mut self) -> &mut Option<NodeName> {
        (*(self as *mut Self).cast::<MaybeUninit<Option<NodeName>>>()).assume_init_mut()
    }

    unsafe fn assume_init_ref(&self) -> &Option<NodeName> {
        (*(self as *const Self).cast::<MaybeUninit<Option<NodeName>>>()).assume_init_ref()
    }
}

#[repr(C)]
pub struct iox2_node_name_t {
    /// cbindgen:rename=internal
    node_name: iox2_node_name_storage_t,
    deleter: fn(*mut iox2_node_name_t),
}

impl iox2_node_name_t {
    pub(crate) fn cast(node_name: iox2_node_name_h) -> *mut Self {
        node_name as *mut _ as *mut Self
    }

    pub(crate) fn cast_node_name(node_name_ptr: iox2_node_name_ptr) -> *const NodeName {
        debug_assert!(!node_name_ptr.is_null());
        let maybe_node_name =
            unsafe { (*(node_name_ptr as *const _ as *const Option<NodeName>)).as_ref() };
        debug_assert!(maybe_node_name.is_some());
        unsafe { maybe_node_name.unwrap_unchecked() as *const _ }
    }

    pub(crate) fn take(&mut self) -> Option<NodeName> {
        unsafe { self.node_name.assume_init_mut().take() }
    }

    fn alloc() -> *mut iox2_node_name_t {
        unsafe { alloc(Layout::new::<iox2_node_name_t>()) as *mut iox2_node_name_t }
    }
    fn dealloc(storage: *mut iox2_node_name_t) {
        unsafe {
            dealloc(storage as *mut _, Layout::new::<iox2_node_name_t>());
        }
    }
}

pub struct iox2_node_name_h_t;
/// The handle for `iox2_node_name_t`. Passing the handle to an function transfers the ownership.
pub type iox2_node_name_h = *mut iox2_node_name_h_t;

pub struct iox2_node_name_ptr_t;
/// The immutable pointer to the underlying `NodeName`
pub type iox2_node_name_ptr = *const iox2_node_name_ptr_t;

pub struct iox2_node_name_mut_ptr_t;
/// The mutable pointer to the underlying `NodeName`
pub type iox2_node_name_mut_ptr = *mut iox2_node_name_mut_ptr_t;

// END type definition

// BEGIN C API

/// This function create a new node name!
///
/// # Arguments
///
/// * `node_name_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_node_name_t`]. If it is a NULL pointer, the storage will be allocated on the heap.
/// * `node_name_str` - Must be valid node name string.
/// * `node_name_len` - The length of the node name string, not including a null termination.
/// * `node_name_handle_ptr` - An uninitialized or dangling [`iox2_node_name_h`] handle which will be initialized by this function call.
///
/// Returns IOX2_OK on success, an [`iox2_semantic_string_error_e`](crate::iox2_semantic_string_error_e) otherwise.
///
/// # Safety
///
/// Terminates if `node_name_str` or `node_name_handle_ptr` is a NULL pointer!
/// It is undefined behavior to pass a `node_name_len` which is larger than the actual length of `node_name_str`!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name_new(
    node_name_struct_ptr: *mut iox2_node_name_t,
    node_name_str: *const c_char,
    node_name_len: c_int,
    node_name_handle_ptr: *mut iox2_node_name_h,
) -> c_int {
    debug_assert!(!node_name_str.is_null());
    debug_assert!(!node_name_handle_ptr.is_null());

    *node_name_handle_ptr = std::ptr::null_mut();

    let mut handle = node_name_struct_ptr;
    fn no_op(_storage: *mut iox2_node_name_t) {}
    let mut deleter: fn(*mut iox2_node_name_t) = no_op;
    if handle.is_null() {
        handle = iox2_node_name_t::alloc();
        deleter = iox2_node_name_t::dealloc;
    }
    debug_assert!(!handle.is_null());

    unsafe {
        (*handle).deleter = deleter;
    }

    let node_name = slice::from_raw_parts(node_name_str as *const _, node_name_len as usize);

    let node_name = if let Ok(node_name) = str::from_utf8(node_name) {
        node_name
    } else {
        deleter(handle);
        return iox2_semantic_string_error_e::INVALID_CONTENT as c_int;
    };

    let node_name = match NodeName::new(node_name) {
        Ok(node_name) => node_name,
        Err(e) => {
            deleter(handle);
            return e.into_c_int();
        }
    };

    unsafe {
        (*handle).node_name.init(node_name);
    }

    *node_name_handle_ptr = handle as *mut _ as *mut _;

    IOX2_OK
}

/// This function casts a [`iox2_node_name_h`] into a [`iox2_node_name_ptr`]
///
/// # Arguments
///
/// * `node_name_handle` obtained by [`iox2_node_name_new`]
///
/// Returns a [`iox2_node_name_ptr`]
///
/// # Safety
///
/// The `node_name_handle` must be a valid handle.
/// The `node_name_handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_node_name_ptr(
    node_name_handle: iox2_node_name_h,
) -> iox2_node_name_ptr {
    debug_assert!(!node_name_handle.is_null());

    (*iox2_node_name_t::cast(node_name_handle))
        .node_name
        .assume_init_ref() as *const _ as *const _
}

/// This function gives access to the node name as a C-style string
///
/// # Arguments
///
/// * `node_name_ptr` obtained by e.g. [`iox2_cast_node_name_ptr`] or a function returning a [`iox2_node_name_ptr`]
/// * `node_name_len` can be used to get the length of the C-style string if not `NULL`
///
/// Returns zero terminated C-style string
///
/// # Safety
///
/// The `node_name_ptr` must be a valid pointer to a node name.
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name_as_c_str(
    node_name_ptr: iox2_node_name_ptr,
    node_name_len: *mut c_int,
) -> *const c_char {
    debug_assert!(!node_name_ptr.is_null());

    let node_name = &*iox2_node_name_t::cast_node_name(node_name_ptr);

    if !node_name_len.is_null() {
        unsafe {
            *node_name_len = node_name.len() as _;
        }
    }

    node_name.as_str().as_ptr() as *const _
}

/// This function needs to be called to destroy the node name!
///
/// In general, this function is not required to call, since [`iox2_node_builder_set_name`](crate::iox2_node_builder_set_name) will consume the [`iox2_node_name_h`] handle.
///
/// # Arguments
///
/// * `node_name_handle` - A valid [`iox2_node_name_h`]
///
/// # Safety
///
/// The `node_name_handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// The corresponding [`iox2_node_name_t`] can be re-used with a call to [`iox2_node_name_new`]!
#[no_mangle]
pub unsafe extern "C" fn iox2_node_name_drop(node_name_handle: iox2_node_name_h) {
    debug_assert!(!node_name_handle.is_null());

    let node_name_struct = &mut (*iox2_node_name_t::cast(node_name_handle));

    node_name_struct.node_name.assume_init_mut().take();
    (node_name_struct.deleter)(node_name_struct);
}

// END C API

#[cfg(test)]
mod test {
    use super::*;

    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn assert_storage_size() {
        // all const functions; if it compiles, the storage size is sufficient
        const _STORAGE_LAYOUT_CHECK: () = iox2_node_name_storage_t::assert_storage_layout();
    }

    #[test]
    fn basic_node_name_test() -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let expected_node_name = NodeName::new("hypnotaod")?;

            let mut node_name_handle: iox2_node_name_h = std::ptr::null_mut();
            let ret_val = iox2_node_name_new(
                std::ptr::null_mut(),
                expected_node_name.as_str().as_ptr() as *const _,
                expected_node_name.len() as _,
                &mut node_name_handle,
            );
            assert_that!(ret_val, eq(IOX2_OK));

            let mut node_name_len = 0;
            let node_name_c_str = iox2_node_name_as_c_str(
                iox2_cast_node_name_ptr(node_name_handle),
                &mut node_name_len,
            );

            let slice = slice::from_raw_parts(node_name_c_str as *const _, node_name_len as _);
            let node_name = str::from_utf8(slice)?;

            assert_that!(node_name, eq(expected_node_name.as_str()));

            iox2_node_name_drop(node_name_handle);

            Ok(())
        }
    }
}