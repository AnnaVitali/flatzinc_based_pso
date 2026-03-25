var float: a;
var float: b;
var bool: r;

constraint r <-> (a <= b);
solve satisfy;
