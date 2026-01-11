mod clang;
mod gcc;
mod intel;
mod msvc;
mod traits;

pub use clang::Clang;
pub use gcc::Gcc;
pub use intel::Intel;
pub use msvc::Msvc;

pub use traits::{Archiver, CCompiler, CppCompiler, Linker};

pub const PROFILE_DEFINE_TEMPLATE: &str = "RF_PROFIL_";
