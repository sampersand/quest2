# Syntax macros in Quest are a way to replace arbitrary sequences of input
# tokens with other tokens. The syntax for the macros is:
#   $syntax <matching pattern> = <replacement pattern> ;
# It will only do the replacement if the match perfectly matches.
$syntax {
	# For the entire matching pattern to match, we must first see the
	# identifier `brainfuck` followed by the token `{`.
	brainfuck {
		# Quest has special syntax for repeating macro sequences. Here we're
		# using `${ ... }`, which means that the `...` contents can be matched
		# zero or more times.
		${
			# Normally, encountering a token in the matching pattern
			# means that the token must be matched exactly. If you
			# prefix that token with `$!`, that means the token _musnt_
			# match. In this case, this means that `}` cannot be the
			# next token. You can think of it as `(?!)` in regex (negative
			# lookahead). The `\` is needed to escape the `}`, so it isnt
			# interpreted as the end of the `${...}` block.
			$!\}

			# Then, the next token can be anything. The pattern of
			# `${$!<TOKEN> $t:token}` means to take as many non-`<TOKEN>`s
			# as you can. So this will read until a `}` is encountered, or EOF.
			$t:token
		} # close the `${...}`

	# After the `${...}`, we must match the closing `}`. This means that
	# `brainfuck { ... <END OF FILE>` will not match.
	}
} = {
	# If that entire thing beforehand matched, replace it with the following:
	(() -> {
		array = [0] * 30000;
		ptr = 0;
		# We're going to expand brainfuck tokens out into quest code. But, since
		# those tokens themselves are part of valid quest expressions, we don't want
		# to expand out those valid expressions. So, we prefix each token we parsed
		# with an `@`, so we can now match, eg, `@ +` instead of just `+`.
		${ @ $t }
	})();
};

# The following are the syntax expansions for valid brainfuck programs.
# Note that for `[` and `]`, the brackets have to be escaped, as they're
# unmatched within their respective expansions.
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
