Integer.'!!' = Integer::factorial;
Integer.choose = (n, k) -> {
	(!!n) / (!!k * !!(n - k))
};

$syntax { ($l:tt $r:tt) } = { $l.choose($r) };

print( (10 4) );
 
