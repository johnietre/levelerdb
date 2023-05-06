mod random;
use random::Random;

fn main() {
    let mut rand = Random::new(4230497161);
    println!("{}", rand.next());
    println!("{}", rand.uniform(30));
    println!("{}", rand.one_in(45));
    println!("{}", rand.skewed(60));
}
