//! C code generator for the sigi compiler.
//!
//! Lowers the IR (`Program`) into a single C source file that uses a
//! fixed stack array and runtime prelude.  The generated code is
//! portable C99 and links against libm.

use crate::ast::{walk_ops, Op, Program, TargetName, VarName};
use std::collections::{BTreeSet, HashSet};

pub struct Codegen {
    program: Program,
}

impl Codegen {
    pub fn new(program: Program) -> Self {
        Self { program }
    }

    pub fn generate(&self) -> String {
        let mut lines = Vec::new();

        // ─── Runtime Prelude ─────────────────────────────────────
        // The prelude contains the stack implementation, array helpers,
        // and math includes — everything needed to make the generated
        // C self-contained.
        lines.push(include_str!("../runtime/prelude.c").to_string());

        // ─── Foreign Headers ─────────────────────────────────────
        let headers: BTreeSet<&str> = self
            .program
            .foreign
            .iter()
            .filter(|f| !f.is_user_source())
            .map(|f| f.header.as_str())
            .collect();
        for header in headers {
            lines.push(format!("#include <{header}>"));
        }
        for f in self.program.foreign.iter().filter(|f| f.is_user_source()) {
            let params = if f.arity == 0 {
                "void".to_string()
            } else {
                vec!["double"; f.arity].join(", ")
            };
            lines.push(format!("double {}({params});", f.name));
        }

        // ─── Named Variable Declarations ─────────────────────────
        let mut named_vars = HashSet::new();
        self.collect_vars(&self.program.main_code, &mut named_vars);
        for fn_ in &self.program.functions {
            self.collect_vars(&fn_.body, &mut named_vars);
        }

        let mut sorted_vars: Vec<_> = named_vars.into_iter().collect();
        sorted_vars.sort();

        for name in sorted_vars {
            lines.push(format!(
                "static double var_{} = 0;",
                self.c_ident(&TargetName::Named(name))
            ));
        }

        // ─── Forward Declarations ────────────────────────────────
        // C requires declarations before use; we emit forward decls for
        // all functions so they can mutually call each other.
        for fn_ in &self.program.functions {
            lines.push(format!(
                "static void func_{}(void);",
                self.c_ident(&fn_.name)
            ));
        }
        lines.push("".to_string());

        // ─── Function Pointer Dispatch ───────────────────────────
        // Function values are indices into `program.functions`; the
        // dispatcher maps an index back to its function. Emitted only when
        // the program calls through a pointer.
        let mut uses_call_ptr = false;
        walk_ops(&self.program.main_code, &mut |op| {
            uses_call_ptr |= matches!(op, Op::CallPtr)
        });
        for fn_ in &self.program.functions {
            walk_ops(&fn_.body, &mut |op| uses_call_ptr |= matches!(op, Op::CallPtr));
        }
        if uses_call_ptr {
            lines.push("static void sigi_call_ptr(double _idx) {".to_string());
            lines.push("    switch ((long long)_idx) {".to_string());
            for (i, fn_) in self.program.functions.iter().enumerate() {
                lines.push(format!(
                    "        case {i}: func_{}(); return;",
                    self.c_ident(&fn_.name)
                ));
            }
            lines.push("    }".to_string());
            lines.push("    fprintf(stderr, \"Invalid function pointer\\n\");".to_string());
            lines.push("    exit(1);".to_string());
            lines.push("}".to_string());
            lines.push("".to_string());
        }

        // ─── Function Definitions ────────────────────────────────
        for fn_ in &self.program.functions {
            lines.push(format!(
                "static void func_{}(void) {{",
                self.c_ident(&fn_.name)
            ));
            for op in &fn_.body {
                lines.extend(self.codegen_op(op, 1));
            }
            lines.push("}".to_string());
            lines.push("".to_string());
        }

        // ─── Main ────────────────────────────────────────────────
        lines.push("int main(void) {".to_string());
        lines.push("    srand((unsigned)time(NULL));".to_string());
        for op in &self.program.main_code {
            lines.extend(self.codegen_op(op, 1));
        }
        lines.push("    return 0;".to_string());
        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Collect all named variables used in a block of ops.
    /// Used to emit static declarations before any code.
    fn collect_vars(&self, ops: &[Op], named_vars: &mut HashSet<String>) {
        for op in ops {
            match op {
                Op::Var(VarName::Named(name)) | Op::StoreNamed(name) => {
                    named_vars.insert(name.clone());
                }
                Op::While(body) | Op::DoWhile(body) | Op::Block(body) | Op::For(body) => {
                    self.collect_vars(body, named_vars)
                }
                Op::Switch(arms) => {
                    for arm in arms {
                        self.collect_vars(&arm.body, named_vars);
                    }
                }
                Op::IfElse {
                    then_body,
                    else_body,
                } => {
                    self.collect_vars(then_body, named_vars);
                    self.collect_vars(else_body, named_vars);
                }
                _ => {}
            }
        }
    }

    /// Convert a TargetName into a valid C identifier string.
    fn c_ident(&self, name: &TargetName) -> String {
        match name {
            TargetName::Index(i) => i.to_string(),
            TargetName::Named(s) => s.replace(|c: char| !c.is_alphanumeric(), "_"),
        }
    }

    /// Emit the C statements for a single IR op at the given indent level.
    fn codegen_op(&self, op: &Op, indent: usize) -> Vec<String> {
        let prefix = "    ".repeat(indent);
        let mut lines = Vec::new();

        match op {
            Op::Push(val) => {
                lines.push(format!("{}push({});", prefix, val));
            }
            Op::Var(name) => match name {
                VarName::Index(i) => lines.push(format!("{}push(vars[{}]);", prefix, i)),
                VarName::Named(n) => lines.push(format!(
                    "{}push(var_{});",
                    prefix,
                    self.c_ident(&TargetName::Named(n.clone()))
                )),
            },
            Op::StoreNamed(name) => {
                lines.push(format!(
                    "{}var_{} = pop();",
                    prefix,
                    self.c_ident(&TargetName::Named(name.clone()))
                ));
            }
            Op::Char(byte) => {
                lines.push(format!("{}push('\\x{:02x}');", prefix, byte));
            }
            Op::String(bytes) => {
                for byte in bytes {
                    lines.push(format!("{}putchar({});", prefix, byte));
                }
            }
            Op::Call(target) => {
                lines.push(format!("{}func_{}();", prefix, self.c_ident(target)));
            }
            Op::While(body) => {
                // sigi's while is a stack-top check: pop is not consumed.
                lines.push(format!("{}while (1) {{", prefix));
                lines.push(format!("{}    if (sp <= 0) break;", prefix));
                lines.push(format!("{}    double _cond = stack[sp - 1];", prefix));
                lines.push(format!("{}    if (_cond == 0.0) break;", prefix));
                for op in body {
                    lines.extend(self.codegen_op(op, indent + 1));
                }
                lines.push(format!("{}}}", prefix));
            }
            Op::Block(body) => {
                lines.push(format!("{}{{", prefix));
                for op in body {
                    lines.extend(self.codegen_op(op, indent + 1));
                }
                lines.push(format!("{}}}", prefix));
            }
            Op::For(body) => {
                lines.push(format!("{prefix}{{"));
                lines.push(format!("{prefix}    long long _count = (long long)pop();"));
                lines.push(format!("{prefix}    for (long long _i = 0; _i < _count; _i++) {{"));
                for op in body {
                    lines.extend(self.codegen_op(op, indent + 2));
                }
                lines.push(format!("{prefix}    }}"));
                lines.push(format!("{prefix}}}"));
            }
            Op::DoWhile(body) => {
                // Body runs before the first check; the condition is the
                // stack top, matching `[ ]`, and `continue` re-checks it.
                lines.push(format!("{prefix}do {{"));
                for op in body {
                    lines.extend(self.codegen_op(op, indent + 1));
                }
                lines.push(format!("{prefix}}} while (sp > 0 && stack[sp - 1] != 0.0);"));
            }
            Op::Break => lines.push(format!("{prefix}break;")),
            Op::Continue => lines.push(format!("{prefix}continue;")),
            Op::Switch(arms) => {
                // Arms keep source order and fall through, as in C.
                lines.push(format!("{prefix}{{"));
                lines.push(format!("{prefix}    long long _sw = (long long)pop();"));
                lines.push(format!("{prefix}    switch (_sw) {{"));
                for arm in arms {
                    match arm.value {
                        Some(v) => lines.push(format!("{prefix}    case {v}LL: ;")),
                        None => lines.push(format!("{prefix}    default: ;")),
                    }
                    for op in &arm.body {
                        lines.extend(self.codegen_op(op, indent + 2));
                    }
                }
                lines.push(format!("{prefix}    }}"));
                lines.push(format!("{prefix}}}"));
            }
            Op::Label(name) => lines.push(format!(
                "{prefix}sigi_l_{}: ;",
                self.c_ident(&TargetName::Named(name.clone()))
            )),
            Op::Goto(name) => lines.push(format!(
                "{prefix}goto sigi_l_{};",
                self.c_ident(&TargetName::Named(name.clone()))
            )),
            Op::FuncRef(target) => {
                let index = self
                    .program
                    .functions
                    .iter()
                    .position(|f| f.name == *target)
                    .expect("parser rejects references to undefined functions");
                lines.push(format!("{prefix}push({index});"));
            }
            Op::CallPtr => lines.push(format!("{prefix}sigi_call_ptr(pop());")),
            Op::FfiCall(name) => {
                let arity = self
                    .program
                    .foreign
                    .iter()
                    .find(|f| &f.name == name)
                    .map(|f| f.arity)
                    .expect("parser rejects calls to undeclared foreign functions");
                let args: Vec<String> = (0..arity).map(|i| format!("_f{i}")).collect();
                lines.push(format!("{prefix}{{"));
                for i in (0..arity).rev() {
                    lines.push(format!("{prefix}    double _f{i} = pop();"));
                }
                lines.push(format!("{prefix}    push({name}({}));", args.join(", ")));
                lines.push(format!("{prefix}}}"));
            }
            Op::IfElse {
                then_body,
                else_body,
            } => {
                lines.push(format!("{}{{", prefix));
                lines.push(format!("{}    double _cond = pop();", prefix));
                lines.push(format!("{}    if (_cond != 0.0) {{", prefix));
                for op in then_body {
                    lines.extend(self.codegen_op(op, indent + 2));
                }
                lines.push(format!("{}    }} else {{", prefix));
                for op in else_body {
                    lines.extend(self.codegen_op(op, indent + 2));
                }
                lines.push(format!("{}    }}", prefix));
                lines.push(format!("{}}}", prefix));
            }
            Op::Simple(kind) => {
                match kind.as_str() {
                    // ─── Arithmetic ──────────────────────────────
                    "ADD" | "SUB" | "MUL" | "DIV" => {
                        let op = match kind.as_str() { "ADD" => "+", "SUB" => "-", "MUL" => "*", _ => "/" };
                        lines.push(format!("{}{{ double b = pop(); double a = pop(); push(a {} b); }}", prefix, op));
                    },
                    "MOD" => lines.push(format!("{}{{ double b = pop(); double a = pop(); push(fmod(a, b)); }}", prefix)),
                    "INTDIV" => lines.push(format!("{prefix}{{ long long b = (long long)pop(); long long a = (long long)pop(); if (b == 0) {{ fprintf(stderr, \"Division by zero\\n\"); exit(1); }} push((double)(a / b)); }}")),
                    // ─── Bitwise and Shift ───────────────────────
                    "BITAND" | "BITOR" | "BITXOR" => {
                        let op = match kind.as_str() { "BITAND" => "&", "BITOR" => "|", _ => "^" };
                        lines.push(format!("{prefix}{{ long long b = (long long)pop(); long long a = (long long)pop(); push((double)(a {op} b)); }}"));
                    },
                    "BITNOT" => lines.push(format!("{prefix}push((double)~(long long)pop());")),
                    "SHL" => lines.push(format!("{prefix}{{ int n = shift_count(pop()); long long a = (long long)pop(); push((double)(long long)((unsigned long long)a << n)); }}")),
                    "SHR" => lines.push(format!("{prefix}{{ int n = shift_count(pop()); long long a = (long long)pop(); push((double)(a >> n)); }}")),
                    // ─── Logical ─────────────────────────────────
                    "LOGAND" => lines.push(format!("{prefix}{{ double b = pop(); double a = pop(); push((a != 0.0 && b != 0.0) ? 1.0 : 0.0); }}")),
                    "LOGOR" => lines.push(format!("{prefix}{{ double b = pop(); double a = pop(); push((a != 0.0 || b != 0.0) ? 1.0 : 0.0); }}")),
                    // ─── Comparison ──────────────────────────────
                    "EQ" | "LT" | "GT" => {
                        let op = match kind.as_str() { "EQ" => "==", "LT" => "<", _ => ">" };
                        lines.push(format!("{}{{ double b = pop(); double a = pop(); push((a {} b) ? 1.0 : 0.0); }}", prefix, op));
                    },
                    "NOT" => lines.push(format!("{}push((pop() == 0.0) ? 1.0 : 0.0);", prefix)),
                    // ─── Stack Operations ────────────────────────
                    "DUP" => lines.push(format!("{}{{ double x = stack[sp - 1]; push(x); }}", prefix)),
                    "SWAP" => lines.push(format!("{}{{ double t = stack[sp - 1]; stack[sp - 1] = stack[sp - 2]; stack[sp - 2] = t; }}", prefix)),
                    "DROP" => lines.push(format!("{}(void)pop();", prefix)),
                    // ─── Math Functions ──────────────────────────
                    "SIN" | "COS" | "TAN" | "SQRT" | "FLOOR" | "LOG" | "EXP" | "ABS" => {
                        let f = match kind.as_str() {
                            "SIN" => "sin", "COS" => "cos", "TAN" => "tan", "SQRT" => "sqrt",
                            "FLOOR" => "floor", "LOG" => "log", "EXP" => "exp", _ => "fabs"
                        };
                        lines.push(format!("{}push({}(pop()));", prefix, f));
                    },
                    "POW" => lines.push(format!("{}{{ double e = pop(); double b = pop(); push(pow(b, e)); }}", prefix)),
                    "ATAN2" => lines.push(format!("{}{{ double y = pop(); double x = pop(); push(atan2(y, x)); }}", prefix)),
                    // ─── System / I/O ────────────────────────────
                    "RAND" => lines.push(format!("{}push((double)rand() / RAND_MAX);", prefix)),
                    "EXIT" => lines.push(format!("{}exit((int)pop());", prefix)),
                    "TIME" => lines.push(format!("{}push((double)time(NULL));", prefix)),
                    "ALEN" => lines.push(format!("{}{{ int id = (int)pop(); push(array_sizes[id]); }}", prefix)),
                    "ALOAD" => lines.push(format!("{}{{ int idx = (int)pop(); int id = (int)pop(); push(*arr_slot(id, idx)); }}", prefix)),
                    "ASTORE" => lines.push(format!("{}{{ int idx = (int)pop(); int id = (int)pop(); double val = pop(); *arr_slot(id, idx) = val; }}", prefix)),
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
