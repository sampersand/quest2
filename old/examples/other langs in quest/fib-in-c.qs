#include <stdio.h>
#if 0
$syntax { $i:ident $name:ident () $body:block } = { $name = () -> $body; };
$syntax { $i:ident $name:ident ($j:ident $arg:ident) $body:block } = { $name = ($arg) -> $body; };
printf = (_, what) -> { what.display() };
$syntax { $cond:tt ? $ift:tt : $iff:tt } = { if($cond, { $ift }, { $iff }) };
$syntax { return $what:tt ; } = { $what };
$syntax { if $cond:tt $ift:block else $iff:block } = { (if)($cond, $ift, $iff) } ;
$syntax { int main (void) $body:block } = { int main () $body main(); } ;
$syntax { $i:ident $name:ident = $value:tt ; } = { $name = $value; };
#endif

long fibonacci(long n) {
	if (n <= 1) {
		return n;
	} else {
		return (fibonacci(n-1) + fibonacci(n-2));
	}
}

int main (void) {
	long max = 10;

	printf("%ld\n", fibonacci(max));
}
