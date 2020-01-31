use super::*;

def_extractor! {[usable: false, searchable: true],
    fn index(&self, _page: u32) -> Result<Vec<Comic>> {
        Ok(vec![])
    }

    fn fetch_chapters(&self, _comic: &mut Comic) -> Result<()> {
        Ok(())
    }
}
