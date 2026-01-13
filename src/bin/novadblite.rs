use novadb_lite::page::slot::is_dead;

fn main() {
    let vars = vec![0u8; 10];
    println!("❤❤❤ tuannm: [novadblite.rs][2][a]: {:?}", vars);
    let vars = 1 << 0;
    println!("❤❤❤ tuannm: [novadblite.rs][4][a]: {:?}", vars);
    let vars = 1 << 1;
    println!("❤❤❤ tuannm: [novadblite.rs][5][b]: {:?}", vars);
    let vars = 1 << 2;
    println!("❤❤❤ tuannm: [novadblite.rs][6][c]: {:?}", vars);

    let vars = is_dead(0);
    println!("❤❤❤ tuannm: [novadblite.rs][13][vars]: {:?}", vars);
}
