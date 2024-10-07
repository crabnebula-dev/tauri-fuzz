//! Definition of a security policy for our fuzzer

use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use thiserror::Error;

pub type FuzzPolicy = Vec<FunctionPolicy>;

/// Policy set around a function
#[derive(Debug, Clone)]
pub struct FunctionPolicy {
    /// Function name
    pub name: String,

    /// Lib in which the function resides
    pub lib: String,

    /// Conditions that define the policy
    pub rule: Rule,

    /// Number of arguments that the function takes
    pub nb_parameters: u32,

    /// Description of the function policy
    pub description: String,

    /// If the function monitored is a Rust function.
    /// Rust function names are mangled during compilation
    pub is_rust_function: bool,
}

impl FunctionPolicy {
    /// Check the function policy in the specified context and if the invocation should be blocked
    pub fn should_block(&mut self, context: &Context) -> bool {
        let should_block = self.rule.should_block(context);
        match should_block {
            Ok(false) => false,
            Ok(true) => {
                log::error!("{}", &self.policy_infringement_message(context));
                // println!("{}", self.policy_infringement_message(context));
                true
            }
            Err(e) => panic!(
                "Error in policy evaluation: {:?}.\nRule: {:?}.\nContext: {:?}",
                e, self.rule, context
            ),
        }
    }

    pub fn policy_infringement_message(&self, context: &Context) -> String {
        format!(
            "Policy was broken at function [{}].\nDescription: {}\nRule: {:?}\nContext: {:?}",
            self.name, self.description, self.rule, context,
        )
    }
}

// NOTE: we used `Arc` for simplicity in `ConditionOnParameters` and `ConditionOnReturnValue` but
// maybe using cloneable `Box` can improve performance such as in this example:
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=6ca48c4cff92370c907ecf4c548ee33c

/// ConditionOnParameters is a closure on the registers containing the parameters of the function.
/// Registers can contain a value but also a pointer depending on the type of the parameters.
pub type ConditionOnParameters = Arc<dyn Fn(&[usize]) -> Result<bool, RuleError>>;
/// ConditionOnReturnValue is a closure on the value contained in the return value register
/// This can contain a value but also a pointer depending on the type of the return value.
pub type ConditionOnReturnValue = Arc<dyn Fn(usize) -> Result<bool, RuleError>>;
/// ConditionOnParameters but with an additional argument that can be used as storage to pass
/// information for later usage
pub type ConditionOnParametersWithStorage =
    Arc<dyn Fn(&[usize], &mut Option<usize>) -> Result<bool, RuleError>>;
/// ConditionOnReturnValue but with an additional argument that can be used for the analysis
pub type ConditionOnReturnValueWithStorage =
    Arc<dyn Fn(usize, &mut Option<usize>) -> Result<bool, RuleError>>;

/// Rule that the function has to adhere to respect the policy
#[derive(Clone)]
pub enum Rule {
    /// Rule is checked on function entry given a condition on parameters
    OnEntry(ConditionOnParameters),

    /// Rule is checked on function exit given the return value
    OnExit(ConditionOnReturnValue),

    /// Rule is checked both on function entry and exit
    /// Last argument is used to store information that was gathered at entry and used at exit.
    /// For example we need to check the value of a pointer that was passed as an argument.
    /// This is common practice in C to store results of a function in mutable pointer given as
    /// argument
    OnEntryAndExit(
        ConditionOnParametersWithStorage,
        ConditionOnReturnValueWithStorage,
        Option<usize>,
    ),
}

impl Debug for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Rule::OnEntry(_) => write!(f, "Rule::OnEntry"),
            Rule::OnExit(_) => write!(f, "Rule::OnExit"),
            Rule::OnEntryAndExit(_, _, _) => write!(f, "Rule::OnEntryAndExit"),
        }
    }
}

/// Errors that may happen when evaluating a rule given a context
#[derive(Debug, Error)]
pub enum RuleError {
    /// The number of parameters given in the context does not match the number of
    /// parameters expected in the context
    #[error(
        "`{0:?}` of parameters given in the context doesn't match the number expected by the rule"
    )]
    NumberOfParametersDontMatch(usize),

    #[error("Error while converting a string")]
    StringConversionError(#[from] std::str::Utf8Error),

    #[error("Error while trying to convert a parameter into Rust type: `{0}`")]
    ParametersTypeConversionError(String),

    /// Generic error that happened during the rule evaluation
    #[error("Error during evaluation: `{0}`")]
    EvaluationError(String),

    /// A value was expected to be found in storage when evaluating a rule at a function exit
    #[error("No value stored for rule evaluation at exit: `{0}`")]
    ExpectedStorageEmpty(String),
}

/// Context when evaluating if a function is respecting the specified policy.
/// TODO Check if parameters order always follow the function signature.
/// Parameters are None when the evaluating after the execution of the targeted function
/// The return value is None when evaluating at the beginning of the function and Some when
/// evaluating at the end of the function.
pub enum Context {
    EntryContext(Vec<usize>),
    LeaveContext(usize),
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Context::EntryContext(parameters) => {
                write!(f, "Function entry with parameters: {:?}", parameters)
            }
            Context::LeaveContext(return_value) => {
                write!(
                    f,
                    "Function exit with return value as usize: {:?}",
                    return_value
                )
            }
        }
    }
}

use Context::*;
use Rule::*;
impl Rule {
    /// Evaluate if rule is true given a context
    /// If it returns true it means that the rule has been verified and does not respect the policy
    /// We don't evaluate "entry" rules when given a "leave" context and vice-versa.
    fn should_block(&mut self, context: &Context) -> Result<bool, RuleError> {
        match self {
            // Evaluate the function on entry
            OnEntry(block_condition) => match context {
                EntryContext(parameters) => block_condition(parameters),
                LeaveContext(_) => Ok(false),
            },

            // We block the function on entry
            OnExit(block_condition) => match context {
                EntryContext(_) => Ok(false),
                LeaveContext(return_value) => block_condition(*return_value),
            },
            // We block the function on entry
            OnEntryAndExit(entry_condition, exit_condition, stored_value) => match context {
                EntryContext(parameters) => entry_condition(parameters, stored_value),
                LeaveContext(return_value) => exit_condition(*return_value, stored_value),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_on_parameters() {
        // Block on function entry
        let mut rule = Rule::OnEntry(crate::block_on_entry());
        let context = EntryContext(vec![]);
        assert!(rule.should_block(&context).unwrap());

        // Check parameters
        let mut rule1 = Rule::OnEntry(Arc::new(|params| Ok(params[0] == 1)));
        let mut rule2 = Rule::OnEntry(Arc::new(|params| Ok(params[1] % 2 == 0)));
        let mut rule3 = Rule::OnEntry(Arc::new(|params| Ok(params[2] == 4)));
        let context = EntryContext(vec![1, 2, 3]);
        assert!(rule1.should_block(&context).unwrap());
        assert!(rule2.should_block(&context).unwrap());
        assert!(!rule3.should_block(&context).unwrap());
    }

    #[test]
    fn rule_on_return_value() {
        let mut rule = Rule::OnExit(Arc::new(|v| Ok(v == 1)));
        let mut rule2 = Rule::OnExit(Arc::new(|v| Ok(v == 4)));
        let context = LeaveContext(1);
        assert!(rule.should_block(&context).unwrap());
        assert!(!rule2.should_block(&context).unwrap());
    }

    #[test]
    fn rule_wrong_context() {
        // Entry context with leave rule
        let context = EntryContext(vec![]);
        let mut rule = Rule::OnExit(Arc::new(|_| Ok(false)));
        assert!(!rule.should_block(&context).unwrap());

        // Leave context with entry rule
        let context = LeaveContext(0);
        let mut rule = Rule::OnEntry(crate::block_on_entry());
        assert!(!rule.should_block(&context).unwrap());

        // Entry context with not enough parameters
        let mut rule = Rule::OnEntry(Arc::new(|params| {
            if params.len() < 3 {
                Err(RuleError::NumberOfParametersDontMatch(params.len()))
            } else {
                Ok(params[2] == 4)
            }
        }));
        let context = EntryContext(vec![1, 2]);
        assert!(rule.should_block(&context).is_err());
    }

    #[test]
    fn rule_on_entry_and_exit() {
        let entry_context = EntryContext(vec![1, 2, 3]);
        let leave_context = LeaveContext(4);
        let mut rule = Rule::OnEntryAndExit(
            Arc::new(|_parameters, storage| {
                *storage = Some(4);
                Ok(false)
            }),
            Arc::new(|_return_value, storage| {
                let stored_value = *storage;
                assert!(stored_value.is_some());
                assert_eq!(4, stored_value.unwrap());
                Ok(false)
            }),
            None,
        );
        assert!(!rule.should_block(&entry_context).unwrap());
        assert!(!rule.should_block(&leave_context).unwrap());
    }
}
