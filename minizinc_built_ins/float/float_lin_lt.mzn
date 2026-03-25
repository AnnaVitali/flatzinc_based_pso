array[1..2] of float: as = [1.0, 1.0];
array[1..2] of var float: bs;
float: c = 5.0;

constraint as[1]*bs[1] + as[2]*bs[2] < c;
solve satisfy;
