use std::fmt::{self, Display};
use super::isa::{*, Binop::*, Instr::*, Val::*, Unop::*};

static STK_SIZE: usize = 1024;
static HEAP_SIZE: usize = 1024;

/// GrumpyVM state.
#[derive(Debug)]
struct State {
    /// Program counter.
    pc: u32,
    /// Frame pointer.
    fp: u32,
    /// The stack, with maximum size STK_SIZE.
    stk: Vec<Val>,
    /// The heap, with maximum size HEAP_SIZE.
    heap: Vec<Val>,
    /// The program being executed, a vector of instructions.
    prog: Vec<Instr>
}

/// Display implementation for State (modify as you wish).
impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "pc: {}\ninstr: {:?}\nfp: {}\nstk: {:?}\nheap: {:?}",
	       self.pc, self.prog[self.pc as usize], self.fp, self.stk, self.heap)?;
	write!(f, "\nheap size: {}", self.heap.len())
    }
}

/// Debug enum (whether to print debug information during execution or not).
#[derive(Clone, Copy)]
pub enum Debug {
    DEBUG,
    NODEBUG
}

/// State methods.
impl State {
    /// Create initial state for given program.
    fn init(prog: Vec<Instr>) -> State {
	State {
	    pc: 0, 
	    fp: 0,
	    stk: Vec::with_capacity(STK_SIZE),
	    heap: Vec::with_capacity(HEAP_SIZE),
	    prog: prog
	}
    }
    /// Push a Val to the stack, checking for overflow.
    fn push(&mut self, v: Val) -> Result<(), String> {
	if self.stk.len() < STK_SIZE {
    	    Ok(self.stk.push(v))
	} else {
	    Err("out of stack space".into())
	}
    }
    /// Pop a Val from the stack, checking for underflow.
    fn pop(&mut self) -> Result<Val, String> {
    	self.stk.pop().ok_or("attempt to pop empty stack".into())
    }
}

/// Evaluate a unary operation on a value.
fn unop(u: Unop, v: Val) -> Result<Val, String> {
    match u {
	Neg => {
	    let b = v.to_bool().ok_or("expected bool")?;
	    Ok(Vbool(!b))
	}
    }
}

/// Evaluate a binary operation on a value.
fn binop(b: Binop, v1: Val, v2: Val) -> Result<Val, String> {
    let i1 = v1.to_i32().ok_or("expected i32")?;
    let i2 = v2.to_i32().ok_or("expected i32")?;
    Ok(match b {
	Add => Vi32(i1 + i2),
	Mul => Vi32(i1 * i2),
	Sub => Vi32(i1 - i2),
	Div => Vi32(i1 / i2),
	Lt => Vbool(i1 <= i2),
	Eq => Vbool(i1 == i2)
    })
}

/// Execute from initial state s.
fn exec(d: Debug, s: &mut State) -> Result<(), String> {
    loop {
	if let Debug::DEBUG = d {
	    println!("{}\n", s)
	}
	if s.pc as usize >= s.prog.len() {
	    return Err("pc out of bounds".into())
	}
	let instr = &s.prog[s.pc as usize];
	s.pc += 1;
	match instr {
	    Push(v) => {
		let v = *v; // Satisfy borrow checker
		s.push(v)?
	    }
	    Pop => { s.pop()?; }
	    Peek(i) => {
		let i = *i as usize; // Satisfy borrow checker
		s.push(s.stk[i])?
	    }
	    Unary(u) => {
		let u = *u; // Satisfy borrow checker
		let v = s.pop()?;
		let i = unop(u, v)?;
		s.stk.push(i)
	    }
	    Binary(b) => {
		let b = *b; // Satisfy borrow checker
	    	let (v1, v2) = (s.pop()?, s.pop()?);
	    	let i = binop(b, v1, v2)?;
	    	s.stk.push(i)
	    }
	    Swap => {
                let v2 = s.pop()?;
                let v1 = s.pop()?;
		s.stk.push(v2);
		s.stk.push(v1);
	    }
	    Alloc => {
                let vinit = s.pop()?;
                let vsize = s.pop()?;
		let size = vsize.to_i32().ok_or("expected i32")? as usize;
		if s.heap.len() + size + 1 < HEAP_SIZE {
		    let loc = s.heap.len();
		    s.heap.push(Vsize(size));
		    s.heap.append(&mut vec![vinit; size]);
		    s.stk.push(Vaddr(loc))
		} else {
		    return Err("out of heap space".into())
		}
	    }
	    Set => {
		let (v, vix, vbase) = (s.pop()?, s.pop()?, s.pop()?);
		let ix = vix.to_i32().ok_or("expected i32")? as usize;
		let base = vbase.to_address().ok_or("expected address")?;
		if base + ix < HEAP_SIZE {
		    if let Vsize(size) = s.heap[base] {
			if ix < size {
			    s.heap[base+ix+1] = v
			} else {
			    return Err("index past end of array".into())
			}
		    } else {
			return Err("expected size at array location".into())
		    }
		} else {
		    return Err("indexing past end of heap".into())
		}
	    }
	    Get => {
                let vix = s.pop()?;
                let vbase = s.pop()?;
		let ix = vix.to_i32().ok_or("expected i32")? as usize;
		let base = vbase.to_address().ok_or("expected address")?;
		if base + ix < HEAP_SIZE {
		    if let Vsize(size) = s.heap[base] {
			if ix < size {
			    s.push(s.heap[base+ix+1])?;
			} else {
			    return Err("index past end of array".into())
			}
		    } else {
			return Err("expected size at array location".into())
		    }
		} else {
		    return Err("indexing past end of heap".into())
		}
	    }
	    Var(i) => {
		let ix = (s.fp + *i) as usize;
		if ix < s.stk.len() {
		    s.push(s.stk[ix])?;
		} else {
		    return Err("variable access past end of stack".into())
		}
	    }
	    Store(i) => {
		let ix = (s.fp + *i) as usize;
		let v = s.pop()?;
		if ix < s.stk.len() {
		    s.stk[ix] = v;
		} else {
		    return Err("store past end of stack".into())
		}
	    }
	    SetFrame(i) => {
		let i = *i; // Satisfy borrow checker
		s.push(Vloc(s.fp))?;
		s.fp = s.stk.len() as u32 - i - 1
	    }
	    Call => {
		if let Vloc(target) = s.pop()? {
		    s.stk.push(Vloc(s.pc));
		    s.pc = target
		} else {
		    return Err("expected loc for call target".into())
		}
	    }
	    Ret => {
		if let (vret, Vloc(pc), Vloc(fp)) = (s.pop()?, s.pop()?, s.pop()?) {
		    s.stk.truncate(s.fp as usize);
		    s.pc = pc;
		    s.fp = fp;
		    s.stk.push(vret)
		} else {
		    return Err("expected location for pc and fp in return".into())
		}
	    }
	    Branch => {
                let vtarget = s.pop()?;
                let vb = s.pop()?;
		let target = vtarget.to_loc().ok_or("expected location")?;
		if vb.to_bool().ok_or("expected bool")? {
		    s.pc = target
		}
	    }
	    Halt => return Ok(())
	}
    }
}

/// Entry point from outside of this module. Run the given program in the VM.
pub fn run(d: Debug, prog: &[Instr]) -> Result<Val, String> {
    let mut s = State::init(prog.into());
    exec(d, &mut s)?;
    s.pop()
}
