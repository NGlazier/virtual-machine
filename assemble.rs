//! Grumpy assembler.
//!
//! This module contains the assembler that translates
//! pseudo-instruction (assembly) programs into native
//! programs by resolving label addresses.

use std::collections::HashMap;
use crate::isa::{*, Instr::*, PInstr::*, Val::*};

/// Translate an assembly program to an equivalent native program.
pub fn assemble(pinstrs : Vec<PInstr>) -> Result<Vec<Instr>, String> {
    // Fill in your solution.
    unimplemented!()
}
