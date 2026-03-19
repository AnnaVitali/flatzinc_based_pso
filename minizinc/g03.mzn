int: n = 10;

array[1..n] of var 0.0..1.0: x;
array[1..n] of var 0.0..1.0: prefix_prod_x;

float: sqrt = sqrt(n);
float: scale = pow(sqrt, n);

var 0.0..1.0: product_x = prefix_prod_x[n];
var float: objective = -scale * product_x;

constraint prefix_prod_x[1] = x[1];
constraint forall(i in 2..n)(prefix_prod_x[i] = prefix_prod_x[i-1] * x[i]);

constraint sum(i in 1..n)(pow(x[i], 2.0)) - 1.0 = 0.0;

solve minimize objective;
