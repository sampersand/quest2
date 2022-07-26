Integer.factorial = (n) -> { n.downto(1).reduce(Integer::'*') };

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
