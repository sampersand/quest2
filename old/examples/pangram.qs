# Define the function `∈` on all objects to be effectively
# a swapped `includes`.
Object."in" = (obj, list) -> { list.includes(obj) };

# Syntax macro to replace `∀ <ident> in <list> : { ... }` 
# with `<list>.iter().are_all(<ident> -> { ... })`
$syntax { forall $name:ident in $list:tt : $body:block } = {
	$list.iter().are_all($name -> $body)
};

# Define the `is_pangram` function
ALPHABET = 'abcdefghijklmnopqrstuvwxyz';
Text.is_pangram = text -> { forall ele in ALPHABET : { ele in text } };

print('a quick brown fox jumps over the lazy dog!'.is_pangram());

