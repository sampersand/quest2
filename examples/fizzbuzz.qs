$syntax { $l:tt .. $r:tt } = { $l.upto($r) };
Integer.'|' = (b, a) -> { 0 == a % b };

fizzbuzz = max -> {
	(1..max).map(n -> {
		(15 | n).then('FizzBuzz'.return);
		(3  | n).then('Fizz'.return);
		(5  | n).then('Buzz'.return);
		n
	})
};

fizzbuzz(100).each(print);

