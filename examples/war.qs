# why arent these in the default namespace, wtf
Null.and_then = (_, _) -> { null };
Object.and_then = (self, op) -> { op(self) };

Card = object({
	'()' = (class, rank_int, suit) -> {
		rank = ifl(rank_int == 10, '10', '234567890JQKA'.to_list()[rank_int - 2]);

		:0.__parents__ = [class];
		:0
	};

	dbg = to_text = self -> { self.rank + self.suit };

	'==' = (self, rhs) -> { self.rank_int == rhs.rank_int };
	'<=>' = (self, rhs) -> { self.rank_int <=> rhs.rank_int };

	ALL_CARDS = 
		2.upto(14)
			.product("♧♢♡♤".to_list())
			.map(list -> { :1(list[0], list[1]) });
});

Pile = object({
	'()' = (class, name, cards) -> {
		:0.__parents__ = [class];
		:0
	};

	to_text = self -> { self.name };
	is_empty = self -> { self.cards.is_empty() };

	draw = self -> { self.cards.shift() };
	add_all = (self, cards) -> { self.cards.concat(cards) };
});


battle = (p1, p2) -> {
	c1 = [p1.draw()];
	c2 = [p2.draw()];

	while({ c1 == c2 }, {
		p1.draw().and_then(c1.push);
		p1.draw().and_then(c1.push);

		p2.draw().and_then(c2.push);
		p2.draw().and_then(c2.push);
	});

	player = ifl((c1 <=> c2) >= 0, p2, p1);

	print("player ", player, " won: ", c1.concat(c2).join(" "));

	player.add_all(c1.shuffle())
};

deck = List::shuffle(Card.ALL_CARDS+[]);
player1 = Pile("1", deck[0, deck.len() / 2]);
player2 = Pile("2", deck[deck.len() / 2, deck.len() / 2]);

while({ !player1.is_empty().else(player2.is_empty) }, {
	battle(player1, player2)
});

print(ifl(player1.is_empty(), "player 2", "player 1"), " wins");
