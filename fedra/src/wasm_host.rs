use alloc::format;
use wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, ImportsBuilder, Module,
    ModuleImportResolver, ModuleInstance, ModuleRef, NopExternals, RuntimeArgs, RuntimeValue,
    Signature, Trap, ValueType,
};

pub fn setup_default(host: &mut WasmHost) {
    let wasm = include_bytes!("./wasm/add.wasm");
    let module = wasmi::Module::from_buffer(&wasm).expect("Failed to load wasm");
    assert!(module.deny_floating_point().is_ok());

    host.set_instance(&module);
}

pub struct WasmHost<'a> {
    imports: ImportsBuilder<'a>,
    instance: Option<ModuleRef>,
}

impl<'a> WasmHost<'a> {
    pub fn set_instance(&mut self, module: &Module) {
        let instance =
            ModuleInstance::new(module, &self.imports).expect("Failed to instantiate wasm module");

        let instance = instance
            .run_start(&mut NopExternals)
            .expect("WASM start function trapped");

        self.instance = Some(instance);
    }

    pub fn invoke<E: Externals>(
        &self,
        name: &str,
        args: &[RuntimeValue],
        runtime: &mut E,
    ) -> Option<RuntimeValue> {
        let instance = self
            .instance
            .as_ref()
            .expect("No module instance initialized");
        let result = instance
            .invoke_export(name, args, runtime)
            .expect("Failed to invoke the export");

        result
    }
}

impl<'a> Default for WasmHost<'a> {
    fn default() -> Self {
        let mut imports = ImportsBuilder::default();
        imports.push_resolver("host", &RuntimeImportResolver);

        WasmHost {
            imports,
            instance: None,
        }
    }
}

unsafe impl Send for WasmHost<'static> {}

#[derive(Default)]
pub struct Runtime {
    pub temp: i32,
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
            GET_TEMP_FN => Ok(Some(RuntimeValue::from(self.temp))),
            SET_TEMP_FN => {
                let value: i32 = args.nth_checked(0)?;
                self.temp = value;

                Ok(None)
            }
            _ => panic!("Unknown function index"),
        }
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
