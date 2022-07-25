Integer.factorial = (n) -> {
	i = 1;
	while({ n != 0 }, {
		:1.i = i * n;
		:1.n = n - 1;
	});
	i
};


Integer.'!!' = Integer::factorial;
Integer.nCk = (n, k) -> { (!!n) / (!!k * !!(n - k)) };

$syntax {
	(
		$l:($_:group $| $_:literal) $r:($_:group $| $_:literal)
	)
} = {
	$l.nCk($r)
};

print( (10 4) );
