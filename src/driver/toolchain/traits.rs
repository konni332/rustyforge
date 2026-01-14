use std::path::{Path, PathBuf};

use crate::{
    CoreResult, TargetKind,
    driver::{
        cannonical_command::{CannonicalCommand, CannonicalCommandBuilder},
        compiler::Editions,
        runtime::{Profile, Target},
        toolchain::PROFILE_DEFINE_TEMPLATE,
    },
};

/// Abstraction over a C compiler backend (e.g. GCC, Clang, MSVC).
///
/// Implementations are responsible for translating high-level build
/// configuration into concrete compiler invocations and dependency
/// discovery mechanisms.
pub trait CCompiler: Send + Sync {
    /// Creates a new compiler instance.
    ///
    /// Implementations are expected to be lightweight and stateless.
    fn new() -> Self
    where
        Self: Sized;

    /// Generates a compile command for a single C translation unit.
    ///
    /// The returned command must compile `path` into the given object file
    /// without performing any linking.
    ///
    /// # Parameters
    ///
    /// - `path`: Source file to compile.
    /// - `output`: Path of the resulting object file.
    /// - `profile`: Active build profile (flags, optimization, debug, LTO).
    /// - `target`: Target-specific configuration (defines, features, ignores).
    /// - `includes`: List of include directories passed to the compiler.
    fn compile_unit_cmd(
        &self,
        path: &Path,
        output: &Path,
        profile: &Profile,
        target: &Target,
        includes: &[PathBuf],
        editions: Editions,
    ) -> CoreResult<CannonicalCommand>;

    /// Resolves all header file dependencies for a given C source file.
    ///
    /// This is typically implemented using compiler-specific dependency
    /// generation flags (e.g. `-M`, `-MM`, or equivalent).
    ///
    /// The returned paths are used for incremental builds and cache invalidation.
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &Profile,
        target: &Target,
        includes: &[PathBuf],
        editions: Editions,
    ) -> CoreResult<Vec<PathBuf>>;
}

/// Abstraction over a C++ compiler backend (e.g. G++, Clang++, MSVC).
///
/// Semantically identical to [`CCompiler`], but separated to allow
/// toolchain-specific behavior, flags, and compatibility rules.
pub trait CppCompiler: Send + Sync {
    /// Creates a new compiler instance.
    ///
    /// Implementations are expected to be lightweight and stateless.
    fn new() -> Self
    where
        Self: Sized;

    /// Generates a compile command for a single C++ translation unit.
    ///
    /// The returned command must compile `path` into the given object file
    /// without performing any linking.
    fn compile_unit_cmd(
        &self,
        path: &Path,
        output: &Path,
        profile: &Profile,
        target: &Target,
        includes: &[PathBuf],
        editions: Editions,
    ) -> CoreResult<CannonicalCommand>;

    /// Resolves all header file dependencies for a given C++ source file.
    ///
    /// This is used to track include changes and determine whether a
    /// translation unit must be recompiled.
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &Profile,
        target: &Target,
        includes: &[PathBuf],
        editions: Editions,
    ) -> CoreResult<Vec<PathBuf>>;
}

/// Abstraction over a linker backend (e.g. ld, lld, link.exe, compiler drivers).
///
/// The linker is responsible for producing final artifacts such as
/// executables or shared libraries from compiled object files.
pub trait Linker {
    /// Creates a new linker instance.
    ///
    /// Implementations are expected to be lightweight and stateless.
    fn new() -> Self
    where
        Self: Sized;

    /// Generates a link command for a target.
    ///
    /// The linker must select the correct invocation based on whether C++
    /// objects are present, the active profile, and the target type
    /// (binary or library).
    ///
    /// # Parameters
    ///
    /// - `target`: Target metadata (kind, name, flags, defines).
    /// - `profile`: Active build profile.
    /// - `objs`: Object files to link.
    /// - `contains_cpp`: Whether any object file was compiled as C++.
    /// - `lib_dirs`: Library search paths passed to the linker.
    /// - `output`: Path of the resulting artifact.
    fn link_objects(
        &self,
        target: &Target,
        profile: &Profile,
        objs: &[PathBuf],
        contains_cpp: bool,
        lib_dirs: &[PathBuf],
        output: &Path,
    ) -> CoreResult<CannonicalCommand>;
}

/// Abstraction over an archive tool used to build static libraries.
///
/// Typical implementations include `ar`, `llvm-ar`, and `lib.exe`.
pub trait Archiver {
    /// Creates a new archiver instance.
    ///
    /// Implementations are expected to be lightweight and stateless.
    fn new() -> Self
    where
        Self: Sized;

    /// Generates a command that archives object files into a static library.
    ///
    /// The resulting command must produce a deterministic archive suitable
    /// for linking.
    ///
    /// # Parameters
    ///
    /// - `objs`: Object files to archive.
    /// - `output`: Path of the resulting static library.
    fn archiver_objects(&self, objs: &[PathBuf], output: &Path) -> CoreResult<CannonicalCommand>;
}
