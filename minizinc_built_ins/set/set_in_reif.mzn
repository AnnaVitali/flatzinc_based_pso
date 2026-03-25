var 1..5: x;
var set of 1..5: s;
var bool: r;

constraint r <-> (x in s);
solve satisfy;
