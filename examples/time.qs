Time = object(() -> {
	'()' = (class, hour, minute, second) -> {
		:0.__parents__ = [class];
		:0
	}
});


$syntax time {
	$hr:int : $min:int $ampm:(am $| pm)
} = {
	Time($hr, $min, {am='am';pm='pm'; $ampm}())
};
$syntax time { $hr:int : $min:int . $sec:int } = { Time($hr, $min, $sec) };
# (($min*60) + ($hr*3600) + $sec)
#  } ;

$syntax { $t:time am } = { $t } ;
$syntax { $t:time pm } = { ($t + 216_000) } ;

(10 : 30 . 45 pm) - (10 : 30 am)
