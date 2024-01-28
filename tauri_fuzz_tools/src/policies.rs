//! Enumeration of libc functions the fuzzer should analyze
//! Taken from subcategores of https://en.wikipedia.org/wiki/C_standard_library

use std::fmt::Debug;

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

// TODO use closure rather than functions
/// ConditionOnParameters is a closure on the parameters of a function
type ConditionOnParameters = fn(&Vec<usize>) -> Result<bool, RuleError>;
/// ConditionOnReturnValue is a closure on the return value of a function
type ConditionOnReturnValue = fn(usize) -> Result<bool, RuleError>;

/// Rule that the function has to adhere to respect the policy
// TODO to improve perf we can separate entry rule and leave rule into two different types, the UX
// is not as good then
#[derive(Debug, Clone)]
pub enum Rule {
    /// Function is analysed on entry given its parameters
    OnEntry(ConditionOnParameters),

    /// Function is analysed after it's execution given its return value
    OnLeave(ConditionOnReturnValue),
}

/// Errors that may happen when evaluating a rule given a context
#[derive(Debug)]
pub enum RuleError {
    /// The number of parameters given in the context does not match the number of
    /// parameters expected in the context
    NumberOfParametersDontMatch(String),
    /// Generic error that happened during the rule evaluation
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

pub type FuzzPolicy = Vec<FunctionPolicy>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_on_parameters() {
        // Block on function entry
        let rule = Rule::OnEntry(|_| Ok(false));
        let context = EntryContext(vec![]);
        assert!(!rule.is_respected(&context).unwrap());

        // Check parameters
        let rule1 = Rule::OnEntry(|params| Ok(params[0] == 1));
        let rule2 = Rule::OnEntry(|params| Ok(params[1] % 2 == 0));
        let rule3 = Rule::OnEntry(|params| Ok(params[2] == 4));
        let context = EntryContext(vec![1, 2, 3]);
        assert!(rule1.is_respected(&context).unwrap());
        assert!(rule2.is_respected(&context).unwrap());
        assert!(!rule3.is_respected(&context).unwrap());
    }

    #[test]
    fn rule_on_return_value() {
        let rule = Rule::OnLeave(|v| Ok(v == 1));
        let rule2 = Rule::OnLeave(|v| Ok(v == 4));
        let context = LeaveContext(1);
        assert!(rule.is_respected(&context).unwrap());
        assert!(!rule2.is_respected(&context).unwrap());
    }

    #[test]
    fn rule_wrong_context() {
        // Entry context with leave rule
        let context = EntryContext(vec![]);
        let rule = Rule::OnLeave(|_| Ok(false));
        assert!(rule.is_respected(&context).unwrap());

        // Leave context with entry rule
        let context = LeaveContext(0);
        let rule = Rule::OnEntry(|_| Ok(false));
        assert!(rule.is_respected(&context).unwrap());

        // Entry context with not enough parameters
        let rule = Rule::OnEntry(|params| {
            if params.len() < 3 {
                Err(RuleError::NumberOfParametersDontMatch(
                    "Expecting 3 parameters".into(),
                ))
            } else {
                Ok(params[2] == 4)
            }
        });
        let context = EntryContext(vec![1, 2]);
        assert!(rule.is_respected(&context).is_err());
    }
}
