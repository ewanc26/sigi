#include "parser.h"
#include <stdexcept>
#include <string>
Parser::Parser(const std::vector<Token>&t): tokens(t){}
const Token& Parser::peek(int o) const { if(i+o<tokens.size()) return tokens[i+o]; static Token e; e.kind="EOF"; return e; }
const Token& Parser::advance(){ if(i<tokens.size()) return tokens[i++]; return peek(); }
bool Parser::eof() const { return peek().kind=="EOF"; }

Program Parser::parse(){
    Program prog;
    std::vector<SOp> mainOps;
    while(!eof()){
        const Token &t = peek();
        SOp op;
        if(t.kind=="NUM"){ op.kind=SOp::PushNum; op.num=t.value; op.ival=t.ival; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="STRING"){ op.kind=SOp::StringLit; op.strvals=t.strvals; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="CHAR"){ op.kind=SOp::CharLit; op.ival=t.ival; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="LOADVAR"){ if(t.ival < 0 || t.ival >= 100) throw std::runtime_error("Numbered variable index must be between 0 and 99"); op.kind=SOp::LoadVar; op.varIndex=(int)t.ival; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="DOTNAME"){ op.kind=SOp::LoadNamed; op.name=t.name; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="STORENAMED"){ op.kind=SOp::StoreNamed; op.name=t.name; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="TILDE"){ op.kind=SOp::Not; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="AT"){ op.kind=SOp::Dup; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="HASH"){ op.kind=SOp::Swap; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="DOLLAR"){ op.kind=SOp::Drop; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="PLUS"){ op.kind=SOp::Add; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="MINUS"){ op.kind=SOp::Sub; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="STAR"){ op.kind=SOp::Mul; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="SLASH"){ op.kind=SOp::Div; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="PERCENT"){ op.kind=SOp::Mod; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="EQ"){ op.kind=SOp::Eq; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="LT"){ op.kind=SOp::Lt; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="GT"){ op.kind=SOp::Gt; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="BAR"){ op.kind=SOp::Print; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="CARET"){ op.kind=SOp::PrintChar; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="QUESTION"){ op.kind=SOp::Input; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="COLON"){ op.kind=SOp::Store; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="LBRACK"){ op.kind=SOp::WhileStart; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="RBRACK"){ op.kind=SOp::WhileEnd; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="LBRACE"){ op.kind=SOp::IfStart; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="RBRACE"){ op.kind=SOp::IfEnd; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="CALLFUNC"){ op.kind=SOp::CallFunc; op.funcNum=(int)t.ival; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="CALLNAMED"){ op.kind=SOp::CallNamed; op.name=t.name; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="BREAK"){ op.kind=SOp::Break; advance(); mainOps.push_back(op); continue; }

        if(t.kind=="S"){ op.kind=SOp::Sin; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="C"){ op.kind=SOp::Cos; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="T"){ op.kind=SOp::Tan; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="R"){ op.kind=SOp::Sqrt; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="P"){ op.kind=SOp::Pow; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="F"){ op.kind=SOp::Floor; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="L"){ op.kind=SOp::Log; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="E"){ op.kind=SOp::Exp; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="M"){ op.kind=SOp::Abs; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="N"){ op.kind=SOp::Atan2; advance(); mainOps.push_back(op); continue; }
        throw std::runtime_error("Unsupported token " + t.kind + " at line " +
                                 std::to_string(t.line) + ", column " + std::to_string(t.col));
    }
    prog.mainOps = mainOps;
    return prog;
}
