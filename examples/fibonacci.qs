fibonacci = n -> {
	(n <= 1).then(n.return);

	fibonacci(n - 1) + fibonacci(n - 2)
};

print(fibonacci(10));
