var int: a;
var int: b;
var bool: r;

constraint r <-> (a <= b);
solve satisfy;
