extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/asm/mont_mul_384_asm.S")
        .compile("ff12381_asm");
}
