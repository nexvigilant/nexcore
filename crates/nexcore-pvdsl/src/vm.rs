//! # PVDSL Virtual Machine
//!
//! Stack-based bytecode interpreter with native function support.

use super::bytecode::{CompiledProgram, OpCode};
use super::error::{PvdslError, PvdslResult};
use super::runtime::RuntimeValue;
use std::collections::HashMap;

/// Native function type
pub type NativeFunction = fn(&[RuntimeValue]) -> PvdslResult<RuntimeValue>;

/// PVDSL Virtual Machine
pub struct VirtualMachine {
    stack: Vec<RuntimeValue>,
    variables: HashMap<String, RuntimeValue>,
    native_functions: HashMap<(String, String), NativeFunction>,
}

impl VirtualMachine {
    /// Create a new VM with default native functions
    #[must_use]
    pub fn new() -> Self {
        let mut vm = Self {
            stack: Vec::with_capacity(64),
            variables: HashMap::new(),
            native_functions: HashMap::new(),
        };
        vm.register_default_natives();
        vm
    }

    /// Register default native functions from nexcore crates
    fn register_default_natives(&mut self) {
        // Signal detection functions
        self.register_native("signal", "prr", native_prr);
        self.register_native("signal", "ror", native_ror);
        self.register_native("signal", "ic", native_ic);
        self.register_native("signal", "ebgm", native_ebgm);
        self.register_native("signal", "chi_square", native_chi_square);
        self.register_native("signal", "fisher", native_fisher);

        // Sequential signal detection functions
        self.register_native("signal", "sprt", native_sprt);
        self.register_native("signal", "maxsprt", native_maxsprt);
        self.register_native("signal", "cusum", native_cusum);
        self.register_native("signal", "mgps", native_mgps);

        // Causality assessment
        self.register_native("causality", "naranjo", native_naranjo);
        self.register_native("causality", "who_umc", native_who_umc);
        self.register_native("causality", "rucam", native_rucam);

        // MedDRA functions
        self.register_native("meddra", "levenshtein", native_levenshtein);
        self.register_native("meddra", "similarity", native_similarity);

        // Risk analytics functions
        self.register_native("risk", "sar", native_sar);
        self.register_native("risk", "es", native_expected_shortfall);
        self.register_native("risk", "monte_carlo", native_monte_carlo);

        // Date functions
        self.register_native("date", "now", native_date_now);
        self.register_native("date", "diff_days", native_date_diff);

        // Math functions
        self.register_native("math", "abs", native_abs);
        self.register_native("math", "sqrt", native_sqrt);
        self.register_native("math", "pow", native_pow);
        self.register_native("math", "log", native_log);
        self.register_native("math", "ln", native_ln);
        self.register_native("math", "exp", native_exp);
        self.register_native("math", "min", native_min);
        self.register_native("math", "max", native_max);
        self.register_native("math", "floor", native_floor);
        self.register_native("math", "ceil", native_ceil);
        self.register_native("math", "round", native_round);

        // Classification functions
        self.register_native("classify", "hartwig_siegel", native_hartwig_siegel);

        // Chemistry-based capability assessment
        self.register_native("chem", "arrhenius", native_arrhenius);
        self.register_native("chem", "michaelis", native_michaelis);
        self.register_native("chem", "hill", native_hill);
        self.register_native("chem", "henderson", native_henderson);
        self.register_native("chem", "halflife", native_halflife);
        self.register_native("chem", "sqi", native_sqi);
    }

    /// Register a native function
    pub fn register_native(&mut self, namespace: &str, name: &str, func: NativeFunction) {
        self.native_functions
            .insert((namespace.to_string(), name.to_string()), func);
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: &str, value: RuntimeValue) {
        self.variables.insert(name.to_string(), value);
    }

    /// Get a variable value
    #[must_use]
    pub fn get_variable(&self, name: &str) -> Option<&RuntimeValue> {
        self.variables.get(name)
    }

    /// Run a compiled program
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails.
    pub fn run(&mut self, program: &CompiledProgram) -> PvdslResult<Option<RuntimeValue>> {
        let mut pc = 0;

        while pc < program.instructions.len() {
            let instr = program.instructions[pc];

            match instr {
                OpCode::LoadConst(idx) => {
                    let val = program.constants.get(idx as usize).ok_or_else(|| {
                        PvdslError::Execution("Constant index out of bounds".into())
                    })?;
                    self.stack.push(val.clone());
                }
                OpCode::LoadVar(idx) => {
                    let name = program
                        .names
                        .get(idx as usize)
                        .ok_or_else(|| PvdslError::Execution("Name index out of bounds".into()))?;
                    let val = self
                        .variables
                        .get(name)
                        .cloned()
                        .unwrap_or(RuntimeValue::Null);
                    self.stack.push(val);
                }
                OpCode::StoreVar(idx) => {
                    let name = program
                        .names
                        .get(idx as usize)
                        .ok_or_else(|| PvdslError::Execution("Name index out of bounds".into()))?;
                    let val = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    self.variables.insert(name.clone(), val);
                }
                OpCode::PopTop => {
                    self.stack.pop();
                }
                OpCode::BinaryAdd => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.stack.push(RuntimeValue::Number(a + b));
                }
                OpCode::BinarySub => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.stack.push(RuntimeValue::Number(a - b));
                }
                OpCode::BinaryMul => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.stack.push(RuntimeValue::Number(a * b));
                }
                OpCode::BinaryDiv => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    if b == 0.0 {
                        return Err(PvdslError::Execution("Division by zero".into()));
                    }
                    self.stack.push(RuntimeValue::Number(a / b));
                }
                OpCode::BinaryMod => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.stack.push(RuntimeValue::Number(a % b));
                }
                OpCode::CompareEq => {
                    let b = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    self.stack.push(RuntimeValue::Boolean(a == b));
                }
                OpCode::CompareNe => {
                    let b = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    self.stack.push(RuntimeValue::Boolean(a != b));
                }
                OpCode::CompareLt => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.stack.push(RuntimeValue::Boolean(a < b));
                }
                OpCode::CompareLe => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.stack.push(RuntimeValue::Boolean(a <= b));
                }
                OpCode::CompareGt => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.stack.push(RuntimeValue::Boolean(a > b));
                }
                OpCode::CompareGe => {
                    let b = self.pop_number()?;
                    let a = self.pop_number()?;
                    self.stack.push(RuntimeValue::Boolean(a >= b));
                }
                OpCode::LogicalAnd => {
                    let b = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    self.stack
                        .push(RuntimeValue::Boolean(a.is_truthy() && b.is_truthy()));
                }
                OpCode::LogicalOr => {
                    let b = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    self.stack
                        .push(RuntimeValue::Boolean(a.is_truthy() || b.is_truthy()));
                }
                OpCode::Jump(target) => {
                    pc = target as usize;
                    continue;
                }
                OpCode::JumpIfFalse(target) => {
                    let val = self.stack.pop().ok_or(PvdslError::StackUnderflow)?;
                    if !val.is_truthy() {
                        pc = target as usize;
                        continue;
                    }
                }
                OpCode::CallFunction(_arg_count) => {
                    // User-defined functions not yet supported
                    return Err(PvdslError::Execution(
                        "User-defined function calls not yet supported".into(),
                    ));
                }
                OpCode::CallNamespaced(ns_idx, name_idx, arg_count) => {
                    let ns = program.names.get(ns_idx as usize).ok_or_else(|| {
                        PvdslError::Execution("Namespace index out of bounds".into())
                    })?;
                    let name = program.names.get(name_idx as usize).ok_or_else(|| {
                        PvdslError::Execution("Function name index out of bounds".into())
                    })?;

                    // Collect arguments from stack
                    let mut args = Vec::with_capacity(arg_count as usize);
                    for _ in 0..arg_count {
                        args.push(self.stack.pop().ok_or(PvdslError::StackUnderflow)?);
                    }
                    args.reverse();

                    // Look up and call native function
                    let key = (ns.clone(), name.clone());
                    let func = self
                        .native_functions
                        .get(&key)
                        .ok_or_else(|| PvdslError::FunctionNotFound(format!("{ns}::{name}")))?;
                    let result = func(&args)?;
                    self.stack.push(result);
                }
                OpCode::Return => {
                    return Ok(self.stack.pop());
                }
                OpCode::BuildList(count) => {
                    let mut list = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        list.push(self.stack.pop().ok_or(PvdslError::StackUnderflow)?);
                    }
                    list.reverse();
                    self.stack.push(RuntimeValue::List(list));
                }
                OpCode::Nop => {}
            }

            pc += 1;
        }

        Ok(self.stack.pop())
    }

    fn pop_number(&mut self) -> PvdslResult<f64> {
        match self.stack.pop() {
            Some(RuntimeValue::Number(n)) => Ok(n),
            Some(other) => Err(PvdslError::TypeError {
                expected: "number".into(),
                actual: format!("{other:?}"),
            }),
            None => Err(PvdslError::StackUnderflow),
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

// === Native Function Implementations ===

fn native_prr(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 4 {
        return Err(PvdslError::InvalidArgument(
            "prr requires 4 arguments (a, b, c, d)".into(),
        ));
    }
    let a = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u64;
    let b = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as u64;
    let c = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as u64;
    let d = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as u64;

    let table = nexcore_pv_core::ContingencyTable::new(a, b, c, d);
    let criteria = nexcore_pv_core::SignalCriteria::evans();
    let result = nexcore_pv_core::calculate_prr(&table, &criteria);
    Ok(RuntimeValue::Number(result.point_estimate))
}

fn native_ror(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 4 {
        return Err(PvdslError::InvalidArgument(
            "ror requires 4 arguments (a, b, c, d)".into(),
        ));
    }
    let a = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u64;
    let b = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as u64;
    let c = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as u64;
    let d = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as u64;

    let table = nexcore_pv_core::ContingencyTable::new(a, b, c, d);
    let criteria = nexcore_pv_core::SignalCriteria::evans();
    let result = nexcore_pv_core::calculate_ror(&table, &criteria);
    Ok(RuntimeValue::Number(result.point_estimate))
}

fn native_ic(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 4 {
        return Err(PvdslError::InvalidArgument(
            "ic requires 4 arguments (a, b, c, d)".into(),
        ));
    }
    let a = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u64;
    let b = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as u64;
    let c = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as u64;
    let d = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as u64;

    let table = nexcore_pv_core::ContingencyTable::new(a, b, c, d);
    let criteria = nexcore_pv_core::SignalCriteria::evans();
    let result = nexcore_pv_core::calculate_ic(&table, &criteria);
    Ok(RuntimeValue::Number(result.point_estimate))
}

fn native_ebgm(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 4 {
        return Err(PvdslError::InvalidArgument(
            "ebgm requires 4 arguments (a, b, c, d)".into(),
        ));
    }
    let a = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u64;
    let b = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as u64;
    let c = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as u64;
    let d = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as u64;

    let table = nexcore_pv_core::ContingencyTable::new(a, b, c, d);
    let criteria = nexcore_pv_core::SignalCriteria::evans();
    let result = nexcore_pv_core::calculate_ebgm(&table, &criteria);
    Ok(RuntimeValue::Number(result.point_estimate))
}

fn native_naranjo(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 5 {
        return Err(PvdslError::InvalidArgument(
            "naranjo requires 5 arguments".into(),
        ));
    }
    let temporal = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as i32;
    let dechallenge = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as i32;
    let rechallenge = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as i32;
    let alternatives = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as i32;
    let previous = args[4].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[4]),
    })? as i32;

    let result = nexcore_pv_core::causality::calculate_naranjo_quick(
        temporal,
        dechallenge,
        rechallenge,
        alternatives,
        previous,
    );
    Ok(RuntimeValue::Number(f64::from(result.score)))
}

fn native_levenshtein(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 2 {
        return Err(PvdslError::InvalidArgument(
            "levenshtein requires 2 string arguments".into(),
        ));
    }
    let a = args[0].as_string().ok_or_else(|| PvdslError::TypeError {
        expected: "string".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let b = args[1].as_string().ok_or_else(|| PvdslError::TypeError {
        expected: "string".into(),
        actual: format!("{:?}", args[1]),
    })?;

    let result = crate::text::levenshtein(a, b);
    Ok(RuntimeValue::Number(result.distance as f64))
}

fn native_abs(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 1 {
        return Err(PvdslError::InvalidArgument(
            "abs requires 1 argument".into(),
        ));
    }
    let n = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    Ok(RuntimeValue::Number(n.abs()))
}

fn native_sqrt(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 1 {
        return Err(PvdslError::InvalidArgument(
            "sqrt requires 1 argument".into(),
        ));
    }
    let n = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    Ok(RuntimeValue::Number(n.sqrt()))
}

fn native_pow(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 2 {
        return Err(PvdslError::InvalidArgument(
            "pow requires 2 arguments".into(),
        ));
    }
    let base = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let exp = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    Ok(RuntimeValue::Number(base.powf(exp)))
}

fn native_log(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 1 {
        return Err(PvdslError::InvalidArgument(
            "log requires 1 argument".into(),
        ));
    }
    let n = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    Ok(RuntimeValue::Number(n.log10()))
}

fn native_ln(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 1 {
        return Err(PvdslError::InvalidArgument("ln requires 1 argument".into()));
    }
    let n = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    Ok(RuntimeValue::Number(n.ln()))
}

fn native_exp(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 1 {
        return Err(PvdslError::InvalidArgument(
            "exp requires 1 argument".into(),
        ));
    }
    let n = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    Ok(RuntimeValue::Number(n.exp()))
}

fn native_min(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 2 {
        return Err(PvdslError::InvalidArgument(
            "min requires 2 arguments".into(),
        ));
    }
    let a = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let b = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    Ok(RuntimeValue::Number(a.min(b)))
}

fn native_max(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 2 {
        return Err(PvdslError::InvalidArgument(
            "max requires 2 arguments".into(),
        ));
    }
    let a = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let b = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    Ok(RuntimeValue::Number(a.max(b)))
}

fn native_floor(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 1 {
        return Err(PvdslError::InvalidArgument(
            "floor requires 1 argument".into(),
        ));
    }
    let n = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    Ok(RuntimeValue::Number(n.floor()))
}

fn native_ceil(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 1 {
        return Err(PvdslError::InvalidArgument(
            "ceil requires 1 argument".into(),
        ));
    }
    let n = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    Ok(RuntimeValue::Number(n.ceil()))
}

fn native_round(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 1 {
        return Err(PvdslError::InvalidArgument(
            "round requires 1 argument".into(),
        ));
    }
    let n = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    Ok(RuntimeValue::Number(n.round()))
}

// === Signal Detection Extended ===

fn native_chi_square(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 4 {
        return Err(PvdslError::InvalidArgument(
            "chi_square requires 4 arguments (a, b, c, d)".into(),
        ));
    }
    let a = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u64;
    let b = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as u64;
    let c = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as u64;
    let d = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as u64;
    let n = (a + b + c + d) as f64;
    let expected_a = ((a + b) as f64 * (a + c) as f64) / n;
    let chi2 = if expected_a > 0.0 {
        (a as f64 - expected_a).powi(2) / expected_a
    } else {
        0.0
    };
    Ok(RuntimeValue::Number(chi2))
}

fn native_fisher(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 4 {
        return Err(PvdslError::InvalidArgument(
            "fisher requires 4 arguments (a, b, c, d)".into(),
        ));
    }
    let a = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u64;
    let b = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as u64;
    let c = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as u64;
    let d = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as u64;
    let table = nexcore_pv_core::ContingencyTable::new(a, b, c, d);
    let result = nexcore_pv_core::fisher_exact_test(&table);
    Ok(RuntimeValue::Number(result.p_value_two_tailed))
}

// === Causality Extended ===

fn native_who_umc(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 5 {
        return Err(PvdslError::InvalidArgument("who_umc requires 5 arguments (temporal, dechallenge, rechallenge, alternatives, plausibility)".into()));
    }
    let temporal = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as i32;
    let dechallenge = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as i32;
    let rechallenge = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as i32;
    let alternatives = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as i32;
    let plausibility = args[4].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[4]),
    })? as i32;
    let result = nexcore_pv_core::causality::calculate_who_umc_quick(
        temporal,
        dechallenge,
        rechallenge,
        alternatives,
        plausibility,
    );
    let score = match result.category {
        nexcore_pv_core::causality::WhoUmcCategory::Certain => 6.0,
        nexcore_pv_core::causality::WhoUmcCategory::ProbableLikely => 5.0,
        nexcore_pv_core::causality::WhoUmcCategory::Possible => 4.0,
        nexcore_pv_core::causality::WhoUmcCategory::Unlikely => 2.0,
        nexcore_pv_core::causality::WhoUmcCategory::ConditionalUnclassified => 1.0,
        nexcore_pv_core::causality::WhoUmcCategory::UnassessableUnclassifiable => 0.0,
    };
    Ok(RuntimeValue::Number(score))
}

fn native_rucam(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() < 5 {
        return Err(PvdslError::InvalidArgument(
            "rucam requires at least 5 arguments".into(),
        ));
    }
    let time_to_onset = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u32;
    let time_to_resolution = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as u32;
    let alt_causes_score = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as i32;
    let rechall_score = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as i32;
    let prev_info_score = args[4].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[4]),
    })? as i32;
    let onset_score: i32 = if time_to_onset <= 5 {
        2
    } else if time_to_onset <= 90 {
        1
    } else {
        0
    };
    let resolution_score: i32 = if time_to_resolution <= 30 {
        2
    } else if time_to_resolution <= 180 {
        1
    } else {
        0
    };
    let total = onset_score + resolution_score + alt_causes_score + rechall_score + prev_info_score;
    Ok(RuntimeValue::Number(f64::from(total)))
}

// === MedDRA Extended ===

fn native_similarity(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 2 {
        return Err(PvdslError::InvalidArgument(
            "similarity requires 2 string arguments".into(),
        ));
    }
    let a = args[0].as_string().ok_or_else(|| PvdslError::TypeError {
        expected: "string".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let b = args[1].as_string().ok_or_else(|| PvdslError::TypeError {
        expected: "string".into(),
        actual: format!("{:?}", args[1]),
    })?;
    let result = crate::text::levenshtein(a, b);
    Ok(RuntimeValue::Number(result.similarity))
}

// === Risk Analytics ===

fn native_sar(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() < 2 {
        return Err(PvdslError::InvalidArgument(
            "sar requires at least 2 arguments (rates as list, confidence)".into(),
        ));
    }
    let rates = match &args[0] {
        RuntimeValue::List(list) => list
            .iter()
            .filter_map(|v| v.as_number())
            .collect::<Vec<f64>>(),
        _ => {
            return Err(PvdslError::TypeError {
                expected: "list".into(),
                actual: format!("{:?}", args[0]),
            });
        }
    };
    let confidence = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    if rates.is_empty() {
        return Ok(RuntimeValue::Number(0.0));
    }
    let mut sorted = rates.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((1.0 - confidence) * sorted.len() as f64) as usize;
    let sar = sorted
        .get(idx.min(sorted.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0.0);
    Ok(RuntimeValue::Number(sar))
}

fn native_expected_shortfall(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() < 2 {
        return Err(PvdslError::InvalidArgument(
            "es requires at least 2 arguments (rates as list, confidence)".into(),
        ));
    }
    let rates = match &args[0] {
        RuntimeValue::List(list) => list
            .iter()
            .filter_map(|v| v.as_number())
            .collect::<Vec<f64>>(),
        _ => {
            return Err(PvdslError::TypeError {
                expected: "list".into(),
                actual: format!("{:?}", args[0]),
            });
        }
    };
    let confidence = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    if rates.is_empty() {
        return Ok(RuntimeValue::Number(0.0));
    }
    let mut sorted = rates.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let cutoff = ((1.0 - confidence) * sorted.len() as f64) as usize;
    let tail: Vec<f64> = sorted.iter().take(cutoff.max(1)).copied().collect();
    let es = tail.iter().sum::<f64>() / tail.len() as f64;
    Ok(RuntimeValue::Number(es))
}

fn native_monte_carlo(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() < 3 {
        return Err(PvdslError::InvalidArgument(
            "monte_carlo requires 3 arguments (mean, std, simulations)".into(),
        ));
    }
    let mean = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let std = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    let sims = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as u32;
    let mut rng_state: u64 = 42;
    let samples: Vec<f64> = (0..sims)
        .map(|_| {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u1 = (rng_state as f64) / (u64::MAX as f64);
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u2 = (rng_state as f64) / (u64::MAX as f64);
            mean + std
                * (-2.0 * u1.max(1e-10).ln()).sqrt()
                * (2.0 * std::f64::consts::PI * u2).cos()
        })
        .collect();
    let avg = samples.iter().sum::<f64>() / samples.len() as f64;
    Ok(RuntimeValue::Number(avg))
}

// === Date Functions ===

fn native_date_now(_args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as f64)
        .unwrap_or(0.0);
    Ok(RuntimeValue::Number(now))
}

fn native_date_diff(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 2 {
        return Err(PvdslError::InvalidArgument(
            "diff_days requires 2 arguments (timestamp1, timestamp2)".into(),
        ));
    }
    let t1 = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let t2 = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    let diff_secs = (t2 - t1).abs();
    let diff_days = diff_secs / 86400.0;
    Ok(RuntimeValue::Number(diff_days))
}

// === Classification ===

fn native_hartwig_siegel(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() < 3 {
        return Err(PvdslError::InvalidArgument(
            "hartwig_siegel requires 3 arguments (severity, outcome, hospitalization)".into(),
        ));
    }
    let severity = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u32;
    let outcome = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as u32;
    let hospitalization = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as u32;
    let level: u32 = if outcome >= 5 {
        7
    } else if hospitalization > 30 {
        6
    } else if hospitalization > 0 {
        5
    } else if severity >= 3 {
        4
    } else if severity >= 2 {
        3
    } else if severity >= 1 {
        2
    } else {
        1
    };
    Ok(RuntimeValue::Number(f64::from(level)))
}

// === Sequential Signal Detection ===

/// SPRT: Sequential Probability Ratio Test
/// Args: observed, expected, null_rr, alt_rr, alpha, beta
/// Returns: log-likelihood ratio
fn native_sprt(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() < 4 {
        return Err(PvdslError::InvalidArgument(
            "sprt requires at least 4 arguments (observed, expected, null_rr, alt_rr)".into(),
        ));
    }
    let observed = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u32;
    let expected = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    let null_rr = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })?;
    let alt_rr = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })?;

    if expected <= 0.0 || alt_rr <= null_rr {
        return Ok(RuntimeValue::Number(0.0));
    }

    // Calculate log-likelihood ratio: n * ln(rr1/rr0) - E * (rr1 - rr0)
    let n = f64::from(observed);
    let llr = n * (alt_rr / null_rr).ln() - expected * (alt_rr - null_rr);

    Ok(RuntimeValue::Number(llr))
}

/// MaxSPRT: Maximized Sequential Probability Ratio Test
/// Args: observed, expected, alpha (optional), max_expected (optional)
/// Returns: test statistic (2 * LLR)
fn native_maxsprt(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() < 2 {
        return Err(PvdslError::InvalidArgument(
            "maxsprt requires at least 2 arguments (observed, expected)".into(),
        ));
    }
    let observed = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u32;
    let expected = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;

    // Use the existing NexCore implementation
    let config = nexcore_pv_core::signals::sequential::MaxSprtConfig::default();
    match nexcore_pv_core::signals::sequential::calculate_maxsprt(observed, expected, &config) {
        Ok(result) => Ok(RuntimeValue::Number(result.test_statistic)),
        Err(_) => Ok(RuntimeValue::Number(0.0)),
    }
}

/// CuSum: Cumulative Sum control chart
/// Args: values (list), baseline, k (slack), h (control limit)
/// Returns: final CuSum value (upper)
fn native_cusum(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() < 2 {
        return Err(PvdslError::InvalidArgument(
            "cusum requires at least 2 arguments (values_list, baseline)".into(),
        ));
    }
    let values = match &args[0] {
        RuntimeValue::List(list) => list
            .iter()
            .filter_map(|v| v.as_number())
            .collect::<Vec<f64>>(),
        _ => {
            return Err(PvdslError::TypeError {
                expected: "list".into(),
                actual: format!("{:?}", args[0]),
            });
        }
    };
    let baseline = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    let k = args.get(2).and_then(|a| a.as_number()).unwrap_or(0.5);
    let h = args.get(3).and_then(|a| a.as_number()).unwrap_or(5.0);

    if values.is_empty() || baseline <= 0.0 {
        return Ok(RuntimeValue::Number(0.0));
    }

    // Calculate CuSum (upper)
    let mut cusum_upper = 0.0;
    for x in &values {
        let z = (x - baseline) / baseline.sqrt();
        cusum_upper = (cusum_upper + z - k).max(0.0);
    }

    // Return final value (or h if exceeded - indicating signal)
    Ok(RuntimeValue::Number(if cusum_upper > h {
        h
    } else {
        cusum_upper
    }))
}

/// MGPS: Multi-item Gamma Poisson Shrinker (full result)
/// Args: a, b, c, d (contingency table)
/// Returns: Dict with ebgm, eb05, eb95, is_signal
fn native_mgps(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 4 {
        return Err(PvdslError::InvalidArgument(
            "mgps requires 4 arguments (a, b, c, d)".into(),
        ));
    }
    let a = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })? as u64;
    let b = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })? as u64;
    let c = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })? as u64;
    let d = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })? as u64;

    // Use nexcore-pv-core signal types for bayesian module
    use nexcore_pv_core::signals::core::types::{ContingencyTable, SignalCriteria};
    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();

    match nexcore_pv_core::signals::bayesian::calculate_ebgm(&table, &criteria) {
        Ok(result) => {
            // Return as a Dict with full MGPS result
            let mut dict = std::collections::HashMap::new();
            dict.insert(
                "ebgm".to_string(),
                RuntimeValue::Number(result.point_estimate),
            );
            dict.insert("eb05".to_string(), RuntimeValue::Number(result.lower_ci));
            dict.insert("eb95".to_string(), RuntimeValue::Number(result.upper_ci));
            dict.insert(
                "is_signal".to_string(),
                RuntimeValue::Boolean(result.is_signal),
            );
            dict.insert(
                "case_count".to_string(),
                RuntimeValue::Number(result.case_count as f64),
            );
            Ok(RuntimeValue::Dict(dict))
        }
        Err(_) => {
            let mut dict = std::collections::HashMap::new();
            dict.insert("ebgm".to_string(), RuntimeValue::Number(0.0));
            dict.insert("eb05".to_string(), RuntimeValue::Number(0.0));
            dict.insert("eb95".to_string(), RuntimeValue::Number(0.0));
            dict.insert("is_signal".to_string(), RuntimeValue::Boolean(false));
            dict.insert("case_count".to_string(), RuntimeValue::Number(a as f64));
            Ok(RuntimeValue::Dict(dict))
        }
    }
}

// === Chemistry-based Capability Assessment ===

/// Arrhenius: Adoption potential from learning barrier
/// Args: barrier (activation energy), motivation (temperature factor), resources (pre-exponential)
/// Returns: normalized adoption potential [0, 1]
fn native_arrhenius(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 3 {
        return Err(PvdslError::InvalidArgument(
            "arrhenius requires 3 arguments (barrier, motivation, resources)".into(),
        ));
    }
    let barrier = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let motivation = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    let resources = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })?;

    let normalized = crate::chemistry::arrhenius(barrier, motivation, resources);
    Ok(RuntimeValue::Number(normalized))
}

/// Michaelis-Menten: Capacity efficiency from saturation
/// Args: vmax (max capacity), demand (current load), km (half-saturation)
/// Returns: normalized capacity efficiency [0, 1]
fn native_michaelis(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 3 {
        return Err(PvdslError::InvalidArgument(
            "michaelis requires 3 arguments (vmax, demand, km)".into(),
        ));
    }
    let vmax = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let demand = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    let km = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })?;

    let normalized = crate::chemistry::michaelis_menten(vmax, demand, km);
    Ok(RuntimeValue::Number(normalized))
}

/// Hill equation: Synergy coefficient from cooperativity
/// Args: n (hill coefficient), skills (count), threshold (half-max)
/// Returns: normalized synergy [0, 1]
fn native_hill(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 3 {
        return Err(PvdslError::InvalidArgument(
            "hill requires 3 arguments (n, skills, threshold)".into(),
        ));
    }
    let n = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let skills = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    let threshold = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })?;

    let normalized = crate::chemistry::hill(n, skills, threshold);
    Ok(RuntimeValue::Number(normalized))
}

/// Henderson-Hasselbalch: Stability from buffer ratio
/// Args: stabilizing (factors), destabilizing (factors)
/// Returns: normalized stability [0, 1]
fn native_henderson(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 2 {
        return Err(PvdslError::InvalidArgument(
            "henderson requires 2 arguments (stabilizing, destabilizing)".into(),
        ));
    }
    let stabilizing = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let destabilizing = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;

    let normalized = crate::chemistry::henderson_hasselbalch(stabilizing, destabilizing);
    Ok(RuntimeValue::Number(normalized))
}

/// Half-life decay: Freshness from elapsed time
/// Args: elapsed (days since update), half_life (days)
/// Returns: normalized freshness [0, 1]
fn native_halflife(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 2 {
        return Err(PvdslError::InvalidArgument(
            "halflife requires 2 arguments (elapsed, half_life)".into(),
        ));
    }
    let elapsed = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let half_life = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;

    let normalized = crate::chemistry::half_life(elapsed, half_life);
    Ok(RuntimeValue::Number(normalized))
}

/// Unified SQI: All 5 equations combined
/// Args: adoption, capacity, synergy, stability, freshness (all normalized 0-1)
/// Returns: SQI score [0, 10]
fn native_sqi(args: &[RuntimeValue]) -> PvdslResult<RuntimeValue> {
    if args.len() != 5 {
        return Err(PvdslError::InvalidArgument(
            "sqi requires 5 arguments (adoption, capacity, synergy, stability, freshness)".into(),
        ));
    }
    let adoption = args[0].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[0]),
    })?;
    let capacity = args[1].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[1]),
    })?;
    let synergy = args[2].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[2]),
    })?;
    let stability = args[3].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[3]),
    })?;
    let freshness = args[4].as_number().ok_or_else(|| PvdslError::TypeError {
        expected: "number".into(),
        actual: format!("{:?}", args[4]),
    })?;

    use crate::sqi_weights::{
        WEIGHT_ADOPTION, WEIGHT_CAPACITY, WEIGHT_FRESHNESS, WEIGHT_STABILITY, WEIGHT_SYNERGY,
    };

    let weighted_sum = adoption * WEIGHT_ADOPTION
        + capacity * WEIGHT_CAPACITY
        + synergy * WEIGHT_SYNERGY
        + stability * WEIGHT_STABILITY
        + freshness * WEIGHT_FRESHNESS;

    let score = weighted_sum * 10.0;
    Ok(RuntimeValue::Number(score))
}

#[cfg(test)]
mod tests {
    use super::super::bytecode::BytecodeGenerator;
    use super::super::lexer::Lexer;
    use super::super::parser::Parser;
    use super::*;

    fn compile_and_run(source: &str) -> PvdslResult<Option<RuntimeValue>> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser
            .parse()
            .map_err(|e| PvdslError::Execution(e.to_string()))?;
        let compiled = BytecodeGenerator::new().compile(&program);
        let mut vm = VirtualMachine::new();
        vm.run(&compiled)
    }

    #[test]
    fn test_e2e_execution() {
        let result = compile_and_run("x = 42\nreturn x").unwrap().unwrap();
        assert_eq!(result, RuntimeValue::Number(42.0));
    }

    #[test]
    fn test_arithmetic() {
        let result = compile_and_run("x = 2 + 3 * 4\nreturn x").unwrap().unwrap();
        // 2 + (3 * 4) = 14 due to precedence
        assert_eq!(result, RuntimeValue::Number(14.0));
    }

    #[test]
    fn test_comparison() {
        let result = compile_and_run("x = 5 > 3\nreturn x").unwrap().unwrap();
        assert_eq!(result, RuntimeValue::Boolean(true));
    }

    #[test]
    fn test_if_statement() {
        let result = compile_and_run("x = 10\nif x > 5 { y = 1 } else { y = 0 }\nreturn y")
            .unwrap()
            .unwrap();
        assert_eq!(result, RuntimeValue::Number(1.0));
    }

    #[test]
    fn test_signal_prr() {
        let result = compile_and_run("result = signal::prr(10, 90, 100, 9800)\nreturn result")
            .unwrap()
            .unwrap();
        if let RuntimeValue::Number(prr) = result {
            // PRR = (10/100) / (90/9800) ≈ 10.89
            assert!(prr > 9.0 && prr < 12.0, "PRR was {prr}");
        } else {
            panic!("Expected number result");
        }
    }

    #[test]
    fn test_levenshtein() {
        let result =
            compile_and_run("dist = meddra::levenshtein(\"hello\", \"hallo\")\nreturn dist")
                .unwrap()
                .unwrap();
        assert_eq!(result, RuntimeValue::Number(1.0));
    }

    #[test]
    fn test_math_sqrt() {
        let result = compile_and_run("x = math::sqrt(16)\nreturn x")
            .unwrap()
            .unwrap();
        assert_eq!(result, RuntimeValue::Number(4.0));
    }

    #[test]
    fn test_math_pow() {
        let result = compile_and_run("x = math::pow(2, 10)\nreturn x")
            .unwrap()
            .unwrap();
        assert_eq!(result, RuntimeValue::Number(1024.0));
    }

    // === Chemistry Capability Tests ===

    #[test]
    fn test_chem_arrhenius() {
        // barrier=6, motivation=1.2, resources=1.0
        let result = compile_and_run("a = chem::arrhenius(6.0, 1.2, 1.0)\nreturn a")
            .unwrap()
            .unwrap();
        if let RuntimeValue::Number(n) = result {
            assert!(n > 0.0 && n <= 1.0, "Arrhenius result {n} out of range");
        } else {
            panic!("Expected number result");
        }
    }

    #[test]
    fn test_chem_michaelis() {
        // vmax=10000, demand=5000, km=8000
        let result = compile_and_run("c = chem::michaelis(10000.0, 5000.0, 8000.0)\nreturn c")
            .unwrap()
            .unwrap();
        if let RuntimeValue::Number(n) = result {
            assert!(n > 0.0 && n <= 1.0, "Michaelis result {n} out of range");
        } else {
            panic!("Expected number result");
        }
    }

    #[test]
    fn test_chem_hill() {
        // n=1.5, skills=5, threshold=3
        let result = compile_and_run("s = chem::hill(1.5, 5.0, 3.0)\nreturn s")
            .unwrap()
            .unwrap();
        if let RuntimeValue::Number(n) = result {
            assert!(n > 0.0 && n <= 1.0, "Hill result {n} out of range");
        } else {
            panic!("Expected number result");
        }
    }

    #[test]
    fn test_chem_henderson() {
        // stabilizing=8, destabilizing=3
        let result = compile_and_run("st = chem::henderson(8.0, 3.0)\nreturn st")
            .unwrap()
            .unwrap();
        if let RuntimeValue::Number(n) = result {
            assert!(n > 0.0 && n <= 1.0, "Henderson result {n} out of range");
        } else {
            panic!("Expected number result");
        }
    }

    #[test]
    fn test_chem_halflife() {
        // elapsed=60 days, half_life=548 days
        let result = compile_and_run("f = chem::halflife(60.0, 548.0)\nreturn f")
            .unwrap()
            .unwrap();
        if let RuntimeValue::Number(n) = result {
            assert!(
                n > 0.8 && n <= 1.0,
                "Halflife result {n} out of range (expect ~0.93)"
            );
        } else {
            panic!("Expected number result");
        }
    }

    #[test]
    fn test_chem_sqi_composition() {
        // Full SQI calculation via PVDSL
        let source = r#"
            a = chem::arrhenius(6.0, 1.2, 1.0)
            c = chem::michaelis(10000.0, 5000.0, 8000.0)
            s = chem::hill(1.5, 5.0, 3.0)
            st = chem::henderson(8.0, 3.0)
            f = chem::halflife(60.0, 548.0)
            return chem::sqi(a, c, s, st, f)
        "#;
        let result = compile_and_run(source).unwrap().unwrap();
        if let RuntimeValue::Number(sqi) = result {
            assert!(sqi > 5.0 && sqi <= 10.0, "SQI {sqi} out of expected range");
        } else {
            panic!("Expected number result");
        }
    }

    #[test]
    fn test_chem_sqi_direct() {
        // Direct SQI from normalized values
        let result = compile_and_run("sqi = chem::sqi(0.75, 0.62, 0.70, 0.85, 0.93)\nreturn sqi")
            .unwrap()
            .unwrap();
        if let RuntimeValue::Number(sqi) = result {
            // Expected: (0.75×0.20 + 0.62×0.25 + 0.70×0.20 + 0.85×0.20 + 0.93×0.15) × 10 ≈ 7.5
            assert!((sqi - 7.5).abs() < 0.1, "SQI was {sqi}, expected ~7.5");
        } else {
            panic!("Expected number result");
        }
    }
}
