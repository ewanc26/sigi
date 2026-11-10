#include "../lexer/lexer.h"
#include "../parser/parser.h"
#include "../sema/sema.h"
#include "../codegen/codegen.h"

#include <chrono>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <random>
#include <sstream>
#include <stdexcept>
#include <string>
#include <system_error>
#include <utility>
#include <vector>
#include <sys/wait.h>
#include <unistd.h>

namespace {

std::filesystem::path create_temp_directory() {
    const auto base = std::filesystem::temp_directory_path();
    const auto ticks = std::chrono::steady_clock::now().time_since_epoch().count();
    std::random_device random;
    for (int attempt = 0; attempt < 16; ++attempt) {
        const auto candidate = base / ("sigi-" + std::to_string(ticks) + "-" +
                                       std::to_string(random()));
        std::error_code error;
        if (std::filesystem::create_directory(candidate, error)) return candidate;
        if (error && error != std::errc::file_exists) {
            throw std::runtime_error("Cannot create temporary directory: " + error.message());
        }
    }
    throw std::runtime_error("Cannot create a unique temporary directory for --run");
}

struct TemporaryDirectory {
    explicit TemporaryDirectory(std::filesystem::path value) : path(std::move(value)) {}
    ~TemporaryDirectory() {
        std::error_code ignored;
        std::filesystem::remove_all(path, ignored);
    }
    std::filesystem::path path;
};

int run_command(const std::string &program, const std::vector<std::string> &args) {
    std::vector<char *> argv;
    argv.push_back(const_cast<char *>(program.c_str()));
    for (const auto &arg : args) {
        argv.push_back(const_cast<char *>(arg.c_str()));
    }
    argv.push_back(nullptr);

    pid_t pid = fork();
    if (pid == -1) {
        return -1;
    }
    if (pid == 0) {
        execvp(program.c_str(), argv.data());
        _exit(127);
    }
    int status = 0;
    if (waitpid(pid, &status, 0) == -1) {
        return -1;
    }
    if (WIFEXITED(status)) return WEXITSTATUS(status);
    if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);
    return 1;
}

} // namespace

int main(int argc, char **argv) {
    if (argc < 2) {
        std::cerr << "usage: sigic <file.si> -o <output.c> | --run [--cc <compiler>]\n";
        return 1;
    }
    if (std::string(argv[1]) == "--support") {
        std::cout << "Support Sigi development:\n";
        std::cout << "  Ko-fi: https://ko-fi.com/ewancroft\n";
        std::cout << "  GitHub Sponsors: https://github.com/sponsors/ewanc26\n";
        return 0;
    }

    const std::string input = argv[1];
    std::string output;
    bool run = false;
    std::string cc = "gcc";
    for (int i = 2; i < argc; ++i) {
        const std::string arg = argv[i];
        if (arg == "-o" || arg == "--output") {
            if (i + 1 >= argc) {
                std::cerr << "sigic: " << arg << " requires a path\n";
                return 1;
            }
            output = argv[++i];
        } else if (arg == "--run") {
            run = true;
        } else if (arg == "--cc") {
            if (i + 1 >= argc) {
                std::cerr << "sigic: --cc requires a compiler executable path\n";
                return 1;
            }
            cc = argv[++i];
        } else {
            std::cerr << "sigic: unknown option " << arg << "\n";
            return 1;
        }
    }
    if (run && !output.empty()) {
        std::cerr << "sigic: choose either --run or -o/--output, not both\n";
        return 1;
    }
    if (!run && output.empty()) {
        std::cerr << "sigic: specify -o <output.c> or --run\n";
        return 1;
    }

    std::ifstream in(input);
    if (!in) {
        std::cerr << "sigic: cannot read source file: " << input << "\n";
        return 1;
    }
    std::stringstream ss;
    ss << in.rdbuf();
    const std::string source = ss.str();

    try {
        Lexer lexer(source);
        const auto tokens = lexer.tokenize();
        Parser parser(tokens);
        auto program = parser.parse();
        SemanticAnalyzer sema;
        sema.analyze(program);
        const std::string ccode = emitSigi(program);

        if (!run) {
            std::ofstream out(output);
            if (!out) {
                std::cerr << "sigic: cannot create output file: " << output << "\n";
                return 1;
            }
            out << ccode;
            if (!out) {
                std::cerr << "sigic: failed while writing output file: " << output << "\n";
                return 1;
            }
            return 0;
        }

        TemporaryDirectory temp(create_temp_directory());
        const auto cfile = (temp.path / "sigi_out.c").string();
        const auto exefile = (temp.path / "sigi_out").string();
        {
            std::ofstream generated(cfile);
            if (!generated) {
                std::cerr << "sigic: cannot create temporary C source\n";
                return 1;
            }
            generated << ccode;
            if (!generated) {
                std::cerr << "sigic: failed while writing temporary C source\n";
                return 1;
            }
        }

        const int compile_status = run_command(cc, {cfile, "-o", exefile, "-lm"});
        if (compile_status == -1) {
            std::cerr << "sigic: failed to start the host C compiler\n";
            return 1;
        }
        if (compile_status != 0) return compile_status;

        const int run_status = run_command(exefile, {});
        if (run_status == -1) {
            std::cerr << "sigic: failed to start the generated program\n";
            return 1;
        }
        return run_status;
    } catch (const std::exception &e) {
        std::cerr << "sigic: " << e.what() << "\n";
        return 1;
    }
}
