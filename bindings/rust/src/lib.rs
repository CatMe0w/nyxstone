use anyhow::anyhow;
use derive_builder::Builder;
use ffi::create_nyxstone_ffi;

#[derive(Builder)]
#[builder(setter(into), pattern = "owned", build_fn(error = "anyhow::Error", skip))]
/// Public interface for calling nyxstone from rust.
/// # Examples
///
/// ```rust
/// # use nyxstone::{Nyxstone, NyxstoneBuilder, Instruction};
/// # fn main() -> anyhow::Result<()> {
/// let nyxstone = NyxstoneBuilder::default()
///     .with_triple("x86_64")
///     .build()?;
///
/// let instructions = nyxstone.assemble_to_instructions("mov rax, rbx", 0x1000, &[])?;
///
/// assert_eq!(
///      instructions,
///      vec![Instruction {
///          address: 0x1000,
///          assembly: "mov rax, rbx".into(),
///          bytes: vec![0x48, 0x89, 0xd8]
///      }]
/// );
/// # Ok(())
/// # }
/// ```
pub struct Nyxstone {
    /// Specifies the LLVM target triple Nyxstone uses. MUST be set!
    ///
    /// # Parameters
    /// - `value`: The LLVM target triple, for example `x86_64`.
    ///
    /// # Returns
    /// The updated NyxstoneBuilder instance.
    #[builder(field(type = "String"), setter(name = "with_triple"))]
    _triple: (), // Empty variable which holds the triple in the auto-generated `NyxstoneBuilder`.

    /// Specifies the CPU for which LLVM assembles/disassembles internally, which might enable/disable certain features.
    ///
    /// # Parameters
    /// - `value`: The CPU name, for example `corei7`
    ///
    /// # Returns
    /// The updated NyxstoneBuilder instance.
    #[builder(field(type = "String"), setter(name = "with_cpu"))]
    _cpu: (),
    #[builder(field(type = "Vec<String>"), setter(custom))]
    _enabled_features: (), // Empty variable which holds the enabled features in the auto-generated `NyxstoneBuilder`.
    #[builder(field(type = "Vec<String>"), setter(custom))]
    _disabled_features: (), // Empty variable which holds the disabled features in the auto-generated `NyxstoneBuilder`.

    /// Specifies in what format immediates should be represented in the output.
    ///
    /// # Parameters
    /// - `value`: One of the [`IntegerBase`] variants.
    ///
    /// # Returns
    /// The updated NyxstoneBuilder instance.
    #[builder(field(type = "IntegerBase"), setter(name = "with_immediate_style"))]
    _imm_style: (), // Empty variable which holds the hex style for instruction printing in the auto-generated `NyxstoneBuilder`.

    /// The c++ `unique_ptr` holding the actual `NyxstoneFFI` instance.
    /// Is an empty type in the `NyxstoneBuilder`.
    #[builder(setter(skip), field(type = "()"))]
    inner: cxx::UniquePtr<ffi::NyxstoneFFI>,
}

// Re-export
pub type Instruction = ffi::Instruction;
pub type LabelDefinition<'name> = ffi::LabelDefinition<'name>;

/// Configuration options for the integer style of immediates in disassembly output.
pub enum IntegerBase {
    /// Immediates are represented in decimal format.
    Dec = 0,
    /// Immediates are represented in hex format, prepended with 0x, for example: 0xff.
    HexPrefix = 1,
    /// Immediates are represented in hex format, suffixed with h, for example: 0ffh.
    HexSuffix = 2,
}

// Implement default for HexStyle for derive_builder.
impl Default for IntegerBase {
    fn default() -> Self {
        IntegerBase::Dec
    }
}

impl Into<ffi::IntegerBase> for IntegerBase {
    fn into(self) -> ffi::IntegerBase {
        match self {
            IntegerBase::Dec => ffi::IntegerBase::Dec,
            IntegerBase::HexPrefix => ffi::IntegerBase::HexPrefix,
            IntegerBase::HexSuffix => ffi::IntegerBase::HexSuffix,
        }
    }
}

impl Nyxstone {
    /// Translates assembly instructions at a given start address to bytes.
    ///
    /// # Note:
    /// Does not support assembly directives that impact the layout (f. i., .section, .org).
    ///
    /// # Parameters:
    /// - `assembly`: The instructions to assemble.
    /// - `address`: The start location of the instructions.
    /// - `labels`: Additional label definitions by absolute address.
    ///
    /// # Returns:
    /// Ok() and bytecode on success, Err() otherwise.
    pub fn assemble_to_bytes(
        &self,
        assembly: &str,
        address: u64,
        labels: &[LabelDefinition],
    ) -> anyhow::Result<Vec<u8>> {
        self.inner
            .assemble_to_bytes(assembly, address, &labels)
            .map_err(|err| anyhow!("Error during assemble (= '{assembly}' at {address}): {err}."))
    }

    /// Translates assembly instructions at a given start address to instruction details containing bytes.
    ///
    /// # Note:
    /// Does not support assembly directives that impact the layout (f. i., .section, .org).
    ///
    /// # Parameters:
    /// - `assembly`: The instructions to assemble.
    /// - `address`: The start location of the instructions.
    /// - `labels`: Additional label definitions by absolute address.
    ///
    /// # Returns:
    /// Ok() and instruction details on success, Err() otherwise.
    pub fn assemble_to_instructions(
        &self,
        assembly: &str,
        address: u64,
        labels: &[LabelDefinition],
    ) -> anyhow::Result<Vec<Instruction>> {
        self.inner
            .assemble_to_instructions(assembly, address, &labels)
            .map_err(|err| anyhow!("Error during assemble (= '{assembly}' at {address}): {err}."))
    }

    /// Translates bytes to disassembly text at a given start address.
    ///
    /// # Parameters:
    /// - `bytes`: The bytes to be disassembled.
    /// - `address`: The start address of the bytes.
    /// - `count`: Number of instructions to be disassembled. If zero is supplied, all instructions are disassembled.
    ///
    /// # Returns:
    /// Ok() and disassembly text on success, Err() otherwise.
    pub fn disassemble_to_text(&self, bytes: &[u8], address: u64, count: usize) -> anyhow::Result<String> {
        self.inner
            .disassemble_to_text(bytes, address, count)
            .map_err(|err| anyhow!("Error during disassembly: {err}."))
    }

    /// Translates bytes to instruction details containing disassembly text at a given start address.
    ///
    /// # Parameters:
    /// - `bytes`: The bytes to be disassembled.
    /// - `address`: The start address of the bytes.
    /// - `count`: Number of instructions to be disassembled. If zero is supplied, all instructions are disassembled.
    ///
    /// # Returns:
    /// Ok() and Instruction details on success, Err() otherwise.
    pub fn disassemble_to_instructions(
        &self,
        bytes: &[u8],
        address: u64,
        count: usize,
    ) -> anyhow::Result<Vec<Instruction>> {
        self.inner
            .disassemble_to_instructions(bytes, address, count)
            .map_err(|err| anyhow!("Error during disassembly: {err}."))
    }
}

unsafe impl Send for Nyxstone {}

impl NyxstoneBuilder {
    /// Enables a given llvm feature.
    ///
    /// # Parameters
    /// - `feature`: The feature to be enabled, should not contain a prepended '+'.
    ///
    /// # Returns
    /// The updated NyxstoneBuilder instance.
    pub fn with_feature(mut self, feature: &str) -> Self {
        // If the feature is already enabled, we do not need to do anything
        if self._enabled_features.iter().any(|f| f == feature) {
            return self;
        }

        // If the feature is disabled, we need to remove it from the disabled features.
        if let Some(pos) = self._disabled_features.iter().position(|f| f == feature) {
            self._disabled_features.swap_remove(pos); // We do not need the features to be in-order
        }

        self._enabled_features.push(feature.into());

        self
    }

    /// Disables a given llvm feature.
    ///
    /// # Parameters
    /// - `feature`: The feature to be disabled, should not contain a prepended '-'.
    ///
    /// # Returns
    /// The updated NyxstoneBuilder instance.
    pub fn without_feature(mut self, feature: &str) -> Self {
        // If the feature is already disabled, we do not need to do anything
        if self._disabled_features.iter().any(|f| f == feature) {
            return self;
        }

        // If the feature is enabled, we need to remove it from the enabled features.
        if let Some(pos) = self._enabled_features.iter().position(|f| f == feature) {
            self._enabled_features.swap_remove(pos); // We do not need the features to be in-order
        }

        self._disabled_features.push(feature.into());

        self
    }

    /// Builds a Nyxstone instance from the NyxstoneBuilder.
    ///
    /// # Returns
    /// Ok() and the Nyxstone instance on success, Err() otherwise.
    pub fn build(self) -> anyhow::Result<Nyxstone> {
        if self._triple.is_empty() {
            return Err(anyhow::anyhow!("No 'triple' supplied to builder."));
        }

        // Build the features string for LLVM
        // LLVM features are comma-seperated strings representing the feature
        // preprended with a '+' for an enabled feature and a '-' for a disabled feature.
        let features: Vec<_> = self
            ._enabled_features
            .into_iter()
            .map(|mut feature| {
                feature.insert(0, '+');
                feature
            })
            .chain(self._disabled_features.into_iter().map(|mut feature| {
                feature.insert(0, '-');
                feature
            }))
            .collect();

        let features: String = features.join(",");

        Ok(Nyxstone {
            _triple: (),
            _cpu: (),
            _enabled_features: (),
            _disabled_features: (),
            _imm_style: (),

            inner: create_nyxstone_ffi(&self._triple, &self._cpu, &features, self._imm_style.into())
                .map_err(|e| anyhow!(e.what().to_owned()))?,
        })
    }
}

#[cxx::bridge]
mod ffi {
    /// Defines the location of a label by absolute address.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct LabelDefinition<'name> {
        /// Name of the label.
        pub name: &'name str,
        /// Absolute address of the label.
        pub address: u64,
    }

    /// Instruction details
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Instruction {
        /// Absolute address of the instruction.
        address: u64,
        /// Assembly string representing the instruction.
        assembly: String,
        /// Byte code of the instruction.
        bytes: Vec<u8>,
    }

    /// Configuration options for the integer style of immediates in disassembly output.
    pub enum IntegerBase {
        /// Immediates are represented in decimal format.
        Dec = 0,
        /// Immediates are represented in hex format, prepended with 0x, for example: 0xff.
        HexPrefix = 1,
        /// Immediates are represented in hex format, suffixed with h, for example: 0ffh.
        HexSuffix = 2,
    }

    unsafe extern "C++" {
        include!("nyxstone/src/nyxstone_ffi.hpp");

        type NyxstoneFFI;

        /// Constructs a Nyxstone instance for the architecture and cpu specified by the llvm-style target triple and
        /// cpu. Also allows enabling and disabling features via the `features` string.
        /// Features are comma-seperated feature strings, which start with a plus if they should be enabled and a minus
        /// if they should be disabled.
        /// Params:
        /// - triple_name: The llvm triple.
        /// - cpu: The cpu to be used, can be empty
        /// - features: llvm features string (features delimited by `,` with `+` for enable and `-` for disable), can be empty
        /// # Returns
        /// Ok() and UniquePtr holding a NyxstoneFFI on success, Err() otherwise.
        fn create_nyxstone_ffi(
            triple_name: &str,
            cpu: &str,
            features: &str,
            style: IntegerBase,
        ) -> Result<UniquePtr<NyxstoneFFI>>;

        // Translates assembly instructions at a given start address to bytes.
        // Additional label definitions by absolute address may be supplied.
        // Does not support assembly directives that impact the layout (f. i., .section, .org).
        fn assemble_to_bytes(
            self: &NyxstoneFFI,
            assembly: &str,
            address: u64,
            labels: &[LabelDefinition],
        ) -> Result<Vec<u8>>;

        // Translates assembly instructions at a given start address to instruction details containing bytes.
        // Additional label definitions by absolute address may be supplied.
        // Does not support assembly directives that impact the layout (f. i., .section, .org).
        fn assemble_to_instructions(
            self: &NyxstoneFFI,
            assembly: &str,
            address: u64,
            labels: &[LabelDefinition],
        ) -> Result<Vec<Instruction>>;

        // Translates bytes to disassembly text at given start address.
        fn disassemble_to_text(self: &NyxstoneFFI, bytes: &[u8], address: u64, count: usize) -> Result<String>;

        // Translates bytes to instruction details containing disassembly text at a given start address.
        fn disassemble_to_instructions(
            self: &NyxstoneFFI,
            bytes: &[u8],
            address: u64,
            count: usize,
        ) -> Result<Vec<Instruction>>;
    }
}

unsafe impl Send for ffi::NyxstoneFFI {}
