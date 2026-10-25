cmake_minimum_required(VERSION 3.16)
project(sigi CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

add_compile_options(-Wall -Wextra)

include(FetchContent)
FetchContent_Declare(
    catch2
    URL https://github.com/catchorg/Catch2/archive/refs/tags/v3.5.0.tar.gz
    TIMEOUT 60
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
)
FetchContent_MakeAvailable(catch2)

find_package(Python3 REQUIRED COMPONENTS Interpreter)

enable_testing()

add_executable(sigic
    src/lexer/lexer.cpp
    src/parser/parser.cpp
    src/sema/sema.cpp
    src/codegen/codegen.cpp
    src/cli/main.cpp
)
target_include_directories(sigic PRIVATE src)

add_executable(unit_tests
    tests/unit/test_lexer.cpp
    tests/unit/test_parser.cpp
    tests/unit/test_sema.cpp
    tests/unit/test_codegen.cpp
    src/lexer/lexer.cpp
    src/parser/parser.cpp
    src/sema/sema.cpp
    src/codegen/codegen.cpp
)
target_include_directories(unit_tests PRIVATE src)
target_link_libraries(unit_tests PRIVATE Catch2::Catch2WithMain)
add_test(NAME unit_tests COMMAND unit_tests)
add_test(NAME cli_smoke
    COMMAND ${Python3_EXECUTABLE} ${CMAKE_SOURCE_DIR}/tests/smoke.py
            $<TARGET_FILE:sigic> ${CMAKE_C_COMPILER})
