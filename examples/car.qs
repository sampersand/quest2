Car = object(() -> {
	# todo: optional parameters, eg wheels = 4.
	'()' = (class, maker, wheels) -> {
		__parents__ = [class];
		__frame__
	};

	to_text = self -> {
		"A car by " + self.maker + " with " + self.wheels.to_text() + " wheels."
	};

	drive = (self, distance) -> {
		print("vroom vroom, we drove ", distance, " units");
	};
});

car = Car('honda', 4);
print(car);
car.drive(10);

