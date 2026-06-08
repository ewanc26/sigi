use crate::ast::{Program, Op, PushOp, VarOp, CallOp, StoreNamedOp, StringOp, BlockOp, WhileOp, IfElseOp, SimpleOp, TargetName, VarName};
use std::collections::HashSet;

pub struct Codegen {
    program: Program,
}

impl Codegen {
    pub fn new(program: Program) -> Self {
        Self { program }
    }

    pub fn generate(&self) -> String {
        let mut lines = Vec::new();
        lines.push("#include <stdio.h>".to_string());
        lines.push("#include <stdlib.h>".to_string());
        lines.push("#include <math.h>".to_string());
        lines.push("#include <time.h>".to_string());
        lines.push("#include <unistd.h>".to_string());
        lines.push("".to_string());
        lines.push("#define STACK_SIZE 1000".to_string());
        lines.push("#define MAX_ARRAYS 10".to_string());
        lines.push("".to_string());
        lines.push("static double stack[STACK_SIZE];".to_string());
        lines.push("static int sp = 0;".to_string());
        lines.push("static double vars[100];".to_string());
        
        let mut named_vars = HashSet::new();
        self.collect_vars(&self.program.main_code, &mut named_vars);
        for fn_ in &self.program.functions {
            self.collect_vars(&fn_.body, &mut named_vars);
        }
        
        let mut sorted_vars: Vec<_> = named_vars.into_iter().collect();
        sorted_vars.sort();
        
        for name in sorted_vars {
            lines.push(format!("static double var_{} = 0;", self.c_ident(&TargetName::Named(name))));
        }

        lines.push("static double *arrays[MAX_ARRAYS];".to_string());
        lines.push("static int array_sizes[MAX_ARRAYS];".to_string());
        lines.push("".to_string());
        
        // ... add the rest of the runtime functions (arr_init, arr_free, push, pop, etc.) ...
        
        lines.join("\n")
    }
    
    fn collect_vars(&self, ops: &[Op], named_vars: &mut HashSet<String>) {
        for op in ops {
            match op {
                Op::Var(VarName::Named(name)) | Op::StoreNamed(name) => {
                    named_vars.insert(name.clone());
                },
                Op::While(body) | Op::Block(body) => self.collect_vars(body, named_vars),
                Op::IfElse { then_body, else_body } => {
                    self.collect_vars(then_body, named_vars);
                    self.collect_vars(else_body, named_vars);
                },
                _ => {},
            }
        }
    }
    
    fn c_ident(&self, name: &TargetName) -> String {
        match name {
            TargetName::Index(i) => i.to_string(),
            TargetName::Named(s) => s.replace(|c: char| !c.is_alphanumeric(), "_"),
        }
    }
}
