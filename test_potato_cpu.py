from typing import Final

from py_ca_compiler import PyPotatoCPUTester


COMPILER_TESTS_DIR: Final[str] = 'writing-a-c-compiler-tests'

test_path = f'{COMPILER_TESTS_DIR}/tests/chapter_1/valid/return_2.c'
potato_program = PyPotatoCPUTester.compile_from_source(test_path)
result = potato_program.execute()
print('Result:', result)
assert result == 2
