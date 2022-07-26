Integer.'!!' = Integer::factorial;
Integer.choose = (n, k) -> {
	(!!n) / (!!k * !!(n - k))
};

$syntax { (
	$l:($_:group $| $_:literal) $r:($_:group $| $_:literal)
) } = {
	$l.choose($r)
};

print( (10 4) );
