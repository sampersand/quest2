$syntax (brainfuck { ${$!\} $t:token} }) = {
	(() -> {
		array = [0] * 30000;
		ptr = 0;
		${ @ $t }
	}());
};

$syntax { @ > } = { ptr = ptr + 1; };
$syntax { @ < } = { ptr = ptr - 1; };
$syntax { @ + } = { array[ptr] = array[ptr] + 1; };
$syntax { @ - } = { array[ptr] = array[ptr] - 1; };
$syntax { @ . } = { print_no_newline(array[ptr].chr()); };
$syntax { @ , } = { array[ptr] = getchar(); };
$syntax { @ \[ } = { while \({ array[ptr] != 0 }, \{ };
$syntax { @ \] } = { \}\); };


brainfuck {
	+ + + + + + + + + + [ > + + + + + + + > + + + + + + + + + +
	> + + + > + < < < < - ] > + + . > + . + + + + + + + . . + +
	+ . > + + . < < + + + + + + + + + + + + + + + . > . + + + .
	- - - - - - . - - - - - - - - . > + . > .
}
