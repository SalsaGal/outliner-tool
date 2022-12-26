use process::Process;

mod process;

fn main() {
    let process = Process::new(image::open("test/test.png").unwrap()).unwrap();
    process.process().save("test/out.png").unwrap();
}
