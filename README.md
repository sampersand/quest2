# Quest2
The reimagining of quest, much faster and more fun.

## TODOS:
- Macros
- Assignment
- `:0.__set_attr__("X",3); X.__set_attr__("F",4); X.F` `F` isnt defined on `X`.
- I didn't think of it ahead of time, but since I don't have explicit arguments in this block, the way you access eg the first argument is just... the first local variable in the scope. Which is not great, i need a way to distinguish them, i think ill have to have a separate "variables" location or somethin.
- Stackframes don't have access to the outer one, so they cant access variables outside them. I'm going to need to somehow have a partially-initialized stackframe when im building things so they can reference it.
- In the following code, should setting `fib` on the block affect the body? (maybe yes, as it could be a way to do statics))
```
:0.__set_attr__("fib", {
	(_0 <= 1).then(_0.return);

	fib(_0 - 1) + fib(_0 - 2)
});

fib.__set_attr__("fib", fib);
```
- Constants, when modified, retain it between calls. this is not desired. (this only applies to text, i presume, as its the only constant type that is heap allocated)
```
:0.__set_attr__("f", {
	"x".concat("b")
});
print(f());
print(f());
```
