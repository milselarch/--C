from py_ca_compiler import PyPotatoCPUTester

test_path = 'writing-a-c-compiler-tests/tests/chapter_1/valid/return_2.c'
potato_program = PyPotatoCPUTester.compile_from_source(test_path)
result = potato_program.execute()
print('Result:', result)
assert result == 2
