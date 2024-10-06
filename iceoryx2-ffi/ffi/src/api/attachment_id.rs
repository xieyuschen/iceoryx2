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

use std::mem::ManuallyDrop;

use iceoryx2::{
    prelude::AttachmentId,
    service::{ipc, local},
};
use iceoryx2_bb_elementary::static_assert::*;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::iox2_service_type_e;

use super::{AssertNonNullHandle, HandleToType};

// BEGIN types definition
pub(super) union AttachmentIdUnion {
    ipc: ManuallyDrop<AttachmentId<ipc::Service>>,
    local: ManuallyDrop<AttachmentId<local::Service>>,
}

impl AttachmentIdUnion {
    pub(super) fn new_ipc(attachment: AttachmentId<ipc::Service>) -> Self {
        Self {
            ipc: ManuallyDrop::new(attachment),
        }
    }

    pub(super) fn new_local(attachment: AttachmentId<local::Service>) -> Self {
        Self {
            local: ManuallyDrop::new(attachment),
        }
    }
}

#[repr(C)]
#[repr(align(8))] // alignment of Option<AttachmentIdUnion>
pub struct iox2_attachment_id_storage_t {
    internal: [u8; 32], // magic number obtained with size_of::<Option<AttachmentIdUnion>>()
}

#[repr(C)]
#[iceoryx2_ffi(AttachmentIdUnion)]
pub struct iox2_attachment_id_t {
    service_type: iox2_service_type_e,
    value: iox2_attachment_id_storage_t,
    deleter: fn(*mut iox2_attachment_id_t),
}

impl iox2_attachment_id_t {
    pub(super) fn init(
        &mut self,
        service_type: iox2_service_type_e,
        value: AttachmentIdUnion,
        deleter: fn(*mut iox2_attachment_id_t),
    ) {
        self.service_type = service_type;
        self.value.init(value);
        self.deleter = deleter;
    }
}

pub struct iox2_attachment_id_h_t;
/// The owning handle for `iox2_attachment_id_t`. Passing the handle to an function transfers the ownership.
pub type iox2_attachment_id_h = *mut iox2_attachment_id_h_t;
/// The non-owning handle for `iox2_attachment_id_t`. Passing the handle to an function does not transfers the ownership.
pub type iox2_attachment_id_h_ref = *const iox2_attachment_id_h;

impl AssertNonNullHandle for iox2_attachment_id_h {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
    }
}

impl AssertNonNullHandle for iox2_attachment_id_h_ref {
    fn assert_non_null(self) {
        debug_assert!(!self.is_null());
        unsafe {
            debug_assert!(!(*self).is_null());
        }
    }
}

impl HandleToType for iox2_attachment_id_h {
    type Target = *mut iox2_attachment_id_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_attachment_id_h_ref {
    type Target = *mut iox2_attachment_id_t;

    fn as_type(self) -> Self::Target {
        unsafe { *self as *mut _ as _ }
    }
}
// END type definition

// BEGIN C API
#[no_mangle]
pub unsafe extern "C" fn iox2_attachment_id_drop(handle: iox2_attachment_id_h) {
    handle.assert_non_null();

    let attachment_id = &mut *handle.as_type();

    match attachment_id.service_type {
        iox2_service_type_e::IPC => {
            ManuallyDrop::drop(&mut attachment_id.value.as_mut().ipc);
        }
        iox2_service_type_e::LOCAL => {
            ManuallyDrop::drop(&mut attachment_id.value.as_mut().local);
        }
    }
    (attachment_id.deleter)(attachment_id);
}
// END C API
