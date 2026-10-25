#include "codegen.h"
#include <sstream>
#include <fstream>
#include <stdexcept>

std::string emitSigi(const Program &p) {
    std::stringstream ss;
    ss << "#include <stdio.h>\n#include <stdlib.h>\n#include <math.h>\n#include <string.h>\n#include <time.h>\n\n";
    ss << "double stack[10000]; int sp = 0;\n";
    ss << "double vars[100] = {0};\n\n";
    ss << "int main() {\n";
    ss << "    srand((unsigned)time(NULL));\n";
    for (auto &op : p.mainOps) {
        if (op.kind == SOp::PushNum) {
            ss << "    stack[sp++] = " << op.num << ";\n";
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
            ss << "    stack[sp++] = " << op.ival << ";\n";
        } else if (op.kind == SOp::LoadVar) {
            ss << "    stack[sp++] = vars[" << op.varIndex << "];\n";
        } else if (op.kind == SOp::Store) {
            ss << "    { int idx = (int)stack[--sp]; double val = stack[--sp]; if (idx < 0 || idx >= 100) { fprintf(stderr, \"Sigi runtime error: variable index out of range\\n\"); return 1; } vars[idx] = val; }\n";
        } else if (op.kind == SOp::Dup) {
            ss << "    stack[sp] = stack[sp-1]; sp++;\n";
        } else if (op.kind == SOp::Swap) {
            ss << "    { double a=stack[sp-1], b=stack[sp-2]; stack[sp-2]=a; stack[sp-1]=b; }\n";
        } else if (op.kind == SOp::Drop) {
            ss << "    sp--;\n";
        } else if (op.kind == SOp::Add) {
            ss << "    { double b=stack[--sp],a=stack[--sp]; stack[sp++]=a+b; }\n";
        } else if (op.kind == SOp::Sub) {
            ss << "    { double b=stack[--sp],a=stack[--sp]; stack[sp++]=a-b; }\n";
        } else if (op.kind == SOp::Mul) {
            ss << "    { double b=stack[--sp],a=stack[--sp]; stack[sp++]=a*b; }\n";
        } else if (op.kind == SOp::Div) {
            ss << "    { double b=stack[--sp],a=stack[--sp]; stack[sp++]=a/b; }\n";
        } else if (op.kind == SOp::Mod) {
            ss << "    { double b=stack[--sp],a=stack[--sp]; stack[sp++]=fmod(a,b); }\n";
        } else if (op.kind == SOp::Not) {
            ss << "    stack[sp-1] = (stack[sp-1]==0)?1.0:0.0;\n";
        } else if (op.kind == SOp::Eq) {
            ss << "    { double b=stack[--sp],a=stack[--sp]; stack[sp++]=(a==b)?1.0:0.0; }\n";
        } else if (op.kind == SOp::Lt) {
            ss << "    { double b=stack[--sp],a=stack[--sp]; stack[sp++]=(a<b)?1.0:0.0; }\n";
        } else if (op.kind == SOp::Gt) {
            ss << "    { double b=stack[--sp],a=stack[--sp]; stack[sp++]=(a>b)?1.0:0.0; }\n";
        } else if (op.kind == SOp::Print) {
            ss << "    printf(\"%g\\n\", stack[--sp]);\n";
        } else if (op.kind == SOp::PrintChar) {
            ss << "    printf(\"%c\", (char)stack[--sp]); fflush(stdout);\n";
        } else if (op.kind == SOp::Input) {
            ss << "    { double x; scanf(\"%lf\", &x); stack[sp++] = x; }\n";
        } else {
            throw std::runtime_error("Sigi C++ code generator does not yet support this operation");
        }
    }
    ss << "    return 0;\n}\n";
    return ss.str();
}
