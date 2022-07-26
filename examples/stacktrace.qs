foo = () -> { {1 / 0}() };
bar = () -> { foo() };
baz = () -> { bar() };

baz();
