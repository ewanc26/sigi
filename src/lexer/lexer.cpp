#include "lexer.h"
#include <cctype>
#include <stdexcept>
#include <string>

Lexer::Lexer(const std::string &s): source(s){}
char Lexer::peek(int o) const { if (i+o>=source.size()) return 0; return source[i+o]; }
char Lexer::advance(){ if(i>=source.size()) return 0; char c=source[i++]; if(c=='\n'){line++;col=1;}else col++; return c; }
bool Lexer::eof() const { return i>=source.size(); }

std::vector<Token> Lexer::tokenize(){
    std::vector<Token> tokens;
    while(!eof()){
        char c = peek();
        if(c==' '||c=='\t'||c=='\r'||c=='\n'){advance(); continue;}
        if(c=='\\'){ while(!eof() && peek()!='\n') advance(); continue;}
        if(c=='/' && peek(1)=='*'){ advance();advance(); while(!eof()){ if(peek()=='*'&&peek(1)=='/'){advance();advance();break;} advance();} continue;}
        if(c=='"'){
            int sl=line,sc=col; advance();
            std::vector<int> vals;
            while(!eof() && peek()!='"'){
                if(peek()=='\\'){ advance(); char e=advance();
                    if(e=='n') vals.push_back(10);
                    else if(e=='t') vals.push_back(9);
                    else if(e=='r') vals.push_back(13);
                    else if(e=='\\') vals.push_back(92);
                    else if(e=='"') vals.push_back(34);
                    else if(e=='x'){ char h1=advance(),h2=advance(); auto hex=[](char x){if(x>='0'&&x<='9')return x-'0';if(x>='a'&&x<='f')return x-'a'+10;if(x>='A'&&x<='F')return x-'A'+10;return 0;}; vals.push_back(hex(h1)*16+hex(h2)); }
                    else if(std::isdigit(e)){ std::string o;o+=e;if(std::isdigit(peek()))o+=advance();if(std::isdigit(peek()))o+=advance(); vals.push_back(std::stoi(o,nullptr,8));}
                    else vals.push_back((unsigned char)e);
                }else vals.push_back((unsigned char)advance());
            }
            if(peek()=='"') advance();
            tokens.push_back(Token{"STRING",0,0,vals,"",sl,sc}); continue;
        }
        if(c=='\''){
            int sl=line,sc=col; advance();
            int v=0;
            if(peek()=='\\'){ advance(); char e=advance();
                if(e=='n') v=10; else if(e=='t') v=9; else if(e=='r') v=13; else if(e=='\\') v=92; else if(e=='\'') v=39;
                else if(e=='x'){ char h1=advance(),h2=advance(); auto hex=[](char x){if(x>='0'&&x<='9')return x-'0';if(x>='a'&&x<='f')return x-'a'+10;if(x>='A'&&x<='F')return x-'A'+10;return 0;}; v=hex(h1)*16+hex(h2); }
                else if(std::isdigit(e)){ std::string o;o+=e;if(std::isdigit(peek()))o+=advance();if(std::isdigit(peek()))o+=advance(); v=std::stoi(o,nullptr,8);}
                else v=(unsigned char)e;
            }else v=(unsigned char)advance();
            if(peek()=='\'') advance();
            tokens.push_back(Token{"CHAR",0,v,{},{},sl,sc}); continue;
        }
        if(c=='.'){
            int sl=line,sc=col; advance();
            if(std::isalpha(peek())||peek()=='_'){
                std::string name;
                while(std::isalnum(peek())||peek()=='_') name+=advance();
                tokens.push_back(Token{"DOTNAME",0,0,{},{name},sl,sc});
            }else tokens.push_back(Token{"DOT",0,0,{},{},sl,sc});
            continue;
        }
        if(c=='<'){
            int sl=line,sc=col; advance();
            if(std::isdigit(peek())){
                std::string n; while(std::isdigit(peek())) n+=advance();
                tokens.push_back(Token{"LOADVAR",0,(long long)std::stoi(n),{},{},sl,sc});
            }else if(peek()=='>'){ advance(); tokens.push_back(Token{"BREAK",0,0,{},{},sl,sc}); }
            else tokens.push_back(Token{"LT",0,0,{},{},sl,sc});
            continue;
        }
        if(c=='>'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"GT",0,0,{},{},sl,sc}); continue;}
        if(c=='!'){
            int sl=line,sc=col; advance();
            if(std::isdigit(peek())||peek()=='.'||peek()=='-'||peek()=='+'){
                std::string s;
                if(peek()=='0'&&peek(1)=='x'){ s+=advance();s+=advance(); while(std::isxdigit(peek()))s+=advance(); while(std::isalpha(peek()))advance(); long long v=std::stoll(s,nullptr,16); tokens.push_back(Token{"NUM",0,v,{},{},sl,sc}); continue;}
                if(peek()=='0'&&peek(1)=='b'){ s+=advance();s+=advance(); while(peek()=='0'||peek()=='1')s+=advance(); while(std::isalpha(peek()))advance(); long long v=0; for(size_t j=2;j<s.size();j++) v=v*2+(s[j]-'0'); tokens.push_back(Token{"NUM",0,v,{},{},sl,sc}); continue;}
                while(std::isdigit(peek()))s+=advance();
                if(peek()=='.'){ s+=advance(); while(std::isdigit(peek()))s+=advance(); }
                if(peek()=='e'||peek()=='E'){ s+=advance(); if(peek()=='+'||peek()=='-')s+=advance(); while(std::isdigit(peek()))s+=advance(); }
                while(std::isalpha(peek()))advance();
                if(s.find('.')!=std::string::npos){ tokens.push_back(Token{"NUM",(double)std::stod(s),0,{},{},sl,sc}); }
                else if(s.size()>1&&s[0]=='0'){ long long v=0; for(char ch:s) if(ch>='0'&&ch<='7')v=v*8+(ch-'0'); tokens.push_back(Token{"NUM",0,v,{},{},sl,sc}); }
                else { tokens.push_back(Token{"NUM",(double)std::stoll(s),std::stoll(s),{},{},sl,sc}); }
                continue;
            }else tokens.push_back(Token{"BANG",0,0,{},{},sl,sc}); continue;
        }
        if(c=='['){ int sl=line,sc=col; advance(); tokens.push_back(Token{"LBRACK",0,0,{},{},sl,sc}); continue;}
        if(c==']'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"RBRACK",0,0,{},{},sl,sc}); continue;}
        if(c=='{'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"LBRACE",0,0,{},{},sl,sc}); continue;}
        if(c=='}'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"RBRACE",0,0,{},{},sl,sc}); continue;}
        if(c=='('){ int sl=line,sc=col; advance(); if(std::isdigit(peek())){ std::string n; while(std::isdigit(peek()))n+=advance(); tokens.push_back(Token{"CALLFUNC",0,(long long)std::stoi(n),{},{},sl,sc}); if(peek()==')')advance(); continue;} if(peek()=='.'){ advance(); if(std::isalpha(peek())||peek()=='_'){ std::string name; while(std::isalnum(peek())||peek()=='_')name+=advance(); tokens.push_back(Token{"CALLNAMED",0,0,{},{name},sl,sc}); if(peek()==')')advance(); continue;}} if(peek()==')'){advance(); tokens.push_back(Token{"RPAREN",0,0,{},{},sl,sc}); continue;} tokens.push_back(Token{"LPAREN",0,0,{},{},sl,sc}); continue;}
        if(c==')'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"RPAREN",0,0,{},{},sl,sc}); continue;}
        if(c=='@'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"AT",0,0,{},{},sl,sc}); continue;}
        if(c==':'){ int sl=line,sc=col; advance(); if(peek()=='.'){ advance(); std::string name; while(std::isalnum(peek())||peek()=='_')name+=advance(); tokens.push_back(Token{"STORENAMED",0,0,{},{name},sl,sc}); }else tokens.push_back(Token{"COLON",0,0,{},{},sl,sc}); continue;}
        if(c==';'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"SEMI",0,0,{},{},sl,sc}); continue;}
        if(c==','){ int sl=line,sc=col; advance(); tokens.push_back(Token{"COMMA",0,0,{},{},sl,sc}); continue;}
        if(c=='=' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"EQ",0,0,{},{},sl,sc}); continue;}
        if(c=='+' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"PLUS",0,0,{},{},sl,sc}); continue;}
        if(c=='-' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"MINUS",0,0,{},{},sl,sc}); continue;}
        if(c=='*' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"STAR",0,0,{},{},sl,sc}); continue;}
        if(c=='/' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"SLASH",0,0,{},{},sl,sc}); continue;}
        if(c=='%' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"PERCENT",0,0,{},{},sl,sc}); continue;}
        if(c=='~' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"TILDE",0,0,{},{},sl,sc}); continue;}
        if(c=='^' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"CARET",0,0,{},{},sl,sc}); continue;}
        if(c=='?' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"QUESTION",0,0,{},{},sl,sc}); continue;}
        if(c=='|' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"BAR",0,0,{},{},sl,sc}); continue;}
        if(c=='$' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"DOLLAR",0,0,{},{},sl,sc}); continue;}
        if(c=='#' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"HASH",0,0,{},{},sl,sc}); continue;}
        if(c=='@' ){ int sl=line,sc=col; advance(); tokens.push_back(Token{"AT",0,0,{},{},sl,sc}); continue;}
        if(c=='S'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"S",0,0,{},{},sl,sc}); continue;}
        if(c=='C'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"C",0,0,{},{},sl,sc}); continue;}
        if(c=='T'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"T",0,0,{},{},sl,sc}); continue;}
        if(c=='R'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"R",0,0,{},{},sl,sc}); continue;}
        if(c=='P'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"P",0,0,{},{},sl,sc}); continue;}
        if(c=='F'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"F",0,0,{},{},sl,sc}); continue;}
        if(c=='L'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"L",0,0,{},{},sl,sc}); continue;}
        if(c=='E'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"E",0,0,{},{},sl,sc}); continue;}
        if(c=='M'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"M",0,0,{},{},sl,sc}); continue;}
        if(c=='N'){ int sl=line,sc=col; advance(); tokens.push_back(Token{"N",0,0,{},{},sl,sc}); continue;}
        const int unsupported_line = line, unsupported_column = col;
        const char unsupported = advance();
        throw std::runtime_error("Unsupported character '" + std::string(1, unsupported) +
                                 "' at line " + std::to_string(unsupported_line) +
                                 ", column " + std::to_string(unsupported_column));
    }
    return tokens;
}
