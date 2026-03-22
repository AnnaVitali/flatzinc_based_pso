int: n = 20;

array[1..n] of var 0.0..10.0: x;


% Decomposed objective for readability

var 0.0..20.0: sum_cos4 = sum(i in 1..n)(pow(cos(x[i]), 4.0));

array[1..n] of var 0.0..1.0: cos2_x;
constraint forall(i in 1..n)(cos2_x[i] = pow(cos(x[i]), 2.0));

array[1..n] of var 0.0..1.0: prefix_prod_cos2;
constraint prefix_prod_cos2[1] = cos2_x[1];
constraint forall(i in 2..n)(prefix_prod_cos2[i] = prefix_prod_cos2[i-1] * cos2_x[i]);
var 0.0..1.0: prod_cos2 = prefix_prod_cos2[n];

var 0.0..21000.0: weighted_sum_sq = sum(i in 1..n)(i * pow(x[i], 2.0));
var float: objective = -(
    abs(
        (sum_cos4 - 2.0 * prod_cos2) / sqrt(weighted_sum_sq)
    )
);

array[1..n] of var 0.0..10.0: prefix_prod_x;
constraint prefix_prod_x[1] = x[1];
constraint forall(i in 2..n)(prefix_prod_x[i] = prefix_prod_x[i-1] * x[i]);
constraint prefix_prod_x[n] >= 0.75;
constraint sum(i in 1..n)(x[i]) <= 7.5 * n;

solve minimize objective;


