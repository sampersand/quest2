$syntax { alias $from:token $to:token; } = {
	$$syntax {$from} = {$to};
};

alias <- =;
alias echo print;

x <- 10;
echo(x);

