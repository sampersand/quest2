Text = "".__parents__[0];
Boolean = true.__parents__[0];
Null = null.__parents__[0];

Integer.tobool = n -> { n != 0 };
Text.tobool = n -> { n != "" };
Boolean.tobool = n -> { n };
Null.tobool = n -> { false };

Integer.tonum = n -> { n };
Text.tonum = n -> { abort("<todo: string to num>") };
Boolean.tonum = n -> { ifl(n, 1, 0) };
Null.tonum = n -> { 0 };

Integer.tostr = n -> { n.to_text() };
Text.tostr = n -> { n.to_text() };
Boolean.tostr = n -> { n.to_text() };
Null.tostr = n -> { n.to_text() };

Integer.add = (l,r) -> { l + r.tonum() };
Text.add = (l,r) -> { l + r.tostr() };
Integer.'^' = Integer::'**';

Integer.lt = (l,r) -> { l < r.tonum() };
Integer.gt = (l,r) -> { l > r.tonum() };
Text.lt = (l,r) -> { l < r.tostr() };
Text.gt = (l,r) -> { l > r.tostr() };
Boolean.lt = (l,r) -> { (!l).and(r.tobool()) };
Boolean.gt = (l,r) -> { l.and(!r.tobool()) };

# whitespace
$syntax kn { @ $_:(\( $| \) $| \[ $| \] $| \{ $| \} $| :) $r:kn } = { $r };

# primitives
$syntax kn { @ $n:int } = { $n };
$syntax kn { @ $n:text } = { $n };
$syntax kn { @ $n:ident } = { env . $n };
$syntax kn { @ $(T $| TRUE) } = { true };
$syntax kn { @ F } = { false };
$syntax kn { @ N } = { null };
# nullary
$syntax kn { @ P } = { abort("<todo: make prompt in quest>") };
$syntax kn { @ R } = { abort("<todo: make random in quest>") };

#unary
$syntax kn { @ E $r:kn } = { abort("<eval isnt supported until quest gets it>") };
$syntax kn { @ B $r:kn } = { { $r } };
$syntax kn { @ C $r:kn } = { ($r)() };
$syntax kn { @ ` $r:kn } = { abort("<todo: make ` in quest>") };
$syntax kn { @ Q $r:kn } = { exit(($r).to_int()) };
$syntax kn { @ ! $r:kn } = { !($r).tobool() };
$syntax kn { @ L $r:kn } = { ($r).len() };
$syntax kn { @ D $r:kn } = { ($r).tap(print) };
$syntax kn { @ O $r:kn } = { ($r.print(); null) };
$syntax kn { @ A $r:kn } = { abort("<todo: make ascii in quest>") };
$syntax kn { @ ~ $r:kn } = { -($r).tonum() };

#binary
$syntax kn { @ + $l:kn $r:kn } = { (($l).add($r)) };
$syntax kn { @ $op:(- $| * $| / $| % $| ^) $l:kn $r:kn } = { (($l) $op ($r)) };
$syntax kn { @ < $l:kn $r:kn } = { (($l).lt($r)) };
$syntax kn { @ > $l:kn $r:kn } = { (($l).gt($r)) };
$syntax kn { @ ? $l:kn $r:kn } = { (($l) == ($r)) };
$syntax kn { @ & $l:kn $r:kn } = { (x=($l); if(x.tobool(), { $r } , { x })) };
$syntax kn { @ | $l:kn $r:kn } = { (x=($l); if(x.tobool(), { x } , { $r })) };
$syntax kn { @ ; $l:kn $r:kn } = { (($l); ($r)) };
$syntax kn { @ = $l:kn $r:kn } = { ($l = ($r)) };
$syntax kn { @ W $l:kn $r:kn } = { (while({ $l }, { $r }); null) };

# ternary
$syntax kn { @ I $l:kn $m:kn $r:kn } = { if($l, { $m }, { $r }) };
$syntax kn { @ G $l:kn $m:kn $r:kn } = { abort("<todo: substr in quest>"); };

# quaternary
$syntax kn { @ S $l:kn $m:kn $r:kn $x:kn } = { abort("<todo substr>") };

$syntax { knight ${$!XDONE $tkn:token} XDONE } = { (env={:0}(); ${@ $tkn}) };

# and here we go, Knight in Quest! (The only caveat is you can only use single letter names)
knight
	; = fizzbuzz B
		; = n 0
		; = max (+ 1 max)
		: W < (= n + 1 n) max
			: O
				: I ! (% n 15) "FizzBuzz"
				: I ! (% n 5)  "Fizz"
				: I ! (% n 3)  "Buzz"
				                n
	; = max 100
	: C fizzbuzz
XDONE
