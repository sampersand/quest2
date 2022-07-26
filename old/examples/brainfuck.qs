$syntax { @ > } = { ptr = ptr + 1; };
$syntax { @ < } = { ptr = ptr - 1; };
$syntax { @ + } = { array[ptr] = array[ptr] + 1; };
$syntax { @ - } = { array[ptr] = array[ptr] - 1; };
$syntax { @ . } = { print_no_newline(array[ptr].chr()); };
$syntax { @ , } = { array[ptr] = getchar(); };
$syntax { @ \[ } = { while \({ array[ptr] != 0 }, \{ };
$syntax { @ \] } = { \}\); };

$syntax { brainfuck { ${$!\} $t:token} } } = {
    array = 1.upto(30000).map(_ -> { 0 }).to_list();
    ptr = 0;
    ${ @ $t }
};

brainfuck {
    + + + + + + + + + + [ > + + + + + + + > + + + + + + + + + +
    > + + + > + < < < < - ] > + + . > + . + + + + + + + . . + +
    + . > + + . < < + + + + + + + + + + + + + + + . > . + + + .
    - - - - - - . - - - - - - - - . > + . > .
}
