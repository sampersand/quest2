Frame = {:0}().__parents__[0];
object_ = (parents, body) -> {
	frame = body.create_frame();
	(parents != []).then({ frame.__parents__ = parents });
	Frame::resume(frame);
	frame
};

$syntax { object $parents:tt $b:block } = { object_($parents, $b) };
$syntax { object () } = { {:0}() };
$syntax { object () $b:block } = { object_ ([], $b) };
$syntax 50 { object $b:block } = { object_ ([], $b) };

$syntax { $n:ident $b:block } = { $n($b) };
$syntax { while $g:group $b:block } = { while({ $g }, $b); } ;
$syntax { to_list } = { '@list' };
$syntax { fn $name:tt $args:group $body:block } = { $name = $args -> $body; };
$syntax { class $name:ident $args:group $body:block } = { $name = object $args $body; } ;

StopIteration = object();

class Enumerator () {
	class Block (:0,) {
		fn '()' (class, block) {
			frame = block.create_frame();
			__parents__ = [class];
			:0
		}

		fn next (self) {
			self::frame.restart()
		}
	}

	class Func (:0,) {
		fn '()' (class, func) {
			__parents__ = [class];
			:0
		}

		fn next (self) {
			self.func()
		}
	}

	fn map (self, func) {
		Enumerator::Func {
			t = self.next();
			if (t == StopIteration, t.itself, { func(t) })
		}
	}

	fn to_list (self) {
		list = [];

		while (list.push(self.next()); list[-1] != StopIteration) {
			# do nothing
		}

		list.pop(); # to remove the `StopIteration`
		list
	}
}

Integer.upto = (min, max) -> {
	Enumerator::Block {
		ifl(max <= min, StopIteration, (min = min + 1) - 1)
	}
};



iter = 0.upto(10).map(x -> { x * 2 });
print(iter.to_list()); #=> [1,2,3,4,5,6,7,8,9]
