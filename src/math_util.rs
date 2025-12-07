// use std::ops::{Add, Mul, Rem, Sub};
//
// trait TwoDimensional {
//     type Num: Add<Output = Self::Num>
//         + Sub<Output = Self::Num>
//         + Mul<Output = Self::Num>
//         + Rem<Output = Self::Num>
//         + Copy
//         + PartialOrd;
//
//     fn x(&self) -> Self::Num;
//     fn y(&self) -> Self::Num;
//     fn two_dim_tup(self) -> (Self::Num, Self::Num);
//     fn from_two_dim_tup(x: Self::Num, y: Self::Num) -> Self;
// }
//
// #[derive(Clone, Copy, Debug)]
// pub struct Vec2<T>(pub T, pub T);
//
// impl <T: > for Vec2<T> {
//     type Num = T;
//     fn x(&self) -> Self::Num {
//         self.0
//     }
// }
//
// impl<T> Add for Vec2<T> {
//     type Output = T;
//
//     fn add(self, rhs: Self) -> Self::Output {
//         let (x1, y1) = self.two_dim_tup();
//         let (x2, y2) = rhs.two_dim_tup();
//
//         T::from_two_dim_tup(x1 + x2, y1 + y2)
//     }
// }
