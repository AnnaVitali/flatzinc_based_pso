var bool: a;
var bool: b;
var bool: r;

constraint r <-> (a = b);
solve satisfy;
