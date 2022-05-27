use c_macro::c;

fn say_hello() {
    print!("hello");
}

fn main() {
    unsafe {
        c! {
            void x() {
                for (int i=0; i<5; i++) {
                    'say_hello();
                    printf(", %s\n", "world");
                }
            }
        }
    }
}
