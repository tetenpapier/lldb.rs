// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{
    sys, SBFileSpec, SBSection, SBStream, SBSymbolContextList, SBTypeList, SymbolType, TypeClass,
};
use std::ffi::CString;
use std::fmt;

/// An executable image and its associated object and symbol files.
pub struct SBModule {
    /// The underlying raw `SBModuleRef`.
    pub raw: sys::SBModuleRef,
}

impl SBModule {
    /// Construct a new `Some(SBModule)` or `None`.
    pub fn maybe_wrap(raw: sys::SBModuleRef) -> Option<SBModule> {
        if unsafe { sys::SBModuleIsValid(raw) } {
            Some(SBModule { raw })
        } else {
            None
        }
    }

    /// Check whether or not this is a valid `SBModule` value.
    pub fn is_valid(&self) -> bool {
        unsafe { sys::SBModuleIsValid(self.raw) }
    }

    /// The file for the module on the host system that is running LLDB.
    ///
    /// This can differ from the path on the platform since we might
    /// be doing remote debugging.
    pub fn filespec(&self) -> SBFileSpec {
        SBFileSpec::from(unsafe { sys::SBModuleGetFileSpec(self.raw) })
    }

    /// The file for the module as it is known on the remote system on
    /// which it is being debugged.
    ///
    /// For local debugging this is always the same as `SBModule::filespec`.
    /// But remote debugging might mention a file `/usr/lib/liba.dylib`
    /// which might be locally downloaded and cached. In this case the
    /// platform file could be something like:
    /// `/tmp/lldb/platform-cache/remote.host.computer/usr/lib/liba.dylib`
    /// The file could also be cached in a local developer kit directory.
    pub fn platform_filespec(&self) -> SBFileSpec {
        SBFileSpec::from(unsafe { sys::SBModuleGetPlatformFileSpec(self.raw) })
    }

    #[allow(missing_docs)]
    pub fn find_section(&self, name: &str) -> Option<SBSection> {
        let name = CString::new(name).unwrap();
        SBSection::maybe_wrap(unsafe { sys::SBModuleFindSection(self.raw, name.as_ptr()) })
    }

    /// Get an iterator over the [sections] known to this module instance.
    ///
    /// [sections]: SBSection
    pub fn sections(&self) -> SBModuleSectionIter {
        SBModuleSectionIter {
            module: self,
            idx: 0,
        }
    }

    #[allow(missing_docs)]
    pub fn find_functions(&self, name: &str, name_type_mask: u32) -> SBSymbolContextList {
        let name = CString::new(name).unwrap();
        SBSymbolContextList::from(unsafe {
            sys::SBModuleFindFunctions(self.raw, name.as_ptr(), name_type_mask)
        })
    }

    #[allow(missing_docs)]
    pub fn find_symbols(&self, name: &str, symbol_type: SymbolType) -> SBSymbolContextList {
        let name = CString::new(name).unwrap();
        SBSymbolContextList::from(unsafe {
            sys::SBModuleFindSymbols(self.raw, name.as_ptr(), symbol_type)
        })
    }

    /// Get all types matching `type_mask` from the debug info in this
    /// module.
    ///
    /// `type_mask` is a bitfield consisting of one or more type classes.
    /// This allows you to request only structure types, or only class,
    /// structure, and union types. Passing in [`TypeClass::ANY`] will
    /// return all types found in the debug information for this module.
    pub fn types(&self, type_mask: TypeClass) -> SBTypeList {
        SBTypeList::from(unsafe { sys::SBModuleGetTypes(self.raw, type_mask.bits()) })
    }
}

/// Iterate over the [sections] in a [module].
///
/// [sections]: SBSection
/// [module]: SBModule
pub struct SBModuleSectionIter<'d> {
    module: &'d SBModule,
    idx: usize,
}

impl<'d> Iterator for SBModuleSectionIter<'d> {
    type Item = SBSection;

    fn next(&mut self) -> Option<SBSection> {
        if self.idx < unsafe { sys::SBModuleGetNumSections(self.module.raw) } {
            let r = Some(SBSection::from(unsafe {
                sys::SBModuleGetSectionAtIndex(self.module.raw, self.idx)
            }));
            self.idx += 1;
            r
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = unsafe { sys::SBModuleGetNumSections(self.module.raw) };
        (sz - self.idx, Some(sz))
    }
}

impl<'d> ExactSizeIterator for SBModuleSectionIter<'d> {}

impl Clone for SBModule {
    fn clone(&self) -> SBModule {
        SBModule {
            raw: unsafe { sys::CloneSBModule(self.raw) },
        }
    }
}

impl fmt::Debug for SBModule {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let stream = SBStream::new();
        unsafe { sys::SBModuleGetDescription(self.raw, stream.raw) };
        write!(fmt, "SBModule {{ {} }}", stream.data())
    }
}

impl Drop for SBModule {
    fn drop(&mut self) {
        unsafe { sys::DisposeSBModule(self.raw) };
    }
}

impl From<sys::SBModuleRef> for SBModule {
    fn from(raw: sys::SBModuleRef) -> SBModule {
        SBModule { raw }
    }
}

unsafe impl Send for SBModule {}
unsafe impl Sync for SBModule {}

#[cfg(feature = "graphql")]
graphql_object!(SBModule: crate::SBDebugger | &self | {
    field is_valid() -> bool {
        self.is_valid()
    }

    field filespec() -> SBFileSpec {
        self.filespec()
    }

    field platform_filespec() -> SBFileSpec {
        self.platform_filespec()
    }

    field sections() -> Vec<SBSection> {
        self.sections().collect()
    }
});
