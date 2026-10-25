#pragma once
#include <string>
#include <vector>
#include <variant>
#include <memory>

struct SOp {
    enum Kind {
        PushNum, Dup, Swap, Drop,
        Add, Sub, Mul, Div, Mod,
        Eq, Lt, Gt, Not,
        Print, PrintChar, Input,
        StringLit, CharLit,
        Store, LoadVar, StoreNamed, LoadNamed,
        WhileStart, WhileEnd,
        IfStart, IfElse, IfEnd,
        FuncDef, CallFunc, CallNamed,
        Sin,Cos,Tan,Sqrt,Pow,Floor,Log,Exp,Abs,Atan2,
        Break, Continue
    };
    Kind kind;
    double num = 0;
    long long ival = 0;
    std::vector<int> strvals;
    int varIndex = -1;
    std::string name;
    int funcNum = 0;
};

struct Program {
    std::vector<SOp> ops;
    std::vector<SOp> mainOps;
};
