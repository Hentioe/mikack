use super::*;

def_exctractor! {
    fn is_usable(&self) -> bool { false } 
    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        Ok(vec![])
    }
}
