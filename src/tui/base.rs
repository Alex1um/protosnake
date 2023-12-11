use ncurses::addstr;

pub struct Input<'a> {
    name: &'a str,
    raw: String,
    validator: &'static dyn Fn(char) -> bool,
}

impl Input<'_> {
    
    pub fn new<'a>(name: &'a str, validator: &'static dyn Fn(char) -> bool) -> Input<'a> {
        Input { name, raw: String::new(), validator }
    }
    
    pub fn new_str_ascii_default<'a>(name: &'a str, value: &str) -> Input<'a> {
        Input { name, raw: String::from(value), validator: &|c| c.is_ascii_graphic() }
    }
    
    pub fn new_str_ascii<'a>(name: &'a str) -> Input<'a> {
        Input { name, raw: String::new(), validator: &|c| c.is_ascii_graphic() }
    }

    pub fn new_digit<'a>(name: &'a str) -> Input<'a> {
        Input { name, raw: String::new(), validator: &|c| c.is_ascii_digit() }
    }

    pub fn new_digit_default<'a>(name: &'a str, value: i32) -> Input<'a> {
        Input { name, raw: value.to_string(), validator: &|c| c.is_ascii_digit() }
    }

    pub fn pop(&mut self) -> Option<char> {
        self.raw.pop()
    }

    pub fn push(&mut self, c: char) {
        self.raw.push(c);
    }
    
    pub fn print_nc_name(&self) {
        addstr(&self.name);
    }

    pub fn print_nc(&self) {
        addstr(&self.raw);
    }

}