# Define the function `∈` on all objects to be effectively
# a swapped `includes` which iterates over the second argument.
Object."∈" = (obj, list) -> {
	list.iter().includes(obj)
};

# Syntax macro to replace `∀ <ident> ∈ <list> : { ... }` 
# with `<list>.iter().are_all(<ident> -> { ... })`
$syntax { ∀ $name:ident ∈ $list:tt : $body:block } = {
	$list.iter().all($name -> $body)
};

# Define the `is_pangram` function by using the syntax expansion
Text.is_pangram = text -> {
	∀ ele ∈ 'abcdefghijklmnopqrstuvwxyz' : { ele ∈ text }
};

print('a quick brown fox jumps over the lazy dog!'.is_pangram());

