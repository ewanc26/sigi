# CMake generated Testfile for 
# Source directory: /Volumes/Storage/Mac/projects/Developer/Git/sigi
# Build directory: /Volumes/Storage/Mac/projects/Developer/Git/sigi/build
# 
# This file includes the relevant testing commands required for 
# testing this directory and lists subdirectories to be tested as well.
add_test(unit_tests "/Volumes/Storage/Mac/projects/Developer/Git/sigi/build/unit_tests")
set_tests_properties(unit_tests PROPERTIES  _BACKTRACE_TRIPLES "/Volumes/Storage/Mac/projects/Developer/Git/sigi/CMakeLists.txt;44;add_test;/Volumes/Storage/Mac/projects/Developer/Git/sigi/CMakeLists.txt;0;")
subdirs("_deps/catch2-build")
