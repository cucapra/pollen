LIB := ../../target/debug/libflatgfa.a
TEST_GFA := ../../tests/k.gfa

example: example.o $(LIB)
	cc $^ -o $@

.PHONY: test
test: example
	./$^ $(TEST_GFA)
