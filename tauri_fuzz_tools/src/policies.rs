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
                "Error in policy evaluation: {}.\nRule: {:?}.\nContext: {:?}",
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

/// Rule that the function has to adhere to respect the policy
#[derive(Debug, Clone)]
pub enum Rule {
    /// Function is rejected on entry
    OnEntry(),

    /// Reject if parameter at the specified index does not fulfill the condition
    /// Tuple first value corresponds to the parameter index
    OnParameter(usize, ConditionOnValue),

    /// Reject if return value does not fulfill the condition
    OnReturnValue(ConditionOnValue),

    /// Reject if the two rules are also rejected
    And(Box<Self>, Box<Self>),

    /// Reject if at least one of the two is rejected
    Or(Box<Self>, Box<Self>),
}

/// Context when evaluating if a function is respecting the specified policy.
/// TODO Check if parameters order always follow the function signature.
/// Parameters are relevant when evaluating at the beginning of a function but no guarantees
/// are given when using them at the end of the function.
/// The return value is None when evaluating at the beginning of the function and Some when
/// evaluating at the end of the function.
#[derive(Debug)]
pub struct Context {
    pub parameters: Vec<usize>,
    pub return_value: Option<usize>,
}

use Rule::*;
impl Rule {
    /// Evaluate if rule is true given a context
    /// If it returns true it means that the rule has been verified and does not respect the policy
    fn is_respected(&self, context: &Context) -> Result<bool, String> {
        match self {
            // We block the function on entry
            OnEntry() => Ok(false),

            // Check the rule condition on parameters at index [`index`]
            OnParameter(index, condition) => {
                // Don't check if we're at the end of the function
                if let Some(_) = context.return_value {
                    return Ok(true);
                }
                if *index >= context.parameters.len() {
                    Err(String::from(
                        "Condition specified on a parameter that does not exist",
                    ))
                } else {
                    Ok(condition(context.parameters[*index]))
                }
            }

            // Check the rule condition on return value
            OnReturnValue(condition) => {
                if let Some(return_value) = context.return_value {
                    Ok(condition(return_value))
                } else {
                    Err(String::from(
                        "Confition specified on non-existing return value",
                    ))
                }
            }

            And(rule1, rule2) => {
                let eval1 = rule1.is_respected(context);
                let eval2 = rule2.is_respected(context);
                match (eval1, eval2) {
                    (Ok(e1), Ok(e2)) => Ok(e1 && e2),
                    (Err(e), _) => Err(e),
                    (_, Err(e)) => Err(e),
                }
            }

            Or(rule1, rule2) => {
                let eval1 = rule1.is_respected(context);
                let eval2 = rule2.is_respected(context);
                match (eval1, eval2) {
                    (Ok(e1), Ok(e2)) => Ok(e1 || e2),
                    (Err(e), _) => Err(e),
                    (_, Err(e)) => Err(e),
                }
            }
        }
    }
}

pub type FuzzPolicy = Vec<FunctionPolicy>;

// ConditionOnValue is a closure that checks if a value contained in a register
// match the specified condition
// TODO use closure rather than functions
type ConditionOnValue = fn(usize) -> bool;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_on_entry() {
        let rule = Rule::OnEntry();
        let context = Context {
            parameters: vec![],
            return_value: None,
        };
        assert!(!rule.is_respected(&context).unwrap());
    }

    #[test]
    fn rule_on_parameters() {
        let rule1 = Rule::OnParameter(0, |v| v == 1);
        let rule2 = Rule::OnParameter(1, |v| v % 2 == 0);
        let rule3 = Rule::OnParameter(2, |v| v == 5);
        let context = Context {
            parameters: vec![1, 2, 3],
            return_value: None,
        };
        assert!(rule1.is_respected(&context).unwrap());
        assert!(rule2.is_respected(&context).unwrap());
        assert!(!rule3.is_respected(&context).unwrap());
    }

    #[test]
    fn rule_on_return_value() {
        let rule = Rule::OnReturnValue(|v| v == 1);
        let rule2 = Rule::OnReturnValue(|v| v == 4);
        let context = Context {
            parameters: vec![],
            return_value: Some(1),
        };
        assert!(rule.is_respected(&context).unwrap());
        assert!(!rule2.is_respected(&context).unwrap());
    }

    #[test]
    fn rule_and() {
        let context = Context {
            parameters: vec![10, 3],
            return_value: Some(101),
        };
        let rule1 = Rule::OnParameter(0, |v| v == 10);
        let rule2 = Rule::OnReturnValue(|v| v == 4);
        let rule = Rule::And(Box::new(rule1), Box::new(rule2));
        assert!(!rule.is_respected(&context).unwrap());
        let rule1 = Rule::OnParameter(0, |v| v == 10);
        let rule2 = Rule::OnReturnValue(|v| v == 101);
        let rule = Rule::And(Box::new(rule1), Box::new(rule2));
        assert!(rule.is_respected(&context).unwrap());
    }

    #[test]
    fn rule_or() {
        let context = Context {
            parameters: vec![10, 3],
            return_value: Some(101),
        };
        let rule1 = Rule::OnParameter(0, |v| v == 2);
        let rule2 = Rule::OnReturnValue(|v| v == 4);
        let rule = Rule::Or(Box::new(rule1), Box::new(rule2));
        assert!(!rule.is_respected(&context).unwrap());
        let rule1 = Rule::OnParameter(0, |v| v == 2);
        let rule2 = Rule::OnReturnValue(|v| v == 101);
        let rule = Rule::Or(Box::new(rule1), Box::new(rule2));
        assert!(rule.is_respected(&context).unwrap());
    }
}
