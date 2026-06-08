use crate::ast::{Op, Program, Function, TargetName, VarName};
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
        
        // Runtime
        lines.push(include_str!("../runtime/prelude.c").to_string());
        
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

        // Forward declare functions
        for fn_ in &self.program.functions {
            lines.push(format!("static void func_{}(void);", self.c_ident(&fn_.name)));
        }
        lines.push("".to_string());

        // Function definitions
        for fn_ in &self.program.functions {
            lines.push(format!("static void func_{}(void) {{", self.c_ident(&fn_.name)));
            for op in &fn_.body {
                lines.extend(self.codegen_op(op, 1));
            }
            lines.push("}".to_string());
            lines.push("".to_string());
        }

        // Main function
        lines.push("int main(void) {".to_string());
        lines.push("    srand((unsigned)time(NULL));".to_string());
        for op in &self.program.main_code {
            lines.extend(self.codegen_op(op, 1));
        }
        lines.push("    return 0;".to_string());
        lines.push("}".to_string());

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

    fn codegen_op(&self, op: &Op, indent: usize) -> Vec<String> {
        let prefix = "    ".repeat(indent);
        let mut lines = Vec::new();

        match op {
            Op::Push(val) => {
                lines.push(format!("{}push({});", prefix, val));
            },
            Op::Var(name) => {
                match name {
                    VarName::Index(i) => lines.push(format!("{}push(vars[{}]);", prefix, i)),
                    VarName::Named(n) => lines.push(format!("{}push(var_{});", prefix, self.c_ident(&TargetName::Named(n.clone())))),
                }
            },
            Op::StoreNamed(name) => {
                lines.push(format!("{}var_{} = pop();", prefix, self.c_ident(&TargetName::Named(name.clone()))));
            },
            Op::String(val) => {
                for ch in val.chars() {
                    lines.push(format!("{}putchar({});", prefix, ch as u8));
                }
            },
            Op::Call(target) => {
                lines.push(format!("{}func_{}();", prefix, self.c_ident(target)));
            },
            Op::While(body) => {
                lines.push(format!("{}while (1) {{", prefix));
                lines.push(format!("{}    if (sp <= 0) break;", prefix));
                lines.push(format!("{}    double _cond = stack[sp - 1];", prefix));
                lines.push(format!("{}    if (_cond == 0.0) break;", prefix));
                for op in body { lines.extend(self.codegen_op(op, indent + 1)); }
                lines.push(format!("{}}}", prefix));
            },
            Op::Block(body) => {
                lines.push(format!("{}{{", prefix));
                for op in body { lines.extend(self.codegen_op(op, indent + 1)); }
                lines.push(format!("{}}}", prefix));
            },
            Op::IfElse { then_body, else_body } => {
                lines.push(format!("{}{{", prefix));
                lines.push(format!("{}    double _cond = pop();", prefix));
                lines.push(format!("{}    if (_cond != 0.0) {{", prefix));
                for op in then_body { lines.extend(self.codegen_op(op, indent + 2)); }
                lines.push(format!("{}    }} else {{", prefix));
                for op in else_body { lines.extend(self.codegen_op(op, indent + 2)); }
                lines.push(format!("{}    }}", prefix));
                lines.push(format!("{}}}", prefix));
            },
            Op::Simple(kind) => {
                match kind.as_str() {
                    "ADD" | "SUB" | "MUL" | "DIV" => {
                        let op = match kind.as_str() { "ADD" => "+", "SUB" => "-", "MUL" => "*", _ => "/" };
                        lines.push(format!("{}{{ double b = pop(); double a = pop(); push(a {} b); }}", prefix, op));
                    },
                    "MOD" => lines.push(format!("{}{{ double b = pop(); double a = pop(); push(fmod(a, b)); }}", prefix)),
                    "EQ" | "LT" | "GT" => {
                        let op = match kind.as_str() { "EQ" => "==", "LT" => "<", _ => ">" };
                        lines.push(format!("{}{{ double b = pop(); double a = pop(); push((a {} b) ? 1.0 : 0.0); }}", prefix, op));
                    },
                    "NOT" => lines.push(format!("{}push((pop() == 0.0) ? 1.0 : 0.0);", prefix)),
                    "DUP" => lines.push(format!("{}{{ double x = stack[sp - 1]; push(x); }}", prefix)),
                    "SWAP" => lines.push(format!("{}{{ double t = stack[sp - 1]; stack[sp - 1] = stack[sp - 2]; stack[sp - 2] = t; }}", prefix)),
                    "DROP" => lines.push(format!("{}(void)pop();", prefix)),
                    "SIN" | "COS" | "TAN" | "SQRT" | "FLOOR" | "LOG" | "EXP" | "ABS" => {
                        let f = match kind.as_str() { 
                            "SIN" => "sin", "COS" => "cos", "TAN" => "tan", "SQRT" => "sqrt",
                            "FLOOR" => "floor", "LOG" => "log", "EXP" => "exp", _ => "fabs" 
                        };
                        lines.push(format!("{}push({}(pop()));", prefix, f));
                    },
                    "POW" => lines.push(format!("{}{{ double e = pop(); double b = pop(); push(pow(b, e)); }}", prefix)),
                    "ATAN2" => lines.push(format!("{}{{ double y = pop(); double x = pop(); push(atan2(y, x)); }}", prefix)),
                    "RAND" => lines.push(format!("{}push((double)rand() / RAND_MAX);", prefix)),
                    "EXIT" => lines.push(format!("{}exit((int)pop());", prefix)),
                    "TIME" => lines.push(format!("{}push((double)time(NULL));", prefix)),
                    "ALEN" => lines.push(format!("{}{{ int id = (int)pop(); push(array_sizes[id]); }}", prefix)),
                    "ALOAD" => lines.push(format!("{}{{ int idx = (int)pop(); int id = (int)pop(); push(arrays[id][idx]); }}", prefix)),
                    "ASTORE" => lines.push(format!("{}{{ int idx = (int)pop(); int id = (int)pop(); double val = pop(); arrays[id][idx] = val; }}", prefix)),
                    "AINIT" => lines.push(format!("{}{{ int id = (int)pop(); int size = (int)pop(); arr_init(id, size); }}", prefix)),
                    "AFREE" => lines.push(format!("{}{{ int id = (int)pop(); arr_free(id); }}", prefix)),
                    "FILE_OPEN" => lines.push(format!("{}{{ int mode = (int)pop(); int len = (int)pop(); char *path = (char *)malloc(len + 1); for (int i = 0; i < len; i++) path[i] = (char)pop(); path[len] = '\\0'; FILE *f = fopen(path, mode == 0 ? \"r\" : \"w\"); free(path); push((double)fileno(f)); }}", prefix)),
                    "FILE_READ" => lines.push(format!("{}{{ int fd = (int)pop(); int size = (int)pop(); char *buf = (char *)malloc(size); FILE *f = fdopen(fd, \"r\"); int n = fread(buf, 1, size, f); for (int i = n - 1; i >= 0; i--) push((double)buf[i]); push((double)n); free(buf); }}", prefix)),
                    "FILE_WRITE" => lines.push(format!("{}{{ int fd = (int)pop(); int size = (int)pop(); char *buf = (char *)malloc(size); for (int i = size - 1; i >= 0; i--) buf[i] = (char)pop(); FILE *f = fdopen(fd, \"w\"); int n = fwrite(buf, 1, size, f); push((double)n); free(buf); }}", prefix)),
                    "FILE_CLOSE" => lines.push(format!("{}{{ int fd = (int)pop(); FILE *f = fdopen(fd, \"r\"); fclose(f); }}", prefix)),
                    "USLEEP" => lines.push(format!("{}usleep((useconds_t)pop());", prefix)),
                    "PRINT" => lines.push(format!("{prefix}printf(\"%g\\n\", pop());")),
                    "PRINTC" => lines.push(format!("{}putchar((int)pop());", prefix)),
                    "INPUT" => lines.push(format!("{}{{ double x; scanf(\"%lf\", &x); push(x); }}", prefix)),
                    "STORE" => lines.push(format!("{}{{ int idx = (int)pop(); double val = pop(); vars[idx] = val; }}", prefix)),
                    "NOP" => {},
                    _ => {},
                }
            }
        }
        lines
    }
}
