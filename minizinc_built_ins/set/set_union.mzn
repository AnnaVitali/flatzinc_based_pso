var set of 1..5: a;
var set of 1..5: b;
var set of 1..5: c;

constraint c = a union b;
solve satisfy;
