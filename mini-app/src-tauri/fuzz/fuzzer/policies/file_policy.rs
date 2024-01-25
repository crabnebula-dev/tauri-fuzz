use crate::policies::FunctionDenyRule;

const NO_FILE_ACCESS: FunctionDenyRule = FunctionDenyRule {
    name: "fopen".into(),
    block_on_entry: true,
    block_parameters: vec![],
    block_return_value: None,
    description: "File accessed illegally",
};

const NO_FILE_WRITE_ACCESS: LibcFunctionPolicy = LibcFunctionPolicy {
    name: "fopen".into(),
    block_on_entry: true,
    block_parameters: vec![],
    block_return_value: None,
    description: 
};

const 

