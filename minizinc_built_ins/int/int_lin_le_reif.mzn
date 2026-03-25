array[1..2] of int: as = [2, -1];
array[1..2] of var int: bs;
int: c = 3;
var bool: r;

constraint r <-> (as[1]*bs[1] + as[2]*bs[2] <= c);
solve satisfy;
