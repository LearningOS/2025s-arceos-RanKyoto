#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::collections::*;

pub mod hashmap; 
pub use hashmap::HashMap;