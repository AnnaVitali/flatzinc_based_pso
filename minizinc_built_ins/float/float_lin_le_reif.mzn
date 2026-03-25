array[1..2] of float: as = [2.0, -1.0];
array[1..2] of var float: bs;
float: c = 3.0;
var bool: r;

constraint r <-> (as[1]*bs[1] + as[2]*bs[2] <= c);
solve satisfy;
