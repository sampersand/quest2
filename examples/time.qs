Time = object(() -> {
	'()' = (class, hour, minute, second) -> {
		:0.__parents__ = [class];
		:0
	}
});


$syntax! hrmin { $hr:int : $min:int }; # `!` indicates it's just for matching.
$syntax! walltime { $hm:hrmin $[. $sec:int] }; # `$[ ... ]` means contents optional
$syntax { $wt:walltime $ampm:(am $| pm) } = { ... };

$syntax { $hr:int : $min:int } = { ... };
$syntax { $hr:int : $min:int am } = { ... };
$syntax { $hr:int : $min:int pm } = { ... };
$syntax { $hr:int : $min:int . $sec:int } = { ... };
$syntax { $hr:int : $min:int . $sec:int am } = { ... };
$syntax { $hr:int : $min:int . $sec:int pm } = { ... };

$syntax! hrmin { $hr:int : $min:int } = { };
$syntax { @ $h:hrmin } = { $($hr) };

print(@ 1 : 2);
__EOF__

$syntax time {
	$hr:int : $min:int $ampm:(am $| pm)
} = {
	Time($hr, $min, {am='am';pm='pm'; $ampm}())
};

$syntax time { $hr:int : $min:int . $sec:int } = { Time($hr, $min, $sec) };

$syntax { $t:time am } = { $t } ;
$syntax { $t:time pm } = { ($t + 216_000) } ;

(10 : 30 . 45 pm) - (10 : 30 am)
