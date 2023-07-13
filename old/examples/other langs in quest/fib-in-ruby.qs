{ p=><<-"#EOS" };
$syntax { do } = { \{ };
$syntax { end } = { \} };
$syntax { def $name:ident ($arg:ident) } = { $name = ($arg) -> \{ } ;
$syntax { $func:(fizzbuzz$|upto) $arg:($_:ident$|$_:literal) } = { $func($arg) } ;
$syntax { \) do | $n:ident | } = { \).each \( $n -> \{ } ;
$syntax { end end fizzbuzz } = { \}\); \}; fizzbuzz };
$syntax { case } = { \(\{ } ;
$syntax { when } = { \( } ;
$syntax { then $val:literal } = { \).then({ print($val); $val.return(:2) }); };
$syntax { else $val:tt } = { $val \}()\); \{ };
#EOS
puts = print;

def fizzbuzz(max)
	1.upto max do |n|
		puts case
		     when (0 == n % 15) then 'FizzBuzz'
		     when (0 == n %  3) then 'Fizz'
		     when (0 == n %  5) then 'Buzz'
		     else n
		     end
	end
end

fizzbuzz 100
