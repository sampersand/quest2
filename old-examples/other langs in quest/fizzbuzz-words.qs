$syntax { $l:tt divides $r:tt } = { (0 == $r % $l) };
$syntax { is } = { = };
$syntax { within } = { -> };
$syntax { the block ${$!noblock $t:token} noblock } = ( { ${$t} }; );
$syntax { for $var:ident from $min:tt to $max:tt do ${$!done $t:token} done } = {
	$min.upto($max).map($var -> { ${$t} })
};
$syntax { and } = { . };
$syntax { return $what:literal } = { ($what.return); };

fizzbuzz is max within the block
	for n from 1 to max do
		15 divides n and then return 'FizzBuzz'
		3  divides n and then return 'Fizz'
		5  divides n and then return 'Buzz'
		n
	done
noblock

fizzbuzz(100).each(print);
