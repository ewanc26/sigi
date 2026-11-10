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
        if(t.kind=="NUM"){ op.kind=SOp::PushNum; op.num=(t.value==0.0 ? static_cast<double>(t.ival) : t.value); op.ival=t.ival; advance(); mainOps.push_back(op); continue; }
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
        if(t.kind=="DO"){
            if (i + 1 >= tokens.size() || tokens[i+1].kind != "LBRACK") {
                throw std::runtime_error("expected '[' after 'd' at line " +
                                          std::to_string(t.line) + ", column " +
                                          std::to_string(t.col));
            }
            advance(); advance();
            op.kind = SOp::DoWhileStart;
            mainOps.push_back(op);
            continue;
        }
        if(t.kind=="LBRACE"){ op.kind=SOp::IfStart; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="RBRACE"){ op.kind=SOp::IfEnd; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="SEMI"){ op.kind=SOp::IfElse; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="CALLFUNC"){ op.kind=SOp::CallFunc; op.funcNum=(int)t.ival; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="CALLNAMED"){ op.kind=SOp::CallNamed; op.name=t.name; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="BREAK"){ op.kind=SOp::Break; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="CONTINUE"){ op.kind=SOp::Continue; advance(); mainOps.push_back(op); continue; }
        if(t.kind=="FFIDECL"){
            advance();
            const Token &hdr = peek();
            if(hdr.kind!="STRING") throw std::runtime_error("FFI declaration needs a quoted header at line " + std::to_string(hdr.line));
            advance();
            std::string headerStr(hdr.strvals.begin(), hdr.strvals.end());
            const Token &nm = peek();
            if(nm.kind!="DOTNAME") throw std::runtime_error("FFI declaration needs a .name at line " + std::to_string(nm.line));
            advance();
            const Token &ar = peek();
            if(ar.kind!="NUM") throw std::runtime_error("FFI declaration needs '!N' argument count at line " + std::to_string(ar.line));
            int argc = (int)(ar.value==0.0 ? (double)ar.ival : ar.value);
            if(argc < 0 || argc > 26) throw std::runtime_error("FFI argument count out of range at line " + std::to_string(ar.line));
            advance();
            op.kind = SOp::FfiDecl;
            op.name = nm.name;
            op.header = headerStr;
            op.funcNum = argc;
            mainOps.push_back(op);
            continue;
        }
        if(t.kind=="FFICALL"){
            advance();
            const Token &nm = peek();
            if(nm.kind!="DOTNAME") throw std::runtime_error("FFI call needs a .name at line " + std::to_string(nm.line) + ", column " + std::to_string(nm.col));
            advance();
            op.kind = SOp::FfiCall;
            op.name = nm.name;
            mainOps.push_back(op);
            continue;
        }

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
    struct ControlFrame { SOp::Kind kind; bool hasElse = false; };
    std::vector<ControlFrame> controls;
    for (auto &op : mainOps) {
        if (op.kind == SOp::WhileStart || op.kind == SOp::DoWhileStart || op.kind == SOp::IfStart) {
            controls.push_back({op.kind, false});
        } else if (op.kind == SOp::WhileEnd) {
            if (controls.empty() ||
                (controls.back().kind != SOp::WhileStart && controls.back().kind != SOp::DoWhileStart)) {
                throw std::runtime_error("unmatched ']' in control flow");
            }
            // A ']' closing a do-while becomes its own terminator so the
            // code generator can emit the trailing condition.
            if (controls.back().kind == SOp::DoWhileStart) {
                op.kind = SOp::DoWhileEnd;
            }
            controls.pop_back();
        } else if (op.kind == SOp::IfElse) {
            if (controls.empty() || controls.back().kind != SOp::IfStart || controls.back().hasElse) {
                throw std::runtime_error("';' must separate the two branches of an if-else block");
            }
            controls.back().hasElse = true;
        } else if (op.kind == SOp::IfEnd) {
            if (controls.empty() || controls.back().kind != SOp::IfStart) {
                throw std::runtime_error("unmatched '}' in control flow");
            }
            if (!controls.back().hasElse) {
                throw std::runtime_error("if-else block is missing ';' separator");
            }
            controls.pop_back();
        } else if (op.kind == SOp::Break || op.kind == SOp::Continue) {
            bool insideLoop = false;
            for (auto frame = controls.rbegin(); frame != controls.rend(); ++frame) {
                if (frame->kind == SOp::WhileStart || frame->kind == SOp::DoWhileStart) {
                    insideLoop = true;
                    break;
                }
            }
            if (!insideLoop) {
                throw std::runtime_error(op.kind == SOp::Break
                    ? "break used outside a loop" : "continue used outside a loop");
            }
        }
    }
    if (!controls.empty()) {
        throw std::runtime_error(controls.back().kind == SOp::IfStart
            ? "unterminated '{' conditional" : "unterminated '[' loop");
    }
    prog.mainOps = mainOps;
    return prog;
}
