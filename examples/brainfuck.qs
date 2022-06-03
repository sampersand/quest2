$syntax { @ > } = { :-1.ptr = ptr + 1; };
$syntax { @ < } = { :-1.ptr = ptr - 1; };
$syntax { @ + } = { array[ptr] = array[ptr] + 1; };
$syntax { @ - } = { array[ptr] = array[ptr] - 1; };
$syntax { @ . } = { print(array[ptr].dbg()); };
$syntax { @ , } = { array[ptr] = getchar(); };
$syntax { @ \[ } = { while \({ ptr }, \{ };
$syntax { @ \] } = { \}\); };

$syntax { brainfuck { ${$!\} $t:token} } } = {
    array = 1.upto(30000).map({ 0 });
    ptr = 0;
    ${ @ $t }
};

brainfuck {
    + + + + + + + + + + [ > + + + + + + + > + + + + + + + + + + > + + + > + < <
    < < - ] > + + . > + . + + + + + + + . . + + + . > + + . < < + + + + + + + +
    + + + + + + + . > . + + + . - - - - - - . - - - - - - - - . > + . > . 
}

