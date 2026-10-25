#include "../lexer/lexer.h"
#include "../parser/parser.h"
#include "../sema/sema.h"
#include "../codegen/codegen.h"
#include <iostream>
#include <fstream>
#include <sstream>
#include <cstdlib>

int main(int argc, char **argv) {
    if (argc < 2) {
        std::cerr << "usage: sigic <file.si> [-o output] [--run] [--cc gcc]\n";
        return 1;
    }
    if (std::string(argv[1]) == "--support") {
        std::cout << "Support Sigi development:\n";
        std::cout << "  Ko-fi: https://ko-fi.com/ewancroft\n";
        std::cout << "  GitHub Sponsors: https://github.com/sponsors/ewanc26\n";
        return 0;
    }
    std::string input = argv[1];
    std::string output;
    bool run = false;
    std::string cc = "gcc";
    for (int i = 2; i < argc; i++) {
        std::string a = argv[i];
        if (a == "-o" && i+1 < argc) output = argv[++i];
        else if (a == "--run") run = true;
        else if (a == "--cc" && i+1 < argc) cc = argv[++i];
    }
    if (!run && output.empty()) { std::cerr << "Either -o/--output or --run must be specified\n"; return 1; }
    
    std::ifstream in(input);
    if (!in) { std::cerr << "Failed to read source file\n"; return 1; }
    std::stringstream ss; ss << in.rdbuf();
    std::string source = ss.str();
    
    try {
        Lexer lex(source);
        auto tokens = lex.tokenize();
        Parser p(tokens);
        auto prog = p.parse();
        SemanticAnalyzer sema;
        sema.analyze(prog);
        std::string ccode = emitSigi(prog);
        
        if (run) {
            std::string cfile = "/tmp/sigi_out.c";
            std::string exefile = "/tmp/sigi_out";
            std::ofstream out(cfile);
            out << ccode;
            out.close();
            std::string cmd = cc + " " + cfile + " -o " + exefile + " -lm 2>&1";
            int r = system(cmd.c_str());
            if (r != 0) return 1;
            r = system(exefile.c_str());
            return WEXITSTATUS(r);
        } else {
            std::ofstream out(output);
            out << ccode;
        }
    } catch (std::exception &e) {
        std::cerr << "sigic: " << e.what() << "\n";
        return 1;
    }
    return 0;
}
