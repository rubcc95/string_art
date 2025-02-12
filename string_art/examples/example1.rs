#[inline(never)]
#[no_mangle]
fn fn1(a: usize, b: usize) -> usize{
    if a > b{
        unsafe { a.unchecked_sub(b) }
    } else{
        a
    }
}
#[inline(never)]
#[no_mangle]
fn fn2(a: usize, b: usize) -> usize {
    if let Some(diff) = a.checked_sub(b){
        diff
    } else{
        a
    }
}

fn main(){
    fn2(100, 40);
}