#pragma once

#include <fstream>
#include <iostream>
#include <memory>

struct CmdlineArgs
{
private:
    // The ifstream needs somewhere to live so that the std::istream& reference remains valid.
    // I honestly can't believe that it's this difficult to transparently switch between reading
    // from a file or stdin.
    std::unique_ptr<std::ifstream> m_input_file;
    std::unique_ptr<std::ofstream> m_output_file;

public:
    std::istream& input = std::cin;
    std::ostream& output = std::cout;

    CmdlineArgs(const std::string& input_filename, const std::string& output_filename) :
        m_input_file((input_filename.empty() || input_filename == "-")
                         ? nullptr
                         : new std::ifstream(input_filename)),
        m_output_file((output_filename.empty() || output_filename == "-")
                          ? nullptr
                          : new std::ofstream(output_filename)),
        input(m_input_file ? *m_input_file : std::cin),
        output(m_output_file ? *m_output_file : std::cout)
    {
    }

    CmdlineArgs() = default;
    static CmdlineArgs parse_args(int argc, const char* argv[]);
};
