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
	for
		${$!; $init:token} ;
		${$!; $cond:token} ;
		${$!$_:block $incr:($_:tt $| $_:token)}
		$body:block
} = {
	${$init};
	while({ ${$cond} }, {
		$body();
		${$incr}
	});
};

for x=0; x < 10; x=x+1 {
	print(x);
}


$syntax {
	${$!? $cond:($_:tt $| $_:token)}
	? ${$!: $iftrue:($_:tt $| $_:token)}
	: ${$!; $!, $!\) $!\] $!\} $iffalse:($_:tt $| $_:token) }
} = {
	if(${$cond}, {
		${$iftrue}
	}, {
		${$iffalse}
	})
};

for i=0; i < 10; i=i+1 {
	print(0 == i % 2 ? "odd" : "even!");
}


print(1 ? 2 : 3);

# Integer.'!!' = Integer::factorial;
# Integer.choose = (n, k) -> {
# 	(!!n) / (!!k * !!(n - k))
# };
# 
# $syntax { (
# 	$l:($_:group $| $_:literal) $r:($_:group $| $_:literal)
# ) } = {
# 	$l.choose($r)
# };
# 
# print( (10 4) );
