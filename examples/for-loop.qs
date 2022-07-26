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
