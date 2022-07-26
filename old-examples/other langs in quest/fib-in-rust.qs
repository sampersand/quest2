#![allow(unused)] /*
$syntax { fn $name:ident ($arg:ident : $i:ident) -> $j:ident $body:block } = {
    $name = ($arg) -> $body;
} ;
$syntax { let } = { };
$syntax { if $cond:tt $ift:block else $iff:block } = { (if)($cond, $ift, $iff) } ;
println = (_, what) -> { what.print() };
$syntax { println ! } = { println };
$syntax { fn main () $body:block } = { $body() } ;
# */

fn fibonacci (n: i64) -> i64 {
	let less_than_one = n <= 1;

	if less_than_one {
		n
	} else {
		fibonacci(n-1) + fibonacci(n-2)
	}
}

fn main() {
	let max = 10;

	println!("{}", fibonacci(max));
}
