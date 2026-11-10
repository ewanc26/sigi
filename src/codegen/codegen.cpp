#include "codegen.h"
#include <cstdio>
#include <sstream>
#include <fstream>
#include <cctype>
#include <map>
#include <stdexcept>

std::string emitSigi(const Program &p) {
    std::stringstream ss;
    ss << "#include <stdio.h>\n#include <stdlib.h>\n#include <math.h>\n#include <limits.h>\n#include <string.h>\n#include <time.h>\n\n";
    ss << "double stack[10000]; int sp = 0;\n";
    ss << "static void sigi_fail(const char *m) { fprintf(stderr, \"Sigi runtime error: %s\\n\", m); exit(1); }\n";
    ss << "static int sigi_checked_int(double v, const char *m) { if (!isfinite(v) || v < (double)INT_MIN || v > (double)INT_MAX) sigi_fail(m); return (int)v; }\n";
    ss << "static void sigi_push(double v) { if (sp >= 10000) sigi_fail(\"stack overflow\"); stack[sp++] = v; }\n";
    ss << "static double sigi_pop(void) { if (sp <= 0) sigi_fail(\"stack underflow\"); return stack[--sp]; }\n";
    ss << "double vars[100] = {0};\n\n";
    // Foreign C functions: declared with `x "header" .name !N`, called with `c .name`.
    std::map<std::string, int> ffiArity;
    {
        std::map<std::string, std::string> ffiHeader;
        for (auto &op : p.mainOps) {
            if (op.kind != SOp::FfiDecl) continue;
            if (ffiArity.count(op.name) && (ffiArity[op.name] != op.funcNum || ffiHeader[op.name] != op.header)) {
                throw std::runtime_error("conflicting FFI declarations for ." + op.name);
            }
            ffiArity[op.name] = op.funcNum;
            ffiHeader[op.name] = op.header;
        }
        for (auto &h : ffiHeader) {
            for (char ch : h.second) {
                if (!(std::isalnum((unsigned char)ch) || ch=='.' || ch=='_' || ch=='/' || ch=='-')) {
                    throw std::runtime_error("invalid character in FFI header name: " + h.second);
                }
            }
            ss << "#include <" << h.second << ">\n";
        }
        ss << "\n";
        for (auto &op : p.mainOps) {
            if (op.kind != SOp::FfiCall) continue;
            if (!ffiArity.count(op.name)) {
                throw std::runtime_error("FFI call to undeclared function ." + op.name);
            }
        }
    }

    ss << "int main() {\n";
    ss << "    srand((unsigned)time(NULL));\n";
    for (auto &op : p.mainOps) {
        if (op.kind == SOp::PushNum) {
            // Full-precision literal: a std default-formatted stream truncates
            // to 6 significant digits, which silently corrupts programs that
            // carry high-precision constants. %.17g round-trips a double.
            char buf[64];
            snprintf(buf, sizeof buf, "%.17g", op.num);
            ss << "    sigi_push(" << buf << ");\n";
        } else if (op.kind == SOp::StringLit) {
            ss << "    {";
            for (int v : op.strvals) {
                if (v == 10) ss << "putchar(10);";
                else if (v == 9) ss << "putchar(9);";
                else if (v == 34) ss << "putchar(34);";
                else if (v == 92) ss << "putchar(92);";
                else ss << "putchar(" << v << ");";
            }
            ss << " fflush(stdout); }\n";
        } else if (op.kind == SOp::CharLit) {
            ss << "    sigi_push(" << op.ival << ");\n";
        } else if (op.kind == SOp::LoadVar) {
            ss << "    sigi_push(vars[" << op.varIndex << "]);\n";
        } else if (op.kind == SOp::Store) {
            ss << "    { double raw_idx = sigi_pop(); if (!isfinite(raw_idx) || raw_idx < 0 || raw_idx >= 100 || floor(raw_idx) != raw_idx) sigi_fail(\"variable index out of range\"); int idx = (int)raw_idx; double val = sigi_pop(); vars[idx] = val; }\n";
        } else if (op.kind == SOp::Dup) {
            ss << "    if (sp <= 0) sigi_fail(\"stack underflow\"); sigi_push(stack[sp-1]);\n";
        } else if (op.kind == SOp::Swap) {
            ss << "    if (sp < 2) sigi_fail(\"stack underflow\"); { double a=stack[sp-1], b=stack[sp-2]; stack[sp-2]=a; stack[sp-1]=b; }\n";
        } else if (op.kind == SOp::Drop) {
            ss << "    (void)sigi_pop();\n";
        } else if (op.kind == SOp::Add) {
            ss << "    { double b=sigi_pop(),a=sigi_pop(); sigi_push(a+b); }\n";
        } else if (op.kind == SOp::Sub) {
            ss << "    { double b=sigi_pop(),a=sigi_pop(); sigi_push(a-b); }\n";
        } else if (op.kind == SOp::Mul) {
            ss << "    { double b=sigi_pop(),a=sigi_pop(); sigi_push(a*b); }\n";
        } else if (op.kind == SOp::Div) {
            ss << "    { double b=sigi_pop(),a=sigi_pop(); if (b == 0) sigi_fail(\"division by zero\"); sigi_push(a/b); }\n";
        } else if (op.kind == SOp::Mod) {
            ss << "    { double b=sigi_pop(),a=sigi_pop(); if (b == 0) sigi_fail(\"remainder by zero\"); sigi_push(fmod(a,b)); }\n";
        } else if (op.kind == SOp::Not) {
            ss << "    { double a=sigi_pop(); sigi_push(a==0?1.0:0.0); }\n";
        } else if (op.kind == SOp::Eq) {
            ss << "    { double b=sigi_pop(),a=sigi_pop(); sigi_push((a==b)?1.0:0.0); }\n";
        } else if (op.kind == SOp::Lt) {
            ss << "    { double b=sigi_pop(),a=sigi_pop(); sigi_push((a<b)?1.0:0.0); }\n";
        } else if (op.kind == SOp::Gt) {
            ss << "    { double b=sigi_pop(),a=sigi_pop(); sigi_push((a>b)?1.0:0.0); }\n";
        } else if (op.kind == SOp::Print) {
            ss << "    printf(\"%g\\n\", sigi_pop());\n";
        } else if (op.kind == SOp::PrintChar) {
            ss << "    { int value = sigi_checked_int(sigi_pop(), \"character value out of range\"); putchar((unsigned char)value); fflush(stdout); }\n";
        } else if (op.kind == SOp::Input) {
            ss << "    { double x; if (scanf(\"%lf\", &x) != 1) sigi_fail(\"numeric input failed\"); sigi_push(x); }\n";
        } else if (op.kind == SOp::WhileStart) {
            ss << "    while (sp > 0 && stack[sp-1] != 0) {\n";
        } else if (op.kind == SOp::WhileEnd) {
            ss << "    }\n";
        } else if (op.kind == SOp::DoWhileStart) {
            ss << "    do {\n";
        } else if (op.kind == SOp::DoWhileEnd) {
            ss << "    } while (sp > 0 && stack[sp-1] != 0);\n";
        } else if (op.kind == SOp::IfStart) {
            ss << "    if (sigi_pop() != 0) {\n";
        } else if (op.kind == SOp::IfElse) {
            ss << "    } else {\n";
        } else if (op.kind == SOp::IfEnd) {
            ss << "    }\n";
        } else if (op.kind == SOp::Break) {
            ss << "    break;\n";
        } else if (op.kind == SOp::Continue) {
            ss << "    continue;\n";
        } else if (op.kind == SOp::Sin) {
            ss << "    sigi_push(sin(sigi_pop()));\n";
        } else if (op.kind == SOp::Cos) {
            ss << "    sigi_push(cos(sigi_pop()));\n";
        } else if (op.kind == SOp::Tan) {
            ss << "    sigi_push(tan(sigi_pop()));\n";
        } else if (op.kind == SOp::Sqrt) {
            ss << "    sigi_push(sqrt(sigi_pop()));\n";
        } else if (op.kind == SOp::Floor) {
            ss << "    sigi_push(floor(sigi_pop()));\n";
        } else if (op.kind == SOp::Log) {
            ss << "    sigi_push(log(sigi_pop()));\n";
        } else if (op.kind == SOp::Exp) {
            ss << "    sigi_push(exp(sigi_pop()));\n";
        } else if (op.kind == SOp::Abs) {
            ss << "    sigi_push(fabs(sigi_pop()));\n";
        } else if (op.kind == SOp::Pow) {
            // Binary: base then exponent on the stack.
            ss << "    { double b=sigi_pop(),a=sigi_pop(); sigi_push(pow(a,b)); }\n";
        } else if (op.kind == SOp::Atan2) {
            // Binary: y then x on the stack.
            ss << "    { double y=sigi_pop(),x=sigi_pop(); sigi_push(atan2(x,y)); }\n";
        } else if (op.kind == SOp::FfiDecl) {
            // Handled above by emitting the header include; nothing to do here.
        } else if (op.kind == SOp::FfiCall) {
            if (!ffiArity.count(op.name)) {
                throw std::runtime_error("FFI call to undeclared function ." + op.name);
            }
            const int argc = ffiArity[op.name];
            ss << "    {";
            for (int a = argc; a >= 1; --a) {
                ss << " double sigi_arg" << a << "=sigi_pop();";
            }
            ss << " sigi_push(" << op.name << "(";
            for (int a = 1; a <= argc; ++a) {
                if (a > 1) ss << ",";
                ss << "sigi_arg" << a;
            }
            ss << ")); }\n";
        } else {
            throw std::runtime_error("Sigi C++ code generator does not yet support this operation");
        }
    }
    ss << "    return 0;\n}\n";
    return ss.str();
}
