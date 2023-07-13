n=16599;
#$syntax{$l:integer @ $r:integer}={((n<= $l)&(n<= $r))};
Integer.'@'=(l,r)->{(l<=n)&n<=r};
#i=(l,r)->{(l<=n)&n<=r};print(ifl(i(1650,1659),'lp',ifl(i(1553,1603)|i(1689,1694)|i(1702,1714)|i(1837,1901)|i(1952,2022),'Q','K')));
 i=(l,r)->{(l<=n)&n<=r};print(ifl(1650@1659,'lp', ifl(1553@1603|1689@1694|1702@1714|1837@1901|1952@2022,'Q','K')));
#N.B. This is sugar for "King".
#whom = 𝔎𝔦𝔫𝔤; N.B. The `;` is needed here because the parser is bad lol
#
#N.B. Check to see if the date is within Cromwell's time.
#if in(date, MDCL, MDCLIX) { 
#    whom = 𝔏𝔬𝔯𝔡 𝔓𝔯𝔬𝔱𝔢𝔠𝔱𝔬𝔯
#} alas if
#    in(date, MCMLII, MMXXII)
#    || in(date, MDCCCXXXVII, MCMI)
#    || in(date, MDCCII, MDCCXIV)
#    || in(date, MDLIII, MDCIII)
#{
#    whom = 𝔔𝔲𝔢𝔢𝔫
#}
#c=(l,r)->{l<=:1.n};
__EOF__
Kings: 927-1553, 1603-1649, 1660-1702, 1714-1837, 1901-1952, 2022

Queens: 1553-1603, 1689-1694, 1702-1714, 1837-1901, 1952-2022

Neither: 1650-1659
f=m->{
	1.upto(m).map(n->{

	})
	print(m)
};
f(3);
__EOF__
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
