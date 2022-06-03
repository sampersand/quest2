Frame = {:0}().__parents__[0];
object_ = (parents, body) -> {
	frame = body.create_frame();
	(parents != []).then({ frame.__parents__ = parents });
	Frame::restart(frame);
	frame
};

$syntax { object $parents:tt $b:block } = { object_($parents, $b) };
$syntax { object () } = { {:0}() };
$syntax { object () $b:block } = { object_ ([], $b) };
$syntax { object $b:block } = { object_ ([], $b) };

o = object {
	x = 34;
};
