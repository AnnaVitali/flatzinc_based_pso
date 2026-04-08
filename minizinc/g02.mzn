int: n = 20;

var 0.0..10.0: x1;  
var 0.0..10.0: x2;  
var 0.0..10.0: x3;  
var 0.0..10.0: x4; 
var 0.0..10.0: x5;
var 0.0..10.0: x6;  
var 0.0..10.0: x7;  
var 0.0..10.0: x8;  
var 0.0..10.0: x9;  
var 0.0..10.0: x10;
var 0.0..10.0: x11; 
var 0.0..10.0: x12; 
var 0.0..10.0: x13; 
var 0.0..10.0: x14; 
var 0.0..10.0: x15;
var 0.0..10.0: x16; 
var 0.0..10.0: x17; 
var 0.0..10.0: x18; 
var 0.0..10.0: x19; 
var 0.0..10.0: x20;

var 0.0..1.0: c1 = pow(cos(x1),2.0);
var 0.0..1.0: c2 = pow(cos(x2),2.0);
var 0.0..1.0: c3 = pow(cos(x3),2.0);
var 0.0..1.0: c4 = pow(cos(x4),2.0);
var 0.0..1.0: c5 = pow(cos(x5),2.0);
var 0.0..1.0: c6 = pow(cos(x6),2.0);
var 0.0..1.0: c7 = pow(cos(x7),2.0);
var 0.0..1.0: c8 = pow(cos(x8),2.0);
var 0.0..1.0: c9 = pow(cos(x9),2.0);
var 0.0..1.0: c10 = pow(cos(x10),2.0);
var 0.0..1.0: c11 = pow(cos(x11),2.0);
var 0.0..1.0: c12 = pow(cos(x12),2.0);
var 0.0..1.0: c13 = pow(cos(x13),2.0);
var 0.0..1.0: c14 = pow(cos(x14),2.0);
var 0.0..1.0: c15 = pow(cos(x15),2.0);
var 0.0..1.0: c16 = pow(cos(x16),2.0);
var 0.0..1.0: c17 = pow(cos(x17),2.0);
var 0.0..1.0: c18 = pow(cos(x18),2.0);
var 0.0..1.0: c19 = pow(cos(x19),2.0);
var 0.0..1.0: c20 = pow(cos(x20),2.0);

var 0.0..20.0: sum_cos4 =
    pow(cos(x1),4.0) + pow(cos(x2),4.0) + pow(cos(x3),4.0) + pow(cos(x4),4.0) + pow(cos(x5),4.0) +
    pow(cos(x6),4.0) + pow(cos(x7),4.0) + pow(cos(x8),4.0) + pow(cos(x9),4.0) + pow(cos(x10),4.0) +
    pow(cos(x11),4.0) + pow(cos(x12),4.0) + pow(cos(x13),4.0) + pow(cos(x14),4.0) + pow(cos(x15),4.0) +
    pow(cos(x16),4.0) + pow(cos(x17),4.0) + pow(cos(x18),4.0) + pow(cos(x19),4.0) + pow(cos(x20),4.0);


var 0.0..1.0: prod_cos2 =c1 * c2 * c3 * c4 * c5 * c6 * c7 * c8 * c9 * c10 * c11 * c12 * c13 * c14 * c15 * c16 * c17 * c18 * c19 * c20;

var 0.0..21000.0: weighted_sum_sq = 1.0*pow(x1,2.0) + 2.0*pow(x2,2.0) + 3.0*pow(x3,2.0) + 4.0*pow(x4,2.0) + 5.0*pow(x5,2.0) + 6.0*pow(x6,2.0) + 7.0*pow(x7,2.0) + 8.0*pow(x8,2.0) + 9.0*pow(x9,2.0) + 10.0*pow(x10,2.0) + 
    11.0*pow(x11,2.0) + 12.0*pow(x12,2.0) + 13.0*pow(x13,2.0) + 14.0*pow(x14,2.0) + 15.0*pow(x15,2.0) +16.0*pow(x16,2.0) + 17.0*pow(x17,2.0) + 18.0*pow(x18,2.0) + 19.0*pow(x19,2.0) + 20.0*pow(x20,2.0);

var -2.0..20.0: numerator = sum_cos4 - 2.0 * prod_cos2;
var 1e-6..145.0: denominator = sqrt(weighted_sum_sq);

constraint 0.75 - (x1*x2*x3*x4*x5*x6*x7*x8*x9*x10*x11*x12*x13*x14*x15*x16*x17*x18*x19*x20) <= 0.0;
constraint x1+x2+x3+x4+x5+x6+x7+x8+x9+x10+x11+x12+x13+x14+x15+x16+x17+x18+x19+x20 -7.5 * n <= 0.0;

var float: objective = -abs(numerator / denominator);

solve minimize objective;