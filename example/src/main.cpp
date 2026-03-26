#include <iostream>
#include "include/test.h"

void print_args(int argc, char* argv[]) {
    for (int i = 1; i < argc; ++i) {
        std::cout << argv[i] << std::endl;
    }
}

int main(int argc, char* argv[])
{
    print_args(argc, argv);
    test_hello_rmake();
    return 0;
}
