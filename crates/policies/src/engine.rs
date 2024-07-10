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
    /// Check the function policy in the specified context
    pub fn is_respected(&self, context: &Context) -> bool {
        let is_respected = self.rule.is_respected(context);
        match is_respected {
            Ok(true) => true,
            Ok(false) => {
                log::error!("{}", &self.policy_infringement_message(context));
                println!("{}", self.policy_infringement_message(context));
                false
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
// maybe using clonable `Box` can improve performance such as in this example:
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=6ca48c4cff92370c907ecf4c548ee33c

/// ConditionOnParameters is a closure on the registers containing the parameters of the function.
/// Registers can contain a value but also a pointer depending on the type of the parameters.
pub type ConditionOnParameters = Arc<dyn Fn(&[usize]) -> Result<bool, RuleError>>;
/// ConditionOnReturnValue is a closure on the value contained in the return value register
/// This can contain a value but also a pointer depending on the type of the return value.
pub type ConditionOnReturnValue = Arc<dyn Fn(usize) -> Result<bool, RuleError>>;

/// Rule that the function has to adhere to respect the policy
// TODO to improve perf we can separate entry rule and leave rule into two different types, the UX
// is not as good then
#[derive(Clone)]
pub enum Rule {
    /// Rule is checked on function entry given a condition on parameters
    OnEntry(ConditionOnParameters),

    /// Rule is checked on function exit given the return value
    OnLeave(ConditionOnReturnValue),
}

impl Debug for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Rule::OnEntry(_) => write!(f, "Rule::OnEntry"),
            Rule::OnLeave(_) => write!(f, "Rule::OnLeave"),
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
}

/// Context when evaluating if a function is respecting the specified policy.
/// TODO Check if parameters order always follow the function signature.
/// Parameters are None when the evaluating after the execution of the targeted function
/// The return value is None when evaluating at the beginning of the function and Some when
/// evaluating at the end of the function.
#[derive(Debug)]
pub enum Context {
    EntryContext(Vec<usize>),
    LeaveContext(usize),
}

use Context::*;
use Rule::*;
impl Rule {
    /// Evaluate if rule is true given a context
    /// If it returns true it means that the rule has been verified and does not respect the policy
    /// We don't evaluate "entry" rules when given a "leave" context and vice-versa.
    fn is_respected(&self, context: &Context) -> Result<bool, RuleError> {
        match self {
            // Evaluate the function on entry
            OnEntry(condition) => match context {
                EntryContext(parameters) => condition(parameters),
                LeaveContext(_) => Ok(true),
            },

            // We block the function on entry
            OnLeave(condition) => match context {
                EntryContext(_) => Ok(true),
                LeaveContext(return_value) => condition(*return_value),
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
        let rule = Rule::OnEntry(crate::block_on_entry());
        let context = EntryContext(vec![]);
        assert!(!rule.is_respected(&context).unwrap());

        // Check parameters
        let rule1 = Rule::OnEntry(Arc::new(|params| Ok(params[0] == 1)));
        let rule2 = Rule::OnEntry(Arc::new(|params| Ok(params[1] % 2 == 0)));
        let rule3 = Rule::OnEntry(Arc::new(|params| Ok(params[2] == 4)));
        let context = EntryContext(vec![1, 2, 3]);
        assert!(rule1.is_respected(&context).unwrap());
        assert!(rule2.is_respected(&context).unwrap());
        assert!(!rule3.is_respected(&context).unwrap());
    }

    #[test]
    fn rule_on_return_value() {
        let rule = Rule::OnLeave(Arc::new(|v| Ok(v == 1)));
        let rule2 = Rule::OnLeave(Arc::new(|v| Ok(v == 4)));
        let context = LeaveContext(1);
        assert!(rule.is_respected(&context).unwrap());
        assert!(!rule2.is_respected(&context).unwrap());
    }

    #[test]
    fn rule_wrong_context() {
        // Entry context with leave rule
        let context = EntryContext(vec![]);
        let rule = Rule::OnLeave(Arc::new(|_| Ok(false)));
        assert!(rule.is_respected(&context).unwrap());

        // Leave context with entry rule
        let context = LeaveContext(0);
        let rule = Rule::OnEntry(crate::block_on_entry());
        assert!(rule.is_respected(&context).unwrap());

        // Entry context with not enough parameters
        let rule = Rule::OnEntry(Arc::new(|params| {
            if params.len() < 3 {
                Err(RuleError::NumberOfParametersDontMatch(params.len()))
            } else {
                Ok(params[2] == 4)
            }
        }));
        let context = EntryContext(vec![1, 2]);
        assert!(rule.is_respected(&context).is_err());
    }
}
