var set of 1..5: s;
var 0..5: c;

constraint c = card(s);
solve satisfy;

output ["s = \(s)\n", "c = \(c)\n"];
