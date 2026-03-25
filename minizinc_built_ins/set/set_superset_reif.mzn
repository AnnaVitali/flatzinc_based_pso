var set of 1..5: a;
var set of 1..5: b;
var bool: r;

constraint r <-> (a superset b);
solve satisfy;
