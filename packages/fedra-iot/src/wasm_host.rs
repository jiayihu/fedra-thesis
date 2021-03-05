use alloc::boxed::Box;
use alloc::fmt::Formatter;
use alloc::format;
use alloc::string::String;
use wasmi::nan_preserving_float::F32;
use wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, HostError as WasmiHostError,
    ImportsBuilder, Module, ModuleImportResolver, ModuleInstance, ModuleRef, RuntimeArgs,
    RuntimeValue, Signature, StackRecycler, Trap, TrapKind, ValueType,
};

#[derive(Debug)]
pub enum HostError {
    Interpreter(InterpreterError),
    UnknownFunctionIndex(String),
}

impl alloc::fmt::Display for HostError {
    fn fmt(&self, f: &mut Formatter<'_>) -> alloc::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<InterpreterError> for HostError {
    fn from(e: InterpreterError) -> Self {
        HostError::Interpreter(e)
    }
}

impl From<Trap> for HostError {
    fn from(e: Trap) -> Self {
        HostError::Interpreter(InterpreterError::from(e))
    }
}

impl WasmiHostError for HostError {}

pub struct WasmHost<'a> {
    imports: ImportsBuilder<'a>,
    instance: Option<ModuleRef>,
    stack_recycler: StackRecycler,
}

impl<'a> WasmHost<'a> {
    pub fn create_module(buffer: &[u8]) -> Result<Module, HostError> {
        let module = wasmi::Module::from_buffer(buffer)?;

        module.deny_floating_point_64()?;
        module.validate_memory_size(1)?;

        Ok(module)
    }

    pub fn set_instance(&mut self, module: &Module) -> Result<(), HostError> {
        let instance = ModuleInstance::new(module, &self.imports)?;
        let instance = instance.assert_no_start();

        self.instance = Some(instance);

        Ok(())
    }

    pub fn invoke<E: Externals>(
        &mut self,
        name: &str,
        runtime: &mut E,
    ) -> Result<Option<RuntimeValue>, HostError> {
        let instance = self
            .instance
            .as_ref()
            .ok_or(InterpreterError::Function(String::from(
                "Cannot invoke without an instance",
            )))?;

        self.stack_recycler.clear();
        let result =
            instance.invoke_export_with_stack(name, &[], runtime, &mut self.stack_recycler)?;

        Ok(result)
    }

    pub fn setup_default(&mut self) -> Result<(), HostError> {
        let wasm = include_bytes!("./wasm/functions.wasm");
        let module = Self::create_module(wasm)?;

        self.set_instance(&module)?;

        Ok(())
    }
}

const VALUE_STACK_LIMIT: usize = 4 * 1024;
const CALL_STACK_LIMIT: usize = 42;

impl<'a> Default for WasmHost<'a> {
    fn default() -> Self {
        let mut imports = ImportsBuilder::default();
        imports.push_resolver("env", &RuntimeImportResolver);

        let stack_recycler = StackRecycler::with_limits(VALUE_STACK_LIMIT, CALL_STACK_LIMIT);

        WasmHost {
            imports,
            instance: None,
            stack_recycler,
        }
    }
}

unsafe impl Send for WasmHost<'static> {}

pub struct Runtime {
    pub rainfall: f32,
}

const GET_TEMP_FN: usize = 0;
const SET_TEMP_FN: usize = 1;

impl Externals for Runtime {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            GET_TEMP_FN => Ok(Some(RuntimeValue::from(F32::from(self.rainfall)))),
            SET_TEMP_FN => {
                let value: F32 = args.nth_checked(0)?;
                let value: f32 = value.into();
                self.rainfall = value;

                Ok(None)
            }
            _ => Err(
                TrapKind::Host(Box::new(HostError::UnknownFunctionIndex(format!(
                    "Cannot invoke function with index {}",
                    index
                ))))
                .into(),
            ),
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime { rainfall: 1.0f32 }
    }
}

struct RuntimeImportResolver;

impl ModuleImportResolver for RuntimeImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        let func_ref = match field_name {
            "set_temp" => {
                FuncInstance::alloc_host(Signature::new(&[ValueType::I32][..], None), SET_TEMP_FN)
            }
            "get_temp" => {
                FuncInstance::alloc_host(Signature::new(&[][..], Some(ValueType::I32)), GET_TEMP_FN)
            }
            _ => {
                return Err(InterpreterError::Function(format!(
                    "Host module doesn't export function with name {}",
                    field_name
                )));
            }
        };

        Ok(func_ref)
    }
}
