array[1..3] of var bool: as;
var bool: r;

constraint r <-> (as[1] xor as[2] xor as[3]);

solve satisfy;
