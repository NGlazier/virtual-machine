//! GrumpyVM instruction set.
//!
//! This module contains the types of values and instructions
//! supported by GrumpyVM.

use self::{Binop::*, Instr::*, PInstr::*, Unop::*, Val::*};
use crate::{ParseError, FromBytes, ToBytes};
use byteorder::{BigEndian, ByteOrder};
use regex::Regex;
use std::str::FromStr;

/// Heap addresses.
pub type Address = usize;

/// GrumpyVM values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Val {
    // Value types that may appear in GrumpyVM programs:
    /// The unit value.
    Vunit,
    /// 32-bit signed integers.
    Vi32(i32),
    /// Booleans.
    Vbool(bool),
    /// Stack or instruction locations.
    Vloc(u32),
    /// The undefined value.
    Vundef,

    // Value types that are used internally by the language
    // implementation, and may not appear in GrumpyVM programs:
    /// Metadata for heap objects that span multiple values.
    Vsize(usize),
    /// Pointers to heap locations.
    Vaddr(Address),
}

/// Val methods.
impl Val {
    /// Try to extract an i32 from a Val.
    pub fn to_i32(&self) -> Option<i32> {
	match self {
	    Vi32(i) => Some(*i),
	    _ => None
	}
    }
    /// Try to extract a bool from a Val.
    pub fn to_bool(&self) -> Option<bool> {
	match self {
	    Vbool(b) => Some(*b),
	    _ => None
	}
    }
    /// Try to extract a loc (u32) from a Val.
    pub fn to_loc(&self) -> Option<u32> {
	match self {
	    Vloc(loc) => Some(*loc),
	    _ => None
	}
    }
    /// Try to extract an address (usize) from a Val.
    pub fn to_address(&self) -> Option<Address> {
	match self {
	    Vaddr(addr) => Some(*addr),
	    _ => None
	}
    }
}

/// GrumpyVM native instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    /// Push(v): Push value v onto the stack.
    Push(Val),
    /// Pop a value from the stack, discarding it.
    Pop,
    /// Peek(i): Push onto the stack the ith value from the top.
    Peek(u32),
    /// Unary(u): Apply u to the top value on the stack.
    Unary(Unop),
    /// Binary(b): Apply b to the top two values on the stack,
    /// replacing them with the result.
    Binary(Binop),
    /// Swap the top two values.
    Swap,
    /// Allocate an array on the heap.
    Alloc,
    /// Write to a heap-allocated array.
    Set,
    /// Read from a heap-allocated array.
    Get,
    /// Var(i): Get the value at stack position fp+i.
    Var(u32),
    /// Store(i): Store a value at stack position fp+i.
    Store(u32),
    /// SetFrame(i): Set fp = s.stack.len() - i.
    SetFrame(u32),
    /// Function call.
    Call,
    /// Function return.
    Ret,
    /// Conditional jump.
    Branch,
    /// Halt the machine.
    Halt,
}

/// Program labels.
pub type Label = String;

/// Pseudo-instructions, extending native instructions with support
/// for labels. GrumpyVM cannot execute these directly -- they must
/// first be translated by the assembler to native instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum PInstr {
    /// Label the next instruction.
    PLabel(Label),
    /// Push a label onto the stack.
    PPush(Label),
    /// Native machine instruction.
    PI(Instr),
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unop {
    /// Boolean negation.
    Neg,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Binop {
    /// i32 addition.
    Add,
    /// i32 multiplication.
    Mul,
    /// i32 subtraction.
    Sub,
    /// i32 division (raises an error on divide by zero).
    Div,
    /// Returns true if one i32 is less than another, otherwise false.
    Lt,
    /// Returns true if one i32 is equal another, otherwise false.
    Eq,
}

////////////////////////////////////////////////////////////////////////
// FromStr trait implementations
////////////////////////////////////////////////////////////////////////

impl FromStr for Unop {
    type Err = ParseError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "neg" => Ok(Neg),
            _ => Err(ParseError(String::from("unknown unop"))),
        }
    }
}

impl FromStr for Binop {
    type Err = ParseError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "+" => Ok(Add),
            "*" => Ok(Mul),
            "-" => Ok(Sub),
            "/" => Ok(Div),
            "<" => Ok(Lt),
            "==" => Ok(Eq),
            _ => Err(ParseError(String::from("unknown binop"))),
        }
    }
}

impl FromStr for Val {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "tt" => Ok(Vunit),
            "true" => Ok(Vbool(true)),
            "false" => Ok(Vbool(false)),
            "undef" => Ok(Vundef),
            tok => Ok(Vi32(tok.parse()?))
        }
    }
}

impl FromStr for Instr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut toks = s.split_whitespace();
        if let Some(tok) = toks.next() {
            match tok.trim() {
                "push" => {
                    let tok2 = toks.next().unwrap().trim();
                    Ok(Push(Val::from_str(tok2)?))
                }
                "pop" => Ok(Pop),
                "peek" => {
                    let tok2 = toks.next().unwrap().trim();
                    Ok(Peek(tok2.parse()?))
                }
                "unary" => {
                    let tok2 = toks.next().unwrap().trim();
                    Ok(Unary(Unop::from_str(tok2)?))
                }
                "binary" => {
                    let tok2 = toks.next().unwrap().trim();
                    let b = Binop::from_str(tok2)?;
                    Ok(Binary(b))
                }
                "swap" => Ok(Swap),
                "alloc" => Ok(Alloc),
                "get" => Ok(Set),
                "set" => Ok(Get),
                "var" => {
                    let tok2 = toks.next().unwrap().trim();
                    let n = tok2.parse()?;
                    Ok(Var(n))
                }
                "store" => {
                    let tok2 = toks.next().unwrap().trim();
                    let n = tok2.parse()?;
                    Ok(Store(n))
                }
                "setframe" => {
                    let tok2 = toks.next().unwrap().trim();
                    let n = tok2.parse()?;
                    Ok(SetFrame(n))
                }
                "call" => Ok(Call),
                "ret" => Ok(Ret),
                "branch" => Ok(Branch),
                "halt" => Ok(Halt),
                _ => Err(ParseError(format!("unknown op: {}", tok))),
            }
        } else {
            Err(ParseError(String::from("no tokens")))
        }
    }
}

fn parse_label(s: &str) -> Result<Label, ParseError> {
    if Regex::new("(L[a-zA-Z0-9]+)|(_L[a-zA-Z0-9]+)")
        .unwrap()
        .is_match(s)
    {
        Ok(String::from(s))
    } else {
        Err(ParseError(format!("bad label: {}", s)))
    }
}

impl FromStr for PInstr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut toks = s.split_whitespace();
        if let Some(tok) = toks.next() {
            match tok.trim() {
                "push" => {
                    // let suffix = s.strip_prefix("push")?.trim();
                    let tok2 = toks.next().unwrap().trim();
                    if let Ok(lbl) = parse_label(tok2) {
                        Ok(PPush(lbl))
                    } else {
                        let instr = Instr::from_str(s)?;
                        Ok(PI(instr))
                    }
                }
                _ => {
                    if tok.ends_with(":") {
                        let lbl = parse_label(&tok[0..tok.len() - 1])?;
                        Ok(PLabel(lbl))
                    } else {
                        let instr = Instr::from_str(s)?;
                        Ok(PI(instr))
                    }
                }
            }
        } else {
            Err(ParseError(String::from("no tokens")))
        }
    }
}

////////////////////////////////////////////////////////////////////////
// ToBytes trait implementations
////////////////////////////////////////////////////////////////////////

impl ToBytes for u32 {
    fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![0x00; 4];
        BigEndian::write_u32(&mut v, *self);
        v
    }
}

impl ToBytes for i32 {
    fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![0x00; 4];
        BigEndian::write_i32(&mut v, *self);
        v
    }
}

impl ToBytes for Unop {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Neg => vec![0x00],
        }
    }
}

impl ToBytes for Binop {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Add => vec![0x00],
            Mul => vec![0x01],
            Sub => vec![0x02],
            Div => vec![0x03],
            Lt => vec![0x04],
            Eq => vec![0x05],
        }
    }
}

impl ToBytes for Val {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Vunit => vec![0x00],
            Vi32(i) => {
                let mut bs = vec![0x01];
                bs.append(&mut i.to_bytes());
                bs
            }
            Vbool(true) => vec![0x02],
            Vbool(false) => vec![0x03],
            Vloc(l) => {
                let mut bs = vec![0x04];
                bs.append(&mut l.to_bytes());
                bs
            }
            Vundef => vec![0x05],
            _ => panic!("Val::ToBytes: unsupported constructor"),
        }
    }
}

impl ToBytes for Instr {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Push(v) => {
                let mut bs = vec![0x00];
                bs.append(&mut v.to_bytes());
                bs
            }
            Pop => vec![0x01],
            Peek(i) => {
                let mut bs = vec![0x02];
                bs.append(&mut i.to_bytes());
                bs
            }
            Unary(u) => {
                let mut bs = vec![0x03];
                bs.append(&mut u.to_bytes());
                bs
            }
            Binary(b) => {
                let mut bs = vec![0x04];
                bs.append(&mut b.to_bytes());
                bs
            }
            Swap => vec![0x05],
            Alloc => vec![0x06],
            Get => vec![0x07],
            Set => vec![0x08],
            Var(i) => {
                let mut bs = vec![0x09];
                bs.append(&mut i.to_bytes());
                bs
            }
            Store(i) => {
                let mut bs = vec![0x0A];
                bs.append(&mut i.to_bytes());
                bs
            }
            SetFrame(i) => {
                let mut bs = vec![0x0B];
                bs.append(&mut i.to_bytes());
                bs
            }
            Call => vec![0x0C],
            Ret => vec![0x0D],
            Branch => vec![0x0E],
            Halt => vec![0x0F],
        }
    }
}

////////////////////////////////////////////////////////////////////////
// FromBytes trait implementations
////////////////////////////////////////////////////////////////////////

impl FromBytes for u32 {
    type Err = ParseError;
    fn from_bytes<T: Iterator<Item=u8>>(bytes: &mut T) -> Result<u32, ParseError> {
	let v: Vec<u8> = bytes.take(4).collect();
	if v.len() == 4 {
	    Ok(BigEndian::read_u32(&v))
	} else {
	    Err(ParseError("not enough bytes".into()))
	}
    }
}

impl FromBytes for i32 {
    type Err = ParseError;
    fn from_bytes<T: Iterator<Item=u8>>(bytes: &mut T) -> Result<i32, ParseError> {
	let v: Vec<u8> = bytes.take(4).collect();
	if v.len() == 4 {
	    Ok(BigEndian::read_i32(&v))
	} else {
	    Err(ParseError("not enough bytes".into()))
	}
    }
}

impl FromBytes for Unop {
    type Err = ParseError;
    fn from_bytes<T: Iterator<Item=u8>>(bytes: &mut T) -> Result<Unop, ParseError> {
	match bytes.next().ok_or(ParseError("not enough bytes".into()))? {
            0x00 => Ok(Neg),
            b => Err(ParseError(format!("unknown unop code: {}", b))),
	}
    }
}

impl FromBytes for Binop {
    type Err = ParseError;
    fn from_bytes<T: Iterator<Item=u8>>(bytes: &mut T) -> Result<Binop, ParseError> {
	match bytes.next().ok_or(ParseError("not enough bytes".into()))? {
            0x00 => Ok(Add),
            0x01 => Ok(Mul),
            0x02 => Ok(Sub),
            0x03 => Ok(Div),
            0x04 => Ok(Lt),
            0x05 => Ok(Eq),
            b => Err(ParseError(format!("unknown binop code: {}", b))),
	}
    }
}

impl FromBytes for Val {
    type Err = ParseError;
    fn from_bytes<T: Iterator<Item=u8>>(bytes: &mut T) -> Result<Val, ParseError> {
        match bytes.next().ok_or(ParseError("not enough bytes".into()))? {
            0x00 => Ok(Vunit),
            0x01 => Ok(Vi32(i32::from_bytes(bytes)?)),
            0x02 => Ok(Vbool(true)),
            0x03 => Ok(Vbool(false)),
            0x04 => Ok(Vloc(u32::from_bytes(bytes)?)),
            0x05 => Ok(Vundef),
	    b => Err(ParseError(format!("unknown val code: {}", b))),
	}
    }
}

impl FromBytes for Instr {
    type Err = ParseError;
    fn from_bytes<T: Iterator<Item=u8>>(bytes: &mut T) -> Result<Instr, ParseError> {
	match bytes.next().ok_or(ParseError("not enough bytes".into()))? {
            0x00 => Ok(Push(Val::from_bytes(bytes)?)),
            0x01 => Ok(Pop),
            0x02 => Ok(Peek(u32::from_bytes(bytes)?)),
            0x03 => Ok(Unary(Unop::from_bytes(bytes)?)),
	    0x04 => Ok(Binary(Binop::from_bytes(bytes)?)),
            0x05 => Ok(Swap),
            0x06 => Ok(Alloc),
            0x07 => Ok(Set),
            0x08 => Ok(Get),
            0x09 => Ok(Var(u32::from_bytes(bytes)?)),
            0x0A => Ok(Store(u32::from_bytes(bytes)?)),
            0x0B => Ok(SetFrame(u32::from_bytes(bytes)?)),
            0x0C => Ok(Call),
            0x0D => Ok(Ret),
            0x0E => Ok(Branch),
            0x0F => Ok(Halt),
            b => Err(ParseError(format!("unknown instr code: {}", b))),
	}
    }
}

impl FromBytes for Vec<Instr> {
    type Err = ParseError;
    fn from_bytes<T: Iterator<Item=u8>>(bytes: &mut T) -> Result<Vec<Instr>, ParseError> {
	let n = u32::from_bytes(bytes)?;

	let mut v = Vec::new();
	for _ in 0..n {
	    v.push(Instr::from_bytes(bytes)?)
	}

	Ok(v)
    }
}

// Put all your test cases in this module.
#[cfg(test)]
mod tests {
    use super::*;

    // Example test case.
    #[test]
    fn test_1() {
        assert_eq!(Instr::from_str("push 123").unwrap(), Push(Vi32(123)));
        assert_eq!(PInstr::from_str("Labc123:").unwrap(),
		   PLabel(String::from("Labc123"))
        );
    }
}
