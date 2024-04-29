use alloc::vec::Vec;
use core::num::NonZeroU32;

#[derive(Copy, Clone)]
struct Fixup {
    target_label: Label,
    instruction_offset: usize,
    instruction_length: u8,
    kind: FixupKind,
}

pub struct Assembler {
    origin: u64,
    code: Vec<u8>,
    labels: Vec<isize>,
    fixups: Vec<Fixup>,
    guaranteed_capacity: usize,
}

#[allow(clippy::derivable_impls)]
impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
pub struct AssembledCode<'a>(&'a mut Assembler);

impl<'a> core::ops::Deref for AssembledCode<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0.code
    }
}

impl<'a> From<AssembledCode<'a>> for Vec<u8> {
    fn from(code: AssembledCode<'a>) -> Vec<u8> {
        core::mem::take(&mut code.0.code)
    }
}

impl<'a> Drop for AssembledCode<'a> {
    fn drop(&mut self) {
        self.0.clear();
    }
}

impl Assembler {
    pub const fn new() -> Self {
        Assembler {
            origin: 0,
            code: Vec::new(),
            labels: Vec::new(),
            fixups: Vec::new(),
            guaranteed_capacity: 0,
        }
    }

    pub fn origin(&self) -> u64 {
        self.origin
    }

    pub fn set_origin(&mut self, origin: u64) {
        self.origin = origin;
    }

    pub fn current_address(&self) -> u64 {
        self.origin + self.code.len() as u64
    }

    pub fn forward_declare_label(&mut self) -> Label {
        let label = self.labels.len() as u32;
        self.labels.push(isize::MAX);
        Label::from_raw(label)
    }

    pub fn create_label(&mut self) -> Label {
        let label = self.labels.len() as u32;
        self.labels.push(self.code.len() as isize);
        Label::from_raw(label)
    }

    pub fn define_label(&mut self, label: Label) -> &mut Self {
        assert_eq!(
            self.labels[label.raw() as usize],
            isize::MAX,
            "tried to redefine an already defined label"
        );
        self.labels[label.raw() as usize] = self.code.len() as isize;
        self
    }

    pub fn push_with_label<T>(&mut self, label: Label, instruction: Instruction<T>) -> &mut Self
    where
        T: core::fmt::Display,
    {
        self.define_label(label);
        self.push(instruction)
    }

    #[inline]
    pub fn get_label_origin_offset(&self, label: Label) -> Option<isize> {
        let offset = self.labels[label.raw() as usize];
        if offset == isize::MAX {
            None
        } else {
            Some(offset)
        }
    }

    pub fn get_label_origin_offset_or_panic(&self, label: Label) -> isize {
        self.get_label_origin_offset(label)
            .expect("tried to fetch a label offset for a label that was not defined")
    }

    pub fn set_label_origin_offset(&mut self, label: Label, offset: isize) {
        self.labels[label.raw() as usize] = offset;
    }

    #[inline(always)]
    fn add_fixup(&mut self, instruction_offset: usize, instruction_length: usize, target_label: Label, kind: FixupKind) {
        debug_assert!((target_label.raw() as usize) < self.labels.len());
        debug_assert!(
            (kind.offset() as usize) < instruction_length,
            "instruction is {} bytes long and yet its target fixup starts at {}",
            instruction_length,
            kind.offset()
        );
        debug_assert!((kind.length() as usize) < instruction_length);
        debug_assert!((kind.offset() as usize + kind.length() as usize) <= instruction_length);
        self.fixups.push(Fixup {
            target_label,
            instruction_offset,
            instruction_length: instruction_length as u8,
            kind,
        });
    }

    #[inline(always)]
    pub fn reserve<const INSTRUCTIONS: usize>(&mut self) {
        InstBuf::reserve::<INSTRUCTIONS>(&mut self.code);
        self.guaranteed_capacity = INSTRUCTIONS;
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn push<T>(&mut self, instruction: Instruction<T>) -> &mut Self
    where
        T: core::fmt::Display,
    {
        #[cfg(debug_assertions)]
        log::trace!("{:08x}: {}", self.origin + self.code.len() as u64, instruction);

        if self.guaranteed_capacity == 0 {
            InstBuf::reserve::<1>(&mut self.code);
            self.guaranteed_capacity = 1;
        }

        let instruction_offset = self.code.len();

        // SAFETY: We've reserved space for at least one instruction.
        unsafe {
            instruction.bytes.encode_into_vec_unsafe(&mut self.code);
        }
        self.guaranteed_capacity -= 1;

        if let Some((label, fixup)) = instruction.fixup {
            self.add_fixup(instruction_offset, instruction.bytes.len(), label, fixup);
        }

        self
    }

    pub fn push_raw(&mut self, bytes: &[u8]) -> &mut Self {
        self.code.extend_from_slice(bytes);
        self
    }

    pub fn finalize(&mut self) -> AssembledCode {
        for fixup in self.fixups.drain(..) {
            let origin = fixup.instruction_offset + fixup.instruction_length as usize;
            let target_absolute = self.labels[fixup.target_label.raw() as usize];
            if target_absolute == isize::MAX {
                log::trace!("Undefined label found: {}", fixup.target_label);
                continue;
            }

            let opcode = (fixup.kind.0 << 8) >> 8;
            let fixup_offset = fixup.kind.offset();
            let fixup_length = fixup.kind.length();

            if fixup_offset >= 1 {
                self.code[fixup.instruction_offset] = opcode as u8;
                if fixup_offset >= 2 {
                    self.code[fixup.instruction_offset + 1] = (opcode >> 8) as u8;
                    if fixup_offset >= 3 {
                        self.code[fixup.instruction_offset + 2] = (opcode >> 16) as u8;
                    }
                }
            }

            let offset = target_absolute - origin as isize;
            let p = fixup.instruction_offset + fixup_offset as usize;
            if fixup_length == 1 {
                if offset > i8::MAX as isize || offset < i8::MIN as isize {
                    panic!("out of range jump");
                }
                self.code[p] = offset as i8 as u8;
            } else if fixup_length == 4 {
                if offset > i32::MAX as isize || offset < i32::MIN as isize {
                    panic!("out of range jump");
                }
                self.code[p..p + 4].copy_from_slice(&(offset as i32).to_le_bytes());
            } else {
                unreachable!()
            }
        }

        AssembledCode(self)
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn code_mut(&mut self) -> &mut [u8] {
        &mut self.code
    }

    pub fn spare_capacity(&self) -> usize {
        self.code.capacity() - self.code.len()
    }

    pub fn resize(&mut self, size: usize, fill_with: u8) {
        self.code.resize(size, fill_with)
    }

    pub fn reserve_code(&mut self, length: usize) {
        self.code.reserve(length);
    }

    pub fn reserve_labels(&mut self, length: usize) {
        self.labels.reserve(length);
    }

    pub fn reserve_fixups(&mut self, length: usize) {
        self.fixups.reserve(length);
    }

    pub fn clear(&mut self) {
        self.origin = 0;
        self.code.clear();
        self.labels.clear();
        self.fixups.clear();
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Label(NonZeroU32);

impl Label {
    #[inline]
    pub fn raw(self) -> u32 {
        self.0.get() - 1
    }

    #[inline]
    pub fn from_raw(value: u32) -> Self {
        Label(NonZeroU32::new(value + 1).unwrap())
    }
}

impl core::fmt::Display for Label {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_fmt(core::format_args!("<{}>", self.0))
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct FixupKind(u32);

impl FixupKind {
    #[inline]
    const fn offset(self) -> u32 {
        (self.0 >> 24) & 0b11
    }

    #[inline]
    const fn length(self) -> u32 {
        self.0 >> 28
    }

    #[inline]
    pub const fn new_1(opcode: u32, length: u32) -> Self {
        FixupKind((1 << 24) | (length << 28) | opcode)
    }

    #[inline]
    pub const fn new_2(opcode: [u32; 2], length: u32) -> Self {
        let opcode = opcode[0] | (opcode[1] << 8);
        FixupKind((2 << 24) | (length << 28) | opcode)
    }

    #[inline]
    pub const fn new_3(opcode: [u32; 3], length: u32) -> Self {
        let opcode = opcode[0] | (opcode[1] << 8) | (opcode[2] << 16);
        FixupKind((3 << 24) | (length << 28) | opcode)
    }
}

#[derive(Copy, Clone)]
pub struct Instruction<T> {
    pub(crate) instruction: T,
    pub(crate) bytes: InstBuf,
    pub(crate) fixup: Option<(Label, FixupKind)>,
}

impl<T> core::fmt::Debug for Instruction<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.instruction.fmt(fmt)
    }
}

impl<T> core::fmt::Display for Instruction<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.instruction.fmt(fmt)
    }
}

impl<T> Instruction<T> {
    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}

const MAXIMUM_INSTRUCTION_SIZE: usize = 16;

#[derive(Copy, Clone)]
pub struct InstBuf {
    out_1: u64,
    out_2: u64,
    length: usize,
}

#[allow(clippy::new_without_default)]
impl InstBuf {
    #[inline]
    pub fn new() -> Self {
        Self {
            out_1: 0,
            out_2: 0,
            length: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn append(&mut self, byte: u8) {
        if self.length < 8 {
            self.out_1 |= u64::from(byte) << (self.length * 8);
        } else {
            self.out_2 |= u64::from(byte) << ((self.length - 8) * 8);
        }

        self.length += 1;
    }

    #[inline]
    pub fn append2(&mut self, bytes: [u8; 2]) {
        self.append(bytes[0]);
        self.append(bytes[1]);
    }

    #[inline]
    pub fn append4(&mut self, bytes: [u8; 4]) {
        self.append(bytes[0]);
        self.append(bytes[1]);
        self.append(bytes[2]);
        self.append(bytes[3]);
    }

    #[inline]
    unsafe fn encode_into_raw(self, output: *mut u8) {
        core::ptr::write_unaligned(output.cast::<u64>(), u64::from_le(self.out_1));
        core::ptr::write_unaligned(output.add(8).cast::<u64>(), u64::from_le(self.out_2));
    }

    #[allow(clippy::debug_assert_with_mut_call)]
    #[inline]
    unsafe fn encode_into_vec_unsafe(self, output: &mut Vec<u8>) {
        debug_assert!(output.spare_capacity_mut().len() >= MAXIMUM_INSTRUCTION_SIZE);

        self.encode_into_raw(output.spare_capacity_mut().as_mut_ptr().cast());
        let new_length = output.len() + self.length;
        output.set_len(new_length);
    }

    #[cold]
    #[inline(never)]
    fn reserve_impl(output: &mut Vec<u8>, length: usize) {
        output.reserve(length);
    }

    #[inline(always)]
    fn reserve<const INSTRUCTIONS: usize>(output: &mut Vec<u8>) {
        let count = INSTRUCTIONS.checked_mul(MAXIMUM_INSTRUCTION_SIZE).unwrap();
        if output.spare_capacity_mut().len() < count {
            Self::reserve_impl(output, count);
            if output.spare_capacity_mut().len() < count {
                // SAFETY: `reserve` made sure that we have this much capacity, so this is safe.
                unsafe {
                    core::hint::unreachable_unchecked();
                }
            }
        }
    }

    #[inline]
    pub fn from_array<const N: usize>(array: [u8; N]) -> Self {
        if N > MAXIMUM_INSTRUCTION_SIZE {
            panic!();
        }

        let mut out = Self::new();
        for value in array {
            out.append(value);
        }
        out
    }

    pub fn to_vec(self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(MAXIMUM_INSTRUCTION_SIZE);

        // SAFETY: We've reserved space for at least one instruction.
        unsafe {
            self.encode_into_vec_unsafe(&mut vec);
        }

        vec
    }
}

#[test]
fn test_inst_buf() {
    assert_eq!(InstBuf::from_array([0x01]).to_vec(), [0x01]);
    assert_eq!(InstBuf::from_array([0x01, 0x02]).to_vec(), [0x01, 0x02]);
    assert_eq!(InstBuf::from_array([0x01, 0x02, 0x03]).to_vec(), [0x01, 0x02, 0x03]);
    assert_eq!(InstBuf::from_array([0x01, 0x02, 0x03, 0x04]).to_vec(), [0x01, 0x02, 0x03, 0x04]);
    assert_eq!(
        InstBuf::from_array([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]).to_vec(),
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
    );
    assert_eq!(
        InstBuf::from_array([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09]).to_vec(),
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09]
    );
    assert_eq!(
        InstBuf::from_array([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]).to_vec(),
        [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]
    );
}
