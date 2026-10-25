#include "codegen.h"
#include <sstream>
#include <fstream>
#include <stdexcept>

std::string emitSigi(const Program &p) {
    std::stringstream ss;
    ss << "#include <stdio.h>\n#include <stdlib.h>\n#include <math.h>\n#include <string.h>\n#include <time.h>\n\n";
    ss << "double stack[10000]; int sp = 0;\n";
    ss << "static void sigi_fail(const char *m) { fprintf(stderr, \"Sigi runtime error: %s\\n\", m); exit(1); }\n";
    ss << "static void sigi_push(double v) { if (sp >= 10000) sigi_fail(\"stack overflow\"); stack[sp++] = v; }\n";
    ss << "static double sigi_pop(void) { if (sp <= 0) sigi_fail(\"stack underflow\"); return stack[--sp]; }\n";
    ss << "double vars[100] = {0};\n\n";
    ss << "int main() {\n";
    ss << "    srand((unsigned)time(NULL));\n";
    for (auto &op : p.mainOps) {
        if (op.kind == SOp::PushNum) {
            ss << "    sigi_push(" << op.num << ");\n";
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
            ss << "    { int idx = (int)sigi_pop(); double val = sigi_pop(); if (idx < 0 || idx >= 100) sigi_fail(\"variable index out of range\"); vars[idx] = val; }\n";
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
            ss << "    printf(\"%c\", (char)sigi_pop()); fflush(stdout);\n";
        } else if (op.kind == SOp::Input) {
            ss << "    { double x; if (scanf(\"%lf\", &x) != 1) sigi_fail(\"numeric input failed\"); sigi_push(x); }\n";
        } else if (op.kind == SOp::WhileStart) {
            ss << "    while (sp > 0 && stack[sp-1] != 0) {\\n";
        } else if (op.kind == SOp::WhileEnd) {
            ss << "    }\\n";
        } else if (op.kind == SOp::IfStart) {
            ss << "    if (sigi_pop() != 0) {\\n";
        } else if (op.kind == SOp::IfElse) {
            ss << "    } else {\\n";
        } else if (op.kind == SOp::IfEnd) {
            ss << "    }\\n";
        } else if (op.kind == SOp::Break) {
            ss << "    break;\\n";
        } else {
            throw std::runtime_error("Sigi C++ code generator does not yet support this operation");
        }
    }
    ss << "    return 0;\n}\n";
    return ss.str();
}
