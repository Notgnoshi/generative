cmake_minimum_required(VERSION 3.16)
project(generative LANGUAGES CXX)

# Look for pybind11 cmake modules in the Python virtualenv.
execute_process(
    COMMAND python3 -c "import pybind11; print(pybind11.__path__[0] + '/share/cmake/pybind11')"
    COMMAND_ECHO STDOUT
    OUTPUT_VARIABLE GENERATIVE_PYBIND_PATH
    OUTPUT_STRIP_TRAILING_WHITESPACE
)
set(pybind11_DIR ${GENERATIVE_PYBIND_PATH})

find_package(Python COMPONENTS Interpreter Development REQUIRED)
find_package(pybind11 CONFIG REQUIRED)

pybind11_add_module(cpp src/bindings.cpp)
