# Normal for loop
$syntax {
	for $var:ident in $val:tt $body:block
} = {
	$val.iter().each([$var] -> $body);
};

sum = 0;
for x in [1,2,3,4] {
	sum = sum + x;
}
print(sum);

# For loop with `;`.
$syntax {
	for ${$!; $init:token} ; ${$!; $cond:token} ; ${$!; $incr:token}; $body:block
} = {
	${$init};
	while({ ${$cond} }, { $body(); ${$incr} });
};

for x=0; x < 10; x = x + 1; {
	print(x);
}
