Integer.fibonacci = n -> {
	(n <= 1).then(n.return);

	(n - 1).fibonacci() + (n - 2).fibonacci()
};

10.fibonacci().display();


#fibonacci = n -> {
#	(n <= 1).then(n.return);
#
#	fibonacci(n - 1) + fibonacci(n - 2)
#};
#
#print(fibonacci(10));
