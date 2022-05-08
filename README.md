# Quest2
The reimagining of quest, much faster and more fun.

## TODOS:
[ ] Macros
[x] Assignment
[ ] `x = 3; x.y = 4; print(x.y)` `y` isnt defined on `x`.
[ ] Add all typenames to `Kernel`
[x] `:-1` is invalid somehow? (wasnt encoding negative numbers right)
[ ] resume stackframes
[ ] I didn't think of it ahead of time, but since I don't have explicit arguments in this block, the way you access eg the first argument is just... the first local variable in the scope. Which is not great, i need a way to distinguish them, i think ill have to have a separate "variables" location or somethin.
[ ] Stackframes don't have access to the outer one, so they cant access variables outside them. I'm going to need to somehow have a partially-initialized stackframe when im building things so they can reference it.
[ ] In the following code, should setting `fib` on the block affect the body? (maybe yes, as it could be a way to do statics))
```
fib = n -> {
	(n <= 1).then(n.return);

	fib(n - 1) + fib(n - 2)
});

fib.fib = fib;
```
[ ] Constants, when modified, retain it between calls. this is not desired. (this only applies to text, i presume, as its the only constant type that is heap allocated)
```
f = { "x".concat("b") };
print(f());
print(f());
```
