
$syntax { object $parents:tt $b:block } = { object_($parents, $b) };
$syntax { object () } = { {:0}() };
$syntax { object () $b:block } = { object_ ([], $b) };
$syntax 50 { object $b:block } = { object_ ([], $b) };

$syntax { $n:ident $b:block } = { $n($b) };
$syntax { while $g:group $b:block } = { while({ $g }, $b); } ;
$syntax { to_list } = { '@list' };

Frame = {:0}().__parents__[0];
object_ = (parents, body) -> {
	frame = body.create_frame();
	(parents != []).then { frame.__parents__ = parents; };
	Frame::resume(frame);
	frame
};

StopIteration = object();

# Object.becomes = (self, parents) -> { self.__parents__ = parents; self };
# Object.extend = (self, parent) -> { self.__parents__.unshift(parent); self };
# Object.inherit = (self, parent) -> { self.__parents__.push(parent); self };
# 
$syntax {
	'()' = (class ${, $arg:ident} $[,]) -> $b:block;
} = {
	:0.('()') = (class ${, $arg}) -> {
		'()' = $b;
		frame = :0::'()'.create_frame(class ${, $arg});
		frame.__parents__ = [class];
		Frame::resume(frame);
		frame
	};
};

Enumerator = object {
	Block = object (:0,) {
		'()' = (class, block) -> {
			frame = block.create_frame();
		};

		next = self -> { self::frame.restart() };
	};

	Func = object (:0,) {
		'()' = (class, func) -> {
			print(func);
			#:0.__parents__ = [class];
			#:0
		};

		next = self -> { print(self); self.func() };
	};

	map = (self, func) -> {
		Enumerator::Func {
			t = self.next();
			if (t == StopIteration, t.itself, { func(t) })
		}
	};

	to_list = (self) -> {
		list = [];
		while (list.push(self.next()); list[-1] != StopIteration) {
			# do nothing
		}
		list.pop(); # to remove the `StopIteration`
		list
	};
};


Integer.upto = (min, max) -> {
	Enumerator::Block({
		(max <= min).then(StopIteration.return);
		(min = min + 1) - 1
	})
};
	
iter = 0.upto(10).map(x -> { x * 2 });
print(iter.to_list()); #=> [0, 2, 4, 6, 8, 10, 12, 14, 16, 18]
