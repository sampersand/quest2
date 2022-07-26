Integer.'!!' = Integer::factorial;
Integer.nCk = (n, k) -> { (!!n) / (!!k * !!(n - k)) };

$syntax { (
	$l:($_:group $| $_:literal) $r:($_:group $| $_:literal)
) } = {
	$l.nCk($r)
};

ten_choose_four = (10 4);

print(ten_choose_four);
