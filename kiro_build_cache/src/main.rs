#![allow(unused_mut, unused_variables, unused_parens)]
#[tokio::main]
async fn main(){
	let mut sum = 0;
	for x in ((0..10)).step_by(2 as usize) { if (x != 4) {
println!("{}", x);
sum = (sum + x);
} else {
println!("{}", 999);
} }
	println!("{}", sum);
}
