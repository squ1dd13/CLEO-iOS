use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fmt::Display;
use std::io::{Error, ErrorKind, Read, Seek};

#[derive(Debug, Clone)]
pub struct Variable {
    value: i64,
    location: Location,
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

#[derive(Debug, Clone)]
pub struct Array;

#[derive(Debug, Clone)]
pub struct Pointer(i64);

impl Pointer {
    pub fn from_i64(i: i64) -> Pointer {
        Pointer(i)
    }

    pub fn is_local(&self) -> bool {
        self.0 < 0
    }

    pub fn absolute(&self) -> u64 {
        self.0.abs() as u64
    }
}

impl std::fmt::Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_local() {
            write!(f, "{:#x}", self.absolute())
            // f.write_fmt(format_args!("{:#x}", self.absolute()))
        } else {
            f.write_fmt(format_args!("Global({:#x})", self.absolute()))
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
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
    pub fn read(reader: &mut impl std::io::Read) -> std::io::Result<Value> {
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
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unknown type ID '{}'", id),
                ));
            }
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ParamType {
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
pub enum Location {
    /// A value directly written in the instruction bytecode.
    Immediate,

    /// A variable local to the script.
    Local,

    /// A variable shared between scripts.
    Global,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Param {
    param_type: ParamType,
    location: Location,
    is_variadic: bool,
    is_output: bool,
}

impl Param {
    pub fn read(&self, reader: &mut impl std::io::Read) -> std::io::Result<Value> {
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
pub struct Command {
    pub opcode: u16,
    pub name: String,
    pub returns: bool,
    pub params: Vec<Param>,
}

fn load_all_commands() -> std::result::Result<HashMap<u16, Command>, Box<bincode::ErrorKind>> {
    let commands_bin = include_bytes!("../commands.bin");
    bincode::deserialize(commands_bin)
}

pub struct Instr {
    opcode: u16,
    name: String,

    offset: u64,

    bool_inverted: bool,
    args: Vec<Value>,
}

impl Instr {
    pub fn read(
        commands: &HashMap<u16, Command>,
        reader: &mut (impl Read + Seek),
    ) -> std::io::Result<Instr> {
        let offset = reader.stream_position()?;

        let (opcode, bool_inverted) = {
            let opcode_in_file = reader.read_u16::<LittleEndian>()?;

            // The most significant bit (0x8000) is set when the returned
            //  boolean is to be inverted.
            (opcode_in_file & 0x7fff, opcode_in_file & 0x8000 != 0)
        };

        let cmd = match commands.get(&opcode) {
            Some(command) => command,
            None => {
                // If we don't know the opcode, then we can't get the parameter list,
                //  which is necessary for reading the instruction.
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unknown opcode {:#x}", opcode),
                ));
            }
        };

        let mut args = Vec::with_capacity(cmd.params.len());

        for param in cmd.params.iter() {
            args.push(param.read(reader)?);
        }

        Ok(Instr {
            opcode,
            name: cmd.name.clone(),
            offset,
            bool_inverted,
            args,
        })
    }

    pub fn next_offsets(&self, current: u64, offsets: &mut Vec<u64>) {
        // The 'return' command should go to the return address on the call stack,
        //  but we already handle that case when we branch at 'gosub'.
        if self.opcode == 0x0051 {
            return;
        }

        // goto always branches. Everything else is assumed to also go onto the next instruction.
        if self.opcode != 0x0002 {
            offsets.push(current);
        }

        // goto, goto_if_true, goto_if_false, gosub, switch_start and switch_continue can all
        //  branch to every pointer they reference (in theory).
        if let 0x0002 | 0x004c | 0x004d | 0x0050 | 0x0871 | 0x0872 = self.opcode {
            for arg in &self.args {
                if let Value::Pointer(ptr) = arg {
                    if ptr.is_local() {
                        offsets.push(ptr.absolute());
                    }
                }
            }
        }
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
            self.name,
            self.args
                .iter()
                .map(Value::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub fn disassemble(
    commands: &HashMap<u16, Command>,
    reader: &mut std::io::Cursor<&[u8]>,
    instrs: &mut HashMap<u64, Instr>,
) -> std::io::Result<()> {
    let start = std::time::Instant::now();

    let mut cur_offsets: Vec<u64> = Vec::new();
    let mut new_offsets: Vec<u64> = Vec::new();

    cur_offsets.push(0);

    while !cur_offsets.is_empty() {
        for offset in cur_offsets.iter() {
            if instrs.contains_key(offset) {
                continue;
            }

            reader.seek(std::io::SeekFrom::Start(*offset))?;

            let instr = match Instr::read(commands, reader) {
                Ok(instr) => instr,
                Err(err) => {
                    // Log the error and continue - we can't guarantee that the game would go down
                    //  this invalid path, so the script could still run fine (this is the basis
                    //  of many script obfuscation methods).
                    // log::error!("encountered error at {:#x}: {}", *offset, err);

                    continue;
                }
            };

            instr.next_offsets(reader.position(), &mut new_offsets);
            instrs.insert(*offset, instr);
        }

        cur_offsets.clear();
        cur_offsets.append(&mut new_offsets);
    }

    let end = std::time::Instant::now();
    let time_taken = end - start;

    log::info!("Disassembly took {:#?}", time_taken);

    Ok(())
}

// fixme: The command list is fucked. New one needs to be generated (and kept in textual format so it can be edited easily). We don't care about type info.

pub fn check(bytes: &[u8]) -> Result<Option<String>, String> {
    log::info!("OzoneSC v0.1");

    let commands = load_all_commands().map_err(|err| err.to_string())?;

    let mut instruction_map = HashMap::new();

    if let Err(err) = disassemble(
        &commands,
        &mut std::io::Cursor::new(bytes),
        &mut instruction_map,
    ) {
        log::error!("error at end of disassembly: {}", err);
    } else {
        log::info!("finished disassembly");
    }

    // Create an ordered script from the instructions we found by sorting using offsets.
    // let mut instructions: Vec<(&u64, &Instr)> = instruction_map.iter().collect();
    // instructions.sort_unstable_by_key(|pair| pair.0);

    // for instruction in instructions {
    // log::trace!("{}", instruction.1);
    // }

    Ok(None)
}
