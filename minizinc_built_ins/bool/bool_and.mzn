var bool: a;
var bool: b;
var bool: c;

constraint c = (a /\ b);
solve satisfy;
