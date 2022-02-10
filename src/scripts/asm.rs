//! Provides assembly, disassembly and safety checking for SCM code.

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::{self, Cursor, Error, ErrorKind, Seek};

#[derive(Debug, Clone)]
pub struct Variable {
    value: i64,
    location: Location,
}

impl Variable {
    pub fn new_local(value: i64) -> Variable {
        Variable {
            value,
            location: Location::Local,
        }
    }
}

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.location {
            Location::Global => "global",
            Location::Local => "local",
            _ => panic!("Invalid variable location"),
        };

        f.write_fmt(format_args!("{}_{:#x}", prefix, self.value))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Array;

#[derive(Debug, Copy, Clone)]
pub struct Pointer(i64);

impl Pointer {
    fn from_i64(i: i64) -> Pointer {
        Pointer(i)
    }

    fn is_local(&self) -> bool {
        self.0 < 0
    }

    fn absolute(&self) -> u64 {
        self.0.abs() as u64
    }
}

impl std::fmt::Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_local() {
            write!(f, "{:#x}", self.absolute())
        } else {
            f.write_fmt(format_args!("Global({:#x})", self.absolute()))
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    // fixme: This should be i32, because that's what the game expects and what will be encoded in scripts.
    Integer(i64),
    Real(f32),
    String(String),
    Model(i64),
    Pointer(Pointer),
    VarArgs(Vec<Value>),
    Buffer(String),
    Variable(Variable),
    Array(Array),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(int) => int.fmt(f),
            Value::Real(real) => f.write_fmt(format_args!("{}f", real)),
            Value::String(string) => f.write_fmt(format_args!("\"{}\"", string)),
            Value::Pointer(pointer) => pointer.fmt(f),
            Value::Variable(var) => var.fmt(f),
            Value::Array(_) => f.write_str("arr"),
            _ => panic!("wtf"),
        }
    }
}

impl Value {
    pub fn into_basic(self) -> Value {
        match self {
            Value::Model(val) => Value::Integer(val),
            Value::Pointer(val) => Value::Integer(val.0),
            other => other,
        }
    }

    pub fn write(self, writer: &mut impl io::Write) -> Result<usize> {
        Ok(match self.into_basic() {
            Value::Integer(val) => {
                // i32 type code.
                writer.write_u8(0x01)?;
                writer.write_i32::<byteorder::LittleEndian>(val as i32)?;
                5
            }
            Value::Real(val) => {
                writer.write_u8(0x06)?;
                writer.write_f32::<byteorder::LittleEndian>(val)?;
                5
            }
            Value::String(string) => {
                // Variable-length string type code.
                writer.write_u8(0x0e)?;
                writer.write_u8(string.len() as u8)?;

                for c in string.chars() {
                    writer.write_u8(c as u8)?;
                }

                string.len() + 1
            }
            Value::Variable(Variable { value, location }) => {
                // If we're compiling code for the game to run, it shouldn't matter that all
                // variables are written as int/float, even if they are strings: the game doesn't
                // check the type.
                writer.write_u8(match location {
                    Location::Global => 0x02,
                    Location::Local => 0x03,
                    _ => panic!(),
                })?;

                writer.write_u16::<LittleEndian>(value as u16)?;
                3
            }
            _ => todo!(),
        })
    }

    fn read(reader: &mut impl io::Read) -> Result<Value> {
        let id = reader.read_u8()?;

        Ok(match id {
            0x1 => Value::Integer(reader.read_i32::<LittleEndian>()? as i64),
            0x2 => Value::Variable(Variable {
                value: reader.read_u16::<LittleEndian>()? as i64,
                location: Location::Global,
            }),
            0x3 => Value::Variable(Variable {
                value: reader.read_u16::<LittleEndian>()? as i64,
                location: Location::Local,
            }),
            0x4 => Value::Integer(reader.read_i8()? as i64),
            0x5 => Value::Integer(reader.read_i16::<LittleEndian>()? as i64),
            0x6 => Value::Real(reader.read_f32::<LittleEndian>()?),
            0x7 => {
                reader.read_exact(&mut [0; 6])?;
                Value::Array(Array {})
            }
            0x8 => {
                reader.read_exact(&mut [0; 6])?;
                Value::Array(Array {})
            }
            0x9 => {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf[..])?;
                Value::String(
                    buf.iter()
                        .take_while(|v| v != &&0)
                        .map(|v| *v as char)
                        .collect::<String>(),
                )
            }
            0xa => Value::Variable(Variable {
                value: reader.read_u16::<LittleEndian>()? as i64,
                location: Location::Global,
            }),
            0xb => Value::Variable(Variable {
                value: reader.read_u16::<LittleEndian>()? as i64,
                location: Location::Local,
            }),
            0xc => {
                reader.read_exact(&mut [0; 6])?;
                Value::Array(Array {})
            }
            0xd => {
                reader.read_exact(&mut [0; 6])?;
                Value::Array(Array {})
            }
            0xe => {
                let length = reader.read_u8()? as usize;
                let mut vec: Vec<u8> = std::iter::repeat(0u8).take(length).collect();
                reader.read_exact(vec.as_mut_slice())?;
                Value::String(
                    vec.iter()
                        .take_while(|v| v != &&0)
                        .map(|v| *v as char)
                        .collect::<String>(),
                )
            }
            0xf => {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf[..])?;
                Value::String(
                    buf.iter()
                        .take_while(|v| v != &&0)
                        .map(|v| *v as char)
                        .collect::<String>(),
                )
            }
            0x10 => Value::Variable(Variable {
                value: reader.read_u16::<LittleEndian>()? as i64,
                location: Location::Global,
            }),
            0x11 => Value::Variable(Variable {
                value: reader.read_u16::<LittleEndian>()? as i64,
                location: Location::Local,
            }),
            0x12 => {
                reader.read_exact(&mut [0; 6])?;
                Value::Array(Array {})
            }
            0x13 => {
                reader.read_exact(&mut [0; 6])?;
                Value::Array(Array {})
            }

            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown type ID '{}'", id),
                )
                .into());
            }
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum ParamType {
    /// An integral value.
    Integer,

    /// A string value.
    String,

    /// A floating-point value.
    Real,

    /// A model ID.
    Model,

    /// A pointer to a script location.
    Pointer,

    /// A null byte to mark the end of argument lists.
    End,

    /// Long buffer; used only for opcode 05B6.
    Buffer,

    /// Allows any type. Typically used for variadic arguments.
    Any,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
enum Location {
    /// Useless. Included only for compatibility with `commands.bin`.
    // todo: Rebuild `commands.bin` with this variant removed.
    _0,

    /// A variable local to the script.
    Local,

    /// A variable shared between scripts.
    Global,
}

#[derive(Debug, Deserialize, Serialize)]
struct Param {
    param_type: ParamType,
    location: Location,
    is_variadic: bool,
    is_output: bool,
}

impl Param {
    fn read(&self, reader: &mut impl io::Read) -> Result<Value> {
        let value = Value::read(reader)?;

        if let ParamType::Pointer = self.param_type {
            if let Value::Integer(int) = value {
                return Ok(Value::Pointer(Pointer::from_i64(int)));
            }
        }

        Ok(value)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Command {
    opcode: u16,
    name: String,
    returns: bool,
    params: Vec<Param>,
}

fn load_all_commands() -> Result<HashMap<u16, Command>, Box<bincode::ErrorKind>> {
    let commands_bin = include_bytes!("commands.bin");
    bincode::deserialize(commands_bin)
}

pub struct Instr {
    opcode: u16,
    offset: u64,
    bool_inverted: bool,
    args: Vec<Value>,
}

impl Instr {
    pub fn new(opcode: u16, args: Vec<Value>) -> Instr {
        Instr {
            opcode,
            offset: 0,
            bool_inverted: false,
            args,
        }
    }

    pub fn write(self, dest: &mut impl std::io::Write) -> Result<usize> {
        let compiled_opcode = self.opcode | if self.bool_inverted { 0x8000 } else { 0 };
        dest.write_u16::<LittleEndian>(compiled_opcode)?;

        // We track the length of the bytecode written so that the caller can keep track of
        // offsets.
        let mut byte_count = 2;

        for arg in self.args {
            byte_count += arg.write(dest)?;
        }

        Ok(byte_count)
    }

    fn read(commands: &HashMap<u16, Command>, reader: &mut Cursor<&[u8]>) -> Result<Instr> {
        let offset = reader.position();

        let (opcode, bool_inverted) = {
            let opcode_in_file = reader.read_u16::<LittleEndian>()?;

            // The most significant bit (0x8000) is set when the returned boolean is to be
            // inverted.
            (opcode_in_file & 0x7fff, opcode_in_file & 0x8000 != 0)
        };

        let cmd = match commands.get(&opcode) {
            Some(command) => command,
            None => {
                // If we don't know the opcode, then we can't get the parameter list, which is
                // necessary for reading the instruction.
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unknown opcode {:#x}", opcode),
                )
                .into());
            }
        };

        let mut args = Vec::with_capacity(cmd.params.len());

        for param in &cmd.params {
            args.push(param.read(reader)?);
        }

        Ok(Instr {
            opcode,
            offset,
            bool_inverted,
            args,
        })
    }

    fn next_offsets(&self, current: u64) -> Vec<u64> {
        // The 'return' command should go to the return address on the call stack, but we already
        // handle that case when we branch at 'gosub'.
        if self.opcode == 0x0051 {
            return vec![];
        }

        let mut offsets = vec![];

        // goto always branches. Everything else is assumed to also go onto the next instruction.
        if self.opcode != 0x0002 {
            offsets.push(current);
        }

        // goto, goto_if_true, goto_if_false, gosub, switch_start and switch_continue can all
        // branch to every pointer they reference (in theory).
        if let 0x0002 | 0x004c | 0x004d | 0x0050 | 0x0871 | 0x0872 = self.opcode {
            for arg in &self.args {
                if let Value::Pointer(ptr) = arg {
                    if ptr.is_local() {
                        offsets.push(ptr.absolute());
                    }
                }
            }
        }

        offsets
    }

    fn name(&self) -> Option<&'static str> {
        get_commands()
            .get(&self.opcode)
            .map(|cmd| cmd.name.as_str())
    }
}

impl Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:08x} {:04x} {}{}({})",
            self.offset,
            self.opcode,
            if self.bool_inverted { "!" } else { "" },
            self.name().unwrap_or("<no name>"),
            self.args
                .iter()
                .map(Value::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

struct Disassembler<'bytes> {
    commands: &'static HashMap<u16, Command>,
    bytecode: Cursor<&'bytes [u8]>,
    instrs: HashMap<u64, Instr>,
}

impl Disassembler<'_> {
    fn disasm_all(bytecode: &[u8]) -> HashMap<u64, Instr> {
        let mut disassembler = Disassembler {
            commands: get_commands(),
            bytecode: Cursor::new(bytecode),
            instrs: HashMap::new(),
        };

        // Disassemble from offset 0 (the entry point).
        disassembler.disasm(0);
        disassembler.instrs
    }

    fn disasm(&mut self, offset: u64) {
        if self.instrs.contains_key(&offset) {
            // Path already explored.
            return;
        }

        if let Err(err) = self.bytecode.seek(io::SeekFrom::Start(offset)) {
            log::error!("Seek error: {}", err);
            return;
        }

        // Try to read a single instruction.
        let instr = match Instr::read(self.commands, &mut self.bytecode) {
            Ok(v) => v,
            Err(err) => {
                // We actually expect disassembly errors, because a lot of scripts use invalid
                // bytecode as a way to stop disassembly. We just log the error and carry on.
                log::error!("Error during disassembly: {}", err);
                return;
            }
        };

        let next_offsets = instr.next_offsets(self.bytecode.position());
        self.instrs.insert(offset, instr);

        // Disassemble from all of the possible offsets that could come after the instruction. This
        // allows us to explore all the branches of any conventional script.
        for offset in next_offsets {
            self.disasm(offset);
        }
    }
}

fn get_commands() -> &'static HashMap<u16, Command> {
    static COMMANDS_CELL: OnceCell<HashMap<u16, Command>> = OnceCell::new();

    COMMANDS_CELL.get_or_init(|| {
        let loaded = match load_all_commands() {
            Ok(l) => l,
            Err(err) => {
                log::error!("Error loading commands: {}", err);
                return HashMap::new();
            }
        };

        loaded
    })
}

pub enum Issue {
    /// The script uses instructions that aren't implemented in this library.
    NotImpl,

    /// The script uses code that can only work on Android due to architecture differences.
    BadArch,

    // fixme: Script checks can no longer fail, so we should remove this variant.
    /// The script check failed, so we can't say whether or not the script has issues.
    Unchecked,

    /// The script has the same identity as some other script.
    Duplicate(String),
}

impl Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImpl => f.write_str("Requires features unavailable on iOS."),
            Self::BadArch => f.write_str("Contains some iOS-incompatible code."),
            Self::Duplicate(orig_name) => write!(f, "Duplicate of '{}'.", orig_name),
            Self::Unchecked => f.write_str("Script check failed."),
        }
    }
}

/// A record of the problems found in a script.
pub struct CompatReport {
    issues: Vec<Issue>,
}

impl CompatReport {
    pub fn scan(bytecode: &[u8]) -> CompatReport {
        // Disassemble the entire script.
        let instrs = Disassembler::disasm_all(bytecode);

        // todo: Re-assemble all instructions over the top of `bytecode` and look for differences.

        log::info!("Disassembled {} instructions", instrs.len());

        // Look for problematic opcodes.
        let instr_issues = instrs
            .iter()
            .filter_map(|(_, instr)| match instr.opcode {
                0x0dd5 | 0x0dd6 | 0x0de1..=0x0df6 => Some(Issue::NotImpl),
                0x0dd0..=0x0ddb | 0x0dde => Some(Issue::BadArch),

                _ => None,
            })
            .collect();

        for issue in &instr_issues {
            log::info!("Issue: {}", issue);
        }

        CompatReport {
            issues: instr_issues,
        }
    }

    fn main_issue(&self) -> Option<&Issue> {
        todo!()
    }
}
